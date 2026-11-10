use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine::convert,
    model::{ConversionSettings, OutputFormat},
    tools::ToolPaths,
};

fn settings(format: OutputFormat) -> ConversionSettings {
    ConversionSettings {
        format,
        ..Default::default()
    }
}

fn tools() -> Option<ToolPaths> {
    let mut tools = ToolPaths::discover();
    // The validation-only portable engines are outside the release artifacts.
    // A normal installation instead uses automatic or explicitly selected tools.
    let validation =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".tools/libheif-validation/runtime/ucrt64/bin");
    for (slot, name) in [
        (&mut tools.heif_enc, "heif-enc"),
        (&mut tools.heif_dec, "heif-dec"),
    ] {
        let path = validation.join(if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_owned()
        });
        if slot.is_none() && path.is_file() {
            *slot = Some(path);
        }
    }
    if tools.heif_enc.is_none() || tools.heif_dec.is_none() {
        eprintln!(
            "libheif heif-enc/heif-dec unavailable; modern-image integration fixture skipped"
        );
        return None;
    }
    Some(tools)
}

fn command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}

fn convert_one(input: &Path, conversion: &ConversionSettings, tools: &ToolPaths) -> PathBuf {
    let output = convert(input, conversion, tools, &AtomicBool::new(false), |_| {}).unwrap_or_else(
        |error| {
            panic!(
                "{} -> {}: {error:#}",
                input.display(),
                conversion.format.label()
            )
        },
    );
    assert_eq!(output.len(), 1);
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        assert_eq!(fs::metadata(&output[0]).unwrap().file_attributes() & 2, 0);
    }
    output.into_iter().next().unwrap()
}

fn image_fixture(path: &Path) {
    image::RgbaImage::from_fn(80, 40, |x, _| {
        if x < 40 {
            image::Rgba([220, 30, 50, 255])
        } else {
            image::Rgba([30, 60, 200, 96])
        }
    })
    .save(path)
    .unwrap();
}

#[test]
fn modern_images_explain_missing_engines_before_creating_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let native = dir.path().join("source.png");
    image_fixture(&native);
    let modern = dir.path().join("source.heic");
    fs::write(&modern, b"validation must precede decoding").unwrap();
    for (input, target) in [
        (&native, OutputFormat::Heic),
        (&native, OutputFormat::Avif),
        (&modern, OutputFormat::Png),
    ] {
        let mut conversion = settings(target);
        conversion.output_dir = Some(dir.path().join("must-not-exist"));
        let error = convert(
            input,
            &conversion,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("libheif") && error.contains("Installa"),
            "{error}"
        );
        assert!(!conversion.output_dir.unwrap().exists());
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn real_heic_and_avif_roundtrip_keep_size_colour_alpha_and_originals() {
    let Some(tools) = tools() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("foto è speciale (1).png");
    image_fixture(&input);
    let original = fs::read(&input).unwrap();
    for format in [OutputFormat::Heic, OutputFormat::Avif] {
        let mut conversion = settings(format);
        conversion.image.quality = 95;
        let output = convert_one(&input, &conversion, &tools);
        let bytes = fs::read(&output).unwrap();
        assert!(bytes.len() > 24);
        assert_eq!(&bytes[4..8], b"ftyp", "must be a real HEIF container");
        let compatible = if format == OutputFormat::Avif {
            b"avif"
        } else {
            b"heic"
        };
        assert!(
            bytes[..bytes.len().min(128)]
                .windows(4)
                .any(|window| window == compatible)
        );
        let restored = convert_one(&output, &settings(OutputFormat::Png), &tools);
        assert_ne!(restored, input);
        let image = image::open(restored).unwrap().to_rgba8();
        assert_eq!(image.dimensions(), (80, 40));
        for (x, expected) in [(20, [220u8, 30, 50, 255]), (60, [30u8, 60, 200, 96])] {
            let actual = image.get_pixel(x, 20).0;
            for (got, expected) in actual.into_iter().zip(expected) {
                assert!(
                    got.abs_diff(expected) <= 16,
                    "{}: {actual:?} vs {expected}",
                    format.label()
                );
            }
        }
        let mut resized = settings(OutputFormat::Webp);
        resized.image.width = Some(40);
        let output = convert_one(&output, &resized, &tools);
        assert_eq!(image::open(output).unwrap().width(), 40);
    }
    assert_eq!(fs::read(&input).unwrap(), original);
    assert!(!fs::read_dir(dir.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".the-converter-")
    }));
}

#[test]
fn heif_container_rotation_is_applied_once_and_heif_alias_decodes() {
    let Some(tools) = tools() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("orientation.png");
    image_fixture(&source);
    let rotated = dir.path().join("rotated.heif");
    let fixture = command(tools.heif_enc.as_ref().unwrap())
        .args(["--rotate-cw", "90", "-q", "95"])
        .arg(&source)
        .arg("-o")
        .arg(&rotated)
        .output()
        .unwrap();
    assert!(
        fixture.status.success(),
        "{}",
        String::from_utf8_lossy(&fixture.stderr)
    );
    let output = convert_one(&rotated, &settings(OutputFormat::Png), &tools);
    let image = image::open(output).unwrap().to_rgba8();
    assert_eq!(image.dimensions(), (40, 80));
    let top = image.get_pixel(20, 20);
    let bottom = image.get_pixel(20, 60);
    assert!(top[0] > 180 && top[2] < 90, "{top:?}");
    assert!(bottom[2] > 160 && bottom[0] < 90, "{bottom:?}");
}

#[test]
fn broken_or_cancelled_modern_decode_removes_temporary_files() {
    let Some(tools) = tools() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let broken = dir.path().join("broken.avif");
    fs::write(&broken, b"not an AVIF container").unwrap();
    assert!(
        convert(
            &broken,
            &settings(OutputFormat::Png),
            &tools,
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    let input = dir.path().join("source.png");
    image_fixture(&input);
    let modern = convert_one(&input, &settings(OutputFormat::Heic), &tools);
    let cancel = AtomicBool::new(false);
    let count = fs::read_dir(dir.path()).unwrap().count();
    let result = convert(
        &modern,
        &settings(OutputFormat::Png),
        &tools,
        &cancel,
        |progress| {
            if progress.message.starts_with("Lettura HEIC/AVIF") {
                cancel.store(true, Ordering::Relaxed);
            }
        },
    );
    assert!(result.unwrap_err().to_string().contains("annullata"));
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), count);
}
