//! Local PDF page operations. Merge/split retain page content and ordinary links,
//! but rebuild the catalog: document metadata, outlines and tagging are omitted.
//! Interactive forms and signatures require a dedicated editor and are rejected.
use crate::engine::{Progress, cancelled, commit, file_identity, stem, temporary};
use crate::i18n::tr;
use crate::model::ConversionLimits;
use crate::pdf_merge::{MergePageRef, validate_order};
use anyhow::{Context, Result, bail};
use lopdf::{Dictionary, Document, Object, ObjectId, dictionary};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

const MAX_PAGES: usize = 10_000;
const MAX_TREE_DEPTH: usize = 128;
const INHERITED: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];

type Page = (ObjectId, Dictionary);

/// Merge pages in input/file order into one collision-safe `*-unito.pdf`.
pub fn merge(
    inputs: &[PathBuf],
    dir: Option<&Path>,
    cancel: &std::sync::atomic::AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    cancelled(cancel)?;
    if inputs.len() < 2 || inputs.len() > MAX_PAGES {
        bail!(
            "{}",
            crate::trf!(
                "Per unire i PDF seleziona da 2 a {MAX_PAGES} file",
                "To merge PDFs, select 2 to {MAX_PAGES} files"
            )
        );
    }
    let first = canonical_pdf(&inputs[0])?;
    merge_ordered(
        inputs,
        None,
        &first,
        dir,
        &ConversionLimits::default(),
        cancel,
        progress,
    )
}

/// Prepared sources are private snapshots; naming_input is the original label.
pub(crate) fn merge_ordered(
    inputs: &[PathBuf],
    order: Option<&[MergePageRef]>,
    naming_input: &Path,
    dir: Option<&Path>,
    limits: &ConversionLimits,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    cancelled(cancel)?;
    if inputs.is_empty() || inputs.len() > MAX_PAGES {
        bail!(
            "{}",
            crate::trf!(
                "L'unione richiede da 1 a {MAX_PAGES} sorgenti PDF",
                "Merging requires 1 to {MAX_PAGES} PDF sources"
            )
        );
    }
    let mut merged = Document::with_version("1.7");
    let mut source_page_ids = Vec::with_capacity(inputs.len());
    let mut total_pages = 0usize;
    let mut next_id = 1u32;
    for (index, input) in inputs.iter().enumerate() {
        cancelled(cancel)?;
        let (_, source, _) = load(input)?;
        reject_forms(&source)?;
        let pages = pages(&source)?;
        total_pages += pages.len();
        if total_pages > MAX_PAGES {
            bail!(
                "{}",
                crate::trf!(
                    "Il PDF unito non può superare {MAX_PAGES} pagine",
                    "The merged PDF cannot exceed {MAX_PAGES} pages"
                )
            );
        }
        let mut document = rebuild(&source, &pages)?;
        let object_count = u32::try_from(document.objects.len())?;
        next_id
            .checked_add(object_count + 2)
            .context(tr("Troppi oggetti nei PDF", "Too many objects in the PDFs"))?;
        document.renumber_objects_with(next_id);
        next_id = document
            .max_id
            .checked_add(1)
            .context(tr("Troppi oggetti nei PDF", "Too many objects in the PDFs"))?;
        source_page_ids.push(document.get_pages().into_values().collect::<Vec<_>>());
        merged.max_id = document.max_id;
        merged.objects.extend(document.objects);
        progress(Progress {
            fraction: 0.8 * (index + 1) as f32 / inputs.len() as f32,
            message: crate::trf!(
                "Preparato PDF {} di {}",
                "Prepared PDF {} of {}",
                index + 1,
                inputs.len()
            ),
        });
    }
    cancelled(cancel)?;
    let page_ids: Vec<_> = if let Some(order) = order {
        let counts: Vec<_> = source_page_ids
            .iter()
            .map(|pages| pages.len() as u32)
            .collect();
        validate_order(order, &counts)?;
        order
            .iter()
            .map(|page| source_page_ids[page.source][page.page as usize - 1])
            .collect()
    } else {
        source_page_ids.into_iter().flatten().collect()
    };
    attach_pages(&mut merged, &page_ids)?;
    merged.prune_objects();
    merged.renumber_objects();
    compress_streams(&mut merged, cancel)?;
    let dir = destination(naming_input, dir)?;
    let mut output = temporary(&dir, "pdf")?;
    merged.save_to(output.as_file_mut())?;
    cancelled(cancel)?;
    limits.check_bytes(output.as_file().metadata()?.len())?;
    let output = commit(
        output,
        &dir,
        &format!("{}-unito", stem(naming_input)),
        "pdf",
    )?;
    progress(Progress {
        fraction: 1.0,
        message: tr("PDF uniti", "PDFs merged").into(),
    });
    Ok(vec![output])
}

pub(crate) fn merge_page_count(input: &Path, cancel: &AtomicBool) -> Result<u32> {
    cancelled(cancel)?;
    let (_, document, _) = load(input)?;
    reject_forms(&document)?;
    let pages = pages(&document)?;
    for (_, page) in &pages {
        cancelled(cancel)?;
        if let Ok(annotations) = page.get_deref(b"Annots", &document) {
            for annotation in annotations.as_array()? {
                let dictionary = document.dereference(annotation)?.1.as_dict()?;
                if dictionary
                    .get(b"Subtype")
                    .and_then(Object::as_name)
                    .is_ok_and(|subtype| subtype == b"Widget")
                {
                    bail!(
                        "{}",
                        crate::trf!(
                            "Il PDF contiene campi modulo: esportalo come PDF statico prima di unirlo",
                            "The PDF contains form fields: export it as a static PDF before merging"
                        )
                    );
                }
            }
        }
    }
    Ok(u32::try_from(pages.len())?)
}

/// Export one PDF per page. A failed/cancelled batch rolls back files it owns.
pub fn split(
    input: &Path,
    dir: Option<&Path>,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    cancelled(cancel)?;
    let (input, source, _) = load(input)?;
    reject_forms(&source)?;
    let pages = pages(&source)?;
    let dir = destination(&input, dir)?;
    let mut published: Vec<(PathBuf, File)> = Vec::new();
    let result = (|| -> Result<()> {
        for (index, page) in pages.iter().enumerate() {
            cancelled(cancel)?;
            let mut document = rebuild(&source, std::slice::from_ref(page))?;
            document.renumber_objects();
            compress_streams(&mut document, cancel)?;
            let mut output = temporary(&dir, "pdf")?;
            document.save_to(output.as_file_mut())?;
            cancelled(cancel)?;
            // Retain the inode/file identity before committing, so rollback
            // never mistakes a subsequently replaced path for this output.
            let retained = output.as_file().try_clone()?;
            let output = commit(
                output,
                &dir,
                &format!("{}-pagina-{:03}", stem(&input), index + 1),
                "pdf",
            )?;
            published.push((output, retained));
            progress(Progress {
                fraction: (index + 1) as f32 / pages.len() as f32,
                message: crate::trf!(
                    "Separata pagina {} di {}",
                    "Split page {} of {}",
                    index + 1,
                    pages.len()
                ),
            });
        }
        cancelled(cancel)
    })();
    if let Err(error) = result {
        for (path, retained) in &published {
            if let Ok(current) = File::open(path)
                && matches!((file_identity(retained), file_identity(&current)), (Ok(a), Ok(b)) if a == b)
            {
                let _ = fs::remove_file(path);
            }
        }
        return Err(error);
    }
    Ok(published.into_iter().map(|(path, _)| path).collect())
}

/// Lossless stream compression; never publish more bytes than the input snapshot.
/// Catalogs/forms remain intact. Signed PDFs are rejected because rewriting
/// invalidates their byte-range signatures even if the page appearance survives.
pub fn compress(
    input: &Path,
    dir: Option<&Path>,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    cancelled(cancel)?;
    let (input, mut document, original) = load(input)?;
    pages(&document)?;
    progress(Progress {
        fraction: 0.15,
        message: tr(
            "Compressione PDF senza perdita…",
            "Compressing PDF without loss…",
        )
        .into(),
    });
    cancelled(cancel)?;
    document.prune_objects();
    document.renumber_objects();
    compress_streams(&mut document, cancel)?;
    let dir = destination(&input, dir)?;
    let mut output = temporary(&dir, "pdf")?;
    document.save_to(output.as_file_mut())?;
    if output.as_file().metadata()?.len() > original.len() as u64 {
        output.as_file_mut().set_len(0)?;
        use std::io::{Seek, SeekFrom};
        output.as_file_mut().seek(SeekFrom::Start(0))?;
        output.as_file_mut().write_all(&original)?;
    }
    cancelled(cancel)?;
    let output = commit(output, &dir, &format!("{}-compresso", stem(&input)), "pdf")?;
    progress(Progress {
        fraction: 1.0,
        message: tr("PDF salvato senza perdita", "PDF saved without loss").into(),
    });
    Ok(vec![output])
}

fn canonical_pdf(input: &Path) -> Result<PathBuf> {
    let input = fs::canonicalize(input).with_context(|| {
        crate::trf!(
            "PDF non accessibile: {}",
            "PDF is inaccessible: {}",
            input.display()
        )
    })?;
    if !input.is_file()
        || !input
            .extension()
            .is_some_and(|value| value.eq_ignore_ascii_case("pdf"))
    {
        bail!(
            "{}",
            crate::trf!(
                "Seleziona un file PDF esistente",
                "Select an existing PDF file"
            )
        );
    }
    Ok(input)
}

fn load(input: &Path) -> Result<(PathBuf, Document, Vec<u8>)> {
    let input = canonical_pdf(input)?;
    let original =
        fs::read(&input).context(tr("Impossibile leggere il PDF", "Unable to read the PDF"))?;
    let document = Document::load_mem(&original).context(tr(
        "PDF non valido o non leggibile",
        "Invalid or unreadable PDF",
    ))?;
    if document.is_encrypted() {
        bail!(
            "{}",
            crate::trf!(
                "Il PDF è protetto: sbloccalo prima di questa operazione",
                "The PDF is protected: unlock it before this operation"
            )
        );
    }
    if document
        .objects
        .values()
        .any(|object| contains_signature(object, 0))
    {
        bail!(
            "{}",
            crate::trf!(
                "Il PDF contiene firme digitali: questa operazione le invaliderebbe. Usa una copia senza firme",
                "The PDF contains digital signatures: this operation would invalidate them. Use an unsigned copy"
            )
        );
    }
    Ok((input, document, original))
}

fn contains_signature(object: &Object, depth: usize) -> bool {
    if depth > MAX_TREE_DEPTH {
        return false;
    }
    match object {
        Object::Dictionary(dict) => {
            dict.has(b"ByteRange")
                || dict
                    .get(b"Type")
                    .and_then(Object::as_name)
                    .is_ok_and(|value| value == b"Sig")
                || dict
                    .iter()
                    .any(|(_, value)| contains_signature(value, depth + 1))
        }
        Object::Array(values) => values
            .iter()
            .any(|value| contains_signature(value, depth + 1)),
        Object::Stream(stream) => {
            contains_signature(&Object::Dictionary(stream.dict.clone()), depth + 1)
        }
        _ => false,
    }
}

fn reject_forms(document: &Document) -> Result<()> {
    if let Ok(form) = document.catalog()?.get(b"AcroForm") {
        let form = document.dereference(form)?.1.as_dict()?;
        if form.has(b"XFA")
            || form
                .get_deref(b"Fields", document)
                .and_then(Object::as_array)
                .is_ok_and(|fields| !fields.is_empty())
        {
            bail!(
                "{}",
                crate::trf!(
                    "Il PDF contiene moduli interattivi: esportalo come PDF statico prima di unire o separare le pagine",
                    "The PDF contains interactive forms: export it as a static PDF before merging or splitting pages"
                )
            );
        }
    }
    Ok(())
}

fn destination(input: &Path, dir: Option<&Path>) -> Result<PathBuf> {
    let dir = dir.unwrap_or_else(|| input.parent().unwrap());
    fs::create_dir_all(dir).context(tr(
        "Impossibile creare la cartella di destinazione",
        "Unable to create the output folder",
    ))?;
    fs::canonicalize(dir).context(tr(
        "Cartella di destinazione non accessibile",
        "Output folder is inaccessible",
    ))
}

/// Traverse Kids in declared order rather than sorting object IDs. Materialize
/// inherited attributes before old page-tree ancestors become unreachable.
fn pages(document: &Document) -> Result<Vec<Page>> {
    let root = document.catalog()?.get(b"Pages")?.as_reference()?;
    let mut stack = vec![(root, Dictionary::new(), 0usize)];
    let mut seen = BTreeSet::new();
    let mut output = Vec::new();
    while let Some((id, mut inherited, depth)) = stack.pop() {
        if depth > MAX_TREE_DEPTH || !seen.insert(id) {
            bail!(
                "{}",
                crate::trf!(
                    "Albero delle pagine PDF non valido o ciclico",
                    "Invalid or cyclic PDF page tree"
                )
            );
        }
        let dict = document
            .get_dictionary(id)
            .context(tr("Pagina PDF non valida", "Invalid PDF page"))?;
        for key in INHERITED {
            if let Ok(value) = dict.get(key) {
                inherited.set(key, value.clone());
            }
        }
        match dict.get_type()? {
            b"Page" => {
                let mut page = dict.clone();
                for (key, value) in inherited.iter() {
                    if !page.has(key) {
                        page.set(key.clone(), value.clone());
                    }
                }
                if !page.has(b"MediaBox") {
                    bail!(
                        "{}",
                        crate::trf!(
                            "Pagina PDF senza dimensioni MediaBox",
                            "PDF page has no MediaBox dimensions"
                        )
                    );
                }
                if !page.has(b"Resources") {
                    page.set("Resources", Dictionary::new());
                }
                output.push((id, page));
                if output.len() > MAX_PAGES {
                    bail!(
                        "{}",
                        crate::trf!(
                            "Il PDF non può superare {MAX_PAGES} pagine",
                            "The PDF cannot exceed {MAX_PAGES} pages"
                        )
                    );
                }
            }
            b"Pages" => {
                let kids = dict.get_deref(b"Kids", document)?.as_array()?;
                for child in kids.iter().rev() {
                    stack.push((child.as_reference()?, inherited.clone(), depth + 1));
                }
            }
            _ => bail!(
                "{}",
                crate::trf!(
                    "Albero delle pagine PDF non valido",
                    "Invalid PDF page tree"
                )
            ),
        }
    }
    if output.is_empty() {
        bail!(
            "{}",
            crate::trf!("Il PDF non contiene pagine", "The PDF contains no pages")
        );
    }
    Ok(output)
}

fn rebuild(source: &Document, pages: &[Page]) -> Result<Document> {
    let allowed: BTreeSet<ObjectId> = pages.iter().map(|(id, _)| *id).collect();
    let mut output = source.clone();
    for (id, dictionary) in pages {
        let mut page = dictionary.clone();
        // These require discarded document-level structures and cannot retain
        // correct semantics after rebuilding catalogs or selecting one page.
        page.remove(b"StructParents");
        page.remove(b"B");
        page.remove(b"AA");
        if let Ok(annotations) = dictionary.get_deref(b"Annots", source) {
            let mut sanitized = Vec::new();
            for annotation in annotations.as_array()? {
                let mut dict = source.dereference(annotation)?.1.as_dict()?.clone();
                if dict
                    .get(b"Subtype")
                    .and_then(Object::as_name)
                    .is_ok_and(|value| value == b"Widget")
                {
                    bail!(
                        "{}",
                        crate::trf!(
                            "Il PDF contiene campi modulo: esportalo come PDF statico prima di separarlo o unirlo",
                            "The PDF contains form fields: export it as a static PDF before splitting or merging"
                        )
                    );
                }
                dict.remove(b"StructParent");
                dict.remove(b"AA");
                dict.set("P", *id);
                if let Ok(destination) = dict.get(b"Dest").cloned() {
                    match explicit_destination(source, &destination, &allowed, 0)? {
                        Some(destination) => dict.set("Dest", destination),
                        None => {
                            dict.remove(b"Dest");
                        }
                    }
                }
                if let Ok(action) = dict.get(b"A") {
                    let mut action = source.dereference(action)?.1.as_dict()?.clone();
                    action.remove(b"Next");
                    if action
                        .get(b"S")
                        .and_then(Object::as_name)
                        .is_ok_and(|value| value == b"GoTo")
                    {
                        let target = action
                            .get(b"D")
                            .ok()
                            .map(|target| explicit_destination(source, target, &allowed, 0))
                            .transpose()?
                            .flatten();
                        if let Some(target) = target {
                            action.set("D", target);
                            dict.set("A", action);
                        } else {
                            dict.remove(b"A");
                        }
                    } else {
                        dict.set("A", action);
                    }
                }
                if let Object::Reference(annotation_id) = annotation {
                    output
                        .objects
                        .insert(*annotation_id, Object::Dictionary(dict));
                    sanitized.push(annotation.clone());
                } else {
                    sanitized.push(Object::Dictionary(dict));
                }
            }
            page.set("Annots", sanitized);
        }
        output.objects.insert(*id, Object::Dictionary(page));
    }
    attach_pages(
        &mut output,
        &pages.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
    )?;
    output.prune_objects();
    Ok(output)
}

fn attach_pages(document: &mut Document, pages: &[ObjectId]) -> Result<()> {
    let parent = document.add_object(dictionary! {
        "Type" => "Pages", "Kids" => pages.iter().copied().map(Object::Reference).collect::<Vec<_>>(), "Count" => pages.len() as i64,
    });
    for id in pages {
        document
            .get_object_mut(*id)?
            .as_dict_mut()?
            .set("Parent", parent);
    }
    let root = document.add_object(dictionary! {"Type" => "Catalog", "Pages" => parent});
    document.trailer = dictionary! {"Root" => root};
    Ok(())
}

fn explicit_destination(
    document: &Document,
    value: &Object,
    allowed: &BTreeSet<ObjectId>,
    depth: usize,
) -> Result<Option<Object>> {
    if depth > MAX_TREE_DEPTH {
        bail!(
            "{}",
            crate::trf!("Destinazione PDF ciclica", "Cyclic PDF destination")
        );
    }
    let value = document.dereference(value)?.1;
    match value {
        Object::Array(items) => Ok(items
            .first()
            .and_then(|item| item.as_reference().ok())
            .filter(|id| allowed.contains(id))
            .map(|_| value.clone())),
        Object::Dictionary(dict) => match dict.get(b"D") {
            Ok(destination) => explicit_destination(document, destination, allowed, depth + 1),
            Err(_) => Ok(None),
        },
        Object::String(name, _) | Object::Name(name) => {
            if let Ok(destinations) = document
                .catalog()?
                .get_deref(b"Dests", document)
                .and_then(Object::as_dict)
                && let Ok(destination) = destinations.get(name)
            {
                return explicit_destination(document, destination, allowed, depth + 1);
            }
            if let Ok(names) = document
                .catalog()?
                .get_deref(b"Names", document)
                .and_then(Object::as_dict)
                && let Ok(tree) = names.get(b"Dests")
                && let Some(destination) = named_destination(document, tree, name)?
            {
                return explicit_destination(document, &destination, allowed, depth + 1);
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn named_destination(document: &Document, root: &Object, name: &[u8]) -> Result<Option<Object>> {
    let mut stack = vec![(root, 0)];
    let mut seen = BTreeSet::new();
    while let Some((node, depth)) = stack.pop() {
        if depth > MAX_TREE_DEPTH {
            bail!(
                "{}",
                crate::trf!(
                    "Albero dei collegamenti PDF non valido",
                    "Invalid PDF link tree"
                )
            );
        }
        if let Object::Reference(id) = node
            && !seen.insert(*id)
        {
            bail!(
                "{}",
                crate::trf!(
                    "Albero dei collegamenti PDF ciclico",
                    "Cyclic PDF link tree"
                )
            );
        }
        let dict = document.dereference(node)?.1.as_dict()?;
        if let Ok(names) = dict
            .get_deref(b"Names", document)
            .and_then(Object::as_array)
        {
            for pair in names.as_chunks::<2>().0 {
                let key = match &pair[0] {
                    Object::String(bytes, _) | Object::Name(bytes) => bytes,
                    _ => continue,
                };
                if key == name {
                    return Ok(Some(pair[1].clone()));
                }
            }
        }
        if let Ok(kids) = dict.get_deref(b"Kids", document).and_then(Object::as_array) {
            stack.extend(kids.iter().map(|kid| (kid, depth + 1)));
        }
    }
    Ok(None)
}

fn compress_streams(document: &mut Document, cancel: &AtomicBool) -> Result<()> {
    for object in document.objects.values_mut() {
        cancelled(cancel)?;
        if let Object::Stream(stream) = object
            && stream.allows_compression
        {
            stream.compress()?;
        }
    }
    Ok(())
}
