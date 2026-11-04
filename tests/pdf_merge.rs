use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary,
};
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    model::{ConversionSettings, OutputFormat, PdfAction},
    pdf_merge::{MergePageRef, merge_prepared, prepare_pdf_merge},
    tools::ToolPaths,
};

fn settings(output: &Path) -> ConversionSettings {
    let mut settings = ConversionSettings {
        format: OutputFormat::Pdf,
        output_dir: Some(output.to_owned()),
        ..Default::default()
    };
    settings.pdf.action = PdfAction::Merge;
    settings
}

fn pdf(path: &Path, labels: &[&str]) {
    let mut document = Document::with_version("1.5");
    let parent = document.new_object_id();
    let font = document
        .add_object(dictionary! {"Type"=>"Font", "Subtype"=>"Type1", "BaseFont"=>"Helvetica"});
    let mut children = Vec::new();
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
        let content = document.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page = document.add_object(dictionary! {
            "Type"=>"Page", "Parent"=>parent, "Contents"=>content,
        });
        children.push(Object::Reference(page));
    }
    document.objects.insert(
        parent,
        Object::Dictionary(dictionary! {
            "Type"=>"Pages", "Count"=>children.len() as i64, "Kids"=>children,
            "MediaBox"=>vec![0.into(), 0.into(), 200.into(), 100.into()],
            "Resources"=>dictionary! {"Font"=>dictionary! {"F1"=>font}}, "Rotate"=>90,
        }),
    );
    let root = document.add_object(dictionary! {"Type"=>"Catalog", "Pages"=>parent});
    document.trailer.set("Root", root);
    document.save(path).unwrap();
}

fn image(path: &Path, color: [u8; 3]) {
    image::RgbImage::from_pixel(40, 20, image::Rgb(color))
        .save(path)
        .unwrap();
}

fn assert_image_page(document: &Document, page: u32, channel: usize) {
    let id = document.get_pages()[&page];
    let resources = document
        .get_dictionary(id)
        .unwrap()
        .get_deref(b"Resources", document)
        .unwrap()
        .as_dict()
        .unwrap();
    let objects = resources
        .get_deref(b"XObject", document)
        .unwrap()
        .as_dict()
        .unwrap();
    let stream = document
        .dereference(objects.get(b"Im0").unwrap())
        .unwrap()
        .1
        .as_stream()
        .unwrap();
    let decoded = image::load_from_memory(&stream.content).unwrap().to_rgb8();
    let pixel = decoded.get_pixel(decoded.width() / 2, decoded.height() / 2);
    assert!(pixel[channel] > 230, "unexpected image color: {pixel:?}");
    assert!(pixel[(channel + 1) % 3] < 25 && pixel[(channel + 2) % 3] < 25);
}

#[test]
fn mixed_pages_interleave_without_intermediate_outputs_or_overwriting_originals() {
    let directory = tempfile::tempdir().unwrap();
    let blue = directory.path().join("città & mare.png");
    let source = directory.path().join("source.pdf");
    let red = directory.path().join("red.png");
    image(&blue, [0, 0, 255]);
    pdf(&source, &["A1", "A2"]);
    image(&red, [255, 0, 0]);
    let inputs = vec![blue.clone(), source.clone(), red.clone()];
    let originals: Vec<_> = inputs.iter().map(|path| fs::read(path).unwrap()).collect();
    let output_dir = directory.path().join("output");
    let settings = settings(&output_dir);
    let prepared = prepare_pdf_merge(
        &inputs,
        &settings,
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert!(!output_dir.exists());
    assert_eq!(
        prepared
            .sources()
            .iter()
            .map(|source| source.page_count)
            .collect::<Vec<_>>(),
        [1, 2, 1]
    );
    assert_eq!(
        prepared.pages(),
        vec![
            MergePageRef { source: 0, page: 1 },
            MergePageRef { source: 1, page: 1 },
            MergePageRef { source: 1, page: 2 },
            MergePageRef { source: 2, page: 1 },
        ]
    );
    fs::create_dir(&output_dir).unwrap();
    let conflict = output_dir.join("città & mare-unito.pdf");
    fs::write(&conflict, b"external result").unwrap();
    let order = [
        MergePageRef { source: 1, page: 2 },
        MergePageRef { source: 0, page: 1 },
        MergePageRef { source: 1, page: 1 },
        MergePageRef { source: 2, page: 1 },
    ];
    let outputs = merge_prepared(
        &prepared,
        &order,
        &settings,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0].file_name().unwrap(),
        "città & mare-unito (1).pdf"
    );
    let result = Document::load(&outputs[0]).unwrap();
    assert_eq!(result.get_pages().len(), 4);
    assert!(result.extract_text(&[1]).unwrap().contains("A2"));
    assert!(result.extract_text(&[3]).unwrap().contains("A1"));
    assert_image_page(&result, 2, 2);
    assert_image_page(&result, 4, 0);
    for page in [1, 3] {
        assert_eq!(
            result
                .get_dictionary(result.get_pages()[&page])
                .unwrap()
                .get(b"Rotate")
                .unwrap()
                .as_i64()
                .unwrap(),
            90
        );
    }
    assert_eq!(fs::read(&conflict).unwrap(), b"external result");
    assert_eq!(fs::read_dir(&output_dir).unwrap().count(), 2);
    for (input, original) in inputs.iter().zip(originals) {
        assert_eq!(fs::read(input).unwrap(), original);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        assert_eq!(fs::metadata(&outputs[0]).unwrap().file_attributes() & 2, 0);
    }
}

#[test]
fn saving_uses_prepared_snapshots_even_after_originals_are_deleted() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    let picture = directory.path().join("picture.png");
    pdf(&input, &["Captured text"]);
    image(&picture, [0, 0, 255]);
    let settings = settings(&directory.path().join("output"));
    let prepared = prepare_pdf_merge(
        &[input.clone(), picture.clone()],
        &settings,
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    fs::remove_file(input).unwrap();
    fs::remove_file(picture).unwrap();
    let outputs = merge_prepared(
        &prepared,
        &prepared.pages(),
        &settings,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let result = Document::load(&outputs[0]).unwrap();
    assert!(result.extract_text(&[1]).unwrap().contains("Captured text"));
    assert_image_page(&result, 2, 2);
}

#[test]
fn invalid_page_permutations_never_create_the_output_directory() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source.pdf");
    pdf(&source, &["A", "B"]);
    let output = directory.path().join("output");
    let settings = settings(&output);
    let prepared = prepare_pdf_merge(
        &[source],
        &settings,
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    for order in [
        vec![],
        vec![MergePageRef { source: 0, page: 1 }],
        vec![MergePageRef { source: 0, page: 1 }; 2],
        vec![
            MergePageRef { source: 1, page: 1 },
            MergePageRef { source: 0, page: 2 },
        ],
        vec![
            MergePageRef { source: 0, page: 0 },
            MergePageRef { source: 0, page: 2 },
        ],
        vec![
            MergePageRef { source: 0, page: 1 },
            MergePageRef { source: 0, page: 3 },
        ],
    ] {
        assert!(
            merge_prepared(
                &prepared,
                &order,
                &settings,
                &AtomicBool::new(false),
                |_| {}
            )
            .is_err()
        );
        assert!(!output.exists());
    }
}

#[test]
fn incompatible_inputs_and_missing_office_engine_fail_without_destination_writes() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("output");
    let settings = settings(&output);
    for name in ["source.mp3", "source.mp4", "source.unknown", "source.docx"] {
        let input = directory.path().join(name);
        fs::write(&input, b"unchanged input").unwrap();
        assert!(
            prepare_pdf_merge(
                std::slice::from_ref(&input),
                &settings,
                &ToolPaths::default(),
                &AtomicBool::new(false),
                |_| {}
            )
            .is_err()
        );
        assert_eq!(fs::read(input).unwrap(), b"unchanged input");
        assert!(!output.exists());
    }
    assert!(
        prepare_pdf_merge(
            &[],
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert!(
        prepare_pdf_merge(
            &[directory.path().to_owned()],
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
}

#[test]
fn preparation_and_merge_cancellation_publish_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.pdf");
    let second = directory.path().join("second.pdf");
    pdf(&first, &["A"]);
    pdf(&second, &["B"]);
    let inputs = vec![first, second];
    let output = directory.path().join("output");
    let settings = settings(&output);
    let cancel = AtomicBool::new(false);
    let error = prepare_pdf_merge(
        &inputs,
        &settings,
        &ToolPaths::default(),
        &cancel,
        |progress| {
            if progress.message == "Preparata sorgente 1 di 2" {
                cancel.store(true, Ordering::Relaxed);
            }
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("annullata"));
    assert!(!output.exists());
    cancel.store(false, Ordering::Relaxed);
    let prepared =
        prepare_pdf_merge(&inputs, &settings, &ToolPaths::default(), &cancel, |_| {}).unwrap();
    let error = merge_prepared(&prepared, &prepared.pages(), &settings, &cancel, |_| {
        cancel.store(true, Ordering::Relaxed);
    })
    .unwrap_err();
    assert!(error.to_string().contains("annullata"));
    assert!(!output.exists());
    assert!(inputs.iter().all(|input| input.is_file()));
}

#[test]
fn encrypted_signed_and_form_sources_are_rejected_during_preparation() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    let output = directory.path().join("output");
    let settings = settings(&output);
    for kind in ["owner", "opening", "signed", "form", "widget"] {
        pdf(&input, &["Restricted"]);
        let mut document = Document::load(&input).unwrap();
        match kind {
            "owner" | "opening" => {
                document.trailer.set(
                    "ID",
                    vec![
                        Object::string_literal(vec![1; 16]),
                        Object::string_literal(vec![2; 16]),
                    ],
                );
                let state = lopdf::EncryptionState::try_from(lopdf::EncryptionVersion::V2 {
                    document: &document,
                    owner_password: "owner",
                    user_password: if kind == "owner" { "" } else { "secret" },
                    key_length: 128,
                    permissions: lopdf::Permissions::all(),
                })
                .unwrap();
                document.encrypt(&state).unwrap();
            }
            "signed" => {
                let signature = document.add_object(dictionary! {"Type"=>"Sig", "ByteRange"=>vec![0.into(), 1.into(), 2.into(), 1.into()]});
                document
                    .catalog_mut()
                    .unwrap()
                    .set("Perms", dictionary! {"DocMDP"=>signature});
            }
            "form" => document.catalog_mut().unwrap().set(
                "AcroForm",
                dictionary! {"Fields"=>vec![Object::Dictionary(dictionary! {"FT"=>"Tx"})]},
            ),
            "widget" => {
                let page = document.get_pages()[&1];
                let widget = document.add_object(dictionary! {"Subtype"=>"Widget", "FT"=>"Tx"});
                document
                    .get_object_mut(page)
                    .unwrap()
                    .as_dict_mut()
                    .unwrap()
                    .set("Annots", vec![Object::Reference(widget)]);
            }
            _ => unreachable!(),
        }
        document.save(&input).unwrap();
        let original = fs::read(&input).unwrap();
        assert!(
            prepare_pdf_merge(
                std::slice::from_ref(&input),
                &settings,
                &ToolPaths::default(),
                &AtomicBool::new(false),
                |_| {}
            )
            .is_err(),
            "accepted {kind}"
        );
        assert_eq!(fs::read(&input).unwrap(), original);
        assert!(!output.exists());
    }
}

#[test]
fn libreoffice_sources_join_native_images_with_page_level_order() {
    let tools = ToolPaths::discover();
    if tools.soffice.is_none() {
        eprintln!("SKIP mixed Office merge: LibreOffice unavailable");
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let writer = directory.path().join("writer.rtf");
    let html = directory.path().join("page.html");
    let picture = directory.path().join("picture.png");
    fs::write(
        &writer,
        r"{\rtf1\ansi FirstWriterPage\page SecondWriterPage}",
    )
    .unwrap();
    fs::write(
        &html,
        "<!doctype html><html><body><p>HtmlPage</p></body></html>",
    )
    .unwrap();
    image(&picture, [255, 0, 0]);
    let inputs = vec![writer, html, picture];
    let originals: Vec<_> = inputs.iter().map(|path| fs::read(path).unwrap()).collect();
    let output = directory.path().join("output");
    let settings = settings(&output);
    let prepared =
        prepare_pdf_merge(&inputs, &settings, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(
        prepared
            .sources()
            .iter()
            .map(|source| source.page_count)
            .collect::<Vec<_>>(),
        [2, 1, 1]
    );
    assert!(!output.exists());
    let order = [
        MergePageRef { source: 0, page: 2 },
        MergePageRef { source: 1, page: 1 },
        MergePageRef { source: 0, page: 1 },
        MergePageRef { source: 2, page: 1 },
    ];
    let result = merge_prepared(
        &prepared,
        &order,
        &settings,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let document = Document::load(&result[0]).unwrap();
    for (number, label) in [
        (1, "SecondWriterPage"),
        (2, "HtmlPage"),
        (3, "FirstWriterPage"),
    ] {
        assert!(document.extract_text(&[number]).unwrap().contains(label));
    }
    assert_image_page(&document, 4, 0);
    assert_eq!(fs::read_dir(output).unwrap().count(), 1);
    for (input, original) in inputs.iter().zip(originals) {
        assert_eq!(fs::read(input).unwrap(), original);
    }
}
