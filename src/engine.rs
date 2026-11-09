//! Local conversion engine. Inputs are never modified; outputs are committed without clobbering.
use crate::i18n::tr;
use crate::model::{
    ConversionSettings, MediaKind, OutputFormat, PdfAction, VideoCodec, supported_formats,
};
use crate::platform::set_hidden;
use crate::tools::ToolPaths;
use anyhow::{Context, Result, bail};
use image::{DynamicImage, GenericImageView, ImageDecoder, ImageFormat};
use lopdf::{Document, Object, Stream, dictionary};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use tempfile::{NamedTempFile, TempDir};

#[path = "output_plan.rs"]
mod output_plan;
pub(crate) use output_plan::identity as file_identity;
pub use output_plan::{
    PreparedConversion, ReservedOutput, convert_prepared, prepare, release_prepared,
    validate_prepared,
};

#[derive(Debug, Clone)]
pub struct Progress {
    pub fraction: f32,
    pub message: String,
}

pub(crate) fn cancelled(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        bail!(crate::trf!("Conversione annullata", "Conversion cancelled"));
    }
    Ok(())
}

pub(crate) fn hidden(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}

/// Keep tool subprocesses in a Windows job so cancellation/crashes also stop helper processes.
#[cfg(windows)]
struct ProcessJob(std::os::windows::io::OwnedHandle);
#[cfg(windows)]
impl ProcessJob {
    fn attach(child: &Child) -> Result<Self> {
        use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let raw = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if raw.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            let handle = OwnedHandle::from_raw_handle(raw);
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                raw,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            ) == 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            if AssignProcessToJobObject(raw, child.as_raw_handle()) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            Ok(Self(handle))
        }
    }
    fn terminate(&self) {
        use std::os::windows::io::AsRawHandle;
        unsafe {
            windows_sys::Win32::System::JobObjects::TerminateJobObject(self.0.as_raw_handle(), 1);
        }
    }
}

#[cfg(not(windows))]
struct ProcessJob;
#[cfg(not(windows))]
impl ProcessJob {
    fn attach(_: &Child) -> Result<Self> {
        Ok(Self)
    }
    fn terminate(&self) {}
}

pub fn convert(
    input: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    cancelled(cancel)?;
    let input = fs::canonicalize(input).with_context(|| {
        crate::trf!(
            "File non accessibile: {}",
            "File cannot be accessed: {}",
            input.display()
        )
    })?;
    if !input.is_file() {
        bail!(crate::trf!(
            "La selezione deve contenere file, non cartelle",
            "Select files, not folders"
        ));
    }
    let kind = MediaKind::from_path(&input);
    if !supported_formats(kind).contains(&settings.format) {
        bail!(crate::trf!(
            "Conversione non supportata: {:?} → {}",
            "Unsupported conversion: {:?} → {}",
            kind,
            settings.format
        ));
    }
    validate_for_input(&input, settings)?;
    if kind == MediaKind::Pdf
        && settings.format == OutputFormat::Pdf
        && settings.pdf.action == PdfAction::Merge
    {
        bail!(crate::trf!(
            "Per unire i PDF seleziona almeno due file e avvia l'operazione di gruppo",
            "To merge PDFs, select at least two files and start the batch operation"
        ));
    }
    if let Some(reason) = tools.missing_for_input(&input, settings) {
        bail!("{reason}");
    }
    let out_dir = settings
        .output_dir
        .clone()
        .unwrap_or_else(|| input.parent().unwrap().to_owned());
    fs::create_dir_all(&out_dir).context(crate::i18n::tr(
        "Impossibile creare la cartella di destinazione",
        "Unable to create the output folder",
    ))?;
    let out_dir = fs::canonicalize(out_dir)?;
    on_progress(Progress {
        fraction: 0.0,
        message: crate::i18n::tr("Preparazione…", "Preparing…").into(),
    });
    let outputs = match kind {
        MediaKind::Audio | MediaKind::Video => vec![convert_media(
            &input,
            &out_dir,
            settings,
            tools,
            cancel,
            &mut on_progress,
        )?],
        MediaKind::Image if settings.format == OutputFormat::Txt => {
            vec![crate::ocr::convert_image_text(
                &input,
                &out_dir,
                settings,
                tools,
                cancel,
                &mut on_progress,
            )?]
        }
        MediaKind::Image => vec![convert_image(
            &input,
            &out_dir,
            settings,
            tools,
            cancel,
            &mut on_progress,
        )?],
        MediaKind::Pdf => convert_pdf(&input, &out_dir, settings, tools, cancel, &mut on_progress)?,
        MediaKind::Document
            if settings.format == OutputFormat::Pdf
                && settings.image.page.is_some()
                && input
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("txt")) =>
        {
            vec![crate::page_layout::convert_text_pdf(
                &input,
                &out_dir,
                settings,
                cancel,
                &mut on_progress,
            )?]
        }
        MediaKind::Document | MediaKind::Spreadsheet | MediaKind::Html => {
            vec![crate::documents::convert_office(
                &input,
                &out_dir,
                settings,
                tools,
                cancel,
                &mut on_progress,
            )?]
        }
        _ => bail!(crate::trf!("Formato non supportato", "Unsupported format")),
    };
    on_progress(Progress {
        fraction: 1.0,
        message: crate::trf!(
            "Completato: {} file",
            "Output files created: {}",
            outputs.len()
        ),
    });
    Ok(outputs)
}

/// A merge is a single operation over an ordered group, not a per-file conversion.
pub fn merge_pdfs(
    inputs: &[PathBuf],
    settings: &ConversionSettings,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    cancelled(cancel)?;
    if inputs.len() < 2 {
        bail!(crate::trf!(
            "Per unire i PDF seleziona almeno due file compatibili",
            "To merge PDFs, select at least two compatible files"
        ));
    }
    if settings.format != OutputFormat::Pdf || settings.pdf.action != PdfAction::Merge {
        bail!(crate::trf!(
            "L'unione richiede destinazione PDF e operazione Merge",
            "Merging requires PDF output and the Merge action"
        ));
    }
    validate(settings)?;
    let prepared = crate::pdf_merge::prepare_pdf_merge(
        inputs,
        settings,
        &ToolPaths::discover(),
        cancel,
        |update| {
            on_progress(Progress {
                fraction: update.fraction * 0.5,
                message: update.message,
            })
        },
    )?;
    crate::pdf_merge::merge_prepared(&prepared, &prepared.pages(), settings, cancel, |update| {
        on_progress(Progress {
            fraction: 0.5 + update.fraction * 0.5,
            message: update.message,
        })
    })
}

pub(crate) fn validate(s: &ConversionSettings) -> Result<()> {
    if !(8..=512).contains(&s.audio.bitrate_kbps) {
        bail!(crate::trf!(
            "Il bitrate deve essere tra 8 e 512 kbps",
            "Bitrate must be between 8 and 512 kbps"
        ));
    }
    if s.audio.sample_rate != 0 && !(8000..=192000).contains(&s.audio.sample_rate) {
        bail!(crate::trf!(
            "Frequenza audio non valida (8000–192000 Hz oppure 0 per la sorgente)",
            "Invalid sample rate (8000–192000 Hz, or 0 to use the source)"
        ));
    }
    if s.audio.channels > 2 {
        bail!(crate::trf!(
            "Canali audio: 0 = sorgente, 1 = mono, 2 = stereo",
            "Audio channels: 0 = source, 1 = mono, 2 = stereo"
        ));
    }
    if let Some(rates) = s.format.audio_sample_rates()
        && !rates.contains(&s.audio.sample_rate)
    {
        let choices = rates
            .iter()
            .filter(|rate| **rate != 0)
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        bail!(crate::trf!(
            "{}: frequenze audio supportate {choices} Hz (0 = automatica)",
            "{}: supported sample rates {choices} Hz (0 = automatic)",
            s.format.label()
        ));
    }
    if s.video.crf > 51 {
        bail!(crate::trf!("CRF non valido (0–51)", "Invalid CRF (0–51)"));
    }
    if s.video.width.is_some_and(|v| !(16..=16384).contains(&v))
        || s.image.width.is_some_and(|v| !(1..=32768).contains(&v))
    {
        bail!(crate::trf!("Larghezza non valida", "Invalid width"));
    }
    if s.video.fps.is_some_and(|v| !(1..=240).contains(&v)) {
        bail!(crate::trf!(
            "FPS non validi (1–240)",
            "Invalid frame rate (1–240 FPS)"
        ));
    }
    if !(1..=100).contains(&s.image.quality) {
        bail!(crate::trf!(
            "Qualità JPEG non valida (1–100)",
            "Invalid JPEG quality (1–100)"
        ));
    }
    if !(36..=600).contains(&s.pdf.dpi) {
        bail!(crate::trf!(
            "DPI non validi (36–600)",
            "Invalid DPI (36–600)"
        ));
    }
    if s.video.trim_start.is_some_and(|v| !v.is_finite() || v < 0.)
        || s.video.trim_end.is_some_and(|v| !v.is_finite() || v <= 0.)
    {
        bail!(crate::trf!(
            "Tempi di taglio non validi",
            "Invalid trim times"
        ));
    }
    if let Some(end) = s.video.trim_end
        && end <= s.video.trim_start.unwrap_or(0.)
    {
        bail!(crate::trf!(
            "La fine del taglio deve essere successiva all'inizio",
            "The trim end must be after the start"
        ));
    }
    Ok(())
}

fn validate_for_input(input: &Path, settings: &ConversionSettings) -> Result<()> {
    let kind = MediaKind::from_path(input);
    // Loaded plans can outlive a removed conversion route. Reject them before
    // allocating a workspace or starting any optional conversion tool.
    if !supported_formats(kind).contains(&settings.format) {
        bail!(crate::trf!(
            "Conversione non supportata: {:?} → {}",
            "Unsupported conversion: {:?} → {}",
            kind,
            settings.format
        ));
    }
    validate(settings)?;
    if kind == MediaKind::Image {
        if let Some(edit) = &settings.image.edit {
            crate::image_edit::validate(edit)?;
            if !matches!(
                settings.format,
                OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp | OutputFormat::Pdf
            ) {
                bail!(tr(
                    "Le modifiche immagine richiedono una destinazione PNG, JPEG, WebP o PDF",
                    "Image editing requires PNG, JPEG, WebP or PDF output"
                ));
            }
        }
        if settings.format == OutputFormat::Pdf
            && let Some(page) = &settings.image.page
        {
            page.validate()?;
        }
    }
    if kind == MediaKind::Document
        && settings.format == OutputFormat::Pdf
        && input
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
        && let Some(page) = &settings.image.page
    {
        page.validate()?;
    }
    if kind == MediaKind::Image && settings.format == OutputFormat::Txt
        || kind == MediaKind::Pdf
            && settings.ocr.enabled
            && matches!(
                settings.format,
                OutputFormat::Txt | OutputFormat::Docx | OutputFormat::Html
            )
    {
        crate::ocr::validate_language(&settings.ocr.language)?;
    }
    Ok(())
}

pub(crate) fn temporary(dir: &Path, ext: &str) -> Result<NamedTempFile> {
    let file = tempfile::Builder::new()
        .prefix(".the-converter-")
        .suffix(&format!(".{ext}"))
        .tempfile_in(dir)?;
    set_hidden(file.path())?;
    Ok(file)
}

/// Path-based writers can reject an existing Hidden file on Windows. Hide the
/// enclosing directory instead, keeping its guard alive until publication.
pub(crate) fn external_temporary(dir: &Path, ext: &str) -> Result<(TempDir, NamedTempFile)> {
    let stage = tempfile::Builder::new()
        .prefix(".the-converter-external-")
        .tempdir_in(dir)?;
    set_hidden(stage.path())?;
    let file = tempfile::Builder::new()
        .prefix("output-")
        .suffix(&format!(".{ext}"))
        .tempfile_in(stage.path())?;
    Ok((stage, file))
}

/// Adopt a completed file from a caller-owned staging directory without copying.
/// The enclosing TempDir/WorkGuard remains responsible for failed publications;
/// the adopted path must never be an original user input or a symbolic link.
pub(crate) fn staged_temporary(source: &Path, stage: &Path) -> Result<NamedTempFile> {
    let metadata = fs::symlink_metadata(source).context(crate::i18n::tr(
        "Risultato temporaneo non accessibile",
        "Temporary output cannot be accessed",
    ))?;
    if !metadata.file_type().is_file() {
        bail!(crate::trf!(
            "Il risultato temporaneo deve essere un file regolare",
            "Temporary output must be a regular file"
        ));
    }
    let stage = fs::canonicalize(stage).context(crate::i18n::tr(
        "Cartella temporanea non accessibile",
        "Temporary folder cannot be accessed",
    ))?;
    let source = fs::canonicalize(source).context(crate::i18n::tr(
        "Risultato temporaneo non accessibile",
        "Temporary output cannot be accessed",
    ))?;
    if source.parent() != Some(stage.as_path()) {
        bail!(crate::trf!(
            "Il risultato non appartiene alla cartella temporanea della conversione",
            "The output does not belong to this conversion's temporary folder"
        ));
    }
    // Keep this handle across the rename. Reopening the published path could
    // fail or observe a replacement created by another program in the meantime.
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&source)
        .context(crate::i18n::tr(
            "Impossibile aprire il risultato temporaneo",
            "Unable to open the temporary output",
        ))?;
    if file.metadata()?.len() == 0 {
        bail!(crate::trf!(
            "Il convertitore ha prodotto un file vuoto",
            "The converter produced an empty file"
        ));
    }
    let mut path = tempfile::TempPath::try_from_path(source)?;
    // A failed no-clobber attempt must leave the owned source available for
    // another numbered name. Its enclosing staging directory handles cleanup.
    path.disable_cleanup(true);
    Ok(NamedTempFile::from_parts(file, path))
}

/// persist_noclobber performs the final creation without replacing an existing file, including races.
pub(crate) fn commit(
    mut temp: NamedTempFile,
    dir: &Path,
    stem: &str,
    ext: &str,
) -> Result<PathBuf> {
    if temp.as_file().metadata()?.len() == 0 {
        bail!(crate::trf!(
            "Il convertitore ha prodotto un file vuoto",
            "The converter produced an empty file"
        ));
    }
    temp.as_file_mut().flush()?;
    for n in 0..10000u32 {
        let name = if n == 0 {
            format!("{stem}.{ext}")
        } else {
            format!("{stem} ({n}).{ext}")
        };
        let dest = dir.join(name);
        match temp.persist_noclobber(&dest) {
            Ok(_) => return Ok(dest),
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => temp = e.file,
            Err(e) => {
                return Err(e.error).context(crate::i18n::tr(
                    "Impossibile salvare il risultato",
                    "Unable to save the output",
                ));
            }
        }
    }
    bail!(crate::trf!(
        "Troppi file con lo stesso nome nella destinazione",
        "Too many files with the same name in the output folder"
    ))
}

pub(crate) fn stem(input: &Path) -> String {
    input
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn copy_commit(source: &Path, dir: &Path, name: &str, ext: &str) -> Result<PathBuf> {
    let mut temp = temporary(dir, ext)?;
    let mut source = fs::File::open(source)?;
    std::io::copy(&mut source, temp.as_file_mut())?;
    commit(temp, dir, name, ext)
}

fn flatten(image: &DynamicImage) -> DynamicImage {
    let mut rgb = image::RgbImage::new(image.width(), image.height());
    for (x, y, p) in image.to_rgba8().enumerate_pixels() {
        let a = p[3] as u32;
        rgb.put_pixel(
            x,
            y,
            image::Rgb([
                ((p[0] as u32 * a + 255 * (255 - a) + 127) / 255) as u8,
                ((p[1] as u32 * a + 255 * (255 - a) + 127) / 255) as u8,
                ((p[2] as u32 * a + 255 * (255 - a) + 127) / 255) as u8,
            ]),
        );
    }
    DynamicImage::ImageRgb8(rgb)
}

fn convert_image(
    input: &Path,
    dir: &Path,
    s: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    if s.image.edit.is_some() || s.format == OutputFormat::Pdf && s.image.page.is_some() {
        return convert_edited_image(input, dir, s, tools, cancel, progress);
    }
    let modern_workspace = if crate::modern_images::is_modern_input(input) {
        let workspace = tempfile::Builder::new()
            .prefix(".the-converter-heif-")
            .tempdir_in(dir)?;
        set_hidden(workspace.path())?;
        Some(workspace)
    } else {
        None
    };
    let decoded = if let Some(workspace) = &modern_workspace {
        crate::modern_images::decode(input, workspace.path(), tools, cancel, progress)?
    } else {
        input.to_owned()
    };
    let mut decoder = image::ImageReader::open(&decoded)?
        .with_guessed_format()?
        .into_decoder()
        .context(crate::i18n::tr(
            "Immagine non leggibile o codec non supportato",
            "Unreadable image or unsupported codec",
        ))?;
    // into_decoder does not reserve the decoded pixel allocation as ImageReader::decode does.
    if let Some(limit) = image::Limits::default().max_alloc
        && decoder.total_bytes() > limit
    {
        bail!(crate::trf!(
            "L'immagine supera il limite di memoria per la decodifica",
            "The image exceeds the decoding memory limit"
        ));
    }
    let orientation = decoder.orientation().context(crate::i18n::tr(
        "Orientamento immagine non leggibile",
        "Unable to read image orientation",
    ))?;
    let mut image = DynamicImage::from_decoder(decoder).context(crate::i18n::tr(
        "Immagine non leggibile o codec non supportato",
        "Unreadable image or unsupported codec",
    ))?;
    // Bake camera/smartphone EXIF rotation into pixels before resizing or embedding in PDF.
    image.apply_orientation(orientation);
    cancelled(cancel)?;
    progress(Progress {
        fraction: 0.25,
        message: crate::i18n::tr("Elaborazione immagine…", "Processing image…").into(),
    });
    if let Some(width) = s.image.width {
        let height =
            ((image.height() as f64 * width as f64 / image.width() as f64).round() as u32).max(1);
        if (width as u64) * (height as u64) > 150_000_000 {
            bail!(crate::trf!(
                "L'immagine ridimensionata supera 150 megapixel",
                "The resized image exceeds 150 megapixels"
            ));
        }
        image = image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
    }
    if s.format == OutputFormat::Ico && (image.width() > 256 || image.height() > 256) {
        image = image.thumbnail(256, 256);
    }
    if matches!(s.format, OutputFormat::Avif | OutputFormat::Heic) {
        return crate::modern_images::encode(&image, input, dir, s, tools, cancel, progress);
    }
    let mut temp = temporary(dir, s.format.extension())?;
    match s.format {
        OutputFormat::Pdf => image_pdf(&image, s.image.quality, temp.as_file_mut())?,
        OutputFormat::Jpg => {
            image::codecs::jpeg::JpegEncoder::new_with_quality(temp.as_file_mut(), s.image.quality)
                .encode_image(&flatten(&image))?
        }
        _ => {
            let format = match s.format {
                OutputFormat::Png => ImageFormat::Png,
                OutputFormat::Webp => ImageFormat::WebP,
                OutputFormat::Bmp => ImageFormat::Bmp,
                OutputFormat::Tiff => ImageFormat::Tiff,
                OutputFormat::Gif => ImageFormat::Gif,
                OutputFormat::Ico => ImageFormat::Ico,
                OutputFormat::Qoi => ImageFormat::Qoi,
                OutputFormat::Tga => ImageFormat::Tga,
                OutputFormat::Farbfeld => ImageFormat::Farbfeld,
                OutputFormat::OpenExr => ImageFormat::OpenExr,
                OutputFormat::Hdr => ImageFormat::Hdr,
                OutputFormat::Pnm => ImageFormat::Pnm,
                _ => bail!(crate::trf!(
                    "Format image non supportato",
                    "Unsupported image format"
                )),
            };
            if s.format == OutputFormat::Farbfeld {
                DynamicImage::ImageRgba16(image.to_rgba16())
                    .write_to(temp.as_file_mut(), format)?;
            } else if s.format == OutputFormat::OpenExr {
                DynamicImage::ImageRgba32F(image.to_rgba32f())
                    .write_to(temp.as_file_mut(), format)?;
            } else if matches!(
                s.format,
                OutputFormat::Qoi
                    | OutputFormat::Tga
                    | OutputFormat::Webp
                    | OutputFormat::Bmp
                    | OutputFormat::Gif
                    | OutputFormat::Ico
            ) {
                DynamicImage::ImageRgba8(image.to_rgba8()).write_to(temp.as_file_mut(), format)?;
            } else if s.format == OutputFormat::Png && image.color().bits_per_pixel() > 64 {
                // EXR/HDR can decode to floating point, which PNG cannot encode.
                DynamicImage::ImageRgba16(image.to_rgba16())
                    .write_to(temp.as_file_mut(), format)?;
            } else if s.format == OutputFormat::Pnm {
                DynamicImage::ImageRgba16(image.to_rgba16())
                    .write_to(temp.as_file_mut(), format)?;
            } else if s.format == OutputFormat::Hdr && image.has_alpha() {
                // Radiance RGBE has no alpha channel. Composite in floating point
                // so HDR source values are not clipped to an 8-bit intermediate.
                let rgba = image.to_rgba32f();
                let rgb = image::Rgb32FImage::from_fn(image.width(), image.height(), |x, y| {
                    let pixel = rgba.get_pixel(x, y);
                    let alpha = pixel[3].clamp(0.0, 1.0);
                    image::Rgb([
                        pixel[0] * alpha + 1.0 - alpha,
                        pixel[1] * alpha + 1.0 - alpha,
                        pixel[2] * alpha + 1.0 - alpha,
                    ])
                });
                DynamicImage::ImageRgb32F(rgb).write_to(temp.as_file_mut(), format)?;
            } else if s.format == OutputFormat::Hdr {
                DynamicImage::ImageRgb32F(image.to_rgb32f())
                    .write_to(temp.as_file_mut(), format)?;
            } else {
                image.write_to(temp.as_file_mut(), format)?;
            }
        }
    }
    cancelled(cancel)?;
    progress(Progress {
        fraction: 0.9,
        message: crate::i18n::tr("Salvataggio…", "Saving…").into(),
    });
    commit(temp, dir, &stem(input), s.format.extension())
}

fn convert_edited_image(
    input: &Path,
    dir: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    let loaded = crate::image_edit::load_image(input, tools, cancel, |update| {
        progress(Progress {
            fraction: update.fraction * 0.2,
            message: update.message,
        });
    })?;
    let mut edit = settings.image.edit.clone().unwrap_or_default();
    // Existing width remains effective when page layout is used without an edit recipe.
    if settings.image.edit.is_none()
        && let Some(width) = settings.image.width
    {
        edit.resize = Some(crate::image_edit::ResizeSettings {
            width: Some(width),
            ..Default::default()
        });
    }
    let pixels = crate::image_edit::render(&loaded.pixels, &edit, cancel)?;
    progress(Progress {
        fraction: 0.25,
        message: tr("Geometria immagine pronta", "Image geometry ready").into(),
    });
    let mut output_progress = |update: Progress| {
        progress(Progress {
            fraction: 0.25 + update.fraction * 0.7,
            message: update.message,
        });
    };
    let bytes = if settings.format == OutputFormat::Pdf {
        let page = settings.image.page.clone().unwrap_or_default();
        let prepared = crate::page_layout::prepare(
            &[crate::page_layout::PageInput::Image(std::sync::Arc::new(
                pixels,
            ))],
            &page,
            settings.image.quality,
            cancel,
            &mut output_progress,
        )?;
        if edit
            .max_bytes
            .is_some_and(|cap| prepared.exact_bytes() > cap)
        {
            bail!(tr(
                "Il PDF supera il peso massimo: modifica qualità o dimensioni; nessun file è stato salvato",
                "The PDF exceeds the size limit: change quality or dimensions; no file was saved"
            ));
        }
        prepared.bytes().to_vec()
    } else {
        let encoded = crate::image_edit::encode(
            &pixels,
            settings.format,
            settings.image.quality,
            &edit,
            cancel,
            &mut output_progress,
        )?;
        if !encoded.target_met {
            bail!(tr(
                "Il peso massimo non è stato raggiunto: riduci qualità o dimensioni; nessun file è stato salvato",
                "The size limit could not be met: reduce quality or dimensions; no file was saved"
            ));
        }
        encoded.bytes
    };
    cancelled(cancel)?;
    let mut temp = temporary(dir, settings.format.extension())?;
    temp.as_file_mut().write_all(&bytes)?;
    cancelled(cancel)?;
    commit(temp, dir, &stem(input), settings.format.extension())
}

fn image_pdf(image: &DynamicImage, quality: u8, output: &mut impl Write) -> Result<()> {
    let image = flatten(image);
    let (w, h) = image.dimensions();
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality).encode_image(&image)?;
    let mut doc = Document::with_version("1.5");
    let pages = doc.new_object_id();
    let img=doc.add_object(Stream::new(dictionary!{"Type"=>"XObject","Subtype"=>"Image","Width"=>w as i64,"Height"=>h as i64,"ColorSpace"=>"DeviceRGB","BitsPerComponent"=>8,"Filter"=>"DCTDecode"},jpeg));
    // A4 page, oriented like the source, with a 24pt margin; aspect ratio is preserved.
    let (pw, ph) = if w > h {
        (842.0f64, 595.0f64)
    } else {
        (595.0, 842.0)
    };
    let scale = ((pw - 48.) / w as f64).min((ph - 48.) / h as f64);
    let (iw, ih) = (w as f64 * scale, h as f64 * scale);
    let content = format!(
        "q {iw:.4} 0 0 {ih:.4} {:.4} {:.4} cm /Im0 Do Q",
        (pw - iw) / 2.,
        (ph - ih) / 2.
    );
    let contents = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
    let resources = doc.add_object(dictionary! {"XObject"=>dictionary!{"Im0"=>img}});
    let page=doc.add_object(dictionary!{"Type"=>"Page","Parent"=>pages,"MediaBox"=>vec![Object::from(0),Object::from(0),Object::Real(pw as f32),Object::Real(ph as f32)],"Resources"=>resources,"Contents"=>contents});
    doc.objects.insert(
        pages,
        Object::Dictionary(
            dictionary! {"Type"=>"Pages","Kids"=>vec![Object::Reference(page)],"Count"=>1},
        ),
    );
    let catalog = doc.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages});
    doc.trailer.set("Root", catalog);
    doc.compress();
    doc.save_to(output)?;
    Ok(())
}

fn duration(input: &Path, tools: &ToolPaths, cancel: &AtomicBool) -> Option<f64> {
    if cancel.load(Ordering::Relaxed) {
        return None;
    }
    let exe = tools
        .ffprobe
        .as_ref()
        .filter(|path| crate::tools::executable_available(path))?;
    let mut cmd = Command::new(exe);
    hidden(&mut cmd)
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(input)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().ok()?;
    let job = match ProcessJob::attach(&child) {
        Ok(job) => job,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
    };
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                let mut text = String::new();
                child.stdout.take()?.read_to_string(&mut text).ok()?;
                return text
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v > 0.);
            }
            Ok(None) => {}
            Err(_) => return None,
        }
        if start.elapsed() > Duration::from_secs(10) || cancel.load(Ordering::Relaxed) {
            job.terminate();
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        thread::sleep(Duration::from_millis(30));
    }
}

const PDF_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PDF_PROBE_STDOUT_LIMIT: usize = 64 * 1024;
const PDF_PROBE_STDERR_LIMIT: usize = 8 * 1024;
const PDF_OPEN_PASSWORD_ERROR: &str =
    "Il PDF richiede una password di apertura: sbloccalo prima della conversione";

fn pdf_open_password_error() -> &'static str {
    crate::i18n::tr(
        PDF_OPEN_PASSWORD_ERROR,
        "The PDF requires an opening password: unlock it before converting",
    )
}

/// Count pages with the renderer's companion tool before reserving output names.
/// Native parsing is a fallback only when pdfinfo is unavailable.
pub fn pdf_page_count(input: &Path, tools: &ToolPaths, cancel: &AtomicBool) -> Result<usize> {
    cancelled(cancel)?;
    let input = fs::canonicalize(input).context(crate::i18n::tr(
        "Impossibile leggere le pagine del PDF",
        "Unable to read PDF pages",
    ))?;
    if !input.is_file() {
        bail!(crate::trf!(
            "Impossibile leggere le pagine del PDF: seleziona un file",
            "Unable to read PDF pages: select a file"
        ));
    }
    let count = if let Some(executable) = tools
        .pdfinfo
        .as_ref()
        .filter(|path| crate::tools::executable_available(path))
    {
        let mut command = Command::new(executable);
        command.args(["-enc", "UTF-8"]).arg(&input);
        let output = run_pdf_probe(command, cancel, PDF_PROBE_TIMEOUT)?;
        if !output.status.success() {
            let diagnostic = String::from_utf8_lossy(&output.stderr);
            return Err(pdf_error(anyhow::anyhow!(crate::trf!(
                "Impossibile leggere le pagine del PDF: {}",
                "Unable to read PDF pages: {}",
                if diagnostic.trim().is_empty() {
                    "pdfinfo si è interrotto"
                } else {
                    diagnostic.trim()
                }
            ))));
        }
        parse_pdf_page_count(&output.stdout)?
    } else {
        let document = Document::load(&input).context(crate::i18n::tr(
            "Impossibile leggere le pagine del PDF",
            "Unable to read PDF pages",
        ))?;
        cancelled(cancel)?;
        let count = document.get_pages().len();
        // lopdf 0.38 retains Encrypt even after authenticating an empty user
        // password. Such owner-protected documents still have readable pages.
        if document.is_encrypted() && count == 0 {
            bail!(pdf_open_password_error());
        }
        count
    };
    cancelled(cancel)?;
    if count == 0 || count > 10_000 {
        bail!(crate::trf!(
            "Il PDF deve contenere da 1 a 10000 pagine",
            "The PDF must contain between 1 and 10000 pages"
        ));
    }
    Ok(count)
}

/// Poppler reports opening-password failures in English for all PDF exports.
pub(crate) fn pdf_error(error: anyhow::Error) -> anyhow::Error {
    if format!("{error:#}")
        .to_ascii_lowercase()
        .contains("incorrect password")
    {
        anyhow::anyhow!(pdf_open_password_error())
    } else {
        error
    }
}

fn parse_pdf_page_count(stdout: &[u8]) -> Result<usize> {
    let text = String::from_utf8_lossy(stdout);
    // Info dictionary values precede the real Pages field and can contain line
    // breaks. Taking the last field avoids mistaking a title's text for a count.
    text.lines()
        .rev()
        .find_map(|line| line.strip_prefix("Pages:"))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .context(crate::i18n::tr(
            "Impossibile leggere le pagine del PDF: risposta pdfinfo non valida",
            "Unable to read PDF pages: invalid pdfinfo response",
        ))
}

#[derive(Debug)]
struct PdfProbeOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn read_probe_tail(mut reader: impl Read, limit: usize) -> std::io::Result<Vec<u8>> {
    let mut tail = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(tail),
            Ok(count) => {
                tail.extend_from_slice(&buffer[..count]);
                if tail.len() > limit {
                    tail.drain(..tail.len() - limit);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

/// Drain both pipes while polling the process. Reading only after exit can
/// deadlock on large PDF metadata; retaining only tails keeps memory bounded.
fn run_pdf_probe(
    mut command: Command,
    cancel: &AtomicBool,
    timeout: Duration,
) -> Result<PdfProbeOutput> {
    cancelled(cancel)?;
    hidden(&mut command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().context(crate::i18n::tr(
        "Impossibile avviare il conteggio PDF",
        "Unable to start PDF page counting",
    ))?;
    let job = match ProcessJob::attach(&child) {
        Ok(job) => job,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error).context(crate::i18n::tr(
                "Impossibile isolare il processo di conteggio PDF",
                "Unable to isolate the PDF page-counting process",
            ));
        }
    };
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_reader = thread::spawn(move || read_probe_tail(stdout, PDF_PROBE_STDOUT_LIMIT));
    let stderr_reader = thread::spawn(move || read_probe_tail(stderr, PDF_PROBE_STDERR_LIMIT));
    let started = std::time::Instant::now();
    let result = loop {
        if let Err(error) = cancelled(cancel) {
            break Err(error);
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {}
            Err(error) => break Err(error.into()),
        }
        if started.elapsed() >= timeout {
            break Err(anyhow::anyhow!(crate::trf!(
                "Il conteggio delle pagine PDF ha superato il tempo massimo",
                "PDF page counting timed out"
            )));
        }
        thread::sleep(Duration::from_millis(30));
    };
    if result.is_err() {
        job.terminate();
        let _ = child.kill();
        let _ = child.wait();
    }
    // Close the Windows job before joining readers: helpers may inherit pipes.
    drop(job);
    let stdout = stdout_reader.join();
    let stderr = stderr_reader.join();
    let status = result?;
    let stdout = stdout
        .map_err(|_| {
            anyhow::anyhow!(crate::trf!(
                "Lettura del conteggio PDF interrotta",
                "PDF page-count reading was interrupted"
            ))
        })?
        .context(crate::i18n::tr(
            "Impossibile leggere il conteggio PDF",
            "Unable to read the PDF page count",
        ))?;
    let stderr = stderr
        .map_err(|_| {
            anyhow::anyhow!(crate::trf!(
                "Lettura della diagnostica PDF interrotta",
                "PDF diagnostic reading was interrupted"
            ))
        })?
        .context(crate::i18n::tr(
            "Impossibile leggere la diagnostica PDF",
            "Unable to read PDF diagnostics",
        ))?;
    Ok(PdfProbeOutput {
        status,
        stdout,
        stderr,
    })
}

/// The app's shared quality scale is H.264-oriented. VP9 needs a higher CQ
/// value; retain the saved setting while selecting a practical encoding speed.
fn vp9_policy(stored_quality: u8) -> (u8, u8) {
    (
        stored_quality.saturating_add(9).min(63),
        if stored_quality <= 18 { 2 } else { 4 },
    )
}

const VIDEO_WITHOUT_AUDIO_ERROR: &str = "Il video non contiene una traccia audio da estrarre";

fn video_without_audio_error() -> &'static str {
    crate::i18n::tr(
        VIDEO_WITHOUT_AUDIO_ERROR,
        "The video does not contain an audio track to extract",
    )
}

fn audio_stream_presence(success: bool, stdout: &[u8]) -> Option<bool> {
    if !success {
        return None;
    }
    let output = std::str::from_utf8(stdout).ok()?.trim();
    if output.is_empty() {
        Some(false)
    } else {
        output.parse::<u32>().ok().map(|_| true)
    }
}

fn has_audio_stream(input: &Path, tools: &ToolPaths, cancel: &AtomicBool) -> Result<Option<bool>> {
    cancelled(cancel)?;
    let Some(executable) = tools
        .ffprobe
        .as_ref()
        .filter(|path| crate::tools::executable_available(path))
    else {
        return Ok(None);
    };
    let mut command = Command::new(executable);
    command
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=index",
            "-of",
            "csv=p=0",
        ])
        .arg(input);
    // Reuse the protected bounded runner. Probe failures fall back to FFmpeg;
    // cancellation must propagate rather than be treated as an unknown result.
    let output = run_pdf_probe(command, cancel, Duration::from_secs(5));
    cancelled(cancel)?;
    Ok(output
        .ok()
        .and_then(|output| audio_stream_presence(output.status.success(), &output.stdout)))
}

fn convert_media(
    input: &Path,
    dir: &Path,
    s: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    let extracting_audio =
        MediaKind::from_path(input) == MediaKind::Video && s.format.kind() == MediaKind::Audio;
    let audio_stream = if extracting_audio {
        has_audio_stream(input, tools, cancel)?
    } else {
        None
    };
    if audio_stream == Some(false) {
        bail!(video_without_audio_error());
    }
    let (_stage, temp) = external_temporary(dir, s.format.extension())?;
    // Do not launch the same failing/timed-out optional probe a second time.
    let mut duration = if extracting_audio && audio_stream.is_none() {
        None
    } else {
        duration(input, tools, cancel)
    };
    cancelled(cancel)?;
    let start = s.video.trim_start.unwrap_or(0.);
    if let Some(end) = s.video.trim_end {
        duration = Some(end - start);
    } else {
        duration = duration.map(|d| (d - start).max(0.));
    }
    let mut cmd = Command::new(tools.ffmpeg.as_ref().context(crate::i18n::tr(
        "FFmpeg non trovato",
        "FFmpeg was not found",
    ))?);
    hidden(&mut cmd).args([
        "-hide_banner",
        "-nostdin",
        "-y",
        "-loglevel",
        "error",
        "-progress",
        "pipe:1",
        "-nostats",
    ]);
    if start > 0. {
        cmd.args(["-ss", &start.to_string()]);
    }
    // Local media may contain playlist references; never allow them to open network URLs.
    cmd.args(["-protocol_whitelist", "file,pipe"])
        .arg("-i")
        .arg(input);
    if let Some(end) = s.video.trim_end {
        cmd.args(["-t", &(end - start).to_string()]);
    }
    if s.format.kind() == MediaKind::Audio {
        cmd.args(["-map", "0:a:0", "-vn"]);
        let codec = match s.format {
            OutputFormat::Mp3 => "libmp3lame",
            OutputFormat::Wav => "pcm_s16le",
            OutputFormat::Aiff => "pcm_s16be",
            OutputFormat::Flac => "flac",
            OutputFormat::Aac | OutputFormat::M4a => "aac",
            OutputFormat::Ogg => "libvorbis",
            OutputFormat::Opus => "libopus",
            OutputFormat::Wma => "wmav2",
            OutputFormat::Ac3 => "ac3",
            OutputFormat::Au => "pcm_s16be",
            OutputFormat::Caf => "alac",
            _ => unreachable!(),
        };
        cmd.args(["-c:a", codec]);
        if !matches!(
            s.format,
            OutputFormat::Wav
                | OutputFormat::Aiff
                | OutputFormat::Flac
                | OutputFormat::Au
                | OutputFormat::Caf
        ) {
            cmd.args(["-b:a", &format!("{}k", s.audio.bitrate_kbps)]);
        }
        if s.audio.sample_rate != 0 {
            cmd.args(["-ar", &s.audio.sample_rate.to_string()]);
        }
        if s.audio.channels != 0 {
            cmd.args(["-ac", &s.audio.channels.to_string()]);
        }
        if s.audio.normalize {
            cmd.args(["-af", "loudnorm=I=-16:TP=-1.5:LRA=11"]);
        }
    } else if s.format == OutputFormat::Gif {
        let fps = s.video.fps.unwrap_or(12);
        let width = s.video.width.unwrap_or(640);
        cmd.args(["-an","-filter_complex",&format!("[0:v:0]fps={fps},scale={width}:-1:flags=lanczos,split[a][b];[a]palettegen[p];[b][p]paletteuse[v]"),"-map","[v]","-loop","0"]);
    } else {
        cmd.args(["-map", "0:v:0", "-map", "0:a:0?"]);
        match s.format {
            OutputFormat::Webm => {
                let (crf, cpu_used) = vp9_policy(s.video.crf);
                cmd.args([
                    "-c:v",
                    "libvpx-vp9",
                    "-b:v",
                    "0",
                    "-crf",
                    &crf.to_string(),
                    "-row-mt",
                    "1",
                    "-deadline",
                    "good",
                    "-cpu-used",
                    &cpu_used.to_string(),
                    "-c:a",
                    "libopus",
                ]);
            }
            OutputFormat::Avi => {
                cmd.args([
                    "-c:v",
                    "mpeg4",
                    "-q:v",
                    &((s.video.crf as u32 / 2).clamp(2, 31)).to_string(),
                    "-c:a",
                    "libmp3lame",
                ]);
            }
            OutputFormat::Mpeg | OutputFormat::Wmv | OutputFormat::Flv | OutputFormat::ThreeGp => {
                let (video_codec, audio_codec) = match s.format {
                    OutputFormat::Mpeg => ("mpeg2video", "mp2"),
                    OutputFormat::Wmv => ("wmv2", "wmav2"),
                    OutputFormat::Flv => ("flv", "libmp3lame"),
                    OutputFormat::ThreeGp => ("mpeg4", "aac"),
                    _ => unreachable!(),
                };
                cmd.args([
                    "-c:v",
                    video_codec,
                    "-q:v",
                    &((s.video.crf as u32 / 2).clamp(2, 31)).to_string(),
                    "-c:a",
                    audio_codec,
                ]);
                if s.format == OutputFormat::Mpeg && s.video.fps.is_none() {
                    // MPEG-2 accepts a fixed set of frame rates; arbitrary source
                    // rates (e.g. screen recordings at 12 fps) need a valid default.
                    cmd.args(["-r", "25"]);
                }
                if s.format == OutputFormat::Flv && s.audio.sample_rate == 0 {
                    cmd.args(["-ar", "44100"]);
                }
            }
            _ => {
                cmd.args([
                    "-c:v",
                    if s.video.codec == VideoCodec::H265 && s.format != OutputFormat::Ts {
                        "libx265"
                    } else {
                        "libx264"
                    },
                    "-crf",
                    &s.video.crf.to_string(),
                    "-preset",
                    "medium",
                    "-c:a",
                    "aac",
                ]);
                if s.video.codec == VideoCodec::H265
                    && matches!(
                        s.format,
                        OutputFormat::Mp4 | OutputFormat::Mov | OutputFormat::M4v
                    )
                {
                    cmd.args(["-tag:v", "hvc1"]);
                }
            }
        }
        // Even dimensions keep H.264/H.265 4:2:0 encoders happy for odd source dimensions.
        let scale = s
            .video
            .width
            .map(|w| format!("scale={}:-2", w / 2 * 2))
            .unwrap_or_else(|| "scale=trunc(iw/2)*2:trunc(ih/2)*2".into());
        cmd.args([
            "-vf",
            &scale,
            "-pix_fmt",
            "yuv420p",
            "-b:a",
            &format!("{}k", s.audio.bitrate_kbps),
        ]);
        if let Some(fps) = s.video.fps {
            cmd.args(["-r", &fps.to_string()]);
        }
        if s.audio.sample_rate != 0 {
            cmd.args(["-ar", &s.audio.sample_rate.to_string()]);
        }
        if s.audio.channels != 0 {
            cmd.args(["-ac", &s.audio.channels.to_string()]);
        }
        if s.audio.normalize {
            cmd.args(["-af", "loudnorm=I=-16:TP=-1.5:LRA=11"]);
        }
        if matches!(
            s.format,
            OutputFormat::Mp4 | OutputFormat::Mov | OutputFormat::M4v
        ) {
            cmd.args(["-movflags", "+faststart"]);
        }
        if s.format == OutputFormat::M4v {
            // FFmpeg otherwise infers raw MPEG-4 video from .m4v and drops audio.
            cmd.args(["-f", "mp4"]);
        }
    }
    cmd.arg(temp.path());
    run_process(cmd, cancel, duration, progress).map_err(|error| {
        if extracting_audio
            && format!("{error:#}")
                .to_ascii_lowercase()
                .contains("matches no streams")
        {
            anyhow::anyhow!(video_without_audio_error())
        } else {
            error
        }
    })?;
    cancelled(cancel)?;
    commit(temp, dir, &stem(input), s.format.extension())
}

/// Bounded stderr capture and polled child handle ensure cancellation also works during silent tools.
pub(crate) fn run_process(
    mut cmd: Command,
    cancel: &AtomicBool,
    duration: Option<f64>,
    progress: &mut impl FnMut(Progress),
) -> Result<()> {
    hidden(&mut cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().with_context(|| {
        crate::trf!(
            "Impossibile avviare {}",
            "Unable to start {}",
            cmd.get_program().to_string_lossy()
        )
    })?;
    let job = match ProcessJob::attach(&child) {
        Ok(job) => job,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e).context(crate::i18n::tr(
                "Impossibile isolare il processo di conversione",
                "Unable to isolate the conversion process",
            ));
        }
    };
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let (send, recv) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout)
            .lines()
            .map_while(std::result::Result::ok)
        {
            if let Some(time) = line
                .strip_prefix("out_time_us=")
                .and_then(|v| v.parse::<f64>().ok())
            {
                let _ = send.send(time / 1_000_000.);
            }
        }
    });
    let errors = thread::spawn(move || {
        let mut text = Vec::new();
        let mut buf = [0u8; 4096];
        let mut reader = stderr;
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    text.extend_from_slice(&buf[..n]);
                    if text.len() > 32768 {
                        text.drain(..text.len() - 32768);
                    }
                }
            }
        }
        String::from_utf8_lossy(&text).into_owned()
    });
    let result = wait_child(&mut child, &job, cancel, &recv, duration, progress);
    // Close the job before joining readers, including handles inherited by helper subprocesses.
    drop(job);
    let _ = reader.join();
    let diagnostic = errors.join().unwrap_or_default();
    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => bail!(crate::trf!(
            "Conversione non riuscita: {}",
            "Conversion failed: {}",
            if diagnostic.trim().is_empty() {
                "il convertitore si è interrotto"
            } else {
                diagnostic.trim()
            }
        )),
        Err(e) => Err(e),
    }
}

fn formatted_media_time(seconds: f64) -> String {
    let seconds = if seconds.is_finite() && seconds >= 0.0 {
        seconds as u64
    } else {
        0
    };
    if seconds >= 3600 {
        format!(
            "{}:{:02}:{:02}",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        )
    } else {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}

fn conversion_progress_message(processed: f64, duration: Option<f64>) -> String {
    let processed = formatted_media_time(processed);
    match duration.filter(|duration| duration.is_finite() && *duration >= 0.0) {
        Some(duration) => crate::trf!(
            "Conversione… {processed} di {}",
            "Converting… {processed} of {}",
            formatted_media_time(duration)
        ),
        None => crate::trf!("Conversione… {processed}", "Converting… {processed}"),
    }
}

fn wait_child(
    child: &mut Child,
    job: &ProcessJob,
    cancel: &AtomicBool,
    recv: &mpsc::Receiver<f64>,
    duration: Option<f64>,
    progress: &mut impl FnMut(Progress),
) -> Result<std::process::ExitStatus> {
    loop {
        if cancel.load(Ordering::Relaxed) {
            job.terminate();
            let _ = child.kill();
            let _ = child.wait();
            bail!(crate::trf!("Conversione annullata", "Conversion cancelled"));
        }
        for time in recv.try_iter() {
            progress(Progress {
                fraction: duration
                    .filter(|d| *d > 0.)
                    .map(|d| (time / d).clamp(0., 0.99) as f32)
                    .unwrap_or(0.),
                message: conversion_progress_message(time, duration),
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(e) => {
                job.terminate();
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.into());
            }
        }
        thread::sleep(Duration::from_millis(40));
    }
}

fn convert_pdf(
    input: &Path,
    dir: &Path,
    s: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    if s.format == OutputFormat::Pdf {
        return match s.pdf.action {
            PdfAction::Compress => crate::pdf_ops::compress(input, Some(dir), cancel, progress),
            PdfAction::Split => crate::pdf_ops::split(input, Some(dir), cancel, progress),
            PdfAction::Merge => bail!(crate::trf!(
                "L'unione richiede una selezione di PDF",
                "Merging requires a selection of PDF files"
            )),
        };
    }
    if matches!(
        s.format,
        OutputFormat::Txt | OutputFormat::Docx | OutputFormat::Html
    ) {
        return Ok(vec![crate::documents::convert_pdf_text(
            input, dir, s, tools, cancel, progress,
        )?]);
    }
    const PAGES_PER_BLOCK: usize = 10;
    let page_count = pdf_page_count(input, tools, cancel)?;
    let stage = tempfile::Builder::new()
        .prefix(".the-converter-pages-")
        .tempdir_in(dir)?;
    set_hidden(stage.path())?;
    progress(Progress {
        fraction: 0.05,
        message: crate::trf!(
            "Rendering di {page_count} pagine PDF…",
            "Rendering {page_count} PDF pages…"
        ),
    });
    for first in (1..=page_count).step_by(PAGES_PER_BLOCK) {
        cancelled(cancel)?;
        let last = (first + PAGES_PER_BLOCK - 1).min(page_count);
        let mut cmd = Command::new(tools.pdftoppm.as_ref().context(crate::i18n::tr(
            "pdftoppm non trovato",
            "pdftoppm was not found",
        ))?);
        cmd.arg(if s.format == OutputFormat::Jpg {
            "-jpeg"
        } else {
            "-png"
        })
        .args([
            "-r",
            &s.pdf.dpi.to_string(),
            "-f",
            &first.to_string(),
            "-l",
            &last.to_string(),
        ]);
        if s.format == OutputFormat::Jpg {
            cmd.args(["-jpegopt", &format!("quality={}", s.image.quality)]);
        }
        cmd.arg(input).arg(stage.path().join("page"));
        run_process(cmd, cancel, None, progress).map_err(pdf_error)?;
        cancelled(cancel)?;
        progress(Progress {
            fraction: 0.05 + 0.45 * last as f32 / page_count as f32,
            message: crate::trf!(
                "Renderizzate pagine {first}–{last} di {page_count}",
                "Rendered pages {first}–{last} of {page_count}"
            ),
        });
    }
    cancelled(cancel)?;
    let mut pages: Vec<PathBuf> = fs::read_dir(stage.path())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(s.format.extension()))
        .collect();
    let page_number = |path: &Path| {
        path.file_stem()
            .and_then(|n| n.to_str())
            .and_then(|n| n.rsplit('-').next())
            .and_then(|n| n.parse::<usize>().ok())
    };
    pages.sort_by_key(|path| page_number(path));
    if pages.len() != page_count
        || pages
            .iter()
            .enumerate()
            .any(|(index, path)| page_number(path) != Some(index + 1))
    {
        bail!(crate::trf!(
            "Il numero o l'ordine delle pagine esportate da Poppler non corrisponde al PDF: nessun file è stato pubblicato",
            "The number or order of pages exported by Poppler does not match the PDF: no output was published"
        ));
    }
    let mut outputs = Vec::new();
    let result = (|| -> Result<()> {
        for (i, page) in pages.iter().enumerate() {
            cancelled(cancel)?;
            let temporary = staged_temporary(page, stage.path())?;
            let retained = temporary.as_file().try_clone()?;
            let path = commit(
                temporary,
                dir,
                &format!("{}-pagina-{:03}", stem(input), i + 1),
                s.format.extension(),
            )?;
            outputs.push((path, retained));
            progress(Progress {
                fraction: 0.5 + (i + 1) as f32 / pages.len() as f32 * 0.49,
                message: crate::trf!("Pagina {} di {}", "Page {} of {}", i + 1, pages.len()),
            });
        }
        Ok(())
    })();
    if let Err(e) = result {
        for (path, retained) in &outputs {
            if let Ok(current) = fs::File::open(path)
                && matches!((file_identity(retained), file_identity(&current)), (Ok(a), Ok(b)) if a == b)
            {
                let _ = fs::remove_file(path);
            }
        }
        return Err(e);
    }
    Ok(outputs.into_iter().map(|(path, _)| path).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn hidden_handle_temporary_publishes_visible_without_copying_or_overwriting() {
        use std::os::windows::fs::MetadataExt;
        let directory = tempfile::tempdir().unwrap();
        let existing = directory.path().join("result.txt");
        fs::write(&existing, b"original data").unwrap();
        let mut temporary = temporary(directory.path(), "txt").unwrap();
        assert_ne!(
            temporary.as_file().metadata().unwrap().file_attributes() & 2,
            0
        );
        temporary.write_all(b"completed conversion").unwrap();
        let identity = file_identity(temporary.as_file()).unwrap();
        let output = commit(temporary, directory.path(), "result", "txt").unwrap();
        assert_eq!(output, directory.path().join("result (1).txt"));
        let published = fs::File::open(&output).unwrap();
        assert_eq!(file_identity(&published).unwrap(), identity);
        assert_eq!(published.metadata().unwrap().file_attributes() & 2, 0);
        assert_eq!(fs::read(existing).unwrap(), b"original data");
        assert_eq!(fs::read(output).unwrap(), b"completed conversion");
    }

    #[cfg(windows)]
    #[test]
    fn external_temporary_hides_its_directory_and_cleans_it_after_publication_or_failure() {
        use std::os::windows::fs::MetadataExt;
        let directory = tempfile::tempdir().unwrap();
        let (stage, temporary) = external_temporary(directory.path(), "txt").unwrap();
        let stage_path = stage.path().to_owned();
        assert_ne!(stage.path().metadata().unwrap().file_attributes() & 2, 0);
        assert_eq!(
            temporary.as_file().metadata().unwrap().file_attributes() & 2,
            0
        );
        fs::write(temporary.path(), b"external writer result").unwrap();
        let identity = file_identity(temporary.as_file()).unwrap();
        let output = commit(temporary, directory.path(), "result", "txt").unwrap();
        let published = fs::File::open(output).unwrap();
        assert_eq!(file_identity(&published).unwrap(), identity);
        assert_eq!(published.metadata().unwrap().file_attributes() & 2, 0);
        drop(stage);
        assert!(!stage_path.exists());

        let (stage, temporary) = external_temporary(directory.path(), "tmp").unwrap();
        let stage_path = stage.path().to_owned();
        drop(temporary);
        drop(stage);
        assert!(!stage_path.exists());
    }

    #[test]
    fn audio_probe_presence_distinguishes_confirmed_absence_from_unknown_results() {
        assert_eq!(audio_stream_presence(true, b"\r\n "), Some(false));
        assert_eq!(audio_stream_presence(true, b"0\n"), Some(true));
        assert_eq!(audio_stream_presence(true, b"12\r\n"), Some(true));
        for output in [b"error".as_slice(), b"-1", b"0\n1", &[0xff]] {
            assert_eq!(audio_stream_presence(true, output), None);
        }
        assert_eq!(audio_stream_presence(false, b""), None);
        assert_eq!(audio_stream_presence(false, b"0"), None);
    }

    #[test]
    fn cancelled_audio_probe_does_not_return_an_unknown_result() {
        let error = has_audio_stream(
            Path::new("input.mp4"),
            &ToolPaths::default(),
            &AtomicBool::new(true),
        )
        .unwrap_err();
        assert!(error.to_string().contains("annullata"));
    }

    #[test]
    fn vp9_policy_keeps_quality_order_and_valid_codec_limits() {
        let high = vp9_policy(18);
        let balanced = vp9_policy(23);
        let compact = vp9_policy(28);
        assert_eq!((high.0, balanced.0, compact.0), (27, 32, 37));
        assert!(high.1 < balanced.1);
        assert_eq!(balanced.1, compact.1);
        let mapped: Vec<_> = (0..=u8::MAX).map(|quality| vp9_policy(quality).0).collect();
        assert!(mapped.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(mapped.iter().all(|quality| *quality <= 63));
        assert_eq!(vp9_policy(u8::MAX).0, 63);
    }

    #[test]
    fn adopting_staged_files_never_accepts_an_original_outside_the_staging_directory() {
        let directory = tempfile::tempdir().unwrap();
        let stage = directory.path().join("stage");
        fs::create_dir(&stage).unwrap();
        let original = directory.path().join("original.txt");
        fs::write(&original, b"original user data").unwrap();
        assert!(staged_temporary(&original, &stage).is_err());
        assert_eq!(fs::read(original).unwrap(), b"original user data");
    }

    #[test]
    fn failed_adopted_publication_keeps_the_owned_source_for_staging_cleanup() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.tmp");
        let destination = directory.path().join("existing.txt");
        fs::write(&source, b"completed result").unwrap();
        fs::write(&destination, b"unrelated existing data").unwrap();
        let temporary = staged_temporary(&source, directory.path()).unwrap();
        let error = temporary.persist_noclobber(&destination).unwrap_err();
        assert_eq!(error.error.kind(), std::io::ErrorKind::AlreadyExists);
        drop(error);
        assert_eq!(fs::read(source).unwrap(), b"completed result");
        assert_eq!(fs::read(destination).unwrap(), b"unrelated existing data");
    }

    #[test]
    fn pdfinfo_count_uses_the_real_field_after_metadata_and_rejects_invalid_numbers() {
        assert_eq!(
            parse_pdf_page_count(
                b"Title: Invoice\r\nPages: 9999\r\nPages:           12\r\nEncrypted: no\r\n"
            )
            .unwrap(),
            12
        );
        for response in [
            b"Page count: 12".as_slice(),
            b"Pages: -1",
            b"Pages: 12 extra",
            b"Pages: 999999999999999999999999999999999999999999999",
        ] {
            assert!(parse_pdf_page_count(response).is_err());
        }
    }

    #[cfg(windows)]
    fn pdf_probe_test_command(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args([
            "--exact",
            "engine::tests::pdf_probe_fixture_process",
            "--nocapture",
            "--test-threads=1",
        ]);
        command.env("THE_CONVERTER_TEST_PDF_PROBE_MODE", mode);
        command
    }

    #[cfg(windows)]
    #[test]
    fn pdf_probe_fixture_process() {
        let Ok(mode) = std::env::var("THE_CONVERTER_TEST_PDF_PROBE_MODE") else {
            return;
        };
        match mode.as_str() {
            "metadata" => {
                let mut stdout = std::io::stdout().lock();
                stdout.write_all(&vec![b'x'; 262144]).unwrap();
                stdout.write_all(b"\nPages: 12\n").unwrap();
                stdout.flush().unwrap();
                let mut stderr = std::io::stderr().lock();
                stderr.write_all(&vec![b'e'; 131072]).unwrap();
                stderr.flush().unwrap();
            }
            "silent" => {
                if let Some(path) = std::env::var_os("THE_CONVERTER_TEST_PDF_PROBE_CHILD_PID") {
                    fs::write(path, std::process::id().to_string()).unwrap();
                }
                thread::sleep(Duration::from_secs(30));
            }
            "descendant" => {
                let mut child = pdf_probe_test_command("silent");
                assert!(hidden(&mut child).status().unwrap().success());
            }
            _ => panic!("Unknown PDF probe fixture mode: {mode}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn pdf_probe_drains_large_metadata_and_keeps_both_streams_bounded() {
        let command = pdf_probe_test_command("metadata");
        let output = run_pdf_probe(command, &AtomicBool::new(false), PDF_PROBE_TIMEOUT).unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.len(), PDF_PROBE_STDOUT_LIMIT);
        assert_eq!(output.stderr.len(), PDF_PROBE_STDERR_LIMIT);
        assert!(output.stderr.iter().all(|&byte| byte == b'e'));
        assert_eq!(parse_pdf_page_count(&output.stdout).unwrap(), 12);
    }

    #[cfg(windows)]
    #[test]
    fn pdf_probe_timeout_and_cancellation_do_not_wait_for_a_silent_tool() {
        let start = std::time::Instant::now();
        let result = run_pdf_probe(
            pdf_probe_test_command("silent"),
            &AtomicBool::new(false),
            Duration::from_millis(150),
        );
        assert!(result.unwrap_err().to_string().contains("tempo massimo"));
        assert!(start.elapsed() < Duration::from_secs(3));
        let cancel = AtomicBool::new(false);
        let start = std::time::Instant::now();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(150));
                cancel.store(true, Ordering::Relaxed);
            });
            let result =
                run_pdf_probe(pdf_probe_test_command("silent"), &cancel, PDF_PROBE_TIMEOUT);
            assert!(result.unwrap_err().to_string().contains("annullata"));
        });
        assert!(start.elapsed() < Duration::from_secs(3));
    }

    #[cfg(windows)]
    #[test]
    fn pdf_probe_closes_its_job_and_terminates_descendants_on_cancellation() {
        use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
        use windows_sys::Win32::{
            Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
            System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
        };
        let directory = tempfile::tempdir().unwrap();
        let child_pid = directory.path().join("child.pid");
        let mut command = pdf_probe_test_command("descendant");
        command.env("THE_CONVERTER_TEST_PDF_PROBE_CHILD_PID", &child_pid);
        let cancel = AtomicBool::new(false);
        std::thread::scope(|scope| {
            let observer = scope.spawn(|| {
                let observed = (|| -> Result<OwnedHandle> {
                    let started = std::time::Instant::now();
                    while started.elapsed() < Duration::from_secs(15) {
                        if let Some(pid) = fs::read_to_string(&child_pid)
                            .ok()
                            .and_then(|value| value.parse::<u32>().ok())
                        {
                            let raw = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
                            if raw.is_null() {
                                bail!("Cannot observe the descendant fixture process");
                            }
                            let process = unsafe { OwnedHandle::from_raw_handle(raw) };
                            if unsafe { WaitForSingleObject(process.as_raw_handle(), 0) }
                                != WAIT_TIMEOUT
                            {
                                bail!("The descendant fixture exited before cancellation");
                            }
                            return Ok(process);
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    bail!("The descendant fixture did not become ready");
                })();
                let cancelled_at = std::time::Instant::now();
                cancel.store(true, Ordering::Relaxed);
                observed.map(|process| (process, cancelled_at))
            });
            // Startup readiness has its own test budget; cancellation must still
            // close the inherited pipes and kill an already running descendant promptly.
            let result = run_pdf_probe(command, &cancel, Duration::from_secs(30));
            let returned_at = std::time::Instant::now();
            let (process, cancelled_at) = observer.join().unwrap().unwrap();
            assert!(result.unwrap_err().to_string().contains("annullata"));
            assert!(returned_at.duration_since(cancelled_at) < Duration::from_secs(3));
            assert_eq!(
                unsafe { WaitForSingleObject(process.as_raw_handle(), 1000) },
                WAIT_OBJECT_0
            );
        });
    }

    #[cfg(windows)]
    #[test]
    fn stored_probe_script_is_not_executed() {
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("ffprobe.cmd");
        fs::write(
            &script,
            "@echo off\r\n@echo executed>\"%~dp0executed.txt\"\r\n@echo 1.0\r\n",
        )
        .unwrap();
        let tools = ToolPaths {
            ffprobe: Some(script),
            ..Default::default()
        };
        assert!(duration(Path::new("input.wav"), &tools, &AtomicBool::new(false)).is_none());
        assert!(!directory.path().join("executed.txt").exists());
    }

    #[test]
    fn running_ffmpeg_is_cancelled_promptly() {
        let tools = ToolPaths::discover();
        let Some(ffmpeg) = tools.ffmpeg else {
            return;
        };
        let mut cmd = Command::new(ffmpeg);
        cmd.args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-progress",
            "pipe:1",
            "-re",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=30",
            "-f",
            "null",
            "-",
        ]);
        let cancel = AtomicBool::new(false);
        let start = std::time::Instant::now();
        let result = run_process(cmd, &cancel, Some(30.), &mut |_| {
            cancel.store(true, Ordering::Relaxed);
        });
        assert!(result.is_err());
        assert!(cancel.load(Ordering::Relaxed));
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "Cancellation took {:?}",
            start.elapsed()
        );
    }
}

#[cfg(test)]
mod progress_message_tests {
    use super::*;

    #[test]
    fn progress_reports_processed_media_and_known_trimmed_duration() {
        assert_eq!(
            conversion_progress_message(316.9, Some(751.9)),
            "Conversione… 5:16 di 12:31"
        );
        assert_eq!(
            conversion_progress_message(3796.0, Some(7500.0)),
            "Conversione… 1:03:16 di 2:05:00"
        );
        assert_eq!(
            conversion_progress_message(2.4, Some(5.0)),
            "Conversione… 0:02 di 0:05"
        );
        assert_eq!(
            conversion_progress_message(0.0, Some(0.0)),
            "Conversione… 0:00 di 0:00"
        );
    }

    #[test]
    fn unknown_or_invalid_total_durations_do_not_display_a_misleading_total() {
        for duration in [None, Some(-1.0), Some(f64::NAN), Some(f64::INFINITY)] {
            assert_eq!(
                conversion_progress_message(316.0, duration),
                "Conversione… 5:16"
            );
        }
    }

    #[test]
    fn invalid_encoder_timestamps_are_safe_and_do_not_wrap_the_clock() {
        for seconds in [-0.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(formatted_media_time(seconds), "0:00");
        }
        assert_eq!(formatted_media_time(59.999), "0:59");
        assert_eq!(formatted_media_time(60.0), "1:00");
        assert_eq!(formatted_media_time(3600.0), "1:00:00");
    }
}
