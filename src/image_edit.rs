//! Image recipes and exact, reusable encoded previews. No operation changes an input file.
use crate::{engine::Progress, model::OutputFormat, tools::ToolPaths};
use anyhow::{Context, Result, bail, ensure};
use image::{DynamicImage, ImageDecoder, ImageFormat, Rgba, RgbaImage, imageops::FilterType};
use serde::{Deserialize, Serialize};
use std::{
    io::{Cursor, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Bounds apply to the editor, not to existing conversions without an edit recipe.
pub const MAX_PIXELS: u64 = 40_000_000;
pub const MAX_SIDE: u32 = 32_768;
pub const MAX_WEBP_SIDE: u32 = 16_383;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ImageEditSettings {
    pub crop: Option<NormalizedCrop>,
    pub resize: Option<ResizeSettings>,
    pub canvas: Option<CanvasSettings>,
    pub max_bytes: Option<u64>,
    pub allow_downsize: bool,
}

/// Edges in the oriented source, from zero to one. Rounded to pixel boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NormalizedCrop {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResizeSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// With both dimensions present, fit inside that box without stretching.
    pub keep_aspect: bool,
}
impl Default for ResizeSettings {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            keep_aspect: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CanvasAnchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    #[default]
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CanvasFit {
    #[default]
    None,
    Contain,
    Cover,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CanvasSettings {
    pub width: u32,
    pub height: u32,
    pub background: [u8; 4],
    pub anchor: CanvasAnchor,
    pub offset_x: i32,
    pub offset_y: i32,
    pub fit: CanvasFit,
}
impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            background: [0, 0, 0, 0],
            anchor: CanvasAnchor::Center,
            offset_x: 0,
            offset_y: 0,
            fit: CanvasFit::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub pixels: Arc<RgbaImage>,
}

#[derive(Debug, Clone)]
pub struct EncodedImage {
    pub bytes: Vec<u8>,
    /// Decoded from `bytes`, so the preview includes actual compression artifacts.
    pub decoded_preview: RgbaImage,
    pub width: u32,
    pub height: u32,
    pub effective_quality: Option<u8>,
    pub target_met: bool,
    pub format: OutputFormat,
}

fn cancelled(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Relaxed),
        "Elaborazione immagine annullata"
    );
    Ok(())
}

fn dimensions(width: u32, height: u32) -> Result<()> {
    ensure!(
        width > 0 && height > 0 && width <= MAX_SIDE && height <= MAX_SIDE,
        "Dimensioni immagine non valide (1–{MAX_SIDE} pixel per lato)"
    );
    ensure!(
        u64::from(width) * u64::from(height) <= MAX_PIXELS,
        "L’editor supporta al massimo {MAX_PIXELS} pixel"
    );
    Ok(())
}

pub fn validate(settings: &ImageEditSettings) -> Result<()> {
    if let Some(crop) = settings.crop {
        ensure!(
            [crop.left, crop.top, crop.right, crop.bottom]
                .into_iter()
                .all(|edge| edge.is_finite() && (0.0..=1.0).contains(&edge))
                && crop.left < crop.right
                && crop.top < crop.bottom,
            "Ritaglio non valido: seleziona un’area interna all’immagine"
        );
    }
    if let Some(resize) = &settings.resize {
        ensure!(
            resize.width.is_some() || resize.height.is_some(),
            "Indica una dimensione per la scala"
        );
        for value in [resize.width, resize.height].into_iter().flatten() {
            ensure!(
                (1..=MAX_SIDE).contains(&value),
                "Dimensione della scala non valida"
            );
        }
        if !resize.keep_aspect
            && let (Some(width), Some(height)) = (resize.width, resize.height)
        {
            dimensions(width, height)?;
        }
    }
    if let Some(canvas) = &settings.canvas {
        dimensions(canvas.width, canvas.height)?;
    }
    ensure!(
        settings.max_bytes != Some(0),
        "Il peso massimo deve essere maggiore di zero"
    );
    Ok(())
}

/// Load on a worker. Modern HEIF inputs use the same optional decoder as conversions.
pub fn load_image(
    input: &Path,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<LoadedImage> {
    cancelled(cancel)?;
    let workspace = if crate::modern_images::is_modern_input(input) {
        Some(
            tempfile::Builder::new()
                .prefix("convert-here-image-")
                .tempdir()?,
        )
    } else {
        None
    };
    let decoded = if let Some(workspace) = &workspace {
        crate::modern_images::decode(input, workspace.path(), tools, cancel, &mut progress)?
    } else {
        input.to_owned()
    };
    let mut decoder = image::ImageReader::open(decoded)?
        .with_guessed_format()?
        .into_decoder()
        .context("Immagine non leggibile o codec non supportato")?;
    let (width, height) = decoder.dimensions();
    dimensions(width, height)?;
    if let Some(limit) = image::Limits::default().max_alloc {
        ensure!(
            decoder.total_bytes() <= limit,
            "L’immagine supera il limite di memoria per la decodifica"
        );
    }
    let orientation = decoder
        .orientation()
        .context("Orientamento immagine non leggibile")?;
    cancelled(cancel)?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    cancelled(cancel)?;
    progress(Progress {
        fraction: 1.0,
        message: "Immagine pronta per l’editor".into(),
    });
    Ok(LoadedImage {
        pixels: Arc::new(image.into_rgba8()),
    })
}

fn scaled_size(width: u32, height: u32, scale: f64) -> Result<(u32, u32)> {
    ensure!(
        scale.is_finite() && scale > 0.0,
        "Scala immagine non valida"
    );
    let width = (f64::from(width) * scale).round().max(1.0);
    let height = (f64::from(height) * scale).round().max(1.0);
    ensure!(
        width <= f64::from(MAX_SIDE) && height <= f64::from(MAX_SIDE),
        "Immagine scalata troppo grande"
    );
    dimensions(width as u32, height as u32)?;
    Ok((width as u32, height as u32))
}

fn resize_size(width: u32, height: u32, resize: &ResizeSettings) -> Result<(u32, u32)> {
    match (resize.width, resize.height) {
        (Some(w), Some(h)) if !resize.keep_aspect => {
            dimensions(w, h)?;
            Ok((w, h))
        }
        (Some(w), Some(h)) => scaled_size(
            width,
            height,
            (f64::from(w) / f64::from(width)).min(f64::from(h) / f64::from(height)),
        ),
        (Some(w), None) => scaled_size(width, height, f64::from(w) / f64::from(width)),
        (None, Some(h)) => scaled_size(width, height, f64::from(h) / f64::from(height)),
        _ => bail!("Indica una dimensione per la scala"),
    }
}

fn anchor_offset(outer: u32, inner: u32, alignment: i8) -> i64 {
    let delta = i64::from(outer) - i64::from(inner);
    match alignment {
        -1 => 0,
        1 => delta,
        _ => delta / 2,
    }
}

/// Provide premultiplied samples without allocating a second source raster.
/// Choosing the shorter intermediate axis also bounds extreme stretch requests.
fn resized(pixels: &RgbaImage, width: u32, height: u32, cancel: &AtomicBool) -> Result<RgbaImage> {
    dimensions(width, height)?;
    cancelled(cancel)?;
    if pixels.dimensions() == (width, height) {
        return Ok(pixels.clone());
    }
    struct Samples<'a> {
        pixels: &'a RgbaImage,
        transpose: bool,
    }
    impl image::GenericImageView for Samples<'_> {
        type Pixel = Rgba<u16>;
        fn dimensions(&self) -> (u32, u32) {
            let (w, h) = self.pixels.dimensions();
            if self.transpose { (h, w) } else { (w, h) }
        }
        fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
            let p = if self.transpose {
                self.pixels.get_pixel(y, x)
            } else {
                self.pixels.get_pixel(x, y)
            };
            let a = u16::from(p[3]);
            Rgba([
                u16::from(p[0]) * a,
                u16::from(p[1]) * a,
                u16::from(p[2]) * a,
                255 * a,
            ])
        }
    }
    let transpose = u64::from(pixels.height()) * u64::from(width)
        < u64::from(pixels.width()) * u64::from(height);
    let samples = Samples { pixels, transpose };
    let (w, h) = if transpose {
        (height, width)
    } else {
        (width, height)
    };
    let scaled = image::imageops::resize(&samples, w, h, FilterType::Lanczos3);
    cancelled(cancel)?;
    Ok(RgbaImage::from_fn(width, height, |x, y| {
        let p = if transpose {
            scaled.get_pixel(y, x)
        } else {
            scaled.get_pixel(x, y)
        };
        let alpha = u32::from(p[3]);
        if alpha == 0 {
            return Rgba([0, 0, 0, 0]);
        }
        let rgb = [0, 1, 2].map(|c| ((u32::from(p[c]) * 255 + alpha / 2) / alpha).min(255) as u8);
        Rgba([rgb[0], rgb[1], rgb[2], ((alpha + 127) / 255).min(255) as u8])
    }))
}

/// Geometry is deterministic and independent from the encoded output format.
pub fn render(
    source: &RgbaImage,
    settings: &ImageEditSettings,
    cancel: &AtomicBool,
) -> Result<RgbaImage> {
    cancelled(cancel)?;
    validate(settings)?;
    dimensions(source.width(), source.height())?;
    let mut pixels = if let Some(crop) = settings.crop {
        let left = (crop.left * f64::from(source.width())).round() as u32;
        let right = (crop.right * f64::from(source.width())).round() as u32;
        let top = (crop.top * f64::from(source.height())).round() as u32;
        let bottom = (crop.bottom * f64::from(source.height())).round() as u32;
        ensure!(
            left < right && top < bottom,
            "Il ritaglio è più piccolo di un pixel"
        );
        image::imageops::crop_imm(source, left, top, right - left, bottom - top).to_image()
    } else {
        source.clone()
    };
    if let Some(resize) = &settings.resize {
        let (width, height) = resize_size(pixels.width(), pixels.height(), resize)?;
        cancelled(cancel)?;
        if pixels.dimensions() != (width, height) {
            pixels = resized(&pixels, width, height, cancel)?;
        }
    }
    if let Some(canvas) = &settings.canvas {
        cancelled(cancel)?;
        if canvas.fit != CanvasFit::None {
            let x = f64::from(canvas.width) / f64::from(pixels.width());
            let y = f64::from(canvas.height) / f64::from(pixels.height());
            let scale = if canvas.fit == CanvasFit::Contain {
                x.min(y)
            } else {
                x.max(y)
            };
            let (width, height) = scaled_size(pixels.width(), pixels.height(), scale)?;
            pixels = resized(&pixels, width, height, cancel)?;
        }
        let horizontal = match canvas.anchor {
            CanvasAnchor::TopLeft | CanvasAnchor::Left | CanvasAnchor::BottomLeft => -1,
            CanvasAnchor::TopRight | CanvasAnchor::Right | CanvasAnchor::BottomRight => 1,
            _ => 0,
        };
        let vertical = match canvas.anchor {
            CanvasAnchor::TopLeft | CanvasAnchor::Top | CanvasAnchor::TopRight => -1,
            CanvasAnchor::BottomLeft | CanvasAnchor::Bottom | CanvasAnchor::BottomRight => 1,
            _ => 0,
        };
        let x =
            anchor_offset(canvas.width, pixels.width(), horizontal) + i64::from(canvas.offset_x);
        let y =
            anchor_offset(canvas.height, pixels.height(), vertical) + i64::from(canvas.offset_y);
        let mut output =
            RgbaImage::from_pixel(canvas.width, canvas.height, Rgba(canvas.background));
        image::imageops::overlay(&mut output, &pixels, x, y);
        pixels = output;
    }
    cancelled(cancel)?;
    Ok(pixels)
}

fn flattened(pixels: &RgbaImage) -> image::RgbImage {
    image::RgbImage::from_fn(pixels.width(), pixels.height(), |x, y| {
        let pixel = pixels.get_pixel(x, y);
        let a = u32::from(pixel[3]);
        image::Rgb(
            [0, 1, 2].map(|channel| {
                ((u32::from(pixel[channel]) * a + 255 * (255 - a) + 127) / 255) as u8
            }),
        )
    })
}

fn webp(pixels: &RgbaImage, quality: u8) -> Result<Vec<u8>> {
    ensure!(
        pixels.width() <= MAX_WEBP_SIDE && pixels.height() <= MAX_WEBP_SIDE,
        "WebP supporta al massimo {MAX_WEBP_SIDE} pixel per lato"
    );
    // The C encoder reads exactly width*height*4 bytes from a validated image buffer.
    struct Memory(*mut u8);
    impl Drop for Memory {
        fn drop(&mut self) {
            unsafe {
                libwebp_sys::WebPFree(self.0.cast());
            }
        }
    }
    let mut output = Memory(std::ptr::null_mut());
    let size = unsafe {
        libwebp_sys::WebPEncodeRGBA(
            pixels.as_raw().as_ptr(),
            pixels.width() as i32,
            pixels.height() as i32,
            (pixels.width() * 4) as i32,
            f32::from(quality),
            &mut output.0,
        )
    };
    ensure!(
        size > 0 && !output.0.is_null(),
        "L’encoder WebP non ha prodotto un’immagine valida"
    );
    // Memory is allocated by libwebp and remains owned until the copied bytes are complete.
    Ok(unsafe { std::slice::from_raw_parts(output.0, size) }.to_vec())
}

fn encode_once(
    pixels: &RgbaImage,
    format: OutputFormat,
    quality: u8,
    cancel: &AtomicBool,
) -> Result<Vec<u8>> {
    cancelled(cancel)?;
    let bytes = match format {
        OutputFormat::Jpg => {
            let mut bytes = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, quality)
                .encode_image(&flattened(pixels))?;
            bytes
        }
        OutputFormat::Webp => webp(pixels, quality)?,
        OutputFormat::Png => {
            let mut bytes = Cursor::new(Vec::new());
            DynamicImage::ImageRgba8(pixels.clone()).write_to(&mut bytes, ImageFormat::Png)?;
            bytes.into_inner()
        }
        _ => bail!(
            "L’editor esporta immagini PNG, JPEG e WebP; l’impaginazione PDF usa le opzioni pagina"
        ),
    };
    cancelled(cancel)?;
    Ok(bytes)
}

struct Candidate {
    bytes: Vec<u8>,
    quality: Option<u8>,
}

fn budget_encode(
    pixels: &RgbaImage,
    format: OutputFormat,
    quality: u8,
    cap: Option<u64>,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<Candidate> {
    let mut probe = |quality: u8| -> Result<Candidate> {
        progress(Progress {
            fraction: 0.5,
            message: format!("Verifica del peso, qualità {quality}…"),
        });
        Ok(Candidate {
            bytes: encode_once(pixels, format, quality, cancel)?,
            quality: (format != OutputFormat::Png).then_some(quality),
        })
    };
    let initial = probe(quality)?;
    let Some(cap) = cap else { return Ok(initial) };
    if initial.bytes.len() as u64 <= cap || format == OutputFormat::Png {
        return Ok(initial);
    }
    let minimum = probe(1)?;
    if minimum.bytes.len() as u64 > cap {
        return Ok(minimum);
    }
    let mut best = minimum;
    let mut low = 2u8;
    let mut high = quality.saturating_sub(1);
    // Quality/size is not perfectly monotonic: retain only buffers proven to fit.
    while low <= high {
        cancelled(cancel)?;
        let middle = low + (high - low) / 2;
        let candidate = probe(middle)?;
        if candidate.bytes.len() as u64 <= cap {
            best = candidate;
            low = middle + 1;
        } else {
            high = middle.saturating_sub(1)
        }
    }
    Ok(best)
}

/// Encode on a worker. A missed cap is reported honestly; saving refuses that result.
/// Optional downsize scales the complete rendered image, including its canvas.
pub fn encode(
    pixels: &RgbaImage,
    format: OutputFormat,
    quality: u8,
    settings: &ImageEditSettings,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<EncodedImage> {
    cancelled(cancel)?;
    validate(settings)?;
    dimensions(pixels.width(), pixels.height())?;
    ensure!(
        (1..=100).contains(&quality),
        "Qualità immagine non valida (1–100)"
    );
    let mut working = pixels.clone();
    let mut candidate = budget_encode(
        &working,
        format,
        quality,
        settings.max_bytes,
        cancel,
        &mut progress,
    )?;
    if settings.allow_downsize
        && let Some(cap) = settings.max_bytes
    {
        for _ in 0..24 {
            if candidate.bytes.len() as u64 <= cap || working.dimensions() == (1, 1) {
                break;
            }
            cancelled(cancel)?;
            let ratio = ((cap as f64 / candidate.bytes.len() as f64).sqrt() * 0.9).clamp(0.1, 0.8);
            let width = ((f64::from(working.width()) * ratio).floor() as u32).max(1);
            let height = ((f64::from(working.height()) * ratio).floor() as u32).max(1);
            working = resized(pixels, width, height, cancel)?;
            candidate = budget_encode(
                &working,
                format,
                quality,
                settings.max_bytes,
                cancel,
                &mut progress,
            )?;
        }
    }
    let target_met = settings
        .max_bytes
        .is_none_or(|cap| candidate.bytes.len() as u64 <= cap);
    cancelled(cancel)?;
    let decoded_preview = image::load_from_memory(&candidate.bytes)?.into_rgba8();
    progress(Progress {
        fraction: 1.0,
        message: if target_met {
            "Anteprima e peso verificati"
        } else {
            "Peso massimo non raggiunto: riduci le dimensioni o cambia formato"
        }
        .into(),
    });
    Ok(EncodedImage {
        bytes: candidate.bytes,
        width: decoded_preview.width(),
        height: decoded_preview.height(),
        decoded_preview,
        effective_quality: candidate.quality,
        target_met,
        format,
    })
}

/// Publish exact prepared bytes at an exact path. Existing files, aliases and
/// hardlinks are never overwritten, including when the destination races us.
pub fn save_bytes(
    input: Option<&Path>,
    output_path: &Path,
    bytes: &[u8],
    cancel: &AtomicBool,
) -> Result<PathBuf> {
    cancelled(cancel)?;
    ensure!(!bytes.is_empty(), "Nessun contenuto da salvare");
    if let (Some(input), Ok(output)) = (input, output_path.canonicalize()) {
        ensure!(
            input.canonicalize().ok().as_ref() != Some(&output),
            "Scegli un nome diverso dall’immagine originale"
        );
    }
    let parent = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    ensure!(parent.is_dir(), "La cartella di destinazione non esiste");
    ensure!(
        !output_path.try_exists()?,
        "Il file di destinazione esiste già: scegli un altro nome"
    );
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    crate::platform::set_hidden(temporary.path())?;
    for chunk in bytes.chunks(65536) {
        cancelled(cancel)?;
        temporary.write_all(chunk)?;
    }
    temporary.as_file().sync_all()?;
    cancelled(cancel)?;
    temporary
        .persist_noclobber(output_path)
        .map_err(|error| error.error)
        .context("Salvataggio non riuscito: nessun file esistente è stato sovrascritto")?;
    // tempfile clears Hidden/Temporary on Windows before its atomic publication.
    Ok(output_path.to_owned())
}

pub fn save_encoded(
    input: &Path,
    output_path: &Path,
    encoded: &EncodedImage,
    cancel: &AtomicBool,
) -> Result<PathBuf> {
    ensure!(
        encoded.target_met,
        "Il file supera il peso massimo: modifica qualità o dimensioni prima di salvarlo"
    );
    save_bytes(Some(input), output_path, &encoded.bytes, cancel)
}
