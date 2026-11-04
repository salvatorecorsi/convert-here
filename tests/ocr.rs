use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary,
};
use std::{
    fs,
    io::Read,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine,
    model::{ConversionSettings, OutputFormat},
    tools::ToolPaths,
};

const PAGE_ONE: &[u8] = include_bytes!("fixtures/ocr.png");
const PAGE_TWO: &[u8] = include_bytes!("fixtures/ocr-page-two.png");

fn settings(format: OutputFormat) -> ConversionSettings {
    let mut settings = ConversionSettings {
        format,
        ..Default::default()
    };
    settings.ocr.language = "eng".into();
    settings
}

fn tools(pdf: bool) -> Option<ToolPaths> {
    let tools = ToolPaths::discover();
    if tools.tesseract.is_none() || pdf && tools.pdftoppm.is_none() {
        eprintln!(
            "SKIP OCR fixture: local Tesseract{} unavailable",
            if pdf { "/Poppler" } else { "" }
        );
        return None;
    }
    Some(tools)
}

fn scanned_pdf(path: &Path) {
    let mut document = Document::with_version("1.5");
    let pages = document.new_object_id();
    let mut children = Vec::new();
    for image_bytes in [PAGE_ONE, PAGE_TWO] {
        let image = image::load_from_memory(image_bytes).unwrap().to_rgb8();
        let object = document.add_object(Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image", "Width" => image.width(),
                "Height" => image.height(), "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8,
            },
            image.into_raw(),
        ));
        let resources =
            document.add_object(dictionary! {"XObject" => dictionary! {"Image" => object}});
        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![
                        750.into(),
                        0.into(),
                        0.into(),
                        250.into(),
                        0.into(),
                        0.into(),
                    ],
                ),
                Operation::new("Do", vec![Object::Name(b"Image".to_vec())]),
                Operation::new("Q", vec![]),
            ],
        };
        let contents = document.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page = document.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages, "MediaBox" => vec![0.into(),0.into(),750.into(),250.into()],
            "Resources" => resources, "Contents" => contents,
        });
        children.push(Object::Reference(page));
    }
    document.objects.insert(
        pages,
        Object::Dictionary(dictionary! {"Type" => "Pages", "Count" => 2, "Kids" => children}),
    );
    let catalog = document.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages});
    document.trailer.set("Root", catalog);
    document.compress();
    document.save(path).unwrap();
}

#[test]
fn image_to_txt_recognizes_text_locally_and_never_overwrites() {
    let Some(tools) = tools(false) else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("scansione è.png");
    fs::write(&input, PAGE_ONE).unwrap();
    let existing = dir.path().join("scansione è.txt");
    fs::write(&existing, "preserve existing").unwrap();
    let output = engine::convert(
        &input,
        &settings(OutputFormat::Txt),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_ne!(output[0], existing);
    let text = fs::read_to_string(&output[0]).unwrap();
    assert!(
        text.contains("HELLO OCR WORLD") && text.contains("LOCAL TEXT 12345"),
        "{text}"
    );
    assert_eq!(fs::read_to_string(existing).unwrap(), "preserve existing");
    assert_eq!(fs::read(input).unwrap(), PAGE_ONE);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 3);
}

#[test]
fn scanned_pdf_ocr_reconstructs_every_page_as_txt_docx_and_html() {
    let Some(tools) = tools(true) else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("scansione.pdf");
    scanned_pdf(&input);
    let original = fs::read(&input).unwrap();
    for format in [OutputFormat::Txt, OutputFormat::Docx, OutputFormat::Html] {
        let mut conversion = settings(format);
        conversion.ocr.enabled = true;
        let output =
            engine::convert(&input, &conversion, &tools, &AtomicBool::new(false), |_| {}).unwrap();
        let text = if format == OutputFormat::Docx {
            let mut archive = zip::ZipArchive::new(fs::File::open(&output[0]).unwrap()).unwrap();
            let mut document = String::new();
            archive
                .by_name("word/document.xml")
                .unwrap()
                .read_to_string(&mut document)
                .unwrap();
            assert_eq!(document.matches("<w:pageBreakBefore/>").count(), 1);
            document
        } else {
            let text = fs::read_to_string(&output[0]).unwrap();
            if format == OutputFormat::Html {
                assert_eq!(text.matches("<section ").count(), 2);
            }
            if format == OutputFormat::Txt {
                assert_eq!(text.matches('\u{c}').count(), 2);
            }
            text
        };
        let first = text
            .find("HELLO OCR WORLD")
            .unwrap_or_else(|| panic!("{format:?}: {text}"));
        let second = text
            .find("SECOND OCR PAGE")
            .unwrap_or_else(|| panic!("{format:?}: {text}"));
        assert!(first < second);
        assert!(text.contains("DOCUMENT ORDER 67890"));
    }
    assert_eq!(fs::read(input).unwrap(), original);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 4);
}

#[test]
fn cancelling_ocr_on_the_second_page_cleans_partial_text_and_queue_reservations() {
    let Some(tools) = tools(true) else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("annullata.pdf");
    scanned_pdf(&input);
    let mut conversion = settings(OutputFormat::Txt);
    conversion.ocr.enabled = true;
    let plan = engine::prepare(&input, &conversion, &tools).unwrap();
    let cancel = AtomicBool::new(false);
    let result = engine::convert_prepared(&plan, &tools, &cancel, |progress| {
        if progress.message.contains("pagina 2") {
            cancel.store(true, Ordering::Relaxed);
        }
    });
    assert!(result.unwrap_err().to_string().contains("annullata"));
    assert!(!plan.outputs[0].path.exists());
    engine::release_prepared(&plan).unwrap();
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn missing_ocr_engine_language_and_unrecognized_images_do_not_publish_empty_text() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("test.png");
    fs::write(&input, PAGE_ONE).unwrap();
    let result = engine::convert(
        &input,
        &settings(OutputFormat::Txt),
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    );
    assert!(result.unwrap_err().to_string().contains("Tesseract"));
    if let Some(tools) = tools(false) {
        let mut conversion = settings(OutputFormat::Txt);
        conversion.ocr.language = "model_that_does_not_exist".into();
        let result = engine::convert(&input, &conversion, &tools, &AtomicBool::new(false), |_| {});
        let error = format!("{:#}", result.unwrap_err());
        assert!(
            error.contains("modello") || error.contains("Modello") || error.contains("modelli"),
            "{error}"
        );
        let blank = dir.path().join("vuota.png");
        image::RgbImage::from_pixel(300, 100, image::Rgb([255, 255, 255]))
            .save(&blank)
            .unwrap();
        let result = engine::convert(
            &blank,
            &settings(OutputFormat::Txt),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("non ha riconosciuto testo")
        );
        assert!(!dir.path().join("vuota.txt").exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
    assert!(!dir.path().join("test.txt").exists());
}
