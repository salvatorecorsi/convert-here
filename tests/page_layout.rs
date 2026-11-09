use image::{Rgba, RgbaImage};
use lopdf::{Document, Object};
use std::{
    fs,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use the_converter::{
    engine,
    model::{ConversionSettings, OutputFormat},
    page_layout::{
        self, ImageFit, PageInput, PageLayoutSettings, PageOrientation, PaperSize, TextAlign,
    },
    tools::ToolPaths,
};

fn prepare(
    inputs: &[PageInput],
    settings: &PageLayoutSettings,
) -> page_layout::PreparedPageDocument {
    page_layout::prepare(inputs, settings, 90, &AtomicBool::new(false), |_| {}).unwrap()
}

fn text(value: &str) -> PageInput {
    PageInput::Text(Arc::from(value))
}

fn media_box(document: &Document, page: u32) -> Vec<f32> {
    let page = document
        .get_object(document.get_pages()[&page])
        .unwrap()
        .as_dict()
        .unwrap();
    page.get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|number| match number {
            Object::Integer(value) => *value as f32,
            Object::Real(value) => *value,
            _ => panic!("not a number"),
        })
        .collect()
}

#[test]
fn native_paper_sizes_and_orientation_have_physical_media_boxes() {
    for (paper, orientation, expected) in [
        (PaperSize::A4, PageOrientation::Portrait, (210.0, 297.0)),
        (PaperSize::A4, PageOrientation::Landscape, (297.0, 210.0)),
        (PaperSize::A5, PageOrientation::Portrait, (148.0, 210.0)),
        (PaperSize::A5, PageOrientation::Landscape, (210.0, 148.0)),
    ] {
        let settings = PageLayoutSettings {
            paper,
            orientation,
            ..Default::default()
        };
        let prepared = prepare(&[text("")], &settings);
        let document = Document::load_mem(prepared.bytes()).unwrap();
        let dimensions = media_box(&document, 1);
        assert!((dimensions[2] / 72.0 * 25.4 - expected.0).abs() < 0.01);
        assert!((dimensions[3] / 72.0 * 25.4 - expected.1).abs() < 0.01);
        let preview = prepared.preview(0, 400, &AtomicBool::new(false)).unwrap();
        assert_eq!(preview.width().max(preview.height()), 400);
        assert!(preview.pixels().all(|pixel| pixel.0 == [255; 4]));
    }
}

#[test]
fn unicode_text_is_embedded_searchable_and_normalized_without_losing_words() {
    let source = "Cafe\u{301} già € – Ελληνικά\nSeconda riga con parole separate.";
    let prepared = prepare(&[text(source)], &PageLayoutSettings::default());
    let document = Document::load_mem(prepared.bytes()).unwrap();
    let extracted = document.extract_text(&[1]).unwrap();
    assert!(extracted.contains("Café già € – Ελληνικά"), "{extracted:?}");
    assert!(
        extracted.contains("Seconda riga con parole separate."),
        "{extracted:?}"
    );
    assert!(document.objects.values().any(|object| {
        object
            .as_dict()
            .ok()
            .and_then(|dictionary| dictionary.get(b"Subtype").ok())
            .and_then(|subtype| subtype.as_name().ok())
            == Some(b"CIDFontType2")
    }));
    assert!(document.objects.values().any(|object| {
        object
            .as_dict()
            .ok()
            .is_some_and(|dictionary| dictionary.has(b"FontFile2"))
    }));
}

#[test]
fn wrapping_and_pagination_preserve_words_and_explicit_page_breaks() {
    let source = "Parole distinte restano separate anche quando la riga finisce. ".repeat(110);
    let settings = PageLayoutSettings {
        paper: PaperSize::A5,
        font_size_pt: 18.0,
        align: TextAlign::Right,
        ..Default::default()
    };
    let prepared = prepare(
        &[text(&source), text("Prima pagina\u{c}Seconda pagina")],
        &settings,
    );
    assert!(prepared.page_count() > 3);
    let document = Document::load_mem(prepared.bytes()).unwrap();
    let pages: Vec<_> = (1..=prepared.page_count() as u32).collect();
    let extracted = document.extract_text(&pages).unwrap();
    let actual: Vec<_> = extracted.split_whitespace().collect();
    let mut expected: Vec<_> = source.split_whitespace().collect();
    expected.extend(["Prima", "pagina", "Seconda", "pagina"]);
    assert_eq!(actual, expected);
}

#[test]
fn contain_and_cover_previews_follow_embedded_jpeg_and_white_margins() {
    let source = RgbaImage::from_fn(300, 60, |x, _| {
        if x < 100 {
            Rgba([240, 0, 0, 255])
        } else if x < 200 {
            Rgba([0, 200, 0, 255])
        } else {
            Rgba([0, 0, 240, 255])
        }
    });
    let input = PageInput::Image(Arc::new(source));
    let contain = prepare(std::slice::from_ref(&input), &PageLayoutSettings::default());
    let cover = prepare(
        &[input],
        &PageLayoutSettings {
            fit: ImageFit::Cover,
            ..Default::default()
        },
    );
    let cancel = AtomicBool::new(false);
    let contain_preview = contain.preview(0, 400, &cancel).unwrap();
    let cover_preview = cover.preview(0, 400, &cancel).unwrap();
    assert_eq!(contain_preview.get_pixel(0, 0).0, [255; 4]);
    assert_eq!(cover_preview.get_pixel(0, 0).0, [255; 4]);
    let x = cover_preview.width() / 2;
    assert_eq!(contain_preview.get_pixel(x, 24).0, [255; 4]);
    let pixel = cover_preview.get_pixel(x, 24);
    assert!(
        pixel[1] > 150 && pixel[0] < 30 && pixel[2] < 30,
        "{pixel:?}"
    );
}

#[test]
fn pdf_downsample_ignores_hidden_png_rgb_and_matches_white_matte() {
    let temporary = tempfile::tempdir().unwrap();
    let transparent = RgbaImage::from_fn(1600, 800, |x, _| {
        if x < 740 {
            Rgba([255, 0, 0, 0])
        } else if x < 900 {
            Rgba([10, 80, 200, 128])
        } else {
            Rgba([0, 40, 160, 255])
        }
    });
    let matte = RgbaImage::from_fn(1600, 800, |x, y| {
        let pixel = transparent.get_pixel(x, y);
        let alpha = u32::from(pixel[3]);
        let rgb = [0, 1, 2]
            .map(|c| ((u32::from(pixel[c]) * alpha + 255 * (255 - alpha) + 127) / 255) as u8);
        Rgba([rgb[0], rgb[1], rgb[2], 255])
    });
    let transparent_path = temporary.path().join("hidden-red.png");
    let matte_path = temporary.path().join("white-matte.png");
    transparent.save(&transparent_path).unwrap();
    matte.save(&matte_path).unwrap();
    let settings = PageLayoutSettings {
        dpi: 72,
        ..Default::default()
    };
    let actual = prepare(
        &[PageInput::Image(Arc::new(
            image::open(transparent_path).unwrap().to_rgba8(),
        ))],
        &settings,
    );
    let expected = prepare(
        &[PageInput::Image(Arc::new(
            image::open(matte_path).unwrap().to_rgba8(),
        ))],
        &settings,
    );
    let pdf = Document::load_mem(actual.bytes()).unwrap();
    let image_width = pdf
        .objects
        .values()
        .filter_map(|object| object.as_stream().ok())
        .find_map(|stream| {
            if stream
                .dict
                .get(b"Subtype")
                .ok()
                .and_then(|value| value.as_name().ok())
                == Some(b"Image")
            {
                stream
                    .dict
                    .get(b"Width")
                    .ok()
                    .and_then(|width| width.as_i64().ok())
            } else {
                None
            }
        })
        .unwrap();
    assert!(
        image_width < 1600,
        "The fixture must exercise DPI downsampling"
    );
    assert!(
        actual.bytes() == expected.bytes(),
        "Hidden RGB/alpha contaminated the PDF JPEG after downsampling"
    );
    let cancel = AtomicBool::new(false);
    assert!(
        actual.preview(0, 800, &cancel).unwrap() == expected.preview(0, 800, &cancel).unwrap(),
        "Native PDF preview has an alpha halo"
    );
    if let Some(renderer) = ToolPaths::discover().pdftoppm {
        let mut rasters = Vec::new();
        for (name, prepared) in [("actual", actual), ("expected", expected)] {
            let input = temporary.path().join(format!("{name}.pdf"));
            let prefix = temporary.path().join(name);
            fs::write(&input, prepared.bytes()).unwrap();
            let output = std::process::Command::new(&renderer)
                .args(["-singlefile", "-png", "-scale-to", "800"])
                .arg(input)
                .arg(&prefix)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            rasters.push(
                image::open(prefix.with_extension("png"))
                    .unwrap()
                    .to_rgba8(),
            );
        }
        assert!(
            rasters[0] == rasters[1],
            "Poppler PDF raster has an alpha halo"
        );
    } else {
        eprintln!("SKIP Poppler portion: native PDF/JPEG and preview alpha equality verified");
    }
}

#[test]
fn manual_quality_changes_real_pdf_bytes_and_the_prepared_save_is_exact() {
    let input = PageInput::Image(Arc::new(RgbaImage::from_fn(240, 180, |x, y| {
        Rgba([
            ((x * 41 + y * 31) % 256) as u8,
            ((x * 13 + y * 79) % 256) as u8,
            ((x * 97 + y * 17) % 256) as u8,
            255,
        ])
    })));
    let settings = PageLayoutSettings::default();
    let cancel = AtomicBool::new(false);
    let small =
        page_layout::prepare(std::slice::from_ref(&input), &settings, 20, &cancel, |_| {}).unwrap();
    let large = page_layout::prepare(&[input], &settings, 95, &cancel, |_| {}).unwrap();
    assert!(large.exact_bytes() > small.exact_bytes());
    let temporary = tempfile::tempdir().unwrap();
    let original = temporary.path().join("original.txt");
    fs::write(&original, b"keep the source").unwrap();
    let output = temporary.path().join("chosen.pdf");
    assert_eq!(
        page_layout::save_prepared(Some(&original), &output, &small, &cancel).unwrap(),
        output
    );
    assert_eq!(fs::read(&output).unwrap(), small.bytes());
    assert_eq!(fs::metadata(&output).unwrap().len(), small.exact_bytes());
    assert!(page_layout::save_prepared(None, &output, &large, &cancel).is_err());
    assert_eq!(fs::read(&output).unwrap(), small.bytes());
    assert!(page_layout::save_prepared(Some(&original), &original, &large, &cancel).is_err());
    assert_eq!(fs::read(&original).unwrap(), b"keep the source");
}

#[test]
fn selected_txt_page_route_needs_no_office_and_preserves_existing_results() {
    let temporary = tempfile::tempdir().unwrap();
    let input = temporary.path().join("testo.TXT");
    let bytes = "\u{feff}Titolo italiano\r\nCaffè e qualità.".as_bytes();
    fs::write(&input, bytes).unwrap();
    fs::write(temporary.path().join("testo.pdf"), b"existing result").unwrap();
    let mut settings = ConversionSettings {
        format: OutputFormat::Pdf,
        ..Default::default()
    };
    settings.image.page = Some(PageLayoutSettings::default());
    let tools = ToolPaths::default();
    assert!(tools.missing_for_input(&input, &settings).is_none());
    let output =
        engine::convert(&input, &settings, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].file_name().unwrap(), "testo (1).pdf");
    let document = Document::load(&output[0]).unwrap();
    assert!(
        document
            .extract_text(&[1])
            .unwrap()
            .contains("Caffè e qualità.")
    );
    assert_eq!(fs::read(&input).unwrap(), bytes);
    assert_eq!(
        fs::read(temporary.path().join("testo.pdf")).unwrap(),
        b"existing result"
    );
    settings.image.page = None;
    assert!(
        tools
            .missing_for_input(&input, &settings)
            .unwrap()
            .contains("LibreOffice")
    );
    settings.image.page = Some(PageLayoutSettings::default());
    assert!(
        tools
            .missing_for_input(Path::new("document.docx"), &settings)
            .unwrap()
            .contains("LibreOffice")
    );
}

#[test]
fn malformed_settings_text_and_unsupported_characters_fail_before_publication() {
    let cancel = AtomicBool::new(false);
    for settings in [
        PageLayoutSettings {
            margin_mm: f32::NAN,
            ..Default::default()
        },
        PageLayoutSettings {
            margin_mm: 105.0,
            ..Default::default()
        },
        PageLayoutSettings {
            font_size_pt: f32::INFINITY,
            ..Default::default()
        },
        PageLayoutSettings {
            line_height: 0.0,
            ..Default::default()
        },
        PageLayoutSettings {
            dpi: 0,
            ..Default::default()
        },
    ] {
        assert!(settings.validate().is_err());
        assert!(page_layout::prepare(&[text("testo")], &settings, 90, &cancel, |_| {}).is_err());
    }
    let error = page_layout::prepare(
        &[text("Non perdere 🦀")],
        &PageLayoutSettings::default(),
        90,
        &cancel,
        |_| {},
    )
    .unwrap_err();
    assert!(error.to_string().contains("U+1F980"), "{error:#}");
    assert!(
        page_layout::prepare(
            &[text("testo\0nascosto")],
            &PageLayoutSettings::default(),
            90,
            &cancel,
            |_| {}
        )
        .is_err()
    );
    let temporary = tempfile::tempdir().unwrap();
    let input = temporary.path().join("invalid.txt");
    fs::write(&input, [0xff, 0xfe, 0, 0]).unwrap();
    let mut settings = ConversionSettings {
        format: OutputFormat::Pdf,
        ..Default::default()
    };
    settings.image.page = Some(PageLayoutSettings::default());
    assert!(
        page_layout::convert_text_pdf(&input, temporary.path(), &settings, &cancel, |_| {})
            .unwrap_err()
            .to_string()
            .contains("UTF-8")
    );
    assert_eq!(fs::read_dir(temporary.path()).unwrap().count(), 1);
}

#[test]
fn cancellation_preview_bounds_and_hardlinked_originals_are_preserved() {
    let cancel = AtomicBool::new(true);
    assert!(
        page_layout::prepare(
            &[text("testo")],
            &PageLayoutSettings::default(),
            90,
            &cancel,
            |_| {}
        )
        .is_err()
    );
    cancel.store(false, Ordering::Relaxed);
    let prepared = prepare(&[text("testo")], &PageLayoutSettings::default());
    assert!(prepared.preview(0, 0, &cancel).is_err());
    assert!(prepared.preview(0, 2049, &cancel).is_err());
    assert!(prepared.preview(1, 400, &cancel).is_err());
    let temporary = tempfile::tempdir().unwrap();
    let input = temporary.path().join("source.txt");
    let alias = temporary.path().join("alias.pdf");
    fs::write(&input, b"preserve hardlink source").unwrap();
    fs::hard_link(&input, &alias).unwrap();
    assert!(page_layout::save_prepared(Some(&input), &alias, &prepared, &cancel).is_err());
    cancel.store(true, Ordering::Relaxed);
    assert!(prepared.preview(0, 400, &cancel).is_err());
    let absent = temporary.path().join("cancelled.pdf");
    assert!(page_layout::save_prepared(Some(&input), &absent, &prepared, &cancel).is_err());
    assert!(!absent.exists());
    assert_eq!(fs::read(&input).unwrap(), b"preserve hardlink source");
    assert_eq!(fs::read(&alias).unwrap(), b"preserve hardlink source");
}

#[test]
fn cancellation_after_layout_does_not_return_encoded_pdf() {
    let cancel = AtomicBool::new(false);
    let result = page_layout::prepare(
        &[text("Il testo è già impaginato.")],
        &PageLayoutSettings::default(),
        90,
        &cancel,
        |progress| {
            if progress.fraction >= 0.8 {
                cancel.store(true, Ordering::Relaxed);
            }
        },
    );
    assert!(result.is_err());
}

#[test]
fn text_size_and_page_count_limits_fail_before_creating_a_result() {
    let cancel = AtomicBool::new(false);
    let settings = PageLayoutSettings::default();
    let too_large = "x".repeat(8 * 1024 * 1024 + 1);
    assert!(
        page_layout::prepare(&[text(&too_large)], &settings, 90, &cancel, |_| {})
            .unwrap_err()
            .to_string()
            .contains("8 MiB")
    );
    let too_many_pages = "\u{c}".repeat(1000);
    assert!(
        page_layout::prepare(&[text(&too_many_pages)], &settings, 90, &cancel, |_| {})
            .unwrap_err()
            .to_string()
            .contains("1000 pagine")
    );
    assert!(page_layout::prepare(&[], &settings, 90, &cancel, |_| {}).is_err());
}

#[test]
fn actual_poppler_render_matches_native_text_preview_geometry_when_available() {
    let Some(renderer) = ToolPaths::discover().pdftoppm else {
        eprintln!("SKIP: Poppler non disponibile per confronto raster del font PDF incorporato");
        return;
    };
    let prepared = prepare(&[text("Caffè già €")], &PageLayoutSettings::default());
    let temporary = tempfile::tempdir().unwrap();
    let input = temporary.path().join("font.pdf");
    fs::write(&input, prepared.bytes()).unwrap();
    let prefix = temporary.path().join("page");
    let output = std::process::Command::new(renderer)
        .args([
            "-f",
            "1",
            "-l",
            "1",
            "-singlefile",
            "-png",
            "-scale-to",
            "1200",
        ])
        .arg(&input)
        .arg(&prefix)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr)
            .to_ascii_lowercase()
            .contains("syntax error")
    );
    let actual = image::open(prefix.with_extension("png"))
        .unwrap()
        .to_rgba8();
    let expected = prepared.preview(0, 1200, &AtomicBool::new(false)).unwrap();
    let ink = |image: &RgbaImage, label: &str| {
        let mut bounds = (image.width(), image.height(), 0, 0);
        let mut count = 0;
        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel[0] < 100 {
                bounds = (
                    bounds.0.min(x),
                    bounds.1.min(y),
                    bounds.2.max(x),
                    bounds.3.max(y),
                );
                count += 1;
            }
        }
        let minimum = image.pixels().map(|pixel| pixel[0]).min().unwrap();
        assert!(
            count > 10,
            "Il testo non è stato disegnato: {label}, minimo {minimum}, pixel scuri {count}"
        );
        [
            bounds.0 as i32,
            bounds.1 as i32,
            bounds.2 as i32,
            bounds.3 as i32,
        ]
    };
    let (actual, expected) = (ink(&actual, "PDF"), ink(&expected, "anteprima"));
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 3,
            "Bounding box PDF/anteprima divergenti: {actual} / {expected}"
        );
    }
}
