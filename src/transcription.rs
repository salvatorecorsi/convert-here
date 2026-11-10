//! Offline speech recognition. Only the explicit engine/model installation uses the network.
use crate::{
    engine::{Progress, cancelled, commit, run_pdf_probe, run_process, stem, temporary},
    i18n::tr,
    model::{ConversionSettings, SpeechSettings},
    platform::set_hidden,
    tools::ToolPaths,
};
use anyhow::{Context, Result, bail};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
    time::Duration,
};

// Process one bounded PCM buffer at a time, including for long recordings.
const CHUNK_SECONDS: f64 = 600.0;
const MAX_TEXT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_WAVE_BYTES: u64 = 600 * 16_000 * 2 + 65_536;

pub(crate) fn validate_settings(settings: &SpeechSettings) -> Result<()> {
    if settings.language != "auto"
        && (!(2..=3).contains(&settings.language.len())
            || !settings
                .language
                .bytes()
                .all(|byte| byte.is_ascii_lowercase()))
    {
        bail!(tr(
            "Lingua trascrizione non valida: usa auto, it, en o un codice Whisper",
            "Invalid transcription language: use auto, it, en or a Whisper language code"
        ));
    }
    if settings.threads > 32 {
        bail!(tr(
            "Thread di trascrizione: da 1 a 32, oppure 0 per automatico",
            "Transcription threads: 1–32, or 0 for automatic"
        ));
    }
    Ok(())
}

fn selected_interval(duration: f64, settings: &ConversionSettings) -> Result<(f64, f64)> {
    if !duration.is_finite() || duration <= 0.0 {
        bail!(tr(
            "Durata audio non leggibile",
            "Unable to read audio duration"
        ));
    }
    let start = settings.video.trim_start.unwrap_or(0.0);
    let mut end = settings.video.trim_end.unwrap_or(duration).min(duration);
    if let Some(maximum) = settings.limits.max_duration {
        end = end.min(start + maximum);
    }
    if start >= end {
        bail!(tr(
            "Il taglio non contiene audio",
            "The selected interval contains no audio"
        ));
    }
    Ok((start, end))
}

fn audio_duration(input: &Path, probe: &Path, cancel: &AtomicBool) -> Result<f64> {
    let mut command = Command::new(probe);
    command
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_type:format=duration",
            "-of",
            "json",
        ])
        .arg(input);
    let result = run_pdf_probe(command, cancel, Duration::from_secs(15))?;
    if !result.status.success() {
        bail!(
            "{}: {}",
            tr("Analisi audio non riuscita", "Audio analysis failed"),
            String::from_utf8_lossy(&result.stderr).trim()
        );
    }
    let metadata: serde_json::Value = serde_json::from_slice(&result.stdout)?;
    if metadata["streams"]
        .as_array()
        .is_none_or(|streams| streams.is_empty())
    {
        bail!(tr(
            "Il file non contiene una traccia audio",
            "The file contains no audio track"
        ));
    }
    metadata["format"]["duration"]
        .as_str()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
        .context(tr(
            "Durata audio non leggibile",
            "Unable to read audio duration",
        ))
}

pub(crate) fn convert_media_text(
    input: &Path,
    dir: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    cancelled(cancel)?;
    validate_settings(&settings.speech)?;
    let ffmpeg = tools.ffmpeg.as_deref().context(tr(
        "Installa FFmpeg nelle Impostazioni",
        "Install FFmpeg in Settings",
    ))?;
    let ffprobe = tools.ffprobe.as_deref().context(tr(
        "Installa FFmpeg (incluso ffprobe) nelle Impostazioni",
        "Install FFmpeg (including ffprobe) in Settings",
    ))?;
    let whisper = tools.whisper_cli.as_deref().context(tr(
        "Installa Whisper nelle Impostazioni",
        "Install Whisper in Settings",
    ))?;
    let model = tools
        .whisper_model
        .as_deref()
        .filter(|path| crate::tools::model_available(path))
        .context(tr(
            "Installa il modello Whisper nelle Impostazioni",
            "Install the Whisper model in Settings",
        ))?;
    let (start, end) = selected_interval(audio_duration(input, ffprobe, cancel)?, settings)?;
    let count = ((end - start) / CHUNK_SECONDS).ceil() as u64;
    // Reject absurd metadata instead of allocating or scheduling an unbounded job.
    if count > 10_000 {
        bail!(tr(
            "Registrazione troppo lunga: scegli un intervallo più breve",
            "Recording too long: select a shorter interval"
        ));
    }
    let threads = if settings.speech.threads == 0 {
        std::thread::available_parallelism()
            .map_or(4, usize::from)
            .clamp(1, 8)
    } else {
        usize::from(settings.speech.threads)
    };
    let stage = tempfile::Builder::new()
        .prefix(".convert-here-speech-")
        .tempdir_in(dir)?;
    set_hidden(stage.path())?;
    let wave = stage.path().join("audio.wav");
    let text_base = stage.path().join("speech");
    let text_path = text_base.with_extension("txt");
    let mut output = temporary(dir, "txt")?;
    let mut bytes_written = 0_u64;
    for index in 0..count {
        cancelled(cancel)?;
        let offset = start + index as f64 * CHUNK_SECONDS;
        let length = (end - offset).min(CHUNK_SECONDS);
        let message = crate::trf!(
            "Trascrizione {} di {count}…",
            "Transcribing {} of {count}…",
            index + 1
        );
        progress(Progress {
            fraction: index as f32 / count as f32,
            message: message.clone(),
        });
        let mut decode = Command::new(ffmpeg);
        decode
            .args([
                "-hide_banner",
                "-nostdin",
                "-y",
                "-loglevel",
                "error",
                "-progress",
                "pipe:1",
                "-nostats",
                "-ss",
            ])
            .arg(format!("{offset:.6}"))
            .args(["-protocol_whitelist", "file,pipe", "-i"])
            .arg(input)
            .args([
                "-t",
                &format!("{length:.6}"),
                "-map",
                "0:a:0",
                "-vn",
                "-sn",
                "-dn",
                "-ac",
                "1",
                "-ar",
                "16000",
                "-c:a",
                "pcm_s16le",
                "-f",
                "wav",
            ])
            .arg(&wave);
        run_process(decode, cancel, Some(length), &mut |update| {
            progress(Progress {
                fraction: (index as f32 + update.fraction * 0.15) / count as f32,
                message: message.clone(),
            })
        })?;
        cancelled(cancel)?;
        let wave_bytes = fs::metadata(&wave)?.len();
        if wave_bytes > MAX_WAVE_BYTES {
            bail!(tr(
                "Il motore audio ha superato il limite del segmento",
                "The audio engine exceeded the segment size limit"
            ));
        }
        if wave_bytes <= 128 {
            break;
        }
        // No arbitrary command line parameters or network-capable model URLs.
        let mut recognize = Command::new(whisper);
        recognize
            .arg("-m")
            .arg(model)
            .arg("-f")
            .arg(&wave)
            .args([
                "-l",
                &settings.speech.language,
                "-t",
                &threads.to_string(),
                "-ng",
                "-otxt",
                "-np",
                "-of",
            ])
            .arg(&text_base);
        progress(Progress {
            fraction: (index as f32 + 0.15) / count as f32,
            message: message.clone(),
        });
        run_process(recognize, cancel, None, &mut |_| {})?;
        cancelled(cancel)?;
        let length = fs::metadata(&text_path)
            .context(tr(
                "Whisper non ha prodotto il testo",
                "Whisper produced no text file",
            ))?
            .len();
        if bytes_written.saturating_add(length).saturating_add(1) > MAX_TEXT_BYTES {
            bail!(tr(
                "La trascrizione supera il limite di 64 MB",
                "The transcript exceeds the 64 MB limit"
            ));
        }
        let mut text = String::new();
        fs::File::open(&text_path)?
            .take(MAX_TEXT_BYTES + 1)
            .read_to_string(&mut text)?;
        let text = text.trim_matches(['\u{feff}', '\r', '\n', ' ']);
        if !text.is_empty() {
            output.as_file_mut().write_all(text.as_bytes())?;
            output.as_file_mut().write_all(b"\n")?;
            bytes_written += text.len() as u64 + 1;
        }
        fs::remove_file(&wave)?;
        fs::remove_file(&text_path)?;
    }
    cancelled(cancel)?;
    if bytes_written == 0 {
        bail!(tr(
            "Nessun parlato riconosciuto nell’intervallo selezionato",
            "No speech was recognized in the selected interval"
        ));
    }
    commit(output, dir, &stem(input), "txt")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trim_and_duration_cap_are_applied_before_transcribing() {
        let mut settings = ConversionSettings::default();
        settings.video.trim_start = Some(30.0);
        settings.video.trim_end = Some(90.0);
        settings.limits.max_duration = Some(20.0);
        assert_eq!(selected_interval(120.0, &settings).unwrap(), (30.0, 50.0));
        assert!(selected_interval(25.0, &settings).is_err());
        assert!(selected_interval(f64::NAN, &settings).is_err());
    }
    #[test]
    fn language_and_threads_cannot_inject_arguments_or_paths() {
        for language in ["auto", "it", "en", "yue"] {
            assert!(
                validate_settings(&SpeechSettings {
                    language: language.into(),
                    threads: 0
                })
                .is_ok()
            );
        }
        for language in ["", "../it", "--help", "ita+eng", "en -tr", "ITALIAN"] {
            assert!(
                validate_settings(&SpeechSettings {
                    language: language.into(),
                    threads: 0
                })
                .is_err()
            );
        }
        assert!(
            validate_settings(&SpeechSettings {
                threads: 33,
                ..Default::default()
            })
            .is_err()
        );
    }
}
