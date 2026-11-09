//! Native, plain image/text pages. Prepared PDF bytes are also the saved result.
use crate::{
    engine::{Progress, cancelled, commit, external_temporary, stem},
    model::{ConversionSettings, OutputFormat},
};
use ab_glyph::{Font, FontRef, GlyphId, PxScale, point};
use anyhow::{Context, Result, bail, ensure};
use image::{RgbaImage, imageops::FilterType};
use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, atomic::AtomicBool},
};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

const PT_PER_MM: f32 = 72.0 / 25.4;
const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;
const MAX_GLYPHS: usize = 1_000_000;
const MAX_PAGES: usize = 1000;
const MAX_PIXELS: u64 = 40_000_000;
const MAX_IMAGE_BYTES: usize = 96 * 1024 * 1024;
const MAX_PDF_BYTES: usize = 128 * 1024 * 1024;
const MAX_PREVIEW_SIDE: u32 = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaperSize {
    #[default]
    A4,
    A5,
}
impl PaperSize {
    pub fn dimensions_mm(self) -> (f32, f32) {
        match self {
            Self::A4 => (210.0, 297.0),
            Self::A5 => (148.0, 210.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PageOrientation {
    #[default]
    Portrait,
    Landscape,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ImageFit {
    #[default]
    Contain,
    Cover,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PageLayoutSettings {
    pub paper: PaperSize,
    pub orientation: PageOrientation,
    pub margin_mm: f32,
    pub fit: ImageFit,
    /// Maximum image sampling density. Text and page dimensions stay vector units.
    pub dpi: u32,
    pub font_size_pt: f32,
    /// Requested baseline spacing relative to the font size, at least its metrics height.
    pub line_height: f32,
    pub align: TextAlign,
}
impl Default for PageLayoutSettings {
    fn default() -> Self {
        Self {
            paper: PaperSize::A4,
            orientation: PageOrientation::Portrait,
            margin_mm: 12.0,
            fit: ImageFit::Contain,
            dpi: 300,
            font_size_pt: 12.0,
            line_height: 1.25,
            align: TextAlign::Left,
        }
    }
}
impl PageLayoutSettings {
    pub fn page_size_points(&self) -> (f32, f32) {
        let (width, height) = self.paper.dimensions_mm();
        let (width, height) = match self.orientation {
            PageOrientation::Portrait => (width, height),
            PageOrientation::Landscape => (height, width),
        };
        (width * PT_PER_MM, height * PT_PER_MM)
    }

    pub fn validate(&self) -> Result<()> {
        let (width, height) = self.paper.dimensions_mm();
        ensure!(
            self.margin_mm.is_finite()
                && self.margin_mm >= 0.0
                && self.margin_mm * 2.0 < width.min(height) - 1.0,
            "Il margine deve lasciare spazio all'interno della pagina"
        );
        ensure!(
            (72..=600).contains(&self.dpi),
            "DPI pagina: scegli un valore da 72 a 600"
        );
        ensure!(
            self.font_size_pt.is_finite() && (6.0..=96.0).contains(&self.font_size_pt),
            "Dimensione testo: scegli un valore da 6 a 96 pt"
        );
        ensure!(
            self.line_height.is_finite() && (1.0..=3.0).contains(&self.line_height),
            "Interlinea: scegli un valore da 1 a 3"
        );
        Ok(())
    }

    fn content_rect(&self) -> Rect {
        let (width, height) = self.page_size_points();
        let margin = self.margin_mm * PT_PER_MM;
        Rect {
            x: margin,
            y: margin,
            width: width - 2.0 * margin,
            height: height - 2.0 * margin,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PageInput {
    Image(Arc<RgbaImage>),
    Text(Arc<str>),
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}
#[derive(Debug)]
struct ImagePage {
    jpeg: Vec<u8>,
    width: u32,
    height: u32,
    rect: Rect,
}
#[derive(Debug, Clone, Copy)]
struct GlyphPosition {
    cid: u16,
    gid: u16,
    x: f32,
}
#[derive(Debug)]
struct TextLine {
    x: f32,
    baseline: f32,
    glyphs: Vec<GlyphPosition>,
}
#[derive(Debug)]
enum Page {
    Image(ImagePage),
    Text(Vec<TextLine>),
}
#[derive(Debug, Clone, Copy)]
struct FontGlyph {
    cid: u16,
    gid: u16,
    advance: f32,
}

#[derive(Debug)]
pub struct PreparedPageDocument {
    bytes: Vec<u8>,
    pages: Vec<Page>,
    width: f32,
    height: f32,
    font_size_pt: f32,
}
impl PreparedPageDocument {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn exact_bytes(&self) -> u64 {
        self.bytes.len() as u64
    }
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Zero-based page index. Uses the actual embedded JPEG and the PDF's glyph positions.
    pub fn preview(
        &self,
        page_index: usize,
        max_side: u32,
        cancel: &AtomicBool,
    ) -> Result<RgbaImage> {
        cancelled(cancel)?;
        ensure!(
            (1..=MAX_PREVIEW_SIDE).contains(&max_side),
            "Anteprima: il lato massimo deve essere da 1 a 2048 pixel"
        );
        let page = self
            .pages
            .get(page_index)
            .context("Pagina di anteprima non disponibile")?;
        let scale = max_side as f32 / self.width.max(self.height);
        let width = (self.width * scale).round().max(1.0) as u32;
        let height = (self.height * scale).round().max(1.0) as u32;
        let mut canvas = RgbaImage::from_pixel(width, height, image::Rgba([255; 4]));
        match page {
            Page::Image(page) => {
                let pixels = image::load_from_memory(&page.jpeg)?.to_rgba8();
                let (x, y) = (
                    (page.rect.x * scale).round() as i64,
                    (page.rect.y * scale).round() as i64,
                );
                let image_width = (page.rect.width * scale).round().max(1.0) as u32;
                let image_height = (page.rect.height * scale).round().max(1.0) as u32;
                let pixels = image::imageops::resize(
                    &pixels,
                    image_width,
                    image_height,
                    FilterType::Lanczos3,
                );
                cancelled(cancel)?;
                image::imageops::overlay(&mut canvas, &pixels, x, y);
            }
            Page::Text(lines) => {
                let font = FontRef::try_from_slice(font_bytes()?)
                    .context("Font di anteprima non leggibile")?;
                let units = font
                    .units_per_em()
                    .context("Metriche del font non valide")?;
                let font_scale =
                    PxScale::from(self.font_size_pt * scale * font.height_unscaled() / units);
                for line in lines {
                    cancelled(cancel)?;
                    for (index, glyph) in line.glyphs.iter().enumerate() {
                        if index.is_multiple_of(256) {
                            cancelled(cancel)?;
                        }
                        let glyph = GlyphId(glyph.gid).with_scale_and_position(
                            font_scale,
                            point((line.x + glyph.x) * scale, line.baseline * scale),
                        );
                        if let Some(outline) = font.outline_glyph(glyph) {
                            let bounds = outline.px_bounds();
                            outline.draw(|x, y, coverage| {
                                let px = x as i64 + bounds.min.x as i64;
                                let py = y as i64 + bounds.min.y as i64;
                                if px >= 0 && py >= 0 && px < width as i64 && py < height as i64 {
                                    let pixel = canvas.get_pixel_mut(px as u32, py as u32);
                                    for channel in &mut pixel.0[..3] {
                                        *channel =
                                            (*channel as f32 * (1.0 - coverage)).round() as u8;
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
        cancelled(cancel)?;
        Ok(canvas)
    }
}

fn font_bytes() -> Result<&'static [u8]> {
    static FONT: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    FONT.get_or_init(|| {
        eframe::egui::FontDefinitions::default()
            .font_data
            .get("Ubuntu-Light")
            .map(|font| font.font.as_ref().to_vec())
    })
    .as_deref()
    .context("Font Ubuntu incorporato non disponibile")
}

struct BoundedBytes<'a> {
    bytes: Vec<u8>,
    limit: usize,
    cancel: &'a AtomicBool,
}
impl Write for BoundedBytes<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(std::io::Error::other("Operazione annullata"));
        }
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other(
                "Il documento supera il limite di memoria",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn image_page(
    pixels: &RgbaImage,
    settings: &PageLayoutSettings,
    quality: u8,
    cancel: &AtomicBool,
) -> Result<ImagePage> {
    let (width, height) = pixels.dimensions();
    ensure!(
        width > 0 && height > 0 && width as u64 * height as u64 <= 150_000_000,
        "Dimensioni immagine non valide o superiori a 150 megapixel"
    );
    let content = settings.content_rect();
    let (crop_x, crop_y, crop_width, crop_height) = if settings.fit == ImageFit::Cover {
        let aspect = content.width as f64 / content.height as f64;
        if width as f64 / height as f64 > aspect {
            let cropped = (height as f64 * aspect).round().clamp(1.0, width as f64) as u32;
            ((width - cropped) / 2, 0, cropped, height)
        } else {
            let cropped = (width as f64 / aspect).round().clamp(1.0, height as f64) as u32;
            (0, (height - cropped) / 2, width, cropped)
        }
    } else {
        (0, 0, width, height)
    };
    let factor = (content.width / crop_width as f32).min(content.height / crop_height as f32);
    let rect = if settings.fit == ImageFit::Cover {
        content
    } else {
        let (width, height) = (crop_width as f32 * factor, crop_height as f32 * factor);
        Rect {
            x: content.x + (content.width - width) / 2.0,
            y: content.y + (content.height - height) / 2.0,
            width,
            height,
        }
    };
    let target_width = ((rect.width / 72.0 * settings.dpi as f32).ceil() as u32)
        .max(1)
        .min(crop_width);
    let target_height = ((rect.height / 72.0 * settings.dpi as f32).ceil() as u32)
        .max(1)
        .min(crop_height);
    ensure!(
        target_width as u64 * target_height as u64 <= MAX_PIXELS,
        "L'immagine sulla pagina supera 40 megapixel: riduci i DPI"
    );
    cancelled(cancel)?;
    // PDF pages have a white matte. Composite before filtering so RGB hidden
    // behind transparent pixels cannot bleed into visible downsampled edges.
    // A virtual RGB view avoids allocating another full source-sized raster.
    struct WhiteSamples<'a>(&'a RgbaImage);
    impl image::GenericImageView for WhiteSamples<'_> {
        type Pixel = image::Rgb<u8>;
        fn dimensions(&self) -> (u32, u32) {
            self.0.dimensions()
        }
        fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
            let pixel = self.0.get_pixel(x, y);
            let alpha = pixel[3] as u32;
            let flatten = |channel: usize| {
                ((pixel[channel] as u32 * alpha + 255 * (255 - alpha) + 127) / 255) as u8
            };
            image::Rgb([flatten(0), flatten(1), flatten(2)])
        }
    }
    let samples = WhiteSamples(pixels);
    let view = image::imageops::crop_imm(&samples, crop_x, crop_y, crop_width, crop_height);
    let rgb = image::imageops::resize(&*view, target_width, target_height, FilterType::Lanczos3);
    cancelled(cancel)?;
    let mut jpeg = BoundedBytes {
        bytes: Vec::new(),
        limit: MAX_IMAGE_BYTES,
        cancel,
    };
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality)
        .encode_image(&image::DynamicImage::ImageRgb8(rgb))?;
    cancelled(cancel)?;
    Ok(ImagePage {
        jpeg: jpeg.bytes,
        width: target_width,
        height: target_height,
        rect,
    })
}

fn normalize_text(text: &str, cancel: &AtomicBool) -> Result<String> {
    ensure!(text.len() <= MAX_TEXT_BYTES, "Il testo supera 8 MiB");
    let mut plain = String::with_capacity(text.len());
    let mut chars = text
        .strip_prefix('\u{feff}')
        .unwrap_or(text)
        .chars()
        .peekable();
    let mut count = 0usize;
    while let Some(character) = chars.next() {
        if count.is_multiple_of(4096) {
            cancelled(cancel)?;
        }
        count += 1;
        match character {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                plain.push('\n');
            }
            '\t' => plain.push_str("    "),
            '\n' | '\u{c}' => plain.push(character),
            c if c.is_control() => bail!(
                "Il testo contiene un controllo non supportato (U+{:04X})",
                c as u32
            ),
            c => plain.push(c),
        }
        ensure!(
            plain.len() <= MAX_TEXT_BYTES,
            "Il testo normalizzato supera 8 MiB"
        );
    }
    let mut normalized = String::with_capacity(plain.len());
    for (index, character) in plain.nfc().enumerate() {
        if index.is_multiple_of(4096) {
            cancelled(cancel)?;
        }
        normalized.push(character);
        ensure!(
            normalized.len() <= MAX_TEXT_BYTES,
            "Il testo normalizzato supera 8 MiB"
        );
    }
    Ok(normalized)
}

struct TextComposer<'a> {
    settings: &'a PageLayoutSettings,
    glyphs: &'a BTreeMap<char, FontGlyph>,
    font: FontRef<'a>,
    units: f32,
    content: Rect,
    ascent: f32,
    descent: f32,
    advance: f32,
    baseline: f32,
    current_page: Vec<TextLine>,
    pages: &'a mut Vec<Page>,
}
impl<'a> TextComposer<'a> {
    fn new(
        settings: &'a PageLayoutSettings,
        glyphs: &'a BTreeMap<char, FontGlyph>,
        pages: &'a mut Vec<Page>,
    ) -> Result<Self> {
        let font = FontRef::try_from_slice(font_bytes()?).context("Font di testo non leggibile")?;
        let units = font.units_per_em().context("Metriche font non valide")?;
        let ascent = font.ascent_unscaled() / units * settings.font_size_pt;
        let descent = font.descent_unscaled() / units * settings.font_size_pt;
        let content = settings.content_rect();
        ensure!(
            ascent - descent <= content.height,
            "Il testo non entra nei margini: riduci carattere o margini"
        );
        let advance = (settings.font_size_pt * settings.line_height).max(ascent - descent);
        Ok(Self {
            settings,
            glyphs,
            font,
            units,
            content,
            ascent,
            descent,
            advance,
            baseline: content.y + ascent,
            current_page: Vec::new(),
            pages,
        })
    }
    fn kerning(&self, previous: Option<u16>, gid: u16) -> f32 {
        previous.map_or(0.0, |previous| {
            self.font.kern_unscaled(GlyphId(previous), GlyphId(gid)) / self.units
                * self.settings.font_size_pt
        })
    }
    fn new_page(&mut self) -> Result<()> {
        ensure!(
            self.pages.len() < MAX_PAGES,
            "Il documento supera {MAX_PAGES} pagine"
        );
        self.pages
            .push(Page::Text(std::mem::take(&mut self.current_page)));
        self.baseline = self.content.y + self.ascent;
        Ok(())
    }
    fn emit(&mut self, characters: &mut Vec<FontGlyph>) -> Result<()> {
        if self.baseline - self.descent > self.content.y + self.content.height + 0.001 {
            self.new_page()?;
        }
        let mut positions = Vec::with_capacity(characters.len());
        let mut width = 0.0;
        let mut previous = None;
        for glyph in characters.drain(..) {
            width += self.kerning(previous, glyph.gid);
            positions.push(GlyphPosition {
                cid: glyph.cid,
                gid: glyph.gid,
                x: width,
            });
            width += glyph.advance;
            previous = Some(glyph.gid);
        }
        let offset = match self.settings.align {
            TextAlign::Left => 0.0,
            TextAlign::Center => (self.content.width - width) / 2.0,
            TextAlign::Right => self.content.width - width,
        };
        self.current_page.push(TextLine {
            x: self.content.x + offset.max(0.0),
            baseline: self.baseline,
            glyphs: positions,
        });
        self.baseline += self.advance;
        Ok(())
    }
    fn width(&self, chars: impl Iterator<Item = FontGlyph>, mut previous: Option<u16>) -> f32 {
        let mut width = 0.0;
        for glyph in chars {
            width += self.kerning(previous, glyph.gid) + glyph.advance;
            previous = Some(glyph.gid);
        }
        width
    }
    fn paragraph(&mut self, paragraph: &str, cancel: &AtomicBool) -> Result<()> {
        let mut line = Vec::<FontGlyph>::new();
        let mut width = 0.0;
        let mut wrapped = false;
        for word in paragraph.split_word_bounds() {
            cancelled(cancel)?;
            let token: Vec<_> = word.chars().map(|c| self.glyphs[&c]).collect();
            let extra = self.width(token.iter().copied(), line.last().map(|g| g.gid));
            if !line.is_empty() && width + extra > self.content.width {
                self.emit(&mut line)?;
                width = 0.0;
                wrapped = true;
            }
            if wrapped && line.is_empty() && word.chars().all(char::is_whitespace) {
                continue;
            }
            for (index, grapheme) in word.graphemes(true).enumerate() {
                if index.is_multiple_of(256) {
                    cancelled(cancel)?;
                }
                let cluster: Vec<_> = grapheme
                    .chars()
                    .take(129)
                    .map(|c| self.glyphs[&c])
                    .collect();
                ensure!(
                    cluster.len() <= 128,
                    "Il testo contiene una sequenza combinata troppo lunga per l'impaginazione"
                );
                let extra = self.width(cluster.iter().copied(), line.last().map(|g| g.gid));
                if !line.is_empty() && width + extra > self.content.width {
                    self.emit(&mut line)?;
                    width = 0.0;
                }
                let extra = self.width(cluster.iter().copied(), line.last().map(|g| g.gid));
                ensure!(
                    extra <= self.content.width + 0.001,
                    "Un carattere non entra nella riga: riduci carattere o margini"
                );
                line.extend(cluster);
                width += extra;
                wrapped = false;
            }
        }
        self.emit(&mut line)
    }
    fn text(mut self, text: &str, cancel: &AtomicBool) -> Result<()> {
        for (index, page) in text.split('\u{c}').enumerate() {
            if index > 0 {
                self.new_page()?;
            }
            for paragraph in page.split('\n') {
                cancelled(cancel)?;
                self.paragraph(paragraph, cancel)?;
            }
        }
        self.new_page()
    }
}

pub fn prepare(
    inputs: &[PageInput],
    settings: &PageLayoutSettings,
    jpeg_quality: u8,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<PreparedPageDocument> {
    cancelled(cancel)?;
    settings.validate()?;
    ensure!(
        !inputs.is_empty() && inputs.len() <= MAX_PAGES,
        "Seleziona da 1 a {MAX_PAGES} immagini o testi"
    );
    ensure!(
        (1..=100).contains(&jpeg_quality),
        "Qualità JPEG: scegli un valore da 1 a 100"
    );
    let face = ttf_parser::Face::parse(font_bytes()?, 0)
        .map_err(|_| anyhow::anyhow!("Font PDF non leggibile"))?;
    let mut glyphs = BTreeMap::new();
    let mut texts = BTreeMap::new();
    let mut text_bytes = 0usize;
    let mut glyph_count = 0usize;
    for (index, input) in inputs.iter().enumerate() {
        if let PageInput::Text(text) = input {
            text_bytes = text_bytes
                .checked_add(text.len())
                .context("Testo troppo grande")?;
            ensure!(
                text_bytes <= MAX_TEXT_BYTES,
                "Il testo complessivo supera 8 MiB"
            );
            let text = normalize_text(text, cancel)?;
            for (position, character) in text.chars().enumerate() {
                if position.is_multiple_of(4096) {
                    cancelled(cancel)?;
                }
                if matches!(character, '\n' | '\u{c}') {
                    continue;
                }
                glyph_count += 1;
                ensure!(
                    glyph_count <= MAX_GLYPHS,
                    "Il documento supera un milione di caratteri"
                );
                if glyphs.contains_key(&character) {
                    continue;
                }
                let gid = face.glyph_index(character).filter(|gid| gid.0 != 0).with_context(|| format!("Il font incorporato non supporta il carattere U+{:04X}; il testo non è stato esportato", character as u32))?;
                let cid = u16::try_from(glyphs.len() + 1)
                    .context("Troppi caratteri diversi nel documento")?;
                let advance = face
                    .glyph_hor_advance(gid)
                    .context("Metriche del carattere non disponibili")?
                    as f32
                    / face.units_per_em() as f32
                    * settings.font_size_pt;
                glyphs.insert(
                    character,
                    FontGlyph {
                        cid,
                        gid: gid.0,
                        advance,
                    },
                );
            }
            texts.insert(index, text);
        }
    }
    let mut pages = Vec::new();
    let mut image_bytes = 0usize;
    for (index, input) in inputs.iter().enumerate() {
        cancelled(cancel)?;
        match input {
            PageInput::Image(image) => {
                ensure!(
                    pages.len() < MAX_PAGES,
                    "Il documento supera {MAX_PAGES} pagine"
                );
                let page = image_page(image, settings, jpeg_quality, cancel)?;
                image_bytes = image_bytes
                    .checked_add(page.jpeg.len())
                    .context("Documento troppo grande")?;
                ensure!(
                    image_bytes <= MAX_IMAGE_BYTES,
                    "Le immagini del documento superano 96 MiB: riduci qualità o DPI"
                );
                pages.push(Page::Image(page));
            }
            PageInput::Text(_) => {
                TextComposer::new(settings, &glyphs, &mut pages)?.text(&texts[&index], cancel)?
            }
        }
        progress(Progress {
            fraction: (index + 1) as f32 / inputs.len() as f32 * 0.8,
            message: format!("Impaginazione di {} di {}…", index + 1, inputs.len()),
        });
    }
    cancelled(cancel)?;
    let (width, height) = settings.page_size_points();
    progress(Progress {
        fraction: 0.85,
        message: "Preparazione dei byte PDF…".into(),
    });
    let bytes = encode_pdf(&pages, &glyphs, settings, &face, cancel)?;
    progress(Progress {
        fraction: 1.0,
        message: "PDF pronto: anteprima e peso disponibili".into(),
    });
    Ok(PreparedPageDocument {
        bytes,
        pages,
        width,
        height,
        font_size_pt: settings.font_size_pt,
    })
}

fn real(value: f32) -> Object {
    Object::Real(value)
}

fn embed_font(
    doc: &mut Document,
    face: &ttf_parser::Face<'_>,
    glyphs: &BTreeMap<char, FontGlyph>,
) -> Result<lopdf::ObjectId> {
    let scale = 1000.0 / face.units_per_em() as f32;
    let bbox = face.global_bounding_box();
    let font = doc.add_object(Stream::new(
        dictionary! {"Length1" => font_bytes()?.len() as i64},
        font_bytes()?.to_vec(),
    ));
    let descriptor = doc.add_object(dictionary! {
        "Type" => "FontDescriptor", "FontName" => "Ubuntu-Light", "Flags" => 32,
        "FontBBox" => vec![real(bbox.x_min as f32 * scale), real(bbox.y_min as f32 * scale), real(bbox.x_max as f32 * scale), real(bbox.y_max as f32 * scale)],
        "ItalicAngle" => real(face.italic_angle()), "Ascent" => real(face.ascender() as f32 * scale),
        "Descent" => real(face.descender() as f32 * scale), "CapHeight" => real(face.capital_height().unwrap_or(face.ascender()) as f32 * scale),
        "StemV" => 80, "FontFile2" => font,
    });
    let mut mapping = vec![0u8; (glyphs.len() + 1) * 2];
    let mut widths = Vec::with_capacity(glyphs.len() * 2);
    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /ConvertHereUnicode def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let mut ordered: Vec<_> = glyphs.iter().collect();
    ordered.sort_by_key(|(_, glyph)| glyph.cid);
    for block in ordered.chunks(100) {
        cmap.push_str(&format!("{} beginbfchar\n", block.len()));
        for (character, glyph) in block {
            mapping[glyph.cid as usize * 2..glyph.cid as usize * 2 + 2]
                .copy_from_slice(&glyph.gid.to_be_bytes());
            widths.push(Object::Integer(glyph.cid as i64));
            widths.push(Object::Array(vec![real(
                face.glyph_hor_advance(ttf_parser::GlyphId(glyph.gid))
                    .context("Metriche PDF mancanti")? as f32
                    * scale,
            )]));
            let unicode: String = character
                .encode_utf16(&mut [0; 2])
                .iter()
                .map(|unit| format!("{unit:04X}"))
                .collect();
            cmap.push_str(&format!("<{:04X}> <{unicode}>\n", glyph.cid));
        }
        cmap.push_str("endbfchar\n");
    }
    cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    let mapping = doc.add_object(Stream::new(dictionary! {}, mapping));
    let unicode = doc.add_object(Stream::new(dictionary! {}, cmap.into_bytes()));
    let descendant = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "CIDFontType2", "BaseFont" => "Ubuntu-Light",
        "CIDSystemInfo" => dictionary! {"Registry" => Object::string_literal("Adobe"), "Ordering" => Object::string_literal("Identity"), "Supplement" => 0},
        "FontDescriptor" => descriptor, "CIDToGIDMap" => mapping, "DW" => 1000, "W" => Object::Array(widths),
    });
    Ok(doc.add_object(dictionary! {"Type" => "Font", "Subtype" => "Type0", "BaseFont" => "Ubuntu-Light", "Encoding" => "Identity-H", "DescendantFonts" => vec![Object::Reference(descendant)], "ToUnicode" => unicode}))
}

fn encode_pdf(
    pages: &[Page],
    glyphs: &BTreeMap<char, FontGlyph>,
    settings: &PageLayoutSettings,
    face: &ttf_parser::Face<'_>,
    cancel: &AtomicBool,
) -> Result<Vec<u8>> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font = if glyphs.is_empty() {
        None
    } else {
        Some(embed_font(&mut doc, face, glyphs)?)
    };
    let by_cid: BTreeMap<_, _> = glyphs.values().map(|glyph| (glyph.cid, *glyph)).collect();
    let (width, height) = settings.page_size_points();
    let mut children = Vec::with_capacity(pages.len());
    for page in pages {
        cancelled(cancel)?;
        let (resources, operations) = match page {
            Page::Image(page) => {
                let image = doc.add_object(Stream::new(dictionary! {"Type" => "XObject", "Subtype" => "Image", "Width" => page.width as i64, "Height" => page.height as i64, "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8, "Filter" => "DCTDecode"}, page.jpeg.clone()));
                (
                    dictionary! {"XObject" => dictionary! {"Im0" => image}},
                    vec![
                        Operation::new("q", vec![]),
                        Operation::new(
                            "cm",
                            vec![
                                real(page.rect.width),
                                0.into(),
                                0.into(),
                                real(page.rect.height),
                                real(page.rect.x),
                                real(height - page.rect.y - page.rect.height),
                            ],
                        ),
                        Operation::new("Do", vec![Object::Name(b"Im0".to_vec())]),
                        Operation::new("Q", vec![]),
                    ],
                )
            }
            Page::Text(lines) => {
                let mut operations = Vec::with_capacity(lines.len() * 2 + 3);
                let mut resources = dictionary! {};
                if let Some(font) = font {
                    resources.set("Font", dictionary! {"F1" => font});
                    operations.push(Operation::new("BT", vec![]));
                    operations.push(Operation::new(
                        "Tf",
                        vec![Object::Name(b"F1".to_vec()), real(settings.font_size_pt)],
                    ));
                    for line in lines {
                        cancelled(cancel)?;
                        if line.glyphs.is_empty() {
                            continue;
                        }
                        operations.push(Operation::new(
                            "Tm",
                            vec![
                                1.into(),
                                0.into(),
                                0.into(),
                                1.into(),
                                real(line.x),
                                real(height - line.baseline),
                            ],
                        ));
                        let mut array = Vec::new();
                        let mut run = Vec::new();
                        let mut end = 0.0;
                        for (index, glyph) in line.glyphs.iter().enumerate() {
                            if index.is_multiple_of(256) {
                                cancelled(cancel)?;
                            }
                            let offset = glyph.x - end;
                            if offset.abs() > 0.00001 {
                                if !run.is_empty() {
                                    array.push(Object::String(
                                        std::mem::take(&mut run),
                                        lopdf::StringFormat::Hexadecimal,
                                    ));
                                }
                                array.push(real(-offset / settings.font_size_pt * 1000.0));
                            }
                            run.extend_from_slice(&glyph.cid.to_be_bytes());
                            end = glyph.x + by_cid[&glyph.cid].advance;
                        }
                        if !run.is_empty() {
                            array.push(Object::String(run, lopdf::StringFormat::Hexadecimal));
                        }
                        operations.push(Operation::new("TJ", vec![Object::Array(array)]));
                    }
                    operations.push(Operation::new("ET", vec![]));
                }
                (resources, operations)
            }
        };
        let stream = Content { operations }.encode()?;
        let contents = doc.add_object(Stream::new(dictionary! {}, stream));
        let page = doc.add_object(dictionary! {"Type" => "Page", "Parent" => pages_id, "MediaBox" => vec![0.into(), 0.into(), real(width), real(height)], "Resources" => resources, "Contents" => contents});
        children.push(Object::Reference(page));
    }
    doc.objects.insert(
        pages_id,
        Object::Dictionary(
            dictionary! {"Type" => "Pages", "Kids" => children, "Count" => pages.len() as i64},
        ),
    );
    let catalog = doc.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages_id});
    doc.trailer.set("Root", catalog);
    cancelled(cancel)?;
    doc.compress();
    cancelled(cancel)?;
    let mut output = BoundedBytes {
        bytes: Vec::new(),
        limit: MAX_PDF_BYTES,
        cancel,
    };
    doc.save_to(&mut output)?;
    cancelled(cancel)?;
    Ok(output.bytes)
}

/// Plain UTF-8 only. The legacy TXT/Office route remains active without page settings.
pub fn convert_text_pdf(
    input: &Path,
    dir: &Path,
    settings: &ConversionSettings,
    cancel: &AtomicBool,
    progress: impl FnMut(Progress),
) -> Result<PathBuf> {
    cancelled(cancel)?;
    ensure!(
        settings.format == OutputFormat::Pdf
            && input
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("txt")),
        "L'impaginazione del testo richiede un file TXT e destinazione PDF"
    );
    let page = settings
        .image
        .page
        .as_ref()
        .context("Impostazioni pagina mancanti")?;
    page.validate()?;
    let mut file = File::open(input).context("File TXT non accessibile")?;
    ensure!(
        file.metadata()?.len() <= MAX_TEXT_BYTES as u64,
        "Il file TXT supera 8 MiB"
    );
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 65536];
    loop {
        cancelled(cancel)?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        ensure!(
            bytes.len() + count <= MAX_TEXT_BYTES,
            "Il file TXT supera 8 MiB"
        );
        bytes.extend_from_slice(&buffer[..count]);
    }
    let text = String::from_utf8(bytes).context("Il file TXT deve usare la codifica UTF-8")?;
    let prepared = prepare(
        &[PageInput::Text(Arc::from(text))],
        page,
        settings.image.quality,
        cancel,
        progress,
    )?;
    if let Some(limit) = settings.image.edit.as_ref().and_then(|edit| edit.max_bytes) {
        ensure!(
            prepared.exact_bytes() <= limit,
            "Il PDF supera il limite di peso scelto; aumenta il limite o riduci il contenuto"
        );
    }
    let (_stage, mut output) = external_temporary(dir, "pdf")?;
    for chunk in prepared.bytes().chunks(65536) {
        cancelled(cancel)?;
        output.as_file_mut().write_all(chunk)?;
    }
    output.as_file_mut().sync_all()?;
    cancelled(cancel)?;
    commit(output, dir, &stem(input), "pdf")
}

pub fn save_prepared(
    input: Option<&Path>,
    output_path: &Path,
    prepared: &PreparedPageDocument,
    cancel: &AtomicBool,
) -> Result<PathBuf> {
    crate::image_edit::save_bytes(input, output_path, prepared.bytes(), cancel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn bounded_writer_stops_without_retrying_cancellation_or_exceeding_its_limit() {
        let cancel = AtomicBool::new(false);
        let mut writer = BoundedBytes {
            bytes: Vec::new(),
            limit: 4,
            cancel: &cancel,
        };
        writer.write_all(b"1234").unwrap();
        assert!(writer.write_all(b"5").is_err());
        assert_eq!(writer.bytes, b"1234");
        cancel.store(true, Ordering::Relaxed);
        let error = writer.write_all(b"cancel").unwrap_err();
        assert_ne!(error.kind(), std::io::ErrorKind::Interrupted);
        assert!(error.to_string().contains("annullata"));
    }
}
