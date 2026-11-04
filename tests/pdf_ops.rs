use lopdf::{
    Document, Object, ObjectId, Stream,
    content::{Content, Operation},
    dictionary,
};
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine,
    model::{ConversionSettings, OutputFormat, PdfAction},
    pdf_ops,
    tools::ToolPaths,
};

fn fixture(path: &Path, labels: &[&str]) {
    let mut document = Document::with_version("1.5");
    let parent = document.new_object_id();
    let branch = document.new_object_id();
    let font = document.add_object(
        dictionary! {"Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica"},
    );
    let resources = document.add_object(dictionary! {"Font" => dictionary! {"F1" => font}});
    let mut page_ids = Vec::new();
    for label in labels {
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![20.into(), 50.into()]),
                Operation::new("Tj", vec![Object::string_literal(*label)]),
                Operation::new("ET", vec![]),
            ],
        };
        let mut data = content.encode().unwrap();
        // Uncompressed whitespace is losslessly compressible without altering
        // text, image samples or page appearance.
        data.extend(vec![b' '; 8192]);
        let content = document.add_object(Stream::new(dictionary! {}, data));
        page_ids.push(
            document.add_object(
                dictionary! {"Type" => "Page", "Parent" => branch, "Contents" => content},
            ),
        );
    }
    // Deliberately use a page-tree order different from the numeric object IDs.
    page_ids.reverse();
    document.objects.insert(branch, Object::Dictionary(dictionary! {
        "Type" => "Pages", "Parent" => parent, "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => labels.len() as i64, "Rotate" => 90, "CropBox" => vec![10.into(), 10.into(), 190.into(), 90.into()],
    }));
    document.objects.insert(parent, Object::Dictionary(dictionary! {
        "Type" => "Pages", "Kids" => vec![Object::Reference(branch)], "Count" => labels.len() as i64,
        "MediaBox" => vec![0.into(), 0.into(), 200.into(), 100.into()], "Resources" => resources, "Rotate" => 180,
    }));
    let first = page_ids[0];
    let target = page_ids[page_ids.len() - 1];
    let uri = document.add_object(dictionary! {"Type" => "Annot", "Subtype" => "Link", "Rect" => vec![0.into(),0.into(),10.into(),10.into()], "A" => dictionary! {"S" => "URI", "URI" => Object::string_literal("https://example.com/")}, "P" => first});
    let local = document.add_object(dictionary! {"Type" => "Annot", "Subtype" => "Link", "Rect" => vec![0.into(),0.into(),10.into(),10.into()], "A" => dictionary! {"S" => "GoTo", "D" => Object::string_literal("last")}, "P" => first});
    document
        .get_object_mut(first)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set(
            "Annots",
            vec![Object::Reference(uri), Object::Reference(local)],
        );
    let root = document.add_object(dictionary! {"Type" => "Catalog", "Pages" => parent,
        "Names" => dictionary! {"Dests" => dictionary! {"Names" => vec![Object::string_literal("last"),Object::Array(vec![Object::Reference(target),Object::Name(b"Fit".to_vec())])]}},
        "MarkInfo" => dictionary! {"Marked" => true},
    });
    let info =
        document.add_object(dictionary! {"Title" => Object::string_literal("original metadata")});
    document.trailer.set("Root", root);
    document.trailer.set("Info", info);
    document.save(path).unwrap();
}

fn assert_page_attributes(document: &Document, id: ObjectId) {
    let page = document.get_dictionary(id).unwrap();
    assert_eq!(page.get(b"Rotate").unwrap().as_i64().unwrap(), 90);
    assert_eq!(
        page.get(b"MediaBox")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap())
            .collect::<Vec<_>>(),
        vec![0, 0, 200, 100]
    );
    assert_eq!(
        page.get(b"CropBox")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap())
            .collect::<Vec<_>>(),
        vec![10, 10, 190, 90]
    );
    assert!(
        document
            .get_page_fonts(id)
            .unwrap()
            .contains_key(b"F1".as_slice())
    );
}

#[test]
fn merge_preserves_declared_order_inheritance_resources_and_named_local_links() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("città.pdf");
    let second = directory.path().join("secondo.pdf");
    fixture(&first, &["first A", "first B"]);
    fixture(&second, &["second A"]);
    let original = fs::read(&first).unwrap();
    let conflict = directory.path().join("città-unito.pdf");
    fs::write(&conflict, b"preserve external file").unwrap();
    let result = pdf_ops::merge(
        &[first.clone(), second],
        None,
        &AtomicBool::new(false),
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(result[0].file_name().unwrap(), "città-unito (1).pdf");
    let document = Document::load(&result[0]).unwrap();
    let page_ids: Vec<_> = document.get_pages().into_values().collect();
    assert_eq!(page_ids.len(), 3);
    for id in &page_ids {
        assert_page_attributes(&document, *id);
    }
    assert!(document.extract_text(&[1]).unwrap().contains("first B"));
    assert!(document.extract_text(&[2]).unwrap().contains("first A"));
    assert!(document.extract_text(&[3]).unwrap().contains("second A"));
    let annotations = document.get_page_annotations(page_ids[0]).unwrap();
    assert_eq!(annotations.len(), 2);
    let action = annotations[1].get(b"A").unwrap().as_dict().unwrap();
    assert_eq!(
        action.get(b"D").unwrap().as_array().unwrap()[0]
            .as_reference()
            .unwrap(),
        page_ids[1]
    );
    assert!(!document.trailer.has(b"Info"));
    assert!(!document.catalog().unwrap().has(b"Names"));
    assert_eq!(fs::read(&first).unwrap(), original);
    assert_eq!(fs::read(conflict).unwrap(), b"preserve external file");
}

#[test]
fn split_materializes_inherited_page_attributes_and_removes_links_to_excluded_pages() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    fixture(&input, &["A", "B", "C"]);
    let outputs = pdf_ops::split(&input, None, &AtomicBool::new(false), &mut |_| {}).unwrap();
    assert_eq!(outputs.len(), 3);
    for (index, path) in outputs.iter().enumerate() {
        assert_eq!(
            path.file_name().unwrap().to_string_lossy(),
            format!("source-pagina-{:03}.pdf", index + 1)
        );
        let document = Document::load(path).unwrap();
        assert_eq!(document.get_pages().len(), 1);
        let page = document.get_pages()[&1];
        assert_page_attributes(&document, page);
        assert!(
            document
                .extract_text(&[1])
                .unwrap()
                .contains(["C", "B", "A"][index])
        );
        assert_eq!(
            document
                .objects
                .values()
                .filter(|object| object.type_name().is_ok_and(|name| name == b"Page"))
                .count(),
            1,
            "excluded pages leaked through annotation references"
        );
        if index == 0 {
            let annotations = document.get_page_annotations(page).unwrap();
            assert_eq!(annotations.len(), 2);
            assert_eq!(
                annotations[0]
                    .get(b"A")
                    .unwrap()
                    .as_dict()
                    .unwrap()
                    .get(b"S")
                    .unwrap()
                    .as_name()
                    .unwrap(),
                b"URI"
            );
            assert!(!annotations[1].has(b"A"));
        }
    }
}

#[test]
fn compression_never_grows_or_changes_page_text_and_keeps_original_catalog_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    fixture(&input, &["Compress me"]);
    let original = fs::read(&input).unwrap();
    let outputs = pdf_ops::compress(&input, None, &AtomicBool::new(false), &mut |_| {}).unwrap();
    assert_eq!(outputs[0].file_name().unwrap(), "source-compresso.pdf");
    assert!(fs::metadata(&outputs[0]).unwrap().len() <= original.len() as u64);
    let document = Document::load(&outputs[0]).unwrap();
    assert!(document.extract_text(&[1]).unwrap().contains("Compress me"));
    assert!(document.trailer.has(b"Info"));
    assert!(document.catalog().unwrap().has(b"Names"));
    assert_eq!(fs::read(&input).unwrap(), original);
    // Recompressing an already compressed input also obeys the size contract.
    let again = pdf_ops::compress(&outputs[0], None, &AtomicBool::new(false), &mut |_| {}).unwrap();
    assert!(fs::metadata(&again[0]).unwrap().len() <= fs::metadata(&outputs[0]).unwrap().len());
}

#[test]
fn split_cancellation_rolls_back_owned_outputs_but_preserves_replaced_files() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    fixture(&input, &["A", "B"]);
    let cancel = AtomicBool::new(false);
    let result = pdf_ops::split(&input, None, &cancel, &mut |_| {
        cancel.store(true, Ordering::Relaxed)
    });
    assert!(result.unwrap_err().to_string().contains("annullata"));
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    cancel.store(false, Ordering::Relaxed);
    let replaced = directory.path().join("source-pagina-001.pdf");
    let result = pdf_ops::split(&input, None, &cancel, &mut |_| {
        fs::remove_file(&replaced).unwrap();
        fs::write(&replaced, b"external replacement").unwrap();
        cancel.store(true, Ordering::Relaxed);
    });
    assert!(result.is_err());
    assert_eq!(fs::read(replaced).unwrap(), b"external replacement");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}

#[test]
fn encrypted_signed_and_interactive_form_pdfs_fail_before_creating_outputs() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    fixture(&input, &["A"]);
    let mut document = Document::load(&input).unwrap();
    let signature = document.add_object(
        dictionary! {"Type"=>"Sig","ByteRange"=>vec![0.into(),10.into(),20.into(),10.into()]},
    );
    document
        .catalog_mut()
        .unwrap()
        .set("Perms", dictionary! {"DocMDP"=>signature});
    let signed = directory.path().join("signed.pdf");
    document.save(&signed).unwrap();
    let destination = directory.path().join("must-not-exist");
    assert!(
        pdf_ops::compress(
            &signed,
            Some(&destination),
            &AtomicBool::new(false),
            &mut |_| {}
        )
        .unwrap_err()
        .to_string()
        .contains("firme")
    );
    assert!(!destination.exists());
    let mut document = Document::load(&input).unwrap();
    let field=document.add_object(dictionary! {"FT"=>"Tx","T"=>Object::string_literal("Name"),"V"=>Object::string_literal("Person")});
    document.catalog_mut().unwrap().set(
        "AcroForm",
        dictionary! {"Fields"=>vec![Object::Reference(field)]},
    );
    let form = directory.path().join("form.pdf");
    document.save(&form).unwrap();
    assert!(
        pdf_ops::split(
            &form,
            Some(&destination),
            &AtomicBool::new(false),
            &mut |_| {}
        )
        .unwrap_err()
        .to_string()
        .contains("moduli")
    );
    assert!(!destination.exists());
    // Compression retains functioning form dictionaries.
    let compressed = pdf_ops::compress(&form, None, &AtomicBool::new(false), &mut |_| {}).unwrap();
    assert!(
        Document::load(&compressed[0])
            .unwrap()
            .catalog()
            .unwrap()
            .has(b"AcroForm")
    );
    let mut document = Document::load(&input).unwrap();
    document.trailer.set(
        "ID",
        vec![
            Object::string_literal("0123456789abcdef"),
            Object::string_literal("0123456789abcdef"),
        ],
    );
    let state = lopdf::EncryptionState::try_from(lopdf::EncryptionVersion::V2 {
        document: &document,
        owner_password: "owner",
        user_password: "secret",
        key_length: 128,
        permissions: lopdf::Permissions::all(),
    })
    .unwrap();
    document.encrypt(&state).unwrap();
    let encrypted = directory.path().join("encrypted.pdf");
    document.save(&encrypted).unwrap();
    assert!(
        pdf_ops::split(
            &encrypted,
            Some(&destination),
            &AtomicBool::new(false),
            &mut |_| {}
        )
        .is_err()
    );
    assert!(
        pdf_ops::merge(
            &[input, encrypted],
            Some(&destination),
            &AtomicBool::new(false),
            &mut |_| {}
        )
        .is_err()
    );
    assert!(!destination.exists());
}

#[test]
fn invalid_or_cyclic_page_trees_and_pre_cancelled_jobs_do_not_publish() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("cycle.pdf");
    let mut document = Document::with_version("1.5");
    let pages = document.new_object_id();
    document.objects.insert(
        pages,
        Object::Dictionary(
            dictionary! {"Type"=>"Pages","Count"=>1,"Kids"=>vec![Object::Reference(pages)]},
        ),
    );
    let catalog = document.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages});
    document.trailer.set("Root", catalog);
    document.save(&input).unwrap();
    assert!(
        pdf_ops::split(&input, None, &AtomicBool::new(false), &mut |_| {})
            .unwrap_err()
            .to_string()
            .contains("ciclico")
    );
    assert!(
        pdf_ops::compress(&input, None, &AtomicBool::new(true), &mut |_| {})
            .unwrap_err()
            .to_string()
            .contains("annullata")
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

fn pdf_settings(action: PdfAction) -> ConversionSettings {
    let mut settings = ConversionSettings {
        format: OutputFormat::Pdf,
        ..Default::default()
    };
    settings.pdf.action = action;
    settings
}

#[test]
fn queued_split_and_compression_publish_exact_reserved_names_and_clean_their_work() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    fixture(&input, &["A", "B"]);
    let tools = ToolPaths::default();
    let plan = engine::prepare(&input, &pdf_settings(PdfAction::Split), &tools).unwrap();
    assert_eq!(plan.outputs.len(), 2);
    assert_eq!(
        plan.outputs[0].path.file_name().unwrap(),
        "source-pagina-001.pdf"
    );
    assert_eq!(
        plan.outputs[1].path.file_name().unwrap(),
        "source-pagina-002.pdf"
    );
    let result = engine::convert_prepared(&plan, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(
        result,
        plan.outputs
            .iter()
            .map(|output| output.path.clone())
            .collect::<Vec<_>>()
    );
    assert!(!plan.work_dir.exists());
    engine::release_prepared(&plan).unwrap();
    let plan = engine::prepare(&input, &pdf_settings(PdfAction::Compress), &tools).unwrap();
    assert_eq!(
        plan.outputs[0].path.file_name().unwrap(),
        "source-compresso.pdf"
    );
    let result = engine::convert_prepared(&plan, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(result, vec![plan.outputs[0].path.clone()]);
    assert!(fs::metadata(&result[0]).unwrap().len() <= fs::metadata(&input).unwrap().len());
    assert!(!plan.work_dir.exists());
    engine::release_prepared(&plan).unwrap();
    let plan = engine::prepare(&input, &pdf_settings(PdfAction::Split), &tools).unwrap();
    let cancel = AtomicBool::new(false);
    assert!(
        engine::convert_prepared(&plan, &tools, &cancel, |_| {
            cancel.store(true, Ordering::Relaxed)
        })
        .is_err()
    );
    for output in &plan.outputs {
        assert!(!output.path.exists());
    }
    assert!(!plan.work_dir.exists());
    engine::release_prepared(&plan).unwrap();
    assert!(
        fs::read_dir(directory.path())
            .unwrap()
            .all(|entry| entry.unwrap().path().is_file())
    );
}

#[test]
fn merge_is_an_explicit_validated_group_operation_including_engine_routes() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.pdf");
    let second = directory.path().join("second.pdf");
    fixture(&first, &["A"]);
    fixture(&second, &["B"]);
    let mut settings = pdf_settings(PdfAction::Merge);
    let target = directory.path().join("new-directory");
    settings.output_dir = Some(target.clone());
    let tools = ToolPaths::default();
    let error =
        engine::convert(&first, &settings, &tools, &AtomicBool::new(false), |_| {}).unwrap_err();
    assert!(error.to_string().contains("gruppo"));
    assert!(!target.exists());
    assert!(engine::prepare(&first, &settings, &tools).is_err());
    assert!(!target.exists());
    settings.audio.bitrate_kbps = 0;
    assert!(
        engine::merge_pdfs(
            &[first.clone(), second.clone()],
            &settings,
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert!(!target.exists());
    settings.audio.bitrate_kbps = 192;
    let output =
        engine::merge_pdfs(&[first, second], &settings, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(
        output[0].parent().unwrap(),
        fs::canonicalize(target).unwrap()
    );
    let document = Document::load(&output[0]).unwrap();
    assert!(document.extract_text(&[1]).unwrap().contains('A'));
    assert!(document.extract_text(&[2]).unwrap().contains('B'));
}
