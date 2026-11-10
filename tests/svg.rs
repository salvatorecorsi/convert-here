use image::{GenericImageView, Rgba};
use std::{fs, sync::atomic::AtomicBool};
use the_converter::{
    engine, image_edit,
    model::{ConversionSettings, OutputFormat},
    tools::ToolPaths,
};

const SOURCE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32" viewBox="0 0 64 32"><path fill="#0078d4" d="M8 4h48v24H8z"/><circle cx="32" cy="16" r="8" fill="white"/></svg>"##;

#[test]
fn native_svg_outputs_and_pdf_need_no_external_engine() {
    use OutputFormat::*;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("vector.svg");
    fs::write(&input, SOURCE).unwrap();
    let original = fs::read(&input).unwrap();
    for format in [
        Png, Jpg, Webp, Bmp, Tiff, Gif, Ico, Qoi, Tga, Farbfeld, OpenExr, Hdr, Pnm, Pdf,
    ] {
        let settings = ConversionSettings {
            format,
            ..Default::default()
        };
        let outputs = engine::convert(
            &input,
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap_or_else(|error| panic!("{format:?}: {error:#}"));
        assert_eq!(outputs.len(), 1);
        if format == Pdf {
            let document = lopdf::Document::load(&outputs[0]).unwrap();
            assert_eq!(document.get_pages().len(), 1);
        } else {
            assert_eq!(
                image::open(&outputs[0]).unwrap().dimensions(),
                (64, 32),
                "{format:?}"
            );
        }
    }
    assert_eq!(fs::read(&input).unwrap(), original);
}

#[test]
fn svg_editor_and_legacy_width_use_the_same_renderer() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.SVG");
    fs::write(&input, SOURCE).unwrap();
    let cancel = AtomicBool::new(false);
    let loaded = image_edit::load_image(&input, &ToolPaths::default(), &cancel, |_| {}).unwrap();
    assert_eq!(loaded.pixels.dimensions(), (64, 32));
    assert_eq!(loaded.pixels.get_pixel(32, 16), &Rgba([255, 255, 255, 255]));
    let mut settings = ConversionSettings {
        format: OutputFormat::Png,
        ..Default::default()
    };
    settings.image.width = Some(128);
    let output =
        engine::convert(&input, &settings, &ToolPaths::default(), &cancel, |_| {}).unwrap();
    assert_eq!(image::open(&output[0]).unwrap().dimensions(), (128, 64));
}

#[test]
fn svg_size_failure_and_cancellation_publish_nothing_and_preserve_existing_files() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.svg");
    let existing = dir.path().join("source.png");
    fs::write(&input, SOURCE).unwrap();
    fs::write(&existing, b"keep unrelated output").unwrap();
    let mut settings = ConversionSettings {
        format: OutputFormat::Png,
        ..Default::default()
    };
    settings.limits.max_bytes = Some(1);
    assert!(
        engine::convert(
            &input,
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    settings.limits.max_bytes = None;
    assert!(
        engine::convert(
            &input,
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(true),
            |_| {}
        )
        .is_err()
    );
    assert_eq!(fs::read(&input).unwrap(), SOURCE.as_bytes());
    assert_eq!(fs::read(&existing).unwrap(), b"keep unrelated output");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn malformed_or_external_svg_has_no_published_result() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("bad.svg");
    for contents in [
        b"not an SVG".as_slice(),
        b"<svg width='16' height='16'><image href='private.png'/></svg>".as_slice(),
        &[0xff, 0xfe, 0x00],
    ] {
        fs::write(&input, contents).unwrap();
        assert!(
            engine::convert(
                &input,
                &ConversionSettings {
                    format: OutputFormat::Png,
                    ..Default::default()
                },
                &ToolPaths::default(),
                &AtomicBool::new(false),
                |_| {}
            )
            .is_err()
        );
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        assert_eq!(fs::read(&input).unwrap(), contents);
    }
}
