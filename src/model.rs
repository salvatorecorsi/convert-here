use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Audio,
    Video,
    Image,
    Pdf,
    Document,
    Spreadsheet,
    Html,
    Unknown,
}

impl MediaKind {
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "mp3" | "mpga" | "wav" | "flac" | "aac" | "m4a" | "ogg" | "opus" | "aiff" | "aif"
            | "wma" | "alac" | "ape" | "m4b" | "mka" | "mp2" | "oga" | "weba" | "ac3" | "au"
            | "caf" | "amr" | "dts" | "tta" | "voc" | "wv" | "tak" | "w64" | "aifc" | "ra" => {
                Self::Audio
            }
            "mp4" | "m4v" | "mkv" | "mov" | "webm" | "avi" | "wmv" | "mpeg" | "mpg" | "mts"
            | "m2ts" | "ts" | "3gp" | "flv" | "asf" | "vob" | "ogv" | "m2v" | "mxf" | "f4v"
            | "divx" | "3g2" | "m2t" | "mpe" | "mpv" => Self::Video,
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "tif" | "tiff" | "ico" | "qoi"
            | "tga" | "ff" | "hdr" | "exr" | "pnm" | "pam" | "pbm" | "pgm" | "ppm" | "avif"
            | "heic" | "heif" => Self::Image,
            "pdf" => Self::Pdf,
            "doc" | "docx" | "odt" | "rtf" | "txt" | "ppt" | "pptx" | "odp" => Self::Document,
            "xls" | "xlsx" | "xlsm" | "ods" | "csv" => Self::Spreadsheet,
            "html" | "htm" => Self::Html,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    #[default]
    Mp3,
    Wav,
    Flac,
    Aac,
    Ogg,
    Opus,
    Aiff,
    Mp4,
    Mkv,
    Mov,
    Webm,
    Avi,
    Gif,
    Png,
    Jpg,
    Webp,
    Bmp,
    Tiff,
    Ico,
    Pdf,
    Txt,
    M4a,
    Wma,
    Ac3,
    Au,
    Caf,
    Mpeg,
    Wmv,
    Flv,
    Ts,
    M4v,
    ThreeGp,
    Qoi,
    Tga,
    Farbfeld,
    OpenExr,
    Hdr,
    Pnm,
    Csv,
    Xlsx,
    Docx,
    Html,
    Avif,
    Heic,
}

impl OutputFormat {
    pub const ALL: [Self; 44] = [
        Self::Mp3,
        Self::Wav,
        Self::Flac,
        Self::Aac,
        Self::Ogg,
        Self::Opus,
        Self::Aiff,
        Self::Mp4,
        Self::Mkv,
        Self::Mov,
        Self::Webm,
        Self::Avi,
        Self::Gif,
        Self::Png,
        Self::Jpg,
        Self::Webp,
        Self::Bmp,
        Self::Tiff,
        Self::Ico,
        Self::Pdf,
        Self::Txt,
        Self::M4a,
        Self::Wma,
        Self::Ac3,
        Self::Au,
        Self::Caf,
        Self::Mpeg,
        Self::Wmv,
        Self::Flv,
        Self::Ts,
        Self::M4v,
        Self::ThreeGp,
        Self::Qoi,
        Self::Tga,
        Self::Farbfeld,
        Self::OpenExr,
        Self::Hdr,
        Self::Pnm,
        Self::Csv,
        Self::Xlsx,
        Self::Docx,
        Self::Html,
        Self::Avif,
        Self::Heic,
    ];
    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Aac => "aac",
            Self::Ogg => "ogg",
            Self::Opus => "opus",
            Self::Aiff => "aiff",
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Mov => "mov",
            Self::Webm => "webm",
            Self::Avi => "avi",
            Self::Gif => "gif",
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Webp => "webp",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
            Self::Pdf => "pdf",
            Self::Txt => "txt",
            Self::M4a => "m4a",
            Self::Wma => "wma",
            Self::Ac3 => "ac3",
            Self::Au => "au",
            Self::Caf => "caf",
            Self::Mpeg => "mpeg",
            Self::Wmv => "wmv",
            Self::Flv => "flv",
            Self::Ts => "ts",
            Self::M4v => "m4v",
            Self::ThreeGp => "3gp",
            Self::Qoi => "qoi",
            Self::Tga => "tga",
            Self::Farbfeld => "ff",
            Self::OpenExr => "exr",
            Self::Hdr => "hdr",
            Self::Pnm => "pnm",
            Self::Csv => "csv",
            Self::Xlsx => "xlsx",
            Self::Docx => "docx",
            Self::Html => "html",
            Self::Avif => "avif",
            Self::Heic => "heic",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::Wav => "WAV",
            Self::Flac => "FLAC",
            Self::Aac => "AAC",
            Self::Ogg => "OGG",
            Self::Opus => "Opus",
            Self::Aiff => "AIFF",
            Self::Mp4 => "MP4",
            Self::Mkv => "MKV",
            Self::Mov => "MOV",
            Self::Webm => "WebM",
            Self::Avi => "AVI",
            Self::Gif => "GIF",
            Self::Png => "PNG",
            Self::Jpg => "JPEG",
            Self::Webp => "WebP",
            Self::Bmp => "BMP",
            Self::Tiff => "TIFF",
            Self::Ico => "ICO",
            Self::Pdf => "PDF",
            Self::Txt => "TXT",
            Self::M4a => "M4A (AAC)",
            Self::Wma => "WMA",
            Self::Ac3 => "AC3",
            Self::Au => "AU",
            Self::Caf => "CAF (ALAC)",
            Self::Mpeg => "MPEG",
            Self::Wmv => "WMV",
            Self::Flv => "FLV",
            Self::Ts => "MPEG-TS",
            Self::M4v => "M4V",
            Self::ThreeGp => "3GP",
            Self::Qoi => "QOI",
            Self::Tga => "TGA",
            Self::Farbfeld => "Farbfeld",
            Self::OpenExr => "OpenEXR",
            Self::Hdr => "HDR",
            Self::Pnm => "PNM",
            Self::Csv => "CSV",
            Self::Xlsx => "Excel (XLSX)",
            Self::Docx => crate::i18n::tr("Word (DOCX) · testo", "Word (DOCX) · text"),
            Self::Html => crate::i18n::tr("HTML · testo", "HTML · text"),
            Self::Avif => "AVIF",
            Self::Heic => "HEIC",
        }
    }
    pub fn kind(self) -> MediaKind {
        match self {
            Self::Mp3
            | Self::Wav
            | Self::Flac
            | Self::Aac
            | Self::Ogg
            | Self::Opus
            | Self::Aiff
            | Self::M4a
            | Self::Wma
            | Self::Ac3
            | Self::Au
            | Self::Caf => MediaKind::Audio,
            Self::Mp4
            | Self::Mkv
            | Self::Mov
            | Self::Webm
            | Self::Avi
            | Self::Mpeg
            | Self::Wmv
            | Self::Flv
            | Self::Ts
            | Self::M4v
            | Self::ThreeGp => MediaKind::Video,
            Self::Gif
            | Self::Png
            | Self::Jpg
            | Self::Webp
            | Self::Bmp
            | Self::Tiff
            | Self::Ico
            | Self::Qoi
            | Self::Tga
            | Self::Farbfeld
            | Self::OpenExr
            | Self::Hdr
            | Self::Pnm => MediaKind::Image,
            Self::Avif | Self::Heic => MediaKind::Image,
            Self::Pdf => MediaKind::Pdf,
            Self::Txt | Self::Docx => MediaKind::Document,
            Self::Csv | Self::Xlsx => MediaKind::Spreadsheet,
            Self::Html => MediaKind::Html,
        }
    }

    /// Explicit encoder/container constraints; zero lets FFmpeg choose a rate.
    /// Other formats use the general application sample-rate limits.
    pub fn audio_sample_rates(self) -> Option<&'static [u32]> {
        match self {
            Self::Opus | Self::Webm => Some(&[0, 8000, 12000, 16000, 24000, 48000]),
            Self::Ac3 => Some(&[0, 32000, 44100, 48000]),
            Self::Wma | Self::Wmv => {
                Some(&[0, 8000, 11025, 16000, 22050, 24000, 32000, 44100, 48000])
            }
            Self::Flv => Some(&[0, 11025, 22050, 44100]),
            _ => None,
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

pub fn supported_formats(kind: MediaKind) -> Vec<OutputFormat> {
    use OutputFormat::*;
    match kind {
        MediaKind::Audio => vec![Mp3, Wav, Flac, Aac, Ogg, Opus, Aiff, M4a, Wma, Ac3, Au, Caf],
        MediaKind::Video => vec![
            Mp4, Mkv, Mov, Webm, Avi, Gif, Mpeg, Wmv, Flv, Ts, M4v, ThreeGp, Mp3, Wav, Flac, Aac,
            Ogg, Opus, Aiff, M4a, Wma, Ac3, Au, Caf,
        ],
        MediaKind::Image => vec![
            Png, Jpg, Webp, Bmp, Tiff, Gif, Ico, Pdf, Qoi, Tga, Farbfeld, OpenExr, Hdr, Pnm, Avif,
            Heic, Txt,
        ],
        MediaKind::Pdf => vec![Png, Jpg, Txt, Docx, Html, Pdf],
        MediaKind::Document => vec![Pdf],
        MediaKind::Spreadsheet => vec![Pdf, Csv, Xlsx],
        MediaKind::Html => vec![Pdf],
        MediaKind::Unknown => vec![],
    }
}

/// Formats shared by every input, including local OCR and PDF text extraction.
pub fn common_formats<'a>(inputs: impl IntoIterator<Item = &'a Path>) -> Vec<OutputFormat> {
    let mut inputs = inputs.into_iter();
    let Some(first) = inputs.next() else {
        return Vec::new();
    };
    let formats_for = |path: &Path| supported_formats(MediaKind::from_path(path));
    let mut formats = formats_for(first);
    for path in inputs {
        let supported = formats_for(path);
        formats.retain(|format| supported.contains(format));
    }
    formats
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConversionSettings {
    pub format: OutputFormat,
    pub output_dir: Option<PathBuf>,
    pub audio: AudioSettings,
    pub video: VideoSettings,
    pub image: ImageSettings,
    pub pdf: PdfSettings,
    pub csv: CsvSettings,
    pub ocr: OcrSettings,
    pub limits: ConversionLimits,
    pub media_options: std::collections::BTreeMap<String, String>,
}
impl Default for ConversionSettings {
    fn default() -> Self {
        Self {
            format: OutputFormat::Mp3,
            output_dir: None,
            audio: AudioSettings::default(),
            video: VideoSettings::default(),
            image: ImageSettings::default(),
            pdf: PdfSettings::default(),
            csv: CsvSettings::default(),
            ocr: OcrSettings::default(),
            limits: ConversionLimits::default(),
            media_options: std::collections::BTreeMap::new(),
        }
    }
}

/// Optional per-output limits. Old settings omit this field and remain unlimited.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConversionLimits {
    pub max_bytes: Option<u64>,
    /// Maximum audio/video duration in seconds, measured after trim_start.
    pub max_duration: Option<f64>,
}

impl ConversionLimits {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_bytes == Some(0) {
            anyhow::bail!(crate::i18n::tr(
                "Il peso massimo deve essere maggiore di zero",
                "The maximum size must be greater than zero"
            ));
        }
        if self
            .max_duration
            .is_some_and(|value| !value.is_finite() || value <= 0.0 || value > 86400.0)
        {
            anyhow::bail!(crate::i18n::tr(
                "La durata massima deve essere finita, maggiore di zero e non oltre 86400 secondi",
                "The maximum duration must be finite, greater than zero and at most 86400 seconds"
            ));
        }
        Ok(())
    }

    /// Check actual completed bytes, not an estimate of the encoder's output.
    pub fn check_bytes(&self, bytes: u64) -> anyhow::Result<()> {
        match self.max_bytes {
            Some(maximum) if bytes > maximum => anyhow::bail!(
                "{} ({bytes} > {maximum} byte)",
                crate::i18n::tr(
                    "Il risultato supera il peso massimo; nessun risultato oltre il limite è stato salvato",
                    "The output exceeds the size limit; no oversized output was saved"
                )
            ),
            _ => Ok(()),
        }
    }
}

impl ConversionSettings {
    /// An ordinary destination change keeps compatible options and drops recipes
    /// that the newly selected encoder cannot apply. Explicit profiles stay intact.
    pub fn select_output_format(&mut self, format: OutputFormat) {
        self.format = format;
        if !matches!(
            format,
            OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp | OutputFormat::Pdf
        ) {
            self.image.edit = None;
        }
        if format != OutputFormat::Pdf {
            self.image.page = None;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub bitrate_kbps: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub normalize: bool,
}
impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            bitrate_kbps: 192,
            sample_rate: 0,
            channels: 0,
            normalize: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VideoCodec {
    #[default]
    H264,
    H265,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VideoSettings {
    pub codec: VideoCodec,
    pub crf: u8,
    pub width: Option<u32>,
    pub fps: Option<u32>,
    pub trim_start: Option<f64>,
    pub trim_end: Option<f64>,
}
impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            crf: 23,
            width: None,
            fps: None,
            trim_start: None,
            trim_end: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImageSettings {
    pub quality: u8,
    pub width: Option<u32>,
    #[serde(default)]
    pub edit: Option<crate::image_edit::ImageEditSettings>,
    #[serde(default)]
    pub page: Option<crate::page_layout::PageLayoutSettings>,
}
impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            quality: 90,
            width: None,
            edit: None,
            page: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PdfSettings {
    pub dpi: u32,
    pub action: PdfAction,
}
impl Default for PdfSettings {
    fn default() -> Self {
        Self {
            dpi: 150,
            action: PdfAction::Compress,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PdfAction {
    #[default]
    Compress,
    Split,
    Merge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CsvDelimiter {
    #[default]
    Comma,
    Semicolon,
    Tab,
    Pipe,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CsvSettings {
    pub delimiter: CsvDelimiter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OcrSettings {
    pub enabled: bool,
    pub language: String,
}

impl Default for OcrSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            language: "ita+eng".into(),
        }
    }
}
