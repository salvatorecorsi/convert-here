//! Local OCR through Tesseract. No images or recognized text are sent online.
use crate::i18n::tr;
use crate::{
    engine::{Progress, cancelled, commit, run_process, stem, temporary},
    model::{ConversionSettings, MediaKind},
    platform::set_hidden,
    tools::ToolPaths,
};
use anyhow::{Context, Result, bail};
use image::{DynamicImage, ImageDecoder, ImageFormat};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};

const MAX_TEXT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PAGES: usize = 10_000;

pub(crate) fn validate_language(language: &str) -> Result<()> {
    if language.is_empty()
        || language.len() > 128
        || language.split('+').count() > 8
        || language.split('+').any(|code| {
            code.is_empty()
                || code.len() > 32
                || !code
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
    {
        bail!(
            "{}",
            crate::trf!(
                "Lingua OCR non valida: usa i codici Tesseract, per esempio ita, eng o ita+eng",
                "Invalid OCR language: use Tesseract codes, for example ita, eng or ita+eng"
            )
        );
    }
    Ok(())
}

pub(crate) fn convert_image_text(
    input: &Path,
    dir: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    let text = extract_text(input, dir, settings, tools, cancel, progress)?;
    cancelled(cancel)?;
    let mut output = temporary(dir, "txt")?;
    output.as_file_mut().write_all(text.as_bytes())?;
    cancelled(cancel)?;
    commit(output, dir, &stem(input), "txt")
}

/// Returns UTF-8 with form-feed page delimiters, matching Poppler text exports.
/// PDF pages are rendered and recognized one at a time to bound temporary disk use.
pub(crate) fn extract_text(
    input: &Path,
    dir: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<String> {
    cancelled(cancel)?;
    validate_language(&settings.ocr.language)?;
    let tesseract = tools.tesseract.as_ref().context(
        tr("Per OCR locale serve Tesseract. Apri Impostazioni → Motori e seleziona tesseract.exe con i modelli delle lingue richieste.", "Local OCR requires Tesseract. Open Settings → Conversion engines and select tesseract.exe with the required language models."),
    )?;
    let tessdata = adjacent_tessdata(tesseract, &settings.ocr.language)?;
    let stage = tempfile::Builder::new()
        .prefix(".convert-here-ocr-")
        .tempdir_in(dir)?;
    set_hidden(stage.path())?;
    let mut text = String::new();
    match MediaKind::from_path(input) {
        MediaKind::Pdf => {
            let document = lopdf::Document::load(input).context(tr(
                "PDF non leggibile per OCR",
                "PDF cannot be read for OCR",
            ))?;
            if document.is_encrypted() {
                bail!(
                    "{}",
                    crate::trf!(
                        "Il PDF è protetto: sbloccalo prima di usare OCR",
                        "The PDF is protected: unlock it before using OCR"
                    )
                );
            }
            let count = document.get_pages().len();
            if count == 0 || count > MAX_PAGES {
                bail!(
                    "{}",
                    crate::trf!(
                        "OCR richiede un PDF con da 1 a {MAX_PAGES} pagine",
                        "OCR requires a PDF with 1 to {MAX_PAGES} pages"
                    )
                );
            }
            drop(document);
            let pdftoppm = tools.pdftoppm.as_ref().context(
                tr("Per OCR dei PDF serve anche Poppler. Apri Impostazioni → Motori per installarlo oppure seleziona pdftoppm.exe.", "PDF OCR also requires Poppler. Open Settings → Conversion engines to install it or select pdftoppm.exe."),
            )?;
            let image_base = stage.path().join("page");
            let image_path = image_base.with_extension("png");
            for page in 1..=count {
                cancelled(cancel)?;
                progress(Progress {
                    fraction: 0.05 + (page - 1) as f32 / count as f32 * 0.9,
                    message: crate::trf!(
                        "OCR pagina {page} di {count}…",
                        "OCR page {page} of {count}…"
                    ),
                });
                let mut render = Command::new(pdftoppm);
                render
                    .args(["-png", "-singlefile", "-r", &settings.pdf.dpi.to_string()])
                    .args(["-f", &page.to_string(), "-l", &page.to_string()])
                    .arg(input)
                    .arg(&image_base);
                run_process(render, cancel, None, progress).map_err(crate::engine::pdf_error)?;
                cancelled(cancel)?;
                let recognized = recognize_image(
                    &image_path,
                    stage.path(),
                    settings,
                    (tesseract, tessdata.as_deref()),
                    cancel,
                    progress,
                )?;
                if text.len() as u64 + recognized.len() as u64 + 2 > MAX_TEXT_BYTES {
                    bail!(
                        "{}",
                        crate::trf!(
                            "Il testo OCR supera il limite di 64 MB",
                            "OCR text exceeds the 64 MB limit"
                        )
                    );
                }
                text.push_str(recognized.trim_end_matches(['\r', '\n', '\u{c}']));
                text.push('\n');
                text.push('\u{c}');
                fs::remove_file(&image_path)?;
            }
        }
        MediaKind::Image => {
            progress(Progress {
                fraction: 0.1,
                message: tr("Preparazione immagine per OCR…", "Preparing image for OCR…").into(),
            });
            let source = if crate::modern_images::is_modern_input(input) {
                crate::modern_images::decode(input, stage.path(), tools, cancel, progress)?
            } else {
                input.to_owned()
            };
            let mut decoder = image::ImageReader::open(&source)?
                .with_guessed_format()?
                .into_decoder()
                .context(tr(
                    "Immagine non leggibile per OCR",
                    "Image cannot be read for OCR",
                ))?;
            if let Some(limit) = image::Limits::default().max_alloc
                && decoder.total_bytes() > limit
            {
                bail!(
                    "{}",
                    crate::trf!(
                        "L'immagine supera il limite di memoria per OCR",
                        "The image exceeds the OCR memory limit"
                    )
                );
            }
            let orientation = decoder.orientation()?;
            let mut image = DynamicImage::from_decoder(decoder)?;
            image.apply_orientation(orientation);
            cancelled(cancel)?;
            let mut rgb = image::RgbImage::new(image.width(), image.height());
            for (x, y, pixel) in image.to_rgba8().enumerate_pixels() {
                let alpha = pixel[3] as u32;
                rgb.put_pixel(
                    x,
                    y,
                    image::Rgb([
                        ((pixel[0] as u32 * alpha + 255 * (255 - alpha) + 127) / 255) as u8,
                        ((pixel[1] as u32 * alpha + 255 * (255 - alpha) + 127) / 255) as u8,
                        ((pixel[2] as u32 * alpha + 255 * (255 - alpha) + 127) / 255) as u8,
                    ]),
                );
            }
            let normalized = stage.path().join("input.png");
            rgb.save_with_format(&normalized, ImageFormat::Png)?;
            cancelled(cancel)?;
            progress(Progress {
                fraction: 0.3,
                message: tr(
                    "Riconoscimento testo con Tesseract…",
                    "Recognizing text with Tesseract…",
                )
                .into(),
            });
            text = recognize_image(
                &normalized,
                stage.path(),
                settings,
                (tesseract, tessdata.as_deref()),
                cancel,
                progress,
            )?;
        }
        _ => bail!(
            "{}",
            crate::trf!(
                "OCR supporta immagini e PDF",
                "OCR supports images and PDFs"
            )
        ),
    }
    cancelled(cancel)?;
    if text.trim().is_empty() {
        bail!(
            "{}",
            crate::trf!(
                "OCR non ha riconosciuto testo. Controlla lingua, orientamento e qualità della scansione.",
                "OCR did not recognize any text. Check the language, orientation and scan quality."
            )
        );
    }
    Ok(text)
}

fn adjacent_tessdata(executable: &Path, language: &str) -> Result<Option<PathBuf>> {
    let Some(parent) = executable.parent() else {
        return Ok(None);
    };
    let tessdata = parent.join("tessdata");
    if !tessdata.is_dir() {
        // System installations may use a shared data directory or TESSDATA_PREFIX.
        return Ok(None);
    }
    for code in language.split('+') {
        if !tessdata.join(format!("{code}.traineddata")).is_file() {
            bail!(
                "{}",
                crate::trf!(
                    "Modello OCR {code}.traineddata mancante in {}. Installa il modello Tesseract o scegli una lingua disponibile.",
                    "OCR model {code}.traineddata is missing from {}. Install the Tesseract model or choose an available language.",
                    tessdata.display()
                )
            );
        }
    }
    Ok(Some(tessdata))
}

fn recognize_image(
    input: &Path,
    stage: &Path,
    settings: &ConversionSettings,
    engine: (&Path, Option<&Path>),
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<String> {
    cancelled(cancel)?;
    let output_base = stage.join("recognized");
    let output_path = output_base.with_extension("txt");
    let mut command = Command::new(engine.0);
    command.arg(input).arg(&output_base);
    if let Some(tessdata) = engine.1 {
        command.arg("--tessdata-dir").arg(tessdata);
    }
    command.args([
        "-l",
        &settings.ocr.language,
        "--psm",
        "3",
        "--dpi",
        &settings.pdf.dpi.to_string(),
    ]);
    run_process(command, cancel, None, progress).context(tr(
        "OCR non riuscito: controlla Tesseract e i modelli delle lingue selezionate",
        "OCR failed: check Tesseract and the selected language models",
    ))?;
    cancelled(cancel)?;
    let size = fs::metadata(&output_path)
        .context(tr(
            "Tesseract non ha prodotto il testo OCR",
            "Tesseract did not produce OCR text",
        ))?
        .len();
    if size > MAX_TEXT_BYTES {
        bail!(
            "{}",
            crate::trf!(
                "Il testo OCR supera il limite di 64 MB",
                "OCR text exceeds the 64 MB limit"
            )
        );
    }
    let text = fs::read_to_string(&output_path).context(tr(
        "Il testo OCR non è UTF-8 valido",
        "OCR text is not valid UTF-8",
    ))?;
    fs::remove_file(&output_path)?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_codes_reject_options_paths_empty_components_and_excessive_lists() {
        for language in ["eng", "ita+eng", "chi_sim", "srp_latn"] {
            validate_language(language).unwrap();
        }
        for language in [
            "",
            "eng+",
            "+ita",
            "../eng",
            "eng/../../data",
            "--psm 0",
            "eng\nita",
            "a+a+a+a+a+a+a+a+a",
        ] {
            assert!(validate_language(language).is_err(), "{language}");
        }
    }
}
