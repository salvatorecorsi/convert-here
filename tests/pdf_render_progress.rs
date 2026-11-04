use lopdf::{Document, Object, Stream, dictionary};
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine,
    model::{ConversionSettings, OutputFormat},
    tools::ToolPaths,
};

fn page_colour(index: usize) -> [u8; 3] {
    [
        (30 + index % 3 * 80) as u8,
        (30 + index / 3 % 3 * 80) as u8,
        (30 + index / 9 * 80) as u8,
    ]
}

fn coloured_pdf(path: &Path, count: usize) {
    let mut document = Document::with_version("1.5");
    let parent = document.new_object_id();
    let mut children = Vec::new();
    for index in 0..count {
        let [red, green, blue] = page_colour(index);
        let content = format!(
            "{} {} {} rg 0 0 72 36 re f",
            f64::from(red) / 255.0,
            f64::from(green) / 255.0,
            f64::from(blue) / 255.0
        );
        let content = document.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        let page = document.add_object(dictionary! {
            "Type" => "Page", "Parent" => parent, "MediaBox" => vec![0.into(),0.into(),72.into(),36.into()],
            "Resources" => dictionary! {}, "Contents" => content,
        });
        children.push(Object::Reference(page));
    }
    document.objects.insert(
        parent,
        Object::Dictionary(
            dictionary! {"Type" => "Pages", "Kids" => children, "Count" => count as i64},
        ),
    );
    let catalog = document.add_object(dictionary! {"Type" => "Catalog", "Pages" => parent});
    document.trailer.set("Root", catalog);
    document.save(path).unwrap();
}

fn poppler() -> Option<ToolPaths> {
    let tools = ToolPaths::discover();
    if tools.pdftoppm.is_none() {
        eprintln!("SKIP PDF block rendering fixtures: pdftoppm unavailable");
        return None;
    }
    Some(tools)
}

fn settings(format: OutputFormat, dir: &Path, dpi: u32) -> ConversionSettings {
    let mut settings = ConversionSettings {
        format,
        output_dir: Some(dir.to_owned()),
        ..Default::default()
    };
    settings.pdf.dpi = dpi;
    settings.image.quality = 87;
    settings
}

fn staged_images(directory: &Path, format: OutputFormat) -> usize {
    let stage = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".the-converter-pages-")
        })
        .unwrap();
    fs::read_dir(stage)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == format.extension())
        })
        .count()
}

#[test]
fn block_progress_precedes_publication_and_png_jpeg_keep_page_order_dpi_and_names() {
    let Some(tools) = poppler() else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("colori è.pdf");
    coloured_pdf(&input, 23);
    let original = fs::read(&input).unwrap();
    for (format, dpi) in [(OutputFormat::Png, 72), (OutputFormat::Jpg, 144)] {
        let destination = directory.path().join(format.extension());
        fs::create_dir(&destination).unwrap();
        let existing = destination.join(format!("colori è-pagina-011.{}", format.extension()));
        fs::write(&existing, b"unrelated existing output").unwrap();
        let mut fractions = Vec::new();
        let mut stage_counts = Vec::new();
        let outputs = engine::convert(
            &input,
            &settings(format, &destination, dpi),
            &tools,
            &AtomicBool::new(false),
            |progress| {
                if progress.fraction > 0.05 && progress.fraction <= 0.5 {
                    fractions.push(progress.fraction);
                    stage_counts.push(staged_images(&destination, format));
                    let published: Vec<_> = fs::read_dir(&destination)
                        .unwrap()
                        .map(|entry| entry.unwrap().path())
                        .filter(|path| path.is_file() && path != &existing)
                        .collect();
                    assert!(
                        published.is_empty(),
                        "a page was published before rendering finished"
                    );
                }
            },
        )
        .unwrap();
        assert_eq!(stage_counts, [10, 20, 23]);
        assert_eq!(fractions.len(), 3);
        assert!(fractions.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(fractions.last(), Some(&0.5));
        assert_eq!(outputs.len(), 23);
        for (index, output) in outputs.iter().enumerate() {
            let suffix = if index == 10 { " (1)" } else { "" };
            assert_eq!(
                output.file_name().unwrap().to_string_lossy(),
                format!(
                    "colori è-pagina-{:03}{suffix}.{}",
                    index + 1,
                    format.extension()
                )
            );
            let image = image::open(output).unwrap().to_rgb8();
            assert_eq!(image.dimensions(), (dpi, dpi / 2));
            let actual = image.get_pixel(image.width() / 2, image.height() / 2).0;
            for (actual, expected) in actual.into_iter().zip(page_colour(index)) {
                assert!(
                    (i16::from(actual) - i16::from(expected)).abs() < 9,
                    "wrong page colour for {}",
                    output.display()
                );
            }
        }
        assert_eq!(fs::read(&existing).unwrap(), b"unrelated existing output");
        assert!(
            fs::read_dir(destination)
                .unwrap()
                .all(|entry| entry.unwrap().path().is_file())
        );
    }
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn cancellation_after_first_block_cleans_staging_and_preserves_unrelated_files() {
    let Some(tools) = poppler() else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    coloured_pdf(&input, 21);
    let original = fs::read(&input).unwrap();
    let destination = directory.path().join("destination");
    fs::create_dir(&destination).unwrap();
    let unrelated = destination.join("existing.png");
    fs::write(&unrelated, b"existing user file").unwrap();
    let cancel = AtomicBool::new(false);
    let mut completed_blocks = 0;
    let result = engine::convert(
        &input,
        &settings(OutputFormat::Png, &destination, 72),
        &tools,
        &cancel,
        |progress| {
            if progress.fraction > 0.05 && progress.fraction <= 0.5 {
                completed_blocks += 1;
                assert_eq!(staged_images(&destination, OutputFormat::Png), 10);
                cancel.store(true, Ordering::Relaxed);
            }
        },
    );
    assert!(result.unwrap_err().to_string().contains("annullata"));
    assert_eq!(completed_blocks, 1);
    assert_eq!(fs::read_dir(destination).unwrap().count(), 1);
    assert_eq!(fs::read(unrelated).unwrap(), b"existing user file");
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn changed_page_count_is_rejected_before_any_page_is_published() {
    let Some(tools) = poppler() else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("changing.pdf");
    coloured_pdf(&input, 2);
    let destination = directory.path().join("destination");
    fs::create_dir(&destination).unwrap();
    let unrelated = destination.join("keep.txt");
    fs::write(&unrelated, b"preserve me").unwrap();
    let mut changed = false;
    let result = engine::convert(
        &input,
        &settings(OutputFormat::Png, &destination, 72),
        &tools,
        &AtomicBool::new(false),
        |progress| {
            if progress.fraction == 0.05 && !changed {
                // Simulate a document changing between the metadata probe and render.
                coloured_pdf(&input, 1);
                changed = true;
            }
        },
    );
    assert!(changed);
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("nessun file è stato pubblicato")
    );
    assert_eq!(fs::read_dir(destination).unwrap().count(), 1);
    assert_eq!(fs::read(unrelated).unwrap(), b"preserve me");
}

#[test]
fn unexpected_numeric_page_indices_are_rejected_even_when_the_count_matches() {
    let Some(tools) = poppler() else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    coloured_pdf(&input, 2);
    let destination = directory.path().join("destination");
    let mut changed = false;
    let result = engine::convert(
        &input,
        &settings(OutputFormat::Png, &destination, 72),
        &tools,
        &AtomicBool::new(false),
        |progress| {
            if progress.fraction == 0.5 && !changed {
                let stage = fs::read_dir(&destination)
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .find(|path| path.is_dir())
                    .unwrap();
                let second = fs::read_dir(&stage)
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .find(|path| {
                        path.file_stem()
                            .and_then(|name| name.to_str())
                            .and_then(|name| name.rsplit('-').next())
                            .and_then(|number| number.parse::<usize>().ok())
                            == Some(2)
                    })
                    .unwrap();
                fs::rename(&second, second.with_file_name("page-999.png")).unwrap();
                changed = true;
            }
        },
    );
    assert!(changed);
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("nessun file è stato pubblicato")
    );
    assert_eq!(fs::read_dir(destination).unwrap().count(), 0);
}
