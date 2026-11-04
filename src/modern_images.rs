//! HEIC/AVIF support through the optional local libheif command-line tools.
use crate::{
    engine::{Progress, cancelled, commit, external_temporary, run_process, stem},
    model::{ConversionSettings, OutputFormat},
    platform::set_hidden,
    tools::ToolPaths,
};
use anyhow::{Context, Result, bail};
use image::{DynamicImage, ImageFormat};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};

pub(crate) fn is_modern_input(input: &Path) -> bool {
    input
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["avif", "heic", "heif"]
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

/// The caller owns this private workspace and keeps it alive until the returned
/// PNG has been consumed. libheif applies HEIF transforms and normalizes EXIF
/// orientation in its PNG output; the native pipeline must not rotate it again.
pub(crate) fn decode(
    input: &Path,
    workspace: &Path,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    cancelled(cancel)?;
    let executable = tools.heif_dec.as_ref().context(
        "Per leggere HEIC/AVIF serve libheif. Apri Strumenti e seleziona heif-dec.exe con le sue DLL.",
    )?;
    let output = workspace.join("decoded.png");
    progress(Progress {
        fraction: 0.05,
        message: "Lettura HEIC/AVIF con libheif…".into(),
    });
    let mut command = Command::new(executable);
    command.arg("--quiet").arg(input).arg(&output);
    let result = run_process(command, cancel, None, progress);
    cancelled(cancel)?;
    result.context("libheif non ha decodificato l'immagine HEIC/AVIF")?;
    // heif-dec uses numbered filenames for multiple top-level still images.
    // Match the native GIF/TIFF behavior by converting the first image only.
    let output = if output.is_file() {
        output
    } else {
        workspace.join("decoded-1.png")
    };
    if !output.is_file() || fs::metadata(&output)?.len() == 0 {
        bail!(
            "libheif non ha prodotto un'immagine PNG. Verifica che il motore includa il decoder HEVC/AV1 richiesto."
        );
    }
    Ok(output)
}

pub(crate) fn encode(
    image: &DynamicImage,
    input: &Path,
    dir: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    let format = settings.format;
    if !matches!(format, OutputFormat::Avif | OutputFormat::Heic) {
        bail!("Formato libheif non supportato");
    }
    cancelled(cancel)?;
    let executable = tools.heif_enc.as_ref().context(
        "Per creare HEIC/AVIF serve libheif. Apri Strumenti e seleziona heif-enc.exe con le sue DLL e gli encoder HEVC/AV1.",
    )?;
    let stage = tempfile::Builder::new()
        .prefix(".the-converter-heif-")
        .tempdir_in(dir)?;
    set_hidden(stage.path())?;
    let source = stage.path().join("source.png");
    let mut source_file = fs::File::create(&source)?;
    // PNG does not accept floating-point EXR/HDR pixels. Integer pixels keep
    // their precision; floating-point input is represented as normalized16-bit.
    if image.color().bits_per_pixel() > 64 {
        DynamicImage::ImageRgba16(image.to_rgba16())
            .write_to(&mut source_file, ImageFormat::Png)?;
    } else {
        image.write_to(&mut source_file, ImageFormat::Png)?;
    }
    drop(source_file);
    cancelled(cancel)?;
    let (_output_stage, output) = external_temporary(dir, format.extension())?;
    let mut command = Command::new(executable);
    command.args(["-q", &settings.image.quality.to_string()]);
    if format == OutputFormat::Avif {
        command.arg("--avif");
    }
    command.arg(&source).arg("-o").arg(output.path());
    progress(Progress {
        fraction: 0.6,
        message: format!("Codifica {} con libheif…", format.label()),
    });
    let result = run_process(command, cancel, None, progress);
    cancelled(cancel)?;
    result.with_context(|| {
        format!(
            "libheif non ha creato {}. Verifica che la build includa l'encoder richiesto.",
            format.label()
        )
    })?;
    commit(output, dir, &stem(input), format.extension())
}
