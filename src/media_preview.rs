//! Bounded, cancellable source overviews for the single-track media editor.
use crate::{
    engine::{Progress, cancelled, run_pdf_probe},
    i18n::tr,
    model::{ConversionSettings, MediaKind},
    tools::ToolPaths,
};
use anyhow::{Context, Result, ensure};
use image::RgbaImage;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
    time::{Duration, SystemTime},
};

#[derive(Clone, Debug, PartialEq)]
pub struct SourceStamp {
    bytes: u64,
    modified: Option<SystemTime>,
}
impl SourceStamp {
    pub fn read(input: &Path) -> Result<Self> {
        let metadata = fs::metadata(input)?;
        ensure!(
            metadata.is_file(),
            tr(
                "Seleziona un file audio o video",
                "Select an audio or video file"
            )
        );
        Ok(Self {
            bytes: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
    pub fn verify(&self, input: &Path) -> Result<()> {
        ensure!(
            *self == Self::read(input)?,
            tr(
                "Il file sorgente è cambiato: riapri l’editor prima di salvare",
                "The source file has changed: reopen the editor before saving"
            )
        );
        Ok(())
    }
}

pub struct VideoFrame {
    pub time: f64,
    pub pixels: RgbaImage,
}
pub struct MediaPreview {
    pub duration: f64,
    pub has_audio: bool,
    pub has_video: bool,
    pub waveform: Option<RgbaImage>,
    pub frames: Vec<VideoFrame>,
    pub warnings: Vec<String>,
    pub stamp: SourceStamp,
}

struct Metadata {
    duration: f64,
    audio: Option<u32>,
    video: Option<u32>,
    audio_rate: Option<u32>,
}
fn positive_number(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.parse().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
}
fn metadata(bytes: &[u8]) -> Result<Metadata> {
    let json: serde_json::Value = serde_json::from_slice(bytes).context(tr(
        "Metadati audio/video non validi",
        "Invalid audio/video metadata",
    ))?;
    let streams = json["streams"].as_array().context(tr(
        "Nessuna traccia audio o video",
        "No audio or video stream",
    ))?;
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .and_then(|stream| stream["index"].as_u64())
        .and_then(|index| u32::try_from(index).ok());
    let video = streams
        .iter()
        .find(|stream| {
            stream["codec_type"] == "video"
                && stream["disposition"]["attached_pic"].as_u64().unwrap_or(0) == 0
        })
        .and_then(|stream| stream["index"].as_u64())
        .and_then(|index| u32::try_from(index).ok());
    ensure!(
        audio.is_some() || video.is_some(),
        tr(
            "Nessuna traccia audio o video utilizzabile",
            "No usable audio or video stream"
        )
    );
    let duration = positive_number(&json["format"]["duration"])
        .or_else(|| {
            streams
                .iter()
                .filter_map(|stream| positive_number(&stream["duration"]))
                .max_by(f64::total_cmp)
        })
        .context(tr(
            "La durata del file non è disponibile; questo file non può essere tagliato nell’editor",
            "The file duration is unavailable; this file cannot be trimmed in the editor",
        ))?;
    let audio_rate = streams
        .iter()
        .find(|stream| stream["index"].as_u64() == audio.map(u64::from))
        .and_then(|stream| stream["sample_rate"].as_str())
        .and_then(|rate| rate.parse().ok());
    Ok(Metadata {
        duration,
        audio,
        video,
        audio_rate,
    })
}

fn waveform(bytes: &[u8]) -> Result<RgbaImage> {
    let text = std::str::from_utf8(bytes)?;
    let peaks: Vec<f64> = text
        .lines()
        .filter_map(|line| line.strip_prefix("lavfi.astats.Overall.Peak_level="))
        .map(|value| {
            if value == "-inf" {
                Ok(0.0)
            } else {
                let db = value.parse::<f64>()?;
                ensure!(
                    db.is_finite(),
                    tr("Forma d’onda non valida", "Invalid waveform")
                );
                Ok(10.0_f64.powf(db / 20.0).clamp(0.0, 1.0))
            }
        })
        .collect::<Result<_>>()?;
    ensure!(
        !peaks.is_empty() && peaks.len() <= 300,
        tr(
            "Campioni dell’anteprima non validi",
            "Invalid overview samples"
        )
    );
    let mut pixels = RgbaImage::new(1200, 160);
    for x in 0..1200 {
        let peak = peaks[(x as usize * peaks.len() / 1200).min(peaks.len() - 1)];
        let height = (peak.sqrt() * 76.0).round() as u32;
        for y in 79_u32.saturating_sub(height)..=(80 + height).min(159) {
            pixels.put_pixel(x, y, image::Rgba([0, 95, 184, 255]));
        }
    }
    Ok(pixels)
}

fn capture(command: Command, cancel: &AtomicBool, timeout: Duration) -> Result<Vec<u8>> {
    let result = run_pdf_probe(command, cancel, timeout).context(tr(
        "Impossibile preparare l’anteprima audio/video",
        "Could not prepare the audio/video preview",
    ))?;
    ensure!(
        result.status.success(),
        "{}: {}",
        tr(
            "Anteprima audio/video non disponibile",
            "Audio/video preview is unavailable"
        ),
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result.stdout)
}
fn load_asset(path: &Path) -> Result<RgbaImage> {
    ensure!(
        fs::metadata(path)?.len() <= 2 * 1024 * 1024,
        tr("Anteprima troppo grande", "Preview is too large")
    );
    let bytes = fs::read(path)?;
    let dimensions =
        image::ImageReader::with_format(std::io::Cursor::new(&bytes), image::ImageFormat::Png)
            .into_dimensions()?;
    ensure!(
        dimensions.0 > 0 && dimensions.0 <= 1400 && dimensions.1 > 0 && dimensions.1 <= 240,
        tr(
            "Dimensioni anteprima non valide",
            "Invalid preview dimensions"
        )
    );
    Ok(image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?.into_rgba8())
}
fn ffmpeg(tools: &ToolPaths, input: &Path, time: Option<f64>) -> Result<Command> {
    let mut command = Command::new(
        tools
            .ffmpeg
            .as_ref()
            .context(tr("FFmpeg non disponibile", "FFmpeg is unavailable"))?,
    );
    command.args([
        "-hide_banner",
        "-nostdin",
        "-loglevel",
        "error",
        "-y",
        "-filter_threads",
        "1",
        "-filter_complex_threads",
        "1",
    ]);
    if let Some(time) = time {
        command.args(["-ss", &time.to_string()]);
    }
    command
        .args(["-protocol_whitelist", "file,pipe", "-i"])
        .arg(input);
    Ok(command)
}

pub fn prepare(
    input: &Path,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<MediaPreview> {
    cancelled(cancel)?;
    ensure!(
        matches!(
            MediaKind::from_path(input),
            MediaKind::Audio | MediaKind::Video
        ),
        tr(
            "Seleziona un file audio o video",
            "Select an audio or video file"
        )
    );
    let stamp = SourceStamp::read(input)?;
    progress(Progress {
        fraction: 0.0,
        message: tr(
            "Lettura della durata e delle tracce…",
            "Reading duration and streams…",
        )
        .into(),
    });
    let mut probe = Command::new(tools.ffprobe.as_ref().context(tr(
        "Per l’editor serve ffprobe, incluso con FFmpeg",
        "The editor requires ffprobe, included with FFmpeg",
    ))?);
    probe
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-show_entries",
        "format=duration:stream=index,codec_type,duration,sample_rate:stream_disposition=attached_pic",
            "-of",
            "json",
        ])
        .arg(input);
    let info = metadata(&capture(probe, cancel, Duration::from_secs(10))?)?;
    let workspace = tempfile::Builder::new()
        .prefix(".convert-here-preview-")
        .tempdir()?;
    crate::platform::set_hidden(workspace.path())?;
    let mut preview = MediaPreview {
        duration: info.duration,
        has_audio: info.audio.is_some(),
        has_video: info.video.is_some(),
        waveform: None,
        frames: Vec::new(),
        warnings: Vec::new(),
        stamp,
    };
    if let Some(stream) = info.audio {
        if info.duration <= 6.0 * 3600.0 {
            progress(Progress {
                fraction: 0.15,
                message: tr(
                    "Preparazione della forma d’onda…",
                    "Preparing the waveform…",
                )
                .into(),
            });
            let result = (|| -> Result<RgbaImage> {
                let rate = info.audio_rate.context(tr(
                    "Frequenza audio non disponibile",
                    "Audio sample rate unavailable",
                ))?;
                let samples = (f64::from(rate) * info.duration / 256.0).ceil();
                ensure!(
                    (1.0..=1_000_000.0).contains(&samples),
                    tr(
                        "Il file è troppo lungo per la forma d’onda; usa la timeline numerica",
                        "The file is too long for the waveform; use the numeric timeline"
                    )
                );
                let mut command = ffmpeg(tools, input, None)?;
                command.args(["-map",&format!("0:{stream}"),"-vn","-af",&format!("asetnsamples=n={}:p=0,astats=metadata=1:reset=1,ametadata=mode=print:key=lavfi.astats.Overall.Peak_level:file=-",samples as u32),"-threads","1","-f","null","-"]);
                waveform(&capture(command, cancel, Duration::from_secs(30))?)
            })();
            match result {
                Ok(image) => preview.waveform = Some(image),
                Err(error) => {
                    cancelled(cancel)?;
                    preview.warnings.push(format!(
                        "{}: {error:#}",
                        tr("Forma d’onda non disponibile", "Waveform unavailable")
                    ));
                }
            }
        } else {
            preview.warnings.push(tr("Per i file oltre sei ore la forma d’onda non viene caricata; puoi comunque scegliere l’intervallo.", "Waveforms are not loaded for files longer than six hours; you can still select the interval.").into());
        }
    }
    if let Some(stream) = info.video {
        for (index, fraction) in [0.05, 0.35, 0.65, 0.95].into_iter().enumerate() {
            cancelled(cancel)?;
            let time = (info.duration * fraction).min((info.duration - 0.001).max(0.0));
            progress(Progress {
                fraction: 0.5 + index as f32 * 0.1,
                message: tr("Preparazione dei fotogrammi…", "Preparing video frames…").into(),
            });
            let result = (|| -> Result<RgbaImage> {
                let path = workspace.path().join(format!("frame-{index}.png"));
                let mut command = ffmpeg(tools, input, Some(time))?;
                command.args(["-map", &format!("0:{stream}"), "-an", "-vf", "scale=256:144:force_original_aspect_ratio=decrease,pad=256:144:(ow-iw)/2:(oh-ih)/2:color=black", "-frames:v", "1", "-threads", "1", "-c:v", "png", "-f", "image2"]).arg(&path);
                capture(command, cancel, Duration::from_secs(15))?;
                load_asset(&path)
            })();
            match result {
                Ok(pixels) => preview.frames.push(VideoFrame { time, pixels }),
                Err(error) => {
                    cancelled(cancel)?;
                    preview.warnings.push(format!(
                        "{}: {error:#}",
                        tr("Fotogramma non disponibile", "Video frame unavailable")
                    ));
                    break;
                }
            }
        }
    }
    cancelled(cancel)?;
    preview.stamp.verify(input)?;
    progress(Progress {
        fraction: 1.0,
        message: tr("Anteprima pronta", "Preview ready").into(),
    });
    Ok(preview)
}

/// Existing serialized fields are shared by audio, video, profiles and quick jobs.
pub fn settings_for_interval(
    base: &ConversionSettings,
    start: f64,
    end: f64,
    gain_db: f64,
    muted: bool,
) -> Result<ConversionSettings> {
    ensure!(
        start.is_finite() && end.is_finite() && start >= 0.0 && end > start,
        tr(
            "Scegli un intervallo con fine successiva all’inizio",
            "Choose an interval whose end is after its start"
        )
    );
    ensure!(
        gain_db.is_finite() && (-60.0..=20.0).contains(&gain_db),
        tr(
            "Gain non valido: scegli da −60 a +20 dB",
            "Invalid gain: choose −60 to +20 dB"
        )
    );
    let mut settings = base.clone();
    settings.video.trim_start = (start > 0.0).then_some(start);
    settings.video.trim_end = Some(end);
    if muted {
        settings
            .media_options
            .insert("audio_volume".into(), "0".into());
    } else if gain_db.abs() < 1e-10 {
        settings.media_options.remove("audio_volume");
    } else {
        settings.media_options.insert(
            "audio_volume".into(),
            10.0_f64.powf(gain_db / 20.0).min(10.0).to_string(),
        );
    }
    Ok(settings)
}

/// Convert privately, then atomically publish the selected file without replacing
/// an existing file, including the source and any aliases of it.
pub fn save(
    input: &Path,
    destination: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    stamp: &SourceStamp,
    cancel: &AtomicBool,
    progress: impl FnMut(Progress),
) -> Result<PathBuf> {
    cancelled(cancel)?;
    stamp.verify(input)?;
    ensure!(
        destination
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case(settings.format.extension())),
        tr(
            "L’estensione deve corrispondere al formato scelto",
            "The extension must match the selected format"
        )
    );
    ensure!(
        !destination.try_exists()?,
        tr(
            "Il file esiste già: scegli un altro nome",
            "The file already exists: choose another name"
        )
    );
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    ensure!(
        parent.is_dir(),
        tr(
            "La cartella di destinazione non esiste",
            "The destination folder does not exist"
        )
    );
    let stage = tempfile::Builder::new()
        .prefix(".convert-here-media-")
        .tempdir_in(parent)?;
    crate::platform::set_hidden(stage.path())?;
    let mut private = settings.clone();
    private.output_dir = Some(stage.path().to_owned());
    let outputs = crate::engine::convert(input, &private, tools, cancel, progress)?;
    ensure!(
        outputs.len() == 1,
        tr(
            "L’editor richiede un singolo risultato",
            "The editor requires a single output"
        )
    );
    stamp.verify(input)?;
    let mut source = fs::File::open(&outputs[0])?;
    let mut target = tempfile::NamedTempFile::new_in(parent)?;
    crate::platform::set_hidden(target.path())?;
    let mut buffer = [0u8; 65536];
    loop {
        cancelled(cancel)?;
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        target.write_all(&buffer[..read])?;
    }
    target.as_file().sync_all()?;
    cancelled(cancel)?;
    stamp.verify(input)?;
    target
        .persist_noclobber(destination)
        .map_err(|error| error.error)
        .context(tr(
            "Salvataggio non riuscito; nessun file esistente è stato sostituito",
            "Could not save; no existing file was replaced",
        ))?;
    Ok(destination.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn peak_waveform_keeps_high_frequency_energy_and_silence_without_raw_audio_buffers() {
        let image=waveform(b"frame:0\nlavfi.astats.Overall.Peak_level=-18.063656\nframe:1\nlavfi.astats.Overall.Peak_level=-inf\n").unwrap();
        assert_eq!(image.dimensions(), (1200, 160));
        assert!(image.get_pixel(0, 55)[3] > 0);
        assert_eq!(image.get_pixel(1199, 55)[3], 0);
        assert!(waveform(b"lavfi.astats.Overall.Peak_level=NaN\n").is_err());
    }
    #[test]
    fn metadata_excludes_cover_art_and_uses_finite_duration() {
        let info = metadata(br#"{"format":{"duration":"2.5"},"streams":[{"index":0,"codec_type":"video","disposition":{"attached_pic":1}},{"index":1,"codec_type":"audio"}]}"#).unwrap();
        assert_eq!(info.duration, 2.5);
        assert!(info.video.is_none());
        assert_eq!(info.audio, Some(1));
        assert!(
            metadata(
                br#"{"format":{"duration":"NaN"},"streams":[{"index":0,"codec_type":"audio"}]}"#
            )
            .is_err()
        );
    }
    #[test]
    fn interval_and_gain_preserve_other_options_and_do_not_modify_snapshot() {
        let mut base = ConversionSettings::default();
        base.audio.normalize = true;
        base.media_options
            .insert("title".into(), "Preserved".into());
        let selected = settings_for_interval(&base, 1.0, 2.5, -6.020599913279624, false).unwrap();
        assert_eq!(selected.video.trim_start, Some(1.0));
        assert_eq!(selected.video.trim_end, Some(2.5));
        assert!(selected.audio.normalize);
        assert_eq!(selected.media_options["title"], "Preserved");
        assert!(
            (selected.media_options["audio_volume"]
                .parse::<f64>()
                .unwrap()
                - 0.5)
                .abs()
                < 1e-9
        );
        assert!(base.video.trim_start.is_none());
        assert!(!base.media_options.contains_key("audio_volume"));
        assert!(settings_for_interval(&base, 2.0, 1.0, 0.0, false).is_err());
        assert!(settings_for_interval(&base, 0.0, 1.0, f64::NAN, false).is_err());
    }
}
