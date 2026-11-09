//! Typed output-only FFmpeg controls. No raw command lines, scripts or paths.
//! Encoder AVOptions are verified against the selected local FFmpeg before use.
use crate::{
    engine::{cancelled, run_pdf_probe},
    i18n::tr,
    model::{ConversionSettings, MediaKind, OutputFormat, VideoCodec},
    tools::ToolPaths,
};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
    process::Command,
    sync::{Mutex, OnceLock, atomic::AtomicBool},
    time::{Duration, SystemTime},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptionKind {
    Bool,
    Int { min: i64, max: i64 },
    Float { min: f64, max: f64 },
    Choice(&'static [&'static str]),
    Text,
}

#[derive(Clone, Copy, Debug)]
pub struct OptionSpec {
    pub key: &'static str,
    pub label_it: &'static str,
    pub label_en: &'static str,
    /// Stable UI group identifier, not persisted as an option.
    pub group: &'static str,
    pub kind: OptionKind,
}

macro_rules! option {
    ($key:literal, $it:literal, $en:literal, $group:literal, $kind:expr) => {
        OptionSpec {
            key: $key,
            label_it: $it,
            label_en: $en,
            group: $group,
            kind: $kind,
        }
    };
}

const VIDEO_ENCODERS: &[&str] = &[
    "libx264",
    "libx265",
    "mpeg4",
    "mpeg2video",
    "mpeg1video",
    "wmv1",
    "wmv2",
    "flv",
    "h263",
    "libvpx-vp9",
    "libvpx",
    "libaom-av1",
    "libsvtav1",
    "librav1e",
    "ffv1",
    "huffyuv",
    "mjpeg",
    "rawvideo",
    "h264_nvenc",
    "hevc_nvenc",
    "av1_nvenc",
    "h264_qsv",
    "hevc_qsv",
    "av1_qsv",
    "h264_amf",
    "hevc_amf",
    "av1_amf",
];
const AUDIO_ENCODERS: &[&str] = &[
    "libmp3lame",
    "aac",
    "libfdk_aac",
    "libvorbis",
    "libopus",
    "opus",
    "flac",
    "alac",
    "wmav1",
    "wmav2",
    "ac3",
    "ac3_fixed",
    "mp2",
    "pcm_s16le",
    "pcm_s24le",
    "pcm_s32le",
    "pcm_f32le",
    "pcm_f64le",
    "pcm_s16be",
    "pcm_s24be",
    "pcm_s32be",
    "pcm_f32be",
];

pub static SPECS: &[OptionSpec] = &[
    option!(
        "video_encoder",
        "Encoder video",
        "Video encoder",
        "codec",
        OptionKind::Choice(VIDEO_ENCODERS)
    ),
    option!(
        "audio_encoder",
        "Encoder audio",
        "Audio encoder",
        "codec",
        OptionKind::Choice(AUDIO_ENCODERS)
    ),
    option!(
        "preset",
        "Preset encoder",
        "Encoder preset",
        "codec",
        OptionKind::Choice(&[
            "ultrafast",
            "superfast",
            "veryfast",
            "faster",
            "fast",
            "medium",
            "slow",
            "slower",
            "veryslow",
            "placebo",
            "p1",
            "p2",
            "p3",
            "p4",
            "p5",
            "p6",
            "p7"
        ])
    ),
    option!(
        "tune",
        "Ottimizzazione encoder",
        "Encoder tuning",
        "codec",
        OptionKind::Choice(&[
            "film",
            "animation",
            "grain",
            "stillimage",
            "psnr",
            "ssim",
            "fastdecode",
            "zerolatency",
            "hq",
            "ll",
            "ull",
            "lossless"
        ])
    ),
    option!(
        "profile",
        "Profilo video",
        "Video profile",
        "codec",
        OptionKind::Choice(&[
            "baseline",
            "main",
            "high",
            "high10",
            "high422",
            "high444",
            "main10",
            "mainstillpicture"
        ])
    ),
    option!(
        "level",
        "Livello video",
        "Video level",
        "codec",
        OptionKind::Choice(&[
            "1.0", "1.1", "1.2", "1.3", "2.0", "2.1", "2.2", "3.0", "3.1", "3.2", "4.0", "4.1",
            "4.2", "5.0", "5.1", "5.2", "6.0", "6.1", "6.2"
        ])
    ),
    option!(
        "pixel_format",
        "Formato pixel",
        "Pixel format",
        "video",
        OptionKind::Choice(&[
            "yuv420p",
            "yuv422p",
            "yuv444p",
            "yuv420p10le",
            "yuv422p10le",
            "yuv444p10le",
            "nv12",
            "p010le",
            "rgb24",
            "rgba",
            "gray",
            "gray10le"
        ])
    ),
    option!(
        "video_bitrate_kbps",
        "Bitrate video (kbps)",
        "Video bitrate (kbps)",
        "video",
        OptionKind::Int {
            min: 1,
            max: 2_000_000
        }
    ),
    option!(
        "max_rate_kbps",
        "Bitrate massimo (kbps)",
        "Maximum bitrate (kbps)",
        "video",
        OptionKind::Int {
            min: 1,
            max: 2_000_000
        }
    ),
    option!(
        "min_rate_kbps",
        "Bitrate minimo (kbps)",
        "Minimum bitrate (kbps)",
        "video",
        OptionKind::Int {
            min: 0,
            max: 2_000_000
        }
    ),
    option!(
        "buffer_size_kbits",
        "Buffer bitrate (kbit)",
        "Bitrate buffer (kbit)",
        "video",
        OptionKind::Int {
            min: 1,
            max: 8_000_000
        }
    ),
    option!(
        "crf",
        "CRF encoder",
        "Encoder CRF",
        "video",
        OptionKind::Float {
            min: 0.0,
            max: 63.0
        }
    ),
    option!(
        "qscale",
        "Scala qualità video",
        "Video quality scale",
        "video",
        OptionKind::Float {
            min: 1.0,
            max: 31.0
        }
    ),
    option!(
        "gop_size",
        "Intervallo fotogrammi chiave",
        "Keyframe interval",
        "video",
        OptionKind::Int { min: 1, max: 10000 }
    ),
    option!(
        "b_frames",
        "Fotogrammi B",
        "B-frames",
        "video",
        OptionKind::Int { min: 0, max: 16 }
    ),
    option!(
        "reference_frames",
        "Fotogrammi di riferimento",
        "Reference frames",
        "video",
        OptionKind::Int { min: 1, max: 16 }
    ),
    option!(
        "video_fps",
        "FPS di uscita",
        "Output FPS",
        "video",
        OptionKind::Float {
            min: 0.1,
            max: 240.0
        }
    ),
    option!(
        "display_aspect",
        "Proporzioni visualizzate (es. 16/9)",
        "Display aspect (e.g. 16/9)",
        "video",
        OptionKind::Text
    ),
    option!(
        "color_range",
        "Intervallo colore",
        "Color range",
        "video",
        OptionKind::Choice(&["tv", "pc"])
    ),
    option!(
        "color_primaries",
        "Primarie colore",
        "Color primaries",
        "video",
        OptionKind::Choice(&[
            "bt709",
            "bt470m",
            "bt470bg",
            "smpte170m",
            "smpte240m",
            "bt2020"
        ])
    ),
    option!(
        "color_transfer",
        "Trasferimento colore",
        "Color transfer",
        "video",
        OptionKind::Choice(&[
            "bt709",
            "smpte170m",
            "linear",
            "srgb",
            "bt2020-10",
            "bt2020-12",
            "smpte2084",
            "arib-std-b67"
        ])
    ),
    option!(
        "color_space",
        "Spazio colore",
        "Color space",
        "video",
        OptionKind::Choice(&[
            "bt709",
            "fcc",
            "bt470bg",
            "smpte170m",
            "smpte240m",
            "bt2020nc",
            "bt2020c"
        ])
    ),
    option!(
        "audio_bitrate_kbps",
        "Bitrate audio (kbps)",
        "Audio bitrate (kbps)",
        "audio",
        OptionKind::Int { min: 8, max: 2048 }
    ),
    option!(
        "audio_sample_rate",
        "Frequenza audio (Hz)",
        "Audio sample rate (Hz)",
        "audio",
        OptionKind::Int {
            min: 8000,
            max: 192000
        }
    ),
    option!(
        "audio_channels",
        "Canali audio",
        "Audio channels",
        "audio",
        OptionKind::Int { min: 1, max: 8 }
    ),
    option!(
        "audio_sample_format",
        "Formato campioni audio",
        "Audio sample format",
        "audio",
        OptionKind::Choice(&[
            "u8", "u8p", "s16", "s16p", "s32", "s32p", "s64", "s64p", "flt", "fltp", "dbl", "dblp"
        ])
    ),
    option!(
        "audio_compression_level",
        "Livello compressione audio",
        "Audio compression level",
        "audio",
        OptionKind::Int { min: 0, max: 12 }
    ),
    option!(
        "crop_width",
        "Ritaglio: larghezza",
        "Crop width",
        "filters",
        OptionKind::Int { min: 2, max: 32768 }
    ),
    option!(
        "crop_height",
        "Ritaglio: altezza",
        "Crop height",
        "filters",
        OptionKind::Int { min: 2, max: 32768 }
    ),
    option!(
        "crop_x",
        "Ritaglio: posizione X",
        "Crop X",
        "filters",
        OptionKind::Int { min: 0, max: 32768 }
    ),
    option!(
        "crop_y",
        "Ritaglio: posizione Y",
        "Crop Y",
        "filters",
        OptionKind::Int { min: 0, max: 32768 }
    ),
    option!(
        "scale_width",
        "Larghezza video",
        "Video width",
        "filters",
        OptionKind::Int { min: 2, max: 16384 }
    ),
    option!(
        "scale_height",
        "Altezza video",
        "Video height",
        "filters",
        OptionKind::Int { min: 2, max: 16384 }
    ),
    option!(
        "rotation",
        "Rotazione",
        "Rotation",
        "filters",
        OptionKind::Choice(&["0", "90", "180", "270"])
    ),
    option!(
        "horizontal_flip",
        "Rifletti orizzontalmente",
        "Flip horizontally",
        "filters",
        OptionKind::Bool
    ),
    option!(
        "vertical_flip",
        "Rifletti verticalmente",
        "Flip vertically",
        "filters",
        OptionKind::Bool
    ),
    option!(
        "deinterlace",
        "Deinterlacciamento",
        "Deinterlace",
        "filters",
        OptionKind::Choice(&["off", "yadif", "bwdif"])
    ),
    option!(
        "video_fade_in",
        "Dissolvenza video iniziale (s)",
        "Video fade-in (s)",
        "filters",
        OptionKind::Float {
            min: 0.0,
            max: 3600.0
        }
    ),
    option!(
        "video_fade_out_start",
        "Inizio dissolvenza video finale (s)",
        "Video fade-out start (s)",
        "filters",
        OptionKind::Float {
            min: 0.0,
            max: 86400.0
        }
    ),
    option!(
        "video_fade_out_duration",
        "Durata dissolvenza video finale (s)",
        "Video fade-out duration (s)",
        "filters",
        OptionKind::Float {
            min: 0.01,
            max: 3600.0
        }
    ),
    option!(
        "audio_volume",
        "Volume audio (moltiplicatore)",
        "Audio volume multiplier",
        "filters",
        OptionKind::Float {
            min: 0.0,
            max: 10.0
        }
    ),
    option!(
        "audio_highpass_hz",
        "Filtro passa-alto (Hz)",
        "High-pass filter (Hz)",
        "filters",
        OptionKind::Float {
            min: 10.0,
            max: 20000.0
        }
    ),
    option!(
        "audio_lowpass_hz",
        "Filtro passa-basso (Hz)",
        "Low-pass filter (Hz)",
        "filters",
        OptionKind::Float {
            min: 10.0,
            max: 20000.0
        }
    ),
    option!(
        "audio_fade_in",
        "Dissolvenza audio iniziale (s)",
        "Audio fade-in (s)",
        "filters",
        OptionKind::Float {
            min: 0.0,
            max: 3600.0
        }
    ),
    option!(
        "audio_fade_out_start",
        "Inizio dissolvenza audio finale (s)",
        "Audio fade-out start (s)",
        "filters",
        OptionKind::Float {
            min: 0.0,
            max: 86400.0
        }
    ),
    option!(
        "audio_fade_out_duration",
        "Durata dissolvenza audio finale (s)",
        "Audio fade-out duration (s)",
        "filters",
        OptionKind::Float {
            min: 0.01,
            max: 3600.0
        }
    ),
    option!(
        "audio_delay_ms",
        "Ritardo audio (ms)",
        "Audio delay (ms)",
        "filters",
        OptionKind::Int { min: 0, max: 60000 }
    ),
    option!(
        "audio_tempo",
        "Velocità audio",
        "Audio tempo",
        "filters",
        OptionKind::Float { min: 0.5, max: 2.0 }
    ),
    option!(
        "strip_metadata",
        "Rimuovi metadati e capitoli",
        "Remove metadata and chapters",
        "metadata",
        OptionKind::Bool
    ),
    option!("title", "Titolo", "Title", "metadata", OptionKind::Text),
    option!("artist", "Artista", "Artist", "metadata", OptionKind::Text),
    option!("album", "Album", "Album", "metadata", OptionKind::Text),
    option!(
        "comment",
        "Commento",
        "Comment",
        "metadata",
        OptionKind::Text
    ),
    option!("date", "Data", "Date", "metadata", OptionKind::Text),
    option!("genre", "Genere", "Genre", "metadata", OptionKind::Text),
    option!(
        "copyright",
        "Copyright",
        "Copyright",
        "metadata",
        OptionKind::Text
    ),
    option!(
        "track",
        "Numero traccia",
        "Track number",
        "metadata",
        OptionKind::Text
    ),
    option!(
        "language",
        "Lingua metadati",
        "Metadata language",
        "metadata",
        OptionKind::Text
    ),
    option!(
        "threads",
        "Thread encoder (0 = automatici)",
        "Encoder threads (0 = automatic)",
        "performance",
        OptionKind::Int { min: 0, max: 128 }
    ),
    option!(
        "bitexact",
        "Codifica deterministica",
        "Bit-exact encoding",
        "performance",
        OptionKind::Bool
    ),
    option!(
        "mp4_faststart",
        "MP4: avvio rapido",
        "MP4 fast start",
        "performance",
        OptionKind::Bool
    ),
];

fn is_video(format: OutputFormat) -> bool {
    format.kind() == MediaKind::Video
}
fn is_media(format: OutputFormat) -> bool {
    is_video(format) || format.kind() == MediaKind::Audio || format == OutputFormat::Gif
}

fn compatible(key: &str, format: OutputFormat) -> bool {
    if !is_media(format) {
        return false;
    }
    if key.starts_with("codec:") {
        return true;
    }
    if key == "mp4_faststart" {
        return matches!(
            format,
            OutputFormat::Mp4 | OutputFormat::Mov | OutputFormat::M4v
        );
    }
    if key == "audio_compression_level" {
        return matches!(format, OutputFormat::Flac | OutputFormat::Opus);
    }
    if key.starts_with("audio_") {
        return format != OutputFormat::Gif;
    }
    if key.starts_with("crop_")
        || key.starts_with("scale_")
        || matches!(
            key,
            "rotation" | "horizontal_flip" | "vertical_flip" | "deinterlace"
        )
    {
        return is_video(format);
    }
    if key.starts_with("video_")
        || matches!(
            key,
            "preset"
                | "tune"
                | "profile"
                | "level"
                | "pixel_format"
                | "max_rate_kbps"
                | "min_rate_kbps"
                | "buffer_size_kbits"
                | "crf"
                | "qscale"
                | "gop_size"
                | "b_frames"
                | "reference_frames"
                | "display_aspect"
                | "color_range"
                | "color_primaries"
                | "color_transfer"
                | "color_space"
        )
    {
        return is_video(format);
    }
    true
}

pub fn options_for(format: OutputFormat) -> Vec<&'static OptionSpec> {
    SPECS
        .iter()
        .filter(|spec| compatible(spec.key, format))
        .collect()
}

/// Container-specific choices; a compiled encoder can still require hardware.
pub fn video_encoder_choices(format: OutputFormat) -> &'static [&'static str] {
    match format {
        OutputFormat::Mp4 | OutputFormat::M4v | OutputFormat::Mov | OutputFormat::Ts => &[
            "libx264",
            "libx265",
            "mpeg4",
            "mpeg2video",
            "libaom-av1",
            "libsvtav1",
            "librav1e",
            "h264_nvenc",
            "hevc_nvenc",
            "av1_nvenc",
            "h264_qsv",
            "hevc_qsv",
            "av1_qsv",
            "h264_amf",
            "hevc_amf",
            "av1_amf",
        ],
        OutputFormat::Mkv => VIDEO_ENCODERS,
        OutputFormat::Webm => &[
            "libvpx-vp9",
            "libvpx",
            "libaom-av1",
            "libsvtav1",
            "librav1e",
        ],
        OutputFormat::Avi => &["mpeg4", "libx264", "mjpeg", "huffyuv", "ffv1", "rawvideo"],
        OutputFormat::Mpeg => &["mpeg2video", "mpeg1video"],
        OutputFormat::Wmv => &["wmv1", "wmv2"],
        OutputFormat::Flv => &["flv", "libx264"],
        OutputFormat::ThreeGp => &["mpeg4", "libx264", "h263"],
        OutputFormat::Gif => &["gif"],
        _ => &[],
    }
}

pub fn audio_encoder_choices(format: OutputFormat) -> &'static [&'static str] {
    match format {
        OutputFormat::Mp3 => &["libmp3lame"],
        OutputFormat::Wav => &[
            "pcm_s16le",
            "pcm_s24le",
            "pcm_s32le",
            "pcm_f32le",
            "pcm_f64le",
        ],
        OutputFormat::Aiff | OutputFormat::Au => {
            &["pcm_s16be", "pcm_s24be", "pcm_s32be", "pcm_f32be"]
        }
        OutputFormat::Flac => &["flac"],
        OutputFormat::Aac | OutputFormat::M4a => &["aac", "libfdk_aac"],
        OutputFormat::Ogg => &["libvorbis", "libopus", "flac"],
        OutputFormat::Opus => &["libopus", "opus"],
        OutputFormat::Wma | OutputFormat::Wmv => &["wmav2", "wmav1"],
        OutputFormat::Ac3 => &["ac3", "ac3_fixed"],
        OutputFormat::Caf => &["alac", "pcm_s16le", "pcm_s24le", "pcm_s32le", "pcm_f32le"],
        OutputFormat::Webm => &["libopus", "libvorbis"],
        OutputFormat::Mpeg => &["mp2", "ac3"],
        OutputFormat::Flv => &["libmp3lame", "aac"],
        OutputFormat::ThreeGp => &["aac"],
        OutputFormat::Mp4 | OutputFormat::M4v | OutputFormat::Mov => {
            &["aac", "libfdk_aac", "libmp3lame", "ac3", "alac"]
        }
        OutputFormat::Ts => &["aac", "mp2", "ac3"],
        OutputFormat::Avi => &["libmp3lame", "pcm_s16le", "ac3"],
        OutputFormat::Mkv => AUDIO_ENCODERS,
        _ => &[],
    }
}

pub fn encoder_for(settings: &ConversionSettings) -> String {
    let key = if is_video(settings.format) || settings.format == OutputFormat::Gif {
        "video_encoder"
    } else {
        "audio_encoder"
    };
    if let Some(encoder) = settings.media_options.get(key) {
        return encoder.clone();
    }
    if key == "audio_encoder" {
        return audio_encoder_for(settings);
    }
    match settings.format {
        OutputFormat::Webm => "libvpx-vp9",
        OutputFormat::Avi | OutputFormat::ThreeGp => "mpeg4",
        OutputFormat::Mpeg => "mpeg2video",
        OutputFormat::Wmv => "wmv2",
        OutputFormat::Flv => "flv",
        OutputFormat::Gif => "gif",
        _ if settings.video.codec == VideoCodec::H265 && settings.format != OutputFormat::Ts => {
            "libx265"
        }
        _ => "libx264",
    }
    .into()
}

pub fn audio_encoder_for(settings: &ConversionSettings) -> String {
    if let Some(encoder) = settings.media_options.get("audio_encoder") {
        return encoder.clone();
    }
    match settings.format {
        OutputFormat::Mp3 | OutputFormat::Avi | OutputFormat::Flv => "libmp3lame",
        OutputFormat::Wav => "pcm_s16le",
        OutputFormat::Aiff | OutputFormat::Au => "pcm_s16be",
        OutputFormat::Flac => "flac",
        OutputFormat::Ogg => "libvorbis",
        OutputFormat::Opus | OutputFormat::Webm => "libopus",
        OutputFormat::Wma | OutputFormat::Wmv => "wmav2",
        OutputFormat::Ac3 => "ac3",
        OutputFormat::Caf => "alac",
        OutputFormat::Mpeg => "mp2",
        _ => "aac",
    }
    .into()
}

fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        && !value.starts_with('-')
}

fn dynamic_key(key: &str) -> Result<(&str, &str)> {
    let mut parts = key.split(':');
    let (Some("codec"), Some(encoder), Some(parameter), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        bail!(tr(
            "Chiave opzione encoder non valida",
            "Invalid encoder option key"
        ));
    };
    ensure!(
        safe_name(encoder) && safe_name(parameter) && !unsafe_parameter(parameter),
        "{}",
        tr(
            "Opzione encoder non consentita",
            "Encoder option is not allowed"
        )
    );
    Ok((encoder, parameter))
}

fn unsafe_parameter(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    [
        "file",
        "path",
        "url",
        "socket",
        "http",
        "log",
        "stats",
        "dump",
        "script",
        "report",
        "pass",
        "attachment",
    ]
    .iter()
    .any(|fragment| name.contains(fragment))
        || [
            "i",
            "y",
            "n",
            "f",
            "map",
            "c",
            "codec",
            "vf",
            "af",
            "filter_complex",
            "h",
            "help",
            "progress",
            "protocol_whitelist",
            "ss",
            "t",
            "to",
            "fs",
        ]
        .contains(&name.as_str())
}

fn boolean(value: &str) -> Option<bool> {
    match value {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn rational(value: &str) -> bool {
    let Some((numerator, denominator)) = value.split_once('/') else {
        return false;
    };
    matches!((numerator.parse::<i64>(), denominator.parse::<i64>()), (Ok(a), Ok(b)) if a.abs_diff(0) <= 1_000_000_000 && b > 0 && b <= 1_000_000_000)
}

fn validate_value(kind: OptionKind, value: &str) -> bool {
    match kind {
        OptionKind::Bool => boolean(value).is_some(),
        OptionKind::Int { min, max } => {
            value.parse::<i64>().is_ok_and(|v| (min..=max).contains(&v))
        }
        OptionKind::Float { min, max } => value
            .parse::<f64>()
            .is_ok_and(|v| v.is_finite() && (min..=max).contains(&v)),
        OptionKind::Choice(choices) => choices.contains(&value),
        OptionKind::Text => true,
    }
}

pub fn validate(options: &BTreeMap<String, String>) -> Result<()> {
    ensure!(
        options.len() <= 128
            && options
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>()
                <= 32768,
        "{}",
        tr("Troppe opzioni motore", "Too many engine options")
    );
    for (key, value) in options {
        ensure!(
            value.len() <= 4096 && !value.chars().any(char::is_control),
            "{}: {key}",
            tr("Valore opzione non valido", "Invalid option value")
        );
        if key.starts_with("codec:") {
            dynamic_key(key)?;
            ensure!(
                !value.is_empty()
                    && value.len() <= 128
                    && value
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_+|./-".contains(&b)),
                "{}: {key}",
                tr("Valore encoder non sicuro", "Unsafe encoder value")
            );
        } else {
            let spec = SPECS.iter().find(|spec| spec.key == key).with_context(|| {
                format!(
                    "{}: {key}",
                    tr("Opzione motore sconosciuta", "Unknown engine option")
                )
            })?;
            ensure!(
                validate_value(spec.kind, value),
                "{}: {key}",
                tr(
                    "Valore opzione fuori intervallo",
                    "Option value is outside its allowed range"
                )
            );
            if key == "display_aspect" {
                ensure!(
                    rational(value) && !value.starts_with('-') && !value.starts_with("0/"),
                    "{}",
                    tr("Proporzioni non valide", "Invalid display aspect")
                );
            }
        }
    }
    for (left, right) in [
        ("crop_width", "crop_height"),
        ("audio_fade_out_start", "audio_fade_out_duration"),
        ("video_fade_out_start", "video_fade_out_duration"),
    ] {
        ensure!(
            options.contains_key(left) == options.contains_key(right),
            "{}: {left}, {right}",
            tr("Specificare entrambe le opzioni", "Specify both options")
        );
    }
    ensure!(
        !(options.contains_key("crop_x") || options.contains_key("crop_y"))
            || options.contains_key("crop_width"),
        "{}",
        tr(
            "Il ritaglio richiede larghezza e altezza",
            "Cropping requires width and height"
        )
    );
    Ok(())
}

#[derive(Clone, Debug)]
pub struct EncoderOption {
    pub key: String,
    pub name: String,
    pub description: String,
    pub kind: OptionKind,
    pub default: Option<String>,
    /// Runtime enum names. Synthetic pixel/sample-format entries use static keys.
    pub choices: Vec<String>,
}

#[derive(Clone, Debug)]
enum Constraint {
    Number,
    Boolean { automatic: bool },
    Rational,
    Flags,
}

#[derive(Clone, Debug)]
struct Discovered {
    public: EncoderOption,
    constraint: Constraint,
}

#[derive(Clone, Debug)]
struct EncoderHelp {
    options: Vec<Discovered>,
    pixel_formats: Vec<String>,
    sample_formats: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CacheKey {
    path: PathBuf,
    length: u64,
    modified: Option<SystemTime>,
    encoder: String,
}

static ENCODER_CACHE: OnceLock<Mutex<HashMap<CacheKey, EncoderHelp>>> = OnceLock::new();

fn bound(value: &str) -> Option<f64> {
    match value {
        "INT_MIN" => Some(i32::MIN as f64),
        "INT_MAX" => Some(i32::MAX as f64),
        "I64_MIN" | "INT64_MIN" => Some(i64::MIN as f64),
        "I64_MAX" | "INT64_MAX" | "UINT64_MAX" => Some(i64::MAX as f64),
        "UINT32_MAX" => Some(u32::MAX as f64),
        "FLT_MAX" | "DBL_MAX" => Some(f64::MAX),
        value => value.parse::<f64>().ok().filter(|value| value.is_finite()),
    }
}

fn range(description: &str, fallback: (f64, f64)) -> (f64, f64) {
    description
        .split_once("(from ")
        .and_then(|(_, after)| after.split_once(')'))
        .and_then(|(range, _)| range.split_once(" to "))
        .and_then(|(min, max)| Some((bound(min)?, bound(max)?)))
        .filter(|(min, max)| min <= max)
        .unwrap_or(fallback)
}

fn default_value(description: &str) -> Option<String> {
    description
        .split_once("(default ")
        .and_then(|(_, after)| after.split_once(')'))
        .map(|(value, _)| value.trim_matches('"').to_owned())
}

fn parse_help(encoder: &str, text: &str) -> Result<EncoderHelp> {
    ensure!(
        text.lines()
            .any(|line| line.starts_with(&format!("Encoder {encoder} ["))),
        "{}: {encoder}",
        tr(
            "Encoder non disponibile in FFmpeg",
            "Encoder is unavailable in FFmpeg"
        )
    );
    let header_values = |prefix: &str| -> Vec<String> {
        text.lines()
            .find_map(|line| line.trim().strip_prefix(prefix))
            .map(|value| {
                value
                    .split_whitespace()
                    .filter(|value| safe_name(value))
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut help = EncoderHelp {
        options: Vec::new(),
        pixel_formats: header_values("Supported pixel formats:"),
        sample_formats: header_values("Supported sample formats:"),
    };
    let mut current: Option<Discovered> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('-') {
            if let Some(option) = current.take() {
                help.options.push(option);
            }
            let Some((name, after)) = trimmed.split_once(char::is_whitespace) else {
                continue;
            };
            let name = name.trim_start_matches('-');
            if !safe_name(name) || unsafe_parameter(name) {
                continue;
            }
            let Some((_, after)) = after.split_once('<') else {
                continue;
            };
            let Some((value_type, description)) = after.split_once('>') else {
                continue;
            };
            let description = description.trim();
            if !description
                .split_whitespace()
                .next()
                .is_some_and(|flags| flags.contains('E'))
            {
                continue;
            }
            let default = default_value(description);
            let (kind, constraint) = match value_type {
                "boolean" | "bool" => {
                    let automatic = default
                        .as_deref()
                        .is_some_and(|value| value == "auto" || value == "-1");
                    (
                        if automatic {
                            OptionKind::Int { min: -1, max: 1 }
                        } else {
                            OptionKind::Bool
                        },
                        Constraint::Boolean { automatic },
                    )
                }
                "int" | "int64" | "uint64" => {
                    let (min, max) = range(description, (i32::MIN as f64, i32::MAX as f64));
                    (
                        OptionKind::Int {
                            min: min.max(i64::MIN as f64) as i64,
                            max: max.min(i64::MAX as f64) as i64,
                        },
                        Constraint::Number,
                    )
                }
                "float" | "double" => {
                    let (min, max) = range(description, (-f64::MAX, f64::MAX));
                    (OptionKind::Float { min, max }, Constraint::Number)
                }
                "duration" => {
                    let (min, max) = range(description, (-86400.0, 86400.0));
                    (
                        OptionKind::Float {
                            min: min.max(-86400.0),
                            max: max.min(86400.0),
                        },
                        Constraint::Number,
                    )
                }
                "rational" => (OptionKind::Text, Constraint::Rational),
                "flags" => (OptionKind::Text, Constraint::Flags),
                _ => continue, // Strings/binary/dictionaries can contain files or programs.
            };
            current = Some(Discovered {
                public: EncoderOption {
                    key: format!("codec:{encoder}:{name}"),
                    name: name.into(),
                    description: description.into(),
                    kind,
                    default,
                    choices: if matches!(&constraint, Constraint::Boolean { automatic: true }) {
                        vec!["auto".into(), "0".into(), "1".into()]
                    } else {
                        Vec::new()
                    },
                },
                constraint,
            });
        } else if let Some(option) = current.as_mut() {
            // Flag names can omit the numeric code; integer enums include it.
            let fields: Vec<_> = trimmed.split_whitespace().take(3).collect();
            let leaf = match fields.as_slice() {
                [name, flags, ..]
                    if matches!(option.constraint, Constraint::Flags) && flags.contains('E') =>
                {
                    Some(*name)
                }
                [name, value, flags]
                    if flags.contains('E')
                        && (bound(value).is_some()
                            || value
                                .strip_prefix("0x")
                                .is_some_and(|hex| u64::from_str_radix(hex, 16).is_ok())) =>
                {
                    Some(*name)
                }
                _ => None,
            };
            if let Some(name) = leaf
                && safe_name(name)
                && !option
                    .public
                    .choices
                    .iter()
                    .any(|existing| existing == name)
            {
                option.public.choices.push(name.into());
            }
        }
    }
    if let Some(option) = current {
        help.options.push(option);
    }
    help.options.retain(|option| {
        !matches!(option.constraint, Constraint::Flags) || !option.public.choices.is_empty()
    });
    ensure!(
        help.options.len() <= 256,
        "{}",
        tr(
            "Catalogo encoder troppo grande",
            "Encoder catalog is too large"
        )
    );
    Ok(help)
}

fn encoder_help(tools: &ToolPaths, encoder: &str, cancel: &AtomicBool) -> Result<EncoderHelp> {
    cancelled(cancel)?;
    ensure!(
        safe_name(encoder),
        "{}",
        tr("Nome encoder non valido", "Invalid encoder name")
    );
    let path = fs::canonicalize(
        tools
            .ffmpeg
            .as_ref()
            .context(tr("FFmpeg non trovato", "FFmpeg was not found"))?,
    )?;
    let metadata = fs::metadata(&path)?;
    let key = CacheKey {
        path: path.clone(),
        length: metadata.len(),
        modified: metadata.modified().ok(),
        encoder: encoder.into(),
    };
    let cache = ENCODER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(help) = cache
        .lock()
        .map_err(|_| {
            anyhow::anyhow!(tr(
                "Cache encoder non disponibile",
                "Encoder cache is unavailable"
            ))
        })?
        .get(&key)
        .cloned()
    {
        cancelled(cancel)?;
        return Ok(help);
    }
    let mut command = Command::new(path);
    command.args(["-hide_banner", "-h", &format!("encoder={encoder}")]);
    let result = run_pdf_probe(command, cancel, Duration::from_secs(5)).context(tr(
        "Impossibile leggere le opzioni dell'encoder FFmpeg",
        "Unable to read FFmpeg encoder options",
    ))?;
    cancelled(cancel)?;
    ensure!(
        result.status.success(),
        "{}: {}",
        tr(
            "Lettura encoder FFmpeg fallita",
            "FFmpeg encoder query failed"
        ),
        String::from_utf8_lossy(&result.stderr).trim()
    );
    let help = parse_help(encoder, &String::from_utf8_lossy(&result.stdout))?;
    let mut cache = cache.lock().map_err(|_| {
        anyhow::anyhow!(tr(
            "Cache encoder non disponibile",
            "Encoder cache is unavailable"
        ))
    })?;
    if cache.len() >= 128 {
        cache.clear();
    }
    cache.insert(key, help.clone());
    Ok(help)
}

pub fn discover_encoder_options(
    tools: &ToolPaths,
    encoder: &str,
    cancel: &AtomicBool,
) -> Result<Vec<EncoderOption>> {
    let help = encoder_help(tools, encoder, cancel)?;
    let mut options: Vec<_> = help
        .options
        .into_iter()
        .map(|option| option.public)
        .collect();
    for (key, choices) in [
        ("pixel_format", help.pixel_formats),
        ("audio_sample_format", help.sample_formats),
    ] {
        if !choices.is_empty() {
            options.push(EncoderOption {
                key: key.into(),
                name: key.into(),
                description: format!("Supported by {encoder}"),
                kind: OptionKind::Text,
                default: None,
                choices,
            });
        }
    }
    Ok(options)
}

/// Only these controls need curated string choices absent from FFmpeg's help.
/// Pixel/sample formats are returned as synthetic entries by discovery instead.
pub fn encoder_specific_choices(key: &str, encoder: &str) -> Option<&'static [&'static str]> {
    match key {
        "preset" if matches!(encoder, "libx264" | "libx265") => Some(&[
            "ultrafast",
            "superfast",
            "veryfast",
            "faster",
            "fast",
            "medium",
            "slow",
            "slower",
            "veryslow",
            "placebo",
        ]),
        "tune" if encoder == "libx264" => Some(&[
            "film",
            "animation",
            "grain",
            "stillimage",
            "psnr",
            "ssim",
            "fastdecode",
            "zerolatency",
        ]),
        "tune" if encoder == "libx265" => {
            Some(&["grain", "psnr", "ssim", "fastdecode", "zerolatency"])
        }
        "profile" if encoder == "libx264" => {
            Some(&["baseline", "main", "high", "high10", "high422", "high444"])
        }
        "profile" if encoder == "libx265" => Some(&["main", "main10", "mainstillpicture"]),
        "level" if matches!(encoder, "libx264" | "libx265") => Some(&[
            "1.0", "1.1", "1.2", "1.3", "2.0", "2.1", "2.2", "3.0", "3.1", "3.2", "4.0", "4.1",
            "4.2", "5.0", "5.1", "5.2", "6.0", "6.1", "6.2",
        ]),
        "crf"
            if matches!(
                encoder,
                "libx264" | "libx265" | "libvpx-vp9" | "libaom-av1" | "libsvtav1" | "librav1e"
            ) =>
        {
            Some(&[])
        }
        _ => None,
    }
}

fn validate_discovered(option: &Discovered, value: &str) -> bool {
    match option.constraint {
        Constraint::Boolean { automatic } => {
            boolean(value).is_some() || automatic && matches!(value, "auto" | "-1")
        }
        Constraint::Rational => rational(value) || value == "0",
        Constraint::Flags => {
            value == "0"
                || value
                    .trim_start_matches(['+', '-'])
                    .split(['+', '-', '|'])
                    .all(|part| {
                        !part.is_empty()
                            && option.public.choices.iter().any(|choice| choice == part)
                    })
        }
        Constraint::Number => {
            validate_value(option.public.kind, value)
                || option.public.choices.iter().any(|choice| choice == value)
        }
    }
}

fn enabled(options: &BTreeMap<String, String>, key: &str) -> bool {
    options
        .get(key)
        .is_some_and(|value| boolean(value) == Some(true))
}

fn video_filters(settings: &ConversionSettings) -> Option<String> {
    let options = &settings.media_options;
    if !options.keys().any(|key| {
        key.starts_with("crop_")
            || key.starts_with("scale_")
            || key.starts_with("video_fade_")
            || matches!(
                key.as_str(),
                "rotation" | "horizontal_flip" | "vertical_flip" | "deinterlace"
            )
    }) {
        return None;
    }
    let mut filters = Vec::new();
    if let (Some(width), Some(height)) = (options.get("crop_width"), options.get("crop_height")) {
        filters.push(format!(
            "crop={width}:{height}:{}:{}",
            options.get("crop_x").map_or("0", String::as_str),
            options.get("crop_y").map_or("0", String::as_str)
        ));
    }
    if let Some(filter) = options
        .get("deinterlace")
        .filter(|filter| filter.as_str() != "off")
    {
        filters.push(filter.clone());
    }
    match options.get("rotation").map(String::as_str) {
        Some("90") => filters.push("transpose=clock".into()),
        Some("180") => filters.extend(["hflip".into(), "vflip".into()]),
        Some("270") => filters.push("transpose=cclock".into()),
        _ => {}
    }
    if enabled(options, "horizontal_flip") {
        filters.push("hflip".into());
    }
    if enabled(options, "vertical_flip") {
        filters.push("vflip".into());
    }
    if options.contains_key("scale_width") || options.contains_key("scale_height") {
        filters.push(format!(
            "scale={}:{}:flags=lanczos",
            options.get("scale_width").map_or("-2", String::as_str),
            options.get("scale_height").map_or("-2", String::as_str)
        ));
        // Codec baseline is 4:2:0: retain the engine's even-dimension guarantee.
        filters.push("scale=trunc(iw/2)*2:trunc(ih/2)*2".into());
    } else {
        filters.push(
            settings
                .video
                .width
                .map(|width| format!("scale={}:-2", width / 2 * 2))
                .unwrap_or_else(|| "scale=trunc(iw/2)*2:trunc(ih/2)*2".into()),
        );
    }
    if let Some(value) = options.get("video_fade_in") {
        filters.push(format!("fade=t=in:st=0:d={value}"));
    }
    if let (Some(start), Some(duration)) = (
        options.get("video_fade_out_start"),
        options.get("video_fade_out_duration"),
    ) {
        filters.push(format!("fade=t=out:st={start}:d={duration}"));
    }
    Some(filters.join(","))
}

fn audio_filters(settings: &ConversionSettings) -> Option<String> {
    let options = &settings.media_options;
    if !options.keys().any(|key| {
        matches!(
            key.as_str(),
            "audio_volume"
                | "audio_highpass_hz"
                | "audio_lowpass_hz"
                | "audio_delay_ms"
                | "audio_tempo"
                | "audio_fade_in"
                | "audio_fade_out_start"
                | "audio_fade_out_duration"
        )
    }) {
        return None;
    }
    let mut filters = Vec::new();
    if settings.audio.normalize {
        filters.push("loudnorm=I=-16:TP=-1.5:LRA=11".into());
    }
    for (key, filter) in [
        ("audio_volume", "volume"),
        ("audio_highpass_hz", "highpass=f"),
        ("audio_lowpass_hz", "lowpass=f"),
        ("audio_tempo", "atempo"),
    ] {
        if let Some(value) = options.get(key) {
            filters.push(format!("{filter}={value}"));
        }
    }
    if let Some(value) = options.get("audio_delay_ms") {
        filters.push(format!("adelay={value}:all=1"));
    }
    if let Some(value) = options.get("audio_fade_in") {
        filters.push(format!("afade=t=in:st=0:d={value}"));
    }
    if let (Some(start), Some(duration)) = (
        options.get("audio_fade_out_start"),
        options.get("audio_fade_out_duration"),
    ) {
        filters.push(format!("afade=t=out:st={start}:d={duration}"));
    }
    Some(filters.join(","))
}

fn check_static_encoder_values(
    options: &BTreeMap<String, String>,
    video: Option<(&str, &EncoderHelp)>,
    audio: Option<&EncoderHelp>,
) -> Result<()> {
    for key in ["preset", "tune", "profile", "level", "crf"] {
        if let Some(value) = options.get(key) {
            let (encoder, _) = video.context(tr(
                "L'opzione richiede un encoder video",
                "This option requires a video encoder",
            ))?;
            let choices = encoder_specific_choices(key, encoder).with_context(|| {
                format!(
                    "{}: {encoder}, {key}",
                    tr(
                        "Opzione non supportata dall'encoder",
                        "Option is unsupported by this encoder"
                    )
                )
            })?;
            ensure!(
                choices.is_empty() || choices.contains(&value.as_str()),
                "{}: {encoder}, {key}={value}",
                tr(
                    "Valore non supportato dall'encoder",
                    "Value is unsupported by this encoder"
                )
            );
            if key == "crf" && matches!(encoder, "libx264" | "libx265") {
                ensure!(
                    value.parse::<f64>()? <= 51.0,
                    "{}",
                    tr(
                        "CRF H.264/H.265 non valido (0–51)",
                        "Invalid H.264/H.265 CRF (0–51)"
                    )
                );
            }
        }
    }
    for (key, values) in [
        ("pixel_format", video.map(|(_, help)| &help.pixel_formats)),
        (
            "audio_sample_format",
            audio.map(|help| &help.sample_formats),
        ),
    ] {
        if let Some(value) = options.get(key) {
            ensure!(
                values.is_some_and(|choices| choices.iter().any(|choice| choice == value)),
                "{}: {key}={value}",
                tr(
                    "Formato non supportato dall'encoder installato",
                    "Format is unsupported by the installed encoder"
                )
            );
        }
    }
    Ok(())
}

/// Append only validated output controls; the engine adds its unique output last.
pub fn append_args(
    command: &mut Command,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
) -> Result<()> {
    cancelled(cancel)?;
    let options = &settings.media_options;
    validate(options)?;
    if options.is_empty() {
        return Ok(());
    }
    ensure!(
        is_media(settings.format),
        "{}",
        tr(
            "Le opzioni FFmpeg richiedono audio o video",
            "FFmpeg options require audio or video"
        )
    );
    for key in options.keys() {
        ensure!(
            compatible(key, settings.format),
            "{}: {key}",
            tr(
                "Opzione incompatibile con il formato",
                "Option is incompatible with this format"
            )
        );
    }
    let video = (is_video(settings.format) || settings.format == OutputFormat::Gif)
        .then(|| encoder_for(settings));
    let audio = (settings.format != OutputFormat::Gif).then(|| audio_encoder_for(settings));
    if let Some(encoder) = &video {
        ensure!(
            video_encoder_choices(settings.format).contains(&encoder.as_str()),
            "{}: {encoder}",
            tr(
                "Encoder video incompatibile con il contenitore",
                "Video encoder is incompatible with the container"
            )
        );
    }
    if let Some(encoder) = &audio {
        ensure!(
            audio_encoder_choices(settings.format).contains(&encoder.as_str()),
            "{}: {encoder}",
            tr(
                "Encoder audio incompatibile con il contenitore",
                "Audio encoder is incompatible with the container"
            )
        );
    }
    let video_help = video
        .as_ref()
        .map(|encoder| encoder_help(tools, encoder, cancel))
        .transpose()?;
    let audio_help = audio
        .as_ref()
        .map(|encoder| encoder_help(tools, encoder, cancel))
        .transpose()?;
    check_static_encoder_values(
        options,
        video.as_deref().zip(video_help.as_ref()),
        audio_help.as_ref(),
    )?;
    if settings.format == OutputFormat::Opus
        && let Some(level) = options.get("audio_compression_level")
    {
        ensure!(
            level.parse::<i64>()? <= 10,
            "{}",
            tr(
                "Compressione Opus non valida (0–10)",
                "Invalid Opus compression level (0–10)"
            )
        );
    }
    // Validate every discovered parameter before modifying the caller's command.
    let mut private_args = Vec::new();
    for (key, value) in options.iter().filter(|(key, _)| key.starts_with("codec:")) {
        let (encoder, parameter) = dynamic_key(key)?;
        let (help, stream) = if video.as_deref() == Some(encoder) {
            (video_help.as_ref().unwrap(), "v")
        } else if audio.as_deref() == Some(encoder) {
            (audio_help.as_ref().unwrap(), "a")
        } else {
            bail!(
                "{}: {encoder}",
                tr(
                    "L'opzione appartiene a un encoder non selezionato",
                    "The option belongs to an encoder that is not selected"
                )
            );
        };
        let option = help
            .options
            .iter()
            .find(|option| option.public.name == parameter)
            .with_context(|| {
                format!(
                    "{}: {key}",
                    tr(
                        "Opzione non disponibile nell'encoder installato",
                        "Option is unavailable in the installed encoder"
                    )
                )
            })?;
        ensure!(
            validate_discovered(option, value),
            "{}: {key}",
            tr("Valore encoder non valido", "Invalid encoder value")
        );
        private_args.push((format!("-{parameter}:{stream}"), value.clone()));
    }
    cancelled(cancel)?;
    for (key, argument) in [
        ("video_encoder", "-c:v"),
        ("audio_encoder", "-c:a"),
        ("preset", "-preset:v"),
        ("tune", "-tune:v"),
        ("profile", "-profile:v"),
        ("level", "-level:v"),
        ("pixel_format", "-pix_fmt"),
        ("crf", "-crf:v"),
        ("qscale", "-q:v"),
        ("gop_size", "-g:v"),
        ("b_frames", "-bf:v"),
        ("reference_frames", "-refs:v"),
        ("video_fps", "-r:v"),
        ("display_aspect", "-aspect:v"),
        ("color_range", "-color_range:v"),
        ("color_primaries", "-color_primaries:v"),
        ("color_transfer", "-color_trc:v"),
        ("color_space", "-colorspace:v"),
        ("audio_sample_rate", "-ar:a"),
        ("audio_channels", "-ac:a"),
        ("audio_sample_format", "-sample_fmt:a"),
        ("audio_compression_level", "-compression_level:a"),
        ("threads", "-threads"),
    ] {
        if let Some(value) = options.get(key) {
            command.args([argument, value]);
        }
    }
    if options.contains_key("video_encoder")
        && matches!(
            settings.format,
            OutputFormat::Mp4 | OutputFormat::Mov | OutputFormat::M4v
        )
    {
        let hevc = video
            .as_deref()
            .is_some_and(|encoder| encoder == "libx265" || encoder.starts_with("hevc_"));
        command.args(["-tag:v", if hevc { "hvc1" } else { "0" }]);
    }
    for (key, argument) in [
        ("video_bitrate_kbps", "-b:v"),
        ("max_rate_kbps", "-maxrate:v"),
        ("min_rate_kbps", "-minrate:v"),
        ("buffer_size_kbits", "-bufsize:v"),
        ("audio_bitrate_kbps", "-b:a"),
    ] {
        if let Some(value) = options.get(key) {
            command.args([argument, &format!("{value}k")]);
        }
    }
    if let Some(filter) = video_filters(settings) {
        command.args(["-vf", &filter]);
    }
    if let Some(filter) = audio_filters(settings) {
        command.args(["-af", &filter]);
    }
    if enabled(options, "strip_metadata") {
        command.args(["-map_metadata", "-1", "-map_chapters", "-1"]);
    }
    for key in [
        "title",
        "artist",
        "album",
        "comment",
        "date",
        "genre",
        "copyright",
        "track",
        "language",
    ] {
        if let Some(value) = options.get(key) {
            command.args(["-metadata", &format!("{key}={value}")]);
        }
    }
    if enabled(options, "bitexact") {
        command.args(["-fflags", "+bitexact"]);
    }
    if let Some(value) = options.get("mp4_faststart") {
        command.args([
            "-movflags",
            if boolean(value) == Some(true) {
                "+faststart"
            } else {
                "0"
            },
        ]);
    }
    for (argument, value) in private_args {
        command.args([argument, value]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_parses_numeric_enums_flags_auto_and_rejects_file_capable_types() {
        let help = parse_help(
            "test",
            r#"Encoder test [Test encoder]:
    Supported pixel formats: yuv420p yuv420p10le
  -quality <int64> E..V....... Quality (from -2 to 16) (default balanced)
     balanced 4 E..V....... Balanced
     fast 8 E..V....... Fast
  -automatic <boolean> E..V....... Default control (default auto)
  -ratio <rational> E..V....... Fraction
  -delay <duration> E..V....... Delay (from 0 to I64_MAX) (default 0)
  -features <flags> E..V....... Feature bits
     first E..V....... First feature
     second 0x02 E..V....... Second feature
  -filename <string> E..V....... Write a path
  -binary <binary> E..V....... Binary data
  -passlog <int> E..V....... Side-effectful switch (from 0 to 1)
"#,
        )
        .unwrap();
        let get = |name: &str| {
            help.options
                .iter()
                .find(|option| option.public.name == name)
                .unwrap()
        };
        assert!(validate_discovered(get("quality"), "balanced"));
        assert!(validate_discovered(get("quality"), "16"));
        assert!(!validate_discovered(get("quality"), "17"));
        assert!(validate_discovered(get("automatic"), "auto"));
        assert!(validate_discovered(get("automatic"), "-1"));
        assert!(!validate_discovered(get("automatic"), "2"));
        assert!(validate_discovered(get("ratio"), "30000/1001"));
        assert!(!validate_discovered(get("ratio"), "1/0"));
        assert!(!validate_discovered(get("ratio"), "file/input"));
        assert!(validate_discovered(get("delay"), "0.25"));
        assert!(!validate_discovered(get("delay"), "inf"));
        assert!(validate_discovered(get("features"), "+first+second"));
        assert!(!validate_discovered(get("features"), "first+unknown"));
        assert!(!validate_discovered(get("features"), "../../file"));
        assert!(!help.options.iter().any(|option| {
            ["filename", "binary", "passlog"].contains(&option.public.name.as_str())
        }));
    }

    #[test]
    fn encoder_catalog_does_not_accept_diagnostic_or_unrecognized_help_as_availability() {
        assert!(parse_help("missing", "Codec 'missing' is not recognized by FFmpeg.").is_err());
        assert!(parse_help("libx264", "Encoder other [Other]:").is_err());
    }
}
