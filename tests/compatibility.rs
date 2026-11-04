use std::{path::Path, sync::atomic::AtomicBool};
use the_converter::{
    engine,
    model::{ConversionSettings, MediaKind, OutputFormat, common_formats, supported_formats},
    tools::ToolPaths,
};

#[test]
fn selection_offers_only_shared_routes_and_opt_in_transcription() {
    use OutputFormat::*;
    let cases: &[(&[&str], bool, &[OutputFormat])] = &[
        (&[], true, &[]),
        (&["note.txt"], true, &[Pdf]),
        (&["note.txt", "photo.png"], true, &[Pdf]),
        (&["note.txt", "clip.mp4"], true, &[]),
        (
            &["song.WAV", "clip.mp4"],
            true,
            &[Mp3, Wav, Flac, Aac, Ogg, Opus, Aiff, M4a, Wma, Ac3, Au, Caf],
        ),
        (&["song.wav", "report.pdf"], true, &[Txt]),
        (&["song.wav", "report.pdf"], false, &[]),
        (&["report.pdf"], false, &[Png, Jpg, Txt, Docx, Html, Pdf]),
        (&["report.xlsx", "data.csv"], false, &[Pdf, Csv, Xlsx]),
        (&["report.xlsx", "notes.docx", "page.html"], false, &[Pdf]),
        (&["page.HTM", "photo.qoi"], false, &[Pdf]),
        (&["notes.docx"], false, &[Pdf]),
        (&["photo.png", "report.pdf"], true, &[Png, Jpg, Pdf, Txt]),
        (
            &["photo.heic", "photo.AVIF"],
            false,
            &[
                Png, Jpg, Webp, Bmp, Tiff, Gif, Ico, Pdf, Qoi, Tga, Farbfeld, OpenExr, Hdr, Pnm,
                Avif, Heic, Txt,
            ],
        ),
        (&["song.wav", "photo.heic"], true, &[Txt]),
        (&["song.wav", "photo.heic"], false, &[]),
        (&["song.wav", "file.xyz"], true, &[]),
    ];
    for (inputs, enabled, expected) in cases {
        assert_eq!(
            common_formats(inputs.iter().map(Path::new), *enabled),
            *expected,
            "{inputs:?}"
        );
    }
    assert!(!common_formats([Path::new("song.wav")], false).contains(&Txt));
    assert!(common_formats([Path::new("song.wav")], true).contains(&Txt));
}

#[test]
fn unsupported_routes_are_rejected_before_tools_or_output_creation() {
    let folder = tempfile::tempdir().unwrap();
    let inputs = [
        "recording.wav",
        "clip.mp4",
        "photo.png",
        "report.pdf",
        "note.txt",
        "data.csv",
        "book.xlsx",
        "page.html",
        "file.xyz",
    ];
    for name in inputs {
        let input = folder.path().join(name);
        std::fs::write(&input, b"fixture").unwrap();
        let kind = MediaKind::from_path(&input);
        for format in OutputFormat::ALL {
            if supported_formats(kind).contains(&format) {
                continue;
            }
            let output = folder.path().join("must-not-be-created");
            let settings = ConversionSettings {
                format,
                output_dir: Some(output.clone()),
                ..Default::default()
            };
            let result = engine::convert(
                &input,
                &settings,
                &ToolPaths::default(),
                &AtomicBool::new(false),
                |_| {},
            );
            let error = result.unwrap_err().to_string();
            assert!(
                error.contains("Conversione non supportata"),
                "{name} -> {format}: {error}"
            );
            assert!(
                !output.exists(),
                "{name} -> {format} created an output directory"
            );
            let queued = engine::prepare(&input, &settings, &ToolPaths::default());
            assert!(queued.is_err(), "queued {name} -> {format}");
            assert!(!output.exists());
        }
    }
}

#[test]
fn advertised_formats_have_unique_extensions_and_persist_without_loss() {
    let mut extensions = std::collections::HashSet::new();
    for format in OutputFormat::ALL {
        assert!(extensions.insert(format.extension()), "duplicate {format}");
        let json = serde_json::to_string(&ConversionSettings {
            format,
            ..Default::default()
        })
        .unwrap();
        let loaded: ConversionSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.format, format);
        assert_ne!(
            MediaKind::from_path(Path::new(&format!("file.{}", format.extension()))),
            MediaKind::Unknown
        );
    }
}

#[test]
fn stale_ocr_language_only_blocks_routes_that_use_ocr() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("photo.png");
    image::RgbImage::from_pixel(8, 8, image::Rgb([255, 255, 255]))
        .save(&input)
        .unwrap();
    let mut settings = ConversionSettings {
        format: OutputFormat::Png,
        ..Default::default()
    };
    settings.ocr.language.clear();
    assert!(
        engine::convert(
            &input,
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {}
        )
        .is_ok()
    );
    settings.format = OutputFormat::Txt;
    settings.output_dir = Some(dir.path().join("not-created"));
    let error = engine::prepare(&input, &settings, &ToolPaths::default()).unwrap_err();
    assert!(error.to_string().contains("Lingua OCR"));
    assert!(!settings.output_dir.unwrap().exists());
}

#[test]
fn old_settings_load_without_a_key_or_transcription_configuration() {
    let settings: ConversionSettings =
        serde_json::from_str(r#"{"format":"Txt","audio":{"bitrate_kbps":192}}"#).unwrap();
    assert_eq!(settings.transcription.language, None);
    assert_eq!(
        settings.pdf.action,
        the_converter::model::PdfAction::Compress
    );
    assert_eq!(
        settings.csv.delimiter,
        the_converter::model::CsvDelimiter::Comma
    );
    assert!(!settings.ocr.enabled);
    assert_eq!(settings.ocr.language, "ita+eng");
    let json = serde_json::to_string(&settings).unwrap();
    assert!(!json.contains("api_key"));
    let saved_with_language: ConversionSettings =
        serde_json::from_str(r#"{"format":"Txt","transcription":{"language":"it"}}"#).unwrap();
    assert_eq!(
        saved_with_language.transcription.language.as_deref(),
        Some("it")
    );
}
