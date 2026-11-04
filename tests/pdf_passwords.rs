use lopdf::{Document, Object, Stream, dictionary};
use std::{fs, path::Path, sync::atomic::AtomicBool};
use the_converter::{
    engine,
    model::{ConversionSettings, OutputFormat},
    tools::ToolPaths,
};

const PASSWORD_ERROR: &str =
    "Il PDF richiede una password di apertura: sbloccalo prima della conversione";

fn encrypted_pdf(path: &Path, user_password: &str) {
    let mut document = Document::with_version("1.5");
    let parent = document.new_object_id();
    let mut kids = Vec::new();
    for _ in 0..2 {
        let content = document.add_object(Stream::new(
            dictionary! {},
            b"0.2 0.4 0.8 rg 10 10 80 40 re f".to_vec(),
        ));
        let page = document.add_object(dictionary! {
            "Type" => "Page", "Parent" => parent,
            "MediaBox" => vec![0.into(), 0.into(), 200.into(), 100.into()],
            "Resources" => dictionary! {}, "Contents" => content,
        });
        kids.push(Object::Reference(page));
    }
    document.objects.insert(
        parent,
        Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => 2 }),
    );
    let catalog = document.add_object(dictionary! { "Type" => "Catalog", "Pages" => parent });
    document.trailer.set("Root", catalog);
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
        user_password,
        key_length: 128,
        permissions: lopdf::Permissions::all(),
    })
    .unwrap();
    document.encrypt(&state).unwrap();
    document.save(path).unwrap();
}

fn settings(format: OutputFormat) -> ConversionSettings {
    ConversionSettings {
        format,
        ..Default::default()
    }
}

#[test]
fn fallback_reserves_owner_only_pdf_pages_and_rejects_opening_passwords() {
    let dir = tempfile::tempdir().unwrap();
    let owner = dir.path().join("owner-only.pdf");
    let password = dir.path().join("opening-password.pdf");
    encrypted_pdf(&owner, "");
    encrypted_pdf(&password, "secret");
    let loaded = Document::load(&owner).unwrap();
    assert!(loaded.is_encrypted());
    assert_eq!(loaded.get_pages().len(), 2);
    let original = fs::read(&owner).unwrap();
    let tools = ToolPaths {
        pdftoppm: Some(std::env::current_exe().unwrap()),
        ..Default::default()
    };
    for format in [OutputFormat::Png, OutputFormat::Jpg] {
        let plan = engine::prepare(&owner, &settings(format), &tools).unwrap();
        assert_eq!(plan.outputs.len(), 2);
        engine::release_prepared(&plan).unwrap();
        let error = engine::prepare(&password, &settings(format), &tools).unwrap_err();
        assert!(format!("{error:#}").contains(PASSWORD_ERROR), "{error:#}");
    }
    assert_eq!(fs::read(owner).unwrap(), original);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn poppler_owner_only_pdf_converts_directly_and_at_exact_queued_names() {
    let tools = ToolPaths::discover();
    if tools.pdfinfo.is_none() || tools.pdftoppm.is_none() {
        eprintln!("SKIP owner-password rendering: Poppler pdfinfo/pdftoppm unavailable");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("estratto città.pdf");
    encrypted_pdf(&input, "");
    let original = fs::read(&input).unwrap();
    assert_eq!(
        engine::pdf_page_count(&input, &tools, &AtomicBool::new(false)).unwrap(),
        2
    );
    for format in [OutputFormat::Png, OutputFormat::Jpg] {
        let plan = engine::prepare(&input, &settings(format), &tools).unwrap();
        let outputs =
            engine::convert_prepared(&plan, &tools, &AtomicBool::new(false), |_| {}).unwrap();
        assert_eq!(
            outputs,
            plan.outputs
                .iter()
                .map(|output| output.path.clone())
                .collect::<Vec<_>>()
        );
        for output in &outputs {
            assert!(image::open(output).unwrap().width() > 0);
        }
        engine::release_prepared(&plan).unwrap();
        let direct = engine::convert(
            &input,
            &settings(format),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert_eq!(direct.len(), 2);
        assert!(direct.iter().all(|path| !outputs.contains(path)));
    }
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn opening_password_has_same_italian_error_for_probe_queue_images_and_text() {
    let tools = ToolPaths::discover();
    if tools.pdfinfo.is_none() || tools.pdftoppm.is_none() || tools.pdftotext.is_none() {
        eprintln!("SKIP opening-password diagnostics: Poppler unavailable");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("protected.pdf");
    encrypted_pdf(&input, "secret");
    let error = engine::pdf_page_count(&input, &tools, &AtomicBool::new(false)).unwrap_err();
    assert!(format!("{error:#}").contains(PASSWORD_ERROR), "{error:#}");
    for format in [OutputFormat::Png, OutputFormat::Jpg, OutputFormat::Txt] {
        if format != OutputFormat::Txt {
            let error = engine::prepare(&input, &settings(format), &tools).unwrap_err();
            assert!(format!("{error:#}").contains(PASSWORD_ERROR), "{error:#}");
        }
        let error = engine::convert(
            &input,
            &settings(format),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap_err();
        assert!(format!("{error:#}").contains(PASSWORD_ERROR), "{error:#}");
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}
