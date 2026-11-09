use image::{Rgba, RgbaImage};
use serde::Deserialize;
use std::{
    fs,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine::PreparedConversion,
    image_edit::{
        self, CanvasAnchor, CanvasFit, CanvasSettings, ImageEditSettings, NormalizedCrop,
        ResizeSettings,
    },
    model::{ConversionSettings, ImageSettings, OutputFormat},
    queue::QueueJob,
    tools::ToolPaths,
};

fn noise(width: u32, height: u32) -> RgbaImage {
    let mut state = 0x1234_5678_u32;
    RgbaImage::from_fn(width, height, |_, _| {
        let mut channel = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        };
        Rgba([channel(), channel(), channel(), 255])
    })
}

fn decoded(encoded: &image_edit::EncodedImage) -> RgbaImage {
    let actual = image::load_from_memory(&encoded.bytes).unwrap().to_rgba8();
    assert_eq!(actual.dimensions(), (encoded.width, encoded.height));
    assert_eq!(actual, encoded.decoded_preview);
    actual
}

fn rgb_squared_error(actual: &RgbaImage, expected: &RgbaImage) -> u64 {
    assert_eq!(actual.dimensions(), expected.dimensions());
    actual
        .pixels()
        .zip(expected.pixels())
        .map(|(actual, expected)| {
            (0..3)
                .map(|channel| {
                    let delta = i32::from(actual[channel]) - i32::from(expected[channel]);
                    (delta * delta) as u64
                })
                .sum::<u64>()
        })
        .sum()
}

#[test]
fn geometry_crops_source_coordinates_resizes_and_places_pixels_on_a_canvas() {
    let cancel = AtomicBool::new(false);
    let source = RgbaImage::from_fn(4, 2, |x, y| Rgba([x as u8 * 50, y as u8 * 100, 30, 255]));
    assert_eq!(
        image_edit::render(&source, &ImageEditSettings::default(), &cancel).unwrap(),
        source
    );
    let crop = ImageEditSettings {
        crop: Some(NormalizedCrop {
            left: 0.5,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        }),
        ..Default::default()
    };
    let cropped = image_edit::render(&source, &crop, &cancel).unwrap();
    assert_eq!(cropped.dimensions(), (2, 2));
    for y in 0..2 {
        for x in 0..2 {
            assert_eq!(cropped.get_pixel(x, y), source.get_pixel(x + 2, y));
        }
    }

    let solid = RgbaImage::from_pixel(12, 6, Rgba([30, 90, 170, 255]));
    for (resize, dimensions) in [
        (
            ResizeSettings {
                width: Some(6),
                height: None,
                keep_aspect: true,
            },
            (6, 3),
        ),
        (
            ResizeSettings {
                width: Some(6),
                height: Some(5),
                keep_aspect: false,
            },
            (6, 5),
        ),
    ] {
        let resized = image_edit::render(
            &solid,
            &ImageEditSettings {
                resize: Some(resize),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
        assert_eq!(resized.dimensions(), dimensions);
        assert!(
            resized
                .pixels()
                .all(|pixel| *pixel == Rgba([30, 90, 170, 255]))
        );
    }

    // Hidden RGB from transparent pixels must not bleed into a resized edge.
    let transparent_edge = RgbaImage::from_fn(2, 1, |x, _| {
        if x == 0 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 0, 255, 0])
        }
    });
    let reduced_edge = image_edit::render(
        &transparent_edge,
        &ImageEditSettings {
            resize: Some(ResizeSettings {
                width: Some(1),
                height: Some(1),
                keep_aspect: false,
            }),
            ..Default::default()
        },
        &cancel,
    )
    .unwrap();
    let edge = reduced_edge.get_pixel(0, 0);
    assert!(edge[0] >= 250 && edge[1] <= 5 && edge[2] <= 5, "{edge:?}");
    assert!((125..=130).contains(&edge[3]), "{edge:?}");

    // A containing box can exceed the allocation limit while the actual
    // proportional result remains small; validate the result's pixel count.
    let thin = image_edit::render(
        &RgbaImage::from_pixel(100, 1, Rgba([30, 90, 170, 255])),
        &ImageEditSettings {
            resize: Some(ResizeSettings {
                width: Some(7_000),
                height: Some(7_000),
                keep_aspect: true,
            }),
            ..Default::default()
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(thin.dimensions(), (7_000, 70));
    assert_eq!(*thin.get_pixel(3_500, 35), Rgba([30, 90, 170, 255]));

    let foreground = RgbaImage::from_fn(2, 1, |x, _| {
        if x == 0 {
            Rgba([250, 20, 40, 255])
        } else {
            Rgba([20, 60, 240, 255])
        }
    });
    let placed = image_edit::render(
        &foreground,
        &ImageEditSettings {
            canvas: Some(CanvasSettings {
                width: 6,
                height: 5,
                background: [15, 25, 35, 255],
                anchor: CanvasAnchor::BottomRight,
                offset_x: -1,
                offset_y: -1,
                fit: CanvasFit::None,
            }),
            ..Default::default()
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(placed.dimensions(), (6, 5));
    for (x, y, pixel) in placed.enumerate_pixels() {
        let expected = if y == 3 && (3..5).contains(&x) {
            *foreground.get_pixel(x - 3, 0)
        } else {
            Rgba([15, 25, 35, 255])
        };
        assert_eq!(*pixel, expected, "canvas pixel {x},{y}");
    }
    for fit in [CanvasFit::Contain, CanvasFit::Cover] {
        let fitted = image_edit::render(
            &RgbaImage::from_pixel(4, 2, Rgba([20, 210, 30, 255])),
            &ImageEditSettings {
                canvas: Some(CanvasSettings {
                    width: 4,
                    height: 4,
                    background: [210, 20, 30, 255],
                    anchor: CanvasAnchor::Center,
                    offset_x: 0,
                    offset_y: 0,
                    fit,
                }),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
        assert_eq!(fitted.dimensions(), (4, 4));
        for (x, y, pixel) in fitted.enumerate_pixels() {
            let expected = if fit == CanvasFit::Contain && (y == 0 || y == 3) {
                Rgba([210, 20, 30, 255])
            } else {
                Rgba([20, 210, 30, 255])
            };
            assert_eq!(*pixel, expected, "{fit:?}, pixel {x},{y}");
        }
    }
}

#[test]
fn editor_loading_normalizes_camera_orientation_once_before_geometry() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("camera-orientation6.jpg");
    let source = image::RgbImage::from_fn(40, 20, |x, _| {
        if x < 20 {
            image::Rgb([240, 10, 10])
        } else {
            image::Rgb([10, 10, 240])
        }
    });
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 95)
        .encode_image(&source)
        .unwrap();
    // JPEG APP1 with a little-endian TIFF Orientation SHORT = 6 (90° clockwise).
    let exif: &[u8] = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
    let mut oriented = jpeg[..2].to_vec();
    oriented.extend_from_slice(&[0xff, 0xe1]);
    oriented.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
    oriented.extend_from_slice(exif);
    oriented.extend_from_slice(&jpeg[2..]);
    fs::write(&path, &oriented).unwrap();
    let cancel = AtomicBool::new(false);
    let loaded = image_edit::load_image(&path, &ToolPaths::default(), &cancel, |_| {}).unwrap();
    assert_eq!(loaded.pixels.dimensions(), (20, 40));
    let edited = image_edit::render(
        &loaded.pixels,
        &ImageEditSettings {
            crop: Some(NormalizedCrop {
                left: 0.0,
                top: 0.0,
                right: 1.0,
                bottom: 0.5,
            }),
            ..Default::default()
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(edited.dimensions(), (20, 20));
    assert!(edited.get_pixel(10, 10)[0] > 200 && edited.get_pixel(10, 10)[2] < 40);
    let bottom = loaded.pixels.get_pixel(10, 35);
    assert!(bottom[2] > 200 && bottom[0] < 40);
    assert_eq!(fs::read(path).unwrap(), oriented);
}

#[test]
fn jpeg_webp_and_png_previews_are_decoded_from_the_real_encoded_bytes() {
    let cancel = AtomicBool::new(false);
    let source = noise(256, 192);
    let settings = ImageEditSettings::default();
    for format in [OutputFormat::Jpg, OutputFormat::Webp] {
        let low = image_edit::encode(&source, format, 20, &settings, &cancel, |_| {}).unwrap();
        let high = image_edit::encode(&source, format, 95, &settings, &cancel, |_| {}).unwrap();
        assert_eq!(low.format, format);
        assert_eq!(high.format, format);
        assert!(low.target_met && high.target_met);
        assert_eq!(low.effective_quality, Some(20));
        assert_eq!(high.effective_quality, Some(95));
        assert!(high.bytes.len() > low.bytes.len(), "{}", format.label());
        let low_preview = decoded(&low);
        let high_preview = decoded(&high);
        assert!(
            rgb_squared_error(&high_preview, &source) < rgb_squared_error(&low_preview, &source),
            "{} higher quality should retain more source detail",
            format.label()
        );
    }
    let alpha = RgbaImage::from_fn(96, 48, |x, _| match x / 32 {
        0 => Rgba([230, 30, 50, 0]),
        1 => Rgba([20, 190, 40, 96]),
        _ => Rgba([30, 60, 220, 255]),
    });
    for quality in [20, 95] {
        let png = image_edit::encode(
            &alpha,
            OutputFormat::Png,
            quality,
            &settings,
            &cancel,
            |_| {},
        )
        .unwrap();
        assert_eq!(decoded(&png), alpha);
        assert_eq!(png.effective_quality, None);
    }
    let webp =
        image_edit::encode(&alpha, OutputFormat::Webp, 90, &settings, &cancel, |_| {}).unwrap();
    let webp_pixels = decoded(&webp);
    assert!(
        webp_pixels
            .pixels()
            .zip(alpha.pixels())
            .all(|(actual, expected)| actual[3] == expected[3])
    );
    let jpg =
        image_edit::encode(&alpha, OutputFormat::Jpg, 95, &settings, &cancel, |_| {}).unwrap();
    let jpg_pixels = decoded(&jpg);
    assert!(jpg_pixels.pixels().all(|pixel| pixel[3] == 255));
    assert!((0..3).all(|channel| jpg_pixels.get_pixel(8, 24)[channel] >= 250));
}

#[test]
fn requested_byte_limit_is_checked_against_complete_real_files() {
    let cancel = AtomicBool::new(false);
    let source = noise(768, 512);
    let cap = 300_000_u64;
    let settings = ImageEditSettings {
        max_bytes: Some(cap),
        allow_downsize: false,
        ..Default::default()
    };
    for format in [OutputFormat::Jpg, OutputFormat::Webp] {
        let uncapped = image_edit::encode(
            &source,
            format,
            95,
            &ImageEditSettings::default(),
            &cancel,
            |_| {},
        )
        .unwrap();
        assert!(
            uncapped.bytes.len() as u64 > cap,
            "fixture must exercise compression"
        );
        let capped = image_edit::encode(&source, format, 95, &settings, &cancel, |_| {}).unwrap();
        assert!(capped.target_met, "{}", format.label());
        assert!(capped.bytes.len() as u64 <= cap);
        assert_eq!(decoded(&capped).dimensions(), source.dimensions());
        assert!(capped.effective_quality.is_some_and(|quality| quality < 95));
    }
    let unchanged =
        image_edit::encode(&source, OutputFormat::Png, 20, &settings, &cancel, |_| {}).unwrap();
    assert!(!unchanged.target_met);
    assert!(unchanged.bytes.len() as u64 > cap);
    assert_eq!(decoded(&unchanged), source);
    let downsized = image_edit::encode(
        &source,
        OutputFormat::Png,
        95,
        &ImageEditSettings {
            allow_downsize: true,
            ..settings
        },
        &cancel,
        |_| {},
    )
    .unwrap();
    assert!(downsized.target_met);
    assert!(downsized.bytes.len() as u64 <= cap);
    let smaller = decoded(&downsized);
    assert!(smaller.width() < source.width() && smaller.height() < source.height());
    assert!((f64::from(smaller.width()) / f64::from(smaller.height()) - 1.5).abs() < 0.02);
    assert_eq!(downsized.effective_quality, None);
}

#[test]
fn cancellation_and_invalid_geometry_fail_before_allocating_or_publishing() {
    let source = RgbaImage::from_pixel(8, 6, Rgba([30, 60, 90, 255]));
    let cancel = AtomicBool::new(true);
    assert!(image_edit::render(&source, &ImageEditSettings::default(), &cancel).is_err());
    assert!(
        image_edit::encode(
            &source,
            OutputFormat::Webp,
            80,
            &ImageEditSettings::default(),
            &cancel,
            |_| {},
        )
        .is_err()
    );
    cancel.store(false, Ordering::Relaxed);
    for crop in [
        NormalizedCrop {
            left: f64::NAN,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        },
        NormalizedCrop {
            left: 0.8,
            top: 0.0,
            right: 0.2,
            bottom: 1.0,
        },
        NormalizedCrop {
            left: -0.2,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        },
    ] {
        assert!(
            image_edit::render(
                &source,
                &ImageEditSettings {
                    crop: Some(crop),
                    ..Default::default()
                },
                &cancel
            )
            .is_err()
        );
    }
    for dimensions in [(0, 10), (u32::MAX, u32::MAX), (20_000, 20_000)] {
        assert!(
            image_edit::render(
                &source,
                &ImageEditSettings {
                    resize: Some(ResizeSettings {
                        width: Some(dimensions.0),
                        height: Some(dimensions.1),
                        keep_aspect: false,
                    }),
                    ..Default::default()
                },
                &cancel,
            )
            .is_err()
        );
    }
}

#[test]
fn saves_never_clobber_source_or_existing_results_and_refuse_unmet_targets() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("originale è.png");
    let source = noise(64, 48);
    source.save(&input).unwrap();
    let original = fs::read(&input).unwrap();
    let cancel = AtomicBool::new(false);
    let encoded = image_edit::encode(
        &source,
        OutputFormat::Png,
        90,
        &ImageEditSettings::default(),
        &cancel,
        |_| {},
    )
    .unwrap();
    let existing = directory.path().join("esistente.png");
    fs::write(&existing, b"preserve existing result").unwrap();
    assert!(image_edit::save_encoded(&input, &existing, &encoded, &cancel).is_err());
    assert!(image_edit::save_encoded(&input, &input, &encoded, &cancel).is_err());
    assert_eq!(fs::read(&existing).unwrap(), b"preserve existing result");
    assert_eq!(fs::read(&input).unwrap(), original);
    let output = directory.path().join("nuovo.png");
    assert_eq!(
        image_edit::save_encoded(&input, &output, &encoded, &cancel).unwrap(),
        output
    );
    assert_eq!(fs::read(&output).unwrap(), encoded.bytes);
    assert_eq!(image::open(&output).unwrap().to_rgba8(), source);
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        assert_eq!(fs::metadata(&output).unwrap().file_attributes() & 2, 0);
    }
    let unmet = image_edit::encode(
        &source,
        OutputFormat::Png,
        90,
        &ImageEditSettings {
            max_bytes: Some(64),
            allow_downsize: false,
            ..Default::default()
        },
        &cancel,
        |_| {},
    )
    .unwrap();
    assert!(!unmet.target_met && unmet.bytes.len() > 64);
    let rejected = directory.path().join("limite-non-raggiunto.png");
    assert!(image_edit::save_encoded(&input, &rejected, &unmet, &cancel).is_err());
    assert!(!rejected.exists());
    cancel.store(true, Ordering::Relaxed);
    let cancelled = directory.path().join("annullato.png");
    assert!(image_edit::save_encoded(&input, &cancelled, &encoded, &cancel).is_err());
    assert!(!cancelled.exists());
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 3);
}

#[test]
fn batch_engine_preserves_legacy_webp_and_routes_edits_through_verified_native_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("foto-originale.png");
    let source = noise(256, 192);
    source.save(&input).unwrap();
    let original_bytes = fs::read(&input).unwrap();
    let cancel = AtomicBool::new(false);
    let tools = ToolPaths::default();
    let mut settings = ConversionSettings {
        format: OutputFormat::Webp,
        ..Default::default()
    };
    settings.image.quality = 20;
    assert!(settings.image.edit.is_none());

    // Existing batch preferences retain lossless WebP behavior, even when the
    // legacy quality field contains a value that the editor treats as lossy.
    let legacy =
        the_converter::engine::convert(&input, &settings, &tools, &cancel, |_| {}).unwrap();
    assert_eq!(legacy.len(), 1);
    let legacy_pixels = image::open(&legacy[0]).unwrap().into_rgba8();
    assert_eq!(legacy_pixels, source);

    settings.output_dir = Some(directory.path().join("modificati"));
    settings.image.edit = Some(ImageEditSettings::default());
    // A complete editor recipe supersedes the old resize setting rather than
    // applying an unrelated one-pixel resize to the prepared image.
    settings.image.width = Some(1);
    let expected = image_edit::encode(
        &source,
        OutputFormat::Webp,
        20,
        settings.image.edit.as_ref().unwrap(),
        &cancel,
        |_| {},
    )
    .unwrap();
    let edited =
        the_converter::engine::convert(&input, &settings, &tools, &cancel, |_| {}).unwrap();
    assert_eq!(edited.len(), 1);
    assert_eq!(fs::read(&edited[0]).unwrap(), expected.bytes);
    let edited_pixels = image::open(&edited[0]).unwrap().into_rgba8();
    assert_eq!(edited_pixels.dimensions(), source.dimensions());
    assert_eq!(edited_pixels, expected.decoded_preview);
    assert_ne!(edited_pixels, source);

    let capped_directory = directory.path().join("limite-impossibile");
    fs::create_dir(&capped_directory).unwrap();
    settings.output_dir = Some(capped_directory.clone());
    settings.format = OutputFormat::Png;
    settings.image.edit = Some(ImageEditSettings {
        max_bytes: Some(64),
        allow_downsize: false,
        ..Default::default()
    });
    assert!(the_converter::engine::convert(&input, &settings, &tools, &cancel, |_| {}).is_err());
    assert_eq!(fs::read_dir(capped_directory).unwrap().count(), 0);
    assert_eq!(fs::read(input).unwrap(), original_bytes);
}

#[test]
fn old_preferences_and_queued_plans_default_new_image_options_without_migration() {
    let image: ImageSettings = serde_json::from_str(r#"{"quality":74,"width":640}"#).unwrap();
    assert_eq!(image.quality, 74);
    assert_eq!(image.width, Some(640));
    assert!(image.edit.is_none() && image.page.is_none());
    let legacy_settings = serde_json::json!({
        "format": "Png",
        "image": {"quality": 74, "width": 640},
    });
    let preferences: ConversionSettings = serde_json::from_value(legacy_settings.clone()).unwrap();
    assert!(preferences.image.edit.is_none() && preferences.image.page.is_none());

    #[derive(Deserialize)]
    struct PersistedQueueShape {
        #[serde(flatten)]
        view: QueueJob,
        plan: PreparedConversion,
    }
    let queued: PersistedQueueShape = serde_json::from_value(serde_json::json!({
        "id": "a1-b2",
        "input": "foto.png",
        "format": "PNG",
        "state": "Queued",
        "error": null,
        "plan": {
            "id": "a1-b2",
            "input": "foto.png",
            "settings": legacy_settings,
            "output_dir": "risultati",
            "work_dir": "lavoro",
            "outputs": [{"path":"risultati/foto.png","marker":"prenotazione","token":"a1"}],
        },
    }))
    .unwrap();
    assert_eq!(queued.view.id, "a1-b2");
    assert_eq!(queued.plan.settings.format, OutputFormat::Png);
    assert_eq!(queued.plan.settings.image.width, Some(640));
    assert!(queued.plan.settings.image.edit.is_none() && queued.plan.settings.image.page.is_none());
    assert_eq!(queued.plan.outputs.len(), 1);
}
