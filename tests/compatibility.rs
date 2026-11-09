use std::{path::Path, sync::atomic::AtomicBool};
use the_converter::{
    engine,
    model::{ConversionSettings, MediaKind, OutputFormat, common_formats, supported_formats},
    tools::ToolPaths,
};

#[test]
fn selection_offers_only_shared_local_routes() {
    use OutputFormat::*;
    let cases: &[(&[&str], &[OutputFormat])] = &[
        (&[], &[]),
        (&["note.txt"], &[Pdf]),
        (&["note.txt", "photo.png"], &[Pdf]),
        (&["note.txt", "clip.mp4"], &[]),
        (
            &["song.WAV", "clip.mp4"],
            &[Mp3, Wav, Flac, Aac, Ogg, Opus, Aiff, M4a, Wma, Ac3, Au, Caf],
        ),
        (&["song.wav", "report.pdf"], &[]),
        (&["report.pdf"], &[Png, Jpg, Txt, Docx, Html, Pdf]),
        (&["report.xlsx", "data.csv"], &[Pdf, Csv, Xlsx]),
        (&["report.xlsx", "notes.docx", "page.html"], &[Pdf]),
        (&["page.HTM", "photo.qoi"], &[Pdf]),
        (&["notes.docx"], &[Pdf]),
        (&["photo.png", "report.pdf"], &[Png, Jpg, Pdf, Txt]),
        (
            &["photo.heic", "photo.AVIF"],
            &[
                Png, Jpg, Webp, Bmp, Tiff, Gif, Ico, Pdf, Qoi, Tga, Farbfeld, OpenExr, Hdr, Pnm,
                Avif, Heic, Txt,
            ],
        ),
        (&["song.wav", "photo.heic"], &[]),
        (&["song.wav", "file.xyz"], &[]),
    ];
    for (inputs, expected) in cases {
        assert_eq!(
            common_formats(inputs.iter().map(Path::new)),
            *expected,
            "{inputs:?}"
        );
    }
    assert_eq!(
        supported_formats(MediaKind::Audio),
        vec![Mp3, Wav, Flac, Aac, Ogg, Opus, Aiff, M4a, Wma, Ac3, Au, Caf]
    );
    assert!(!common_formats([Path::new("song.wav")]).contains(&Txt));
    assert!(supported_formats(MediaKind::Image).contains(&Txt));
    assert!(supported_formats(MediaKind::Pdf).contains(&Txt));
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
fn old_settings_load_and_obsolete_fields_are_not_written_back() {
    let settings: ConversionSettings =
        serde_json::from_str(r#"{"format":"Txt","audio":{"bitrate_kbps":192}}"#).unwrap();
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
    assert_eq!(saved_with_language.format, OutputFormat::Txt);
    assert!(
        serde_json::to_value(&saved_with_language)
            .unwrap()
            .get("transcription")
            .is_none()
    );
}

#[test]
fn obsolete_audio_text_settings_are_rejected_before_output_or_progress() {
    let directory = tempfile::tempdir().unwrap();
    let audio = directory.path().join("recording.wav");
    let original = b"audio input bytes must not be changed";
    std::fs::write(&audio, original).unwrap();
    let mut settings: ConversionSettings = serde_json::from_str(
        r#"{"format":"Txt","transcription":{"language":"obsolete-language"}}"#,
    )
    .unwrap();
    let output = directory.path().join("not-created");
    settings.output_dir = Some(output.clone());
    let mut progressed = false;
    let error = engine::convert(
        &audio,
        &settings,
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| progressed = true,
    )
    .unwrap_err();
    assert!(error.to_string().contains("Conversione non supportata"));
    assert!(!progressed && !output.exists());
    assert!(
        engine::prepare(&audio, &settings, &ToolPaths::default())
            .unwrap_err()
            .to_string()
            .contains("Conversione non supportata")
    );
    assert!(!output.exists());
    assert_eq!(std::fs::read(audio).unwrap(), original);
}

#[test]
fn cli_rejects_obsolete_audio_text_settings_without_creating_outputs() {
    let directory = tempfile::tempdir().unwrap();
    let audio = directory.path().join("recording.wav");
    let settings = directory.path().join("old-settings.json");
    let output = directory.path().join("not-created");
    std::fs::write(&audio, b"original audio").unwrap();
    std::fs::write(
        &settings,
        br#"{"format":"Txt","transcription":{"language":"it"}}"#,
    )
    .unwrap();
    let config = directory.path().join("config");
    std::fs::create_dir(&config).unwrap();
    let preferences = br#"{"ui_language":"system","settings":{"format":"Flac"}}"#;
    std::fs::write(config.join("preferences.json"), preferences).unwrap();
    for (language, expected) in [
        ("it", "Conversione non supportata"),
        ("en", "Unsupported conversion"),
    ] {
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_the-converter-cli"))
            .env("THE_CONVERTER_CONFIG_DIR", &config)
            .args(["--language", language, "--convert", "--settings"])
            .arg(&settings)
            .arg("--output-dir")
            .arg(&output)
            .arg("--files")
            .arg(&audio)
            .output()
            .unwrap();
        assert!(!result.status.success());
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        assert!(diagnostic.contains(expected), "{language}: {diagnostic}");
        assert!(!output.exists());
        assert_eq!(std::fs::read(&audio).unwrap(), b"original audio");
        assert_eq!(
            std::fs::read(config.join("preferences.json")).unwrap(),
            preferences
        );
    }
}
