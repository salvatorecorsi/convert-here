use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary,
};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine,
    model::{ConversionSettings, CsvDelimiter, OutputFormat},
    tools::ToolPaths,
};

fn settings(format: OutputFormat) -> ConversionSettings {
    ConversionSettings {
        format,
        ..Default::default()
    }
}

fn pdf_fixture(input: &Path, text: &[&str]) {
    let mut doc = Document::with_version("1.5");
    let pages = doc.new_object_id();
    let font = doc.add_object(
        dictionary! {"Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica"},
    );
    let resources = doc.add_object(dictionary! {"Font" => dictionary! {"F1" => font}});
    let mut children = Vec::new();
    for page_text in text {
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![20.into(), 50.into()]),
                Operation::new("Tj", vec![Object::string_literal(*page_text)]),
                Operation::new("ET", vec![]),
            ],
        };
        let content = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page = doc.add_object(dictionary! {"Type" => "Page", "Parent" => pages, "MediaBox" => vec![0.into(), 0.into(), 400.into(), 100.into()], "Resources" => resources, "Contents" => content});
        children.push(Object::Reference(page));
    }
    doc.objects.insert(
        pages,
        Object::Dictionary(
            dictionary! {"Type" => "Pages", "Count" => children.len() as i64, "Kids" => children},
        ),
    );
    let root = doc.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages});
    doc.trailer.set("Root", root);
    doc.save(input).unwrap();
}

fn pdf_tools() -> Option<ToolPaths> {
    let tools = ToolPaths::discover();
    if tools.pdftotext.is_none() {
        eprintln!("SKIP document PDF fixtures: pdftotext unavailable");
        return None;
    }
    Some(tools)
}

#[test]
fn pdf_reconstruction_preserves_text_pages_and_original_without_sidecars() {
    let Some(tools) = pdf_tools() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("città & mare.pdf");
    pdf_fixture(&input, &["Ciao <script> & mare", "Seconda pagina"]);
    let original = fs::read(&input).unwrap();
    let existing = dir.path().join("città & mare.html");
    fs::write(&existing, "keep existing").unwrap();
    #[cfg(windows)]
    let mut saw_hidden_stage = false;
    let html = engine::convert(
        &input,
        &settings(OutputFormat::Html),
        &tools,
        &AtomicBool::new(false),
        |_progress| {
            #[cfg(windows)]
            if _progress.message == "Estrazione del testo incorporato nel PDF…" {
                use std::os::windows::fs::MetadataExt;
                let stages: Vec<_> = fs::read_dir(dir.path())
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .filter(|path| {
                        path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with(".the-converter-external-")
                    })
                    .collect();
                assert_eq!(stages.len(), 1);
                assert_ne!(fs::metadata(&stages[0]).unwrap().file_attributes() & 2, 0);
                let files: Vec<_> = fs::read_dir(&stages[0]).unwrap().collect();
                assert_eq!(files.len(), 1);
                assert_eq!(
                    files[0]
                        .as_ref()
                        .unwrap()
                        .metadata()
                        .unwrap()
                        .file_attributes()
                        & 2,
                    0
                );
                saw_hidden_stage = true;
            }
        },
    )
    .unwrap();
    assert_ne!(html[0], existing);
    assert_eq!(fs::read_to_string(existing).unwrap(), "keep existing");
    let markup = fs::read_to_string(&html[0]).unwrap();
    assert!(markup.contains("Ciao &lt;script&gt; &amp; mare"));
    assert!(!markup.contains("<script>"));
    assert_eq!(markup.matches("<section ").count(), 2);
    let docx = engine::convert(
        &input,
        &settings(OutputFormat::Docx),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let mut archive = zip::ZipArchive::new(fs::File::open(&docx[0]).unwrap()).unwrap();
    let mut document = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut document)
        .unwrap();
    assert!(document.contains("Ciao &lt;script&gt; &amp; mare"));
    assert!(document.contains("Seconda pagina"));
    assert_eq!(document.matches("<w:pageBreakBefore/>").count(), 1);
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        assert!(saw_hidden_stage);
        for output in [&html[0], &docx[0]] {
            assert_eq!(fs::metadata(output).unwrap().file_attributes() & 2, 0);
        }
    }
    assert_eq!(fs::read(input).unwrap(), original);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 4);
}

#[test]
fn reconstructed_pdf_queue_publishes_reserved_path_and_cleans_on_cancellation() {
    let Some(tools) = pdf_tools() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("documento.pdf");
    pdf_fixture(&input, &["Testo modificabile"]);
    let plan = engine::prepare(&input, &settings(OutputFormat::Docx), &tools).unwrap();
    let output = engine::convert_prepared(&plan, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(output, vec![plan.outputs[0].path.clone()]);
    assert!(output[0].is_file());
    engine::release_prepared(&plan).unwrap();
    let cancelled = engine::prepare(&input, &settings(OutputFormat::Html), &tools).unwrap();
    let signal = AtomicBool::new(false);
    let result = engine::convert_prepared(&cancelled, &tools, &signal, |_| {
        signal.store(true, Ordering::Relaxed);
    });
    assert!(result.unwrap_err().to_string().contains("annullata"));
    assert!(!cancelled.outputs[0].path.exists());
    engine::release_prepared(&cancelled).unwrap();
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn scanned_pdf_fails_clearly_and_never_publishes_empty_reconstructions() {
    let Some(tools) = pdf_tools() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("scan.png");
    image::RgbImage::from_pixel(32, 16, image::Rgb([255, 255, 255]))
        .save(&image)
        .unwrap();
    let pdf = engine::convert(
        &image,
        &settings(OutputFormat::Pdf),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    for format in [OutputFormat::Html, OutputFormat::Docx] {
        let result = engine::convert(
            &pdf[0],
            &settings(format),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        );
        assert!(result.unwrap_err().to_string().contains("OCR"));
        assert!(
            !dir.path()
                .join(format!("scan.{}", format.extension()))
                .exists()
        );
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

fn office_tools() -> Option<ToolPaths> {
    let tools = ToolPaths::discover();
    if tools.soffice.is_none() {
        eprintln!("SKIP LibreOffice fixtures: soffice unavailable");
        return None;
    }
    Some(tools)
}

#[test]
fn csv_xlsx_roundtrip_preserves_values_and_formula_text_for_every_delimiter() {
    let Some(tools) = office_tools() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    for (index, (delimiter, separator)) in [
        (CsvDelimiter::Comma, ','),
        (CsvDelimiter::Semicolon, ';'),
        (CsvDelimiter::Tab, '\t'),
        (CsvDelimiter::Pipe, '|'),
    ]
    .into_iter()
    .enumerate()
    {
        let input = dir.path().join(format!("dati {index} è.csv"));
        let original = format!(
            "\"id\"{separator}\"note\"{separator}\"formula\"\r\n\"00123\"{separator}\"città{separator} \"\"mare\"\"\nseconda riga\"{separator}\"=1+2\"\r\n"
        );
        fs::write(&input, &original).unwrap();
        let mut conversion = settings(OutputFormat::Xlsx);
        conversion.csv.delimiter = delimiter;
        let xlsx =
            engine::convert(&input, &conversion, &tools, &AtomicBool::new(false), |_| {}).unwrap();
        assert_eq!(&fs::read(&xlsx[0]).unwrap()[..2], b"PK");
        conversion.format = OutputFormat::Csv;
        let csv = engine::convert(
            &xlsx[0],
            &conversion,
            &tools,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert_ne!(csv[0], input);
        let exported = fs::read_to_string(&csv[0]).unwrap();
        assert!(exported.contains("00123"), "{delimiter:?}: {exported}");
        assert!(
            exported.contains(&format!("città{separator} \"\"mare\"\"\nseconda riga")),
            "{delimiter:?}: {exported}"
        );
        assert!(
            exported.contains("=1+2"),
            "CSV formula was evaluated: {exported}"
        );
        assert!(
            exported.contains(&format!("\"id\"{separator}\"note\"{separator}\"formula\"")),
            "Wrong export delimiter: {exported}"
        );
        assert_eq!(fs::read_to_string(input).unwrap(), original);
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 12);
}

fn two_sheet_xlsx(path: &Path) {
    let mut archive = zip::ZipWriter::new(fs::File::create(path).unwrap());
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let parts = [
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
        ),
        (
            "xl/workbook.xml",
            r#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Prima" sheetId="1" r:id="rId1"/><sheet name="Seconda" sheetId="2" r:id="rId2"/></sheets></workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/></Relationships>"#,
        ),
        (
            "xl/worksheets/sheet1.xml",
            r#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>FIRST_SHEET</t></is></c></row></sheetData></worksheet>"#,
        ),
        (
            "xl/worksheets/sheet2.xml",
            r#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>SECOND_SHEET</t></is></c></row></sheetData></worksheet>"#,
        ),
    ];
    for (name, value) in parts {
        archive.start_file(name, options).unwrap();
        archive.write_all(value.as_bytes()).unwrap();
    }
    archive.finish().unwrap();
}

#[test]
fn spreadsheet_csv_exports_only_first_sheet_and_word_html_export_valid_pdfs() {
    let Some(tools) = office_tools() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("due fogli.xlsx");
    two_sheet_xlsx(&input);
    let output = engine::convert(
        &input,
        &settings(OutputFormat::Csv),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_eq!(output.len(), 1);
    let csv = fs::read_to_string(&output[0]).unwrap();
    assert!(csv.contains("FIRST_SHEET") && !csv.contains("SECOND_SHEET"));
    // Exercise both legacy Word's RTF sibling and HTML with a private profile.
    let rtf = dir.path().join("documento.rtf");
    fs::write(&rtf, r"{\rtf1\ansi Testo Word}").unwrap();
    let html = dir.path().join("pagina.html");
    fs::write(&html, "<!doctype html><html><meta charset=\"UTF-8\"><body><h1>Pagina HTML</h1><p>città &amp; mare</p></body></html>").unwrap();
    for source in [&rtf, &html] {
        let pdf = engine::convert(
            source,
            &settings(OutputFormat::Pdf),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        let document = Document::load(&pdf[0]).unwrap();
        assert!(!document.get_pages().is_empty());
    }
    if tools.pdftotext.is_some() {
        let source_pdf = dir.path().join("testo Word.pdf");
        pdf_fixture(&source_pdf, &["Testo da modificare in Word"]);
        let word = engine::convert(
            &source_pdf,
            &settings(OutputFormat::Docx),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        let exported = engine::convert(
            &word[0],
            &settings(OutputFormat::Pdf),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        let document = Document::load(&exported[0]).unwrap();
        assert_eq!(document.get_pages().len(), 1);
        let text = engine::convert(
            &exported[0],
            &settings(OutputFormat::Txt),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(
            fs::read_to_string(&text[0])
                .unwrap()
                .contains("Testo da modificare in Word")
        );
    }
    let outputs: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert!(outputs.iter().all(|path| path.is_file()));
}
