use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine,
    model::{ConversionSettings, OutputFormat},
    tools::ToolPaths,
};

const NO_AUDIO_ERROR: &str = "Il video non contiene una traccia audio da estrarre";

fn command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}

fn tools() -> Option<ToolPaths> {
    let tools = ToolPaths::discover();
    if tools.ffmpeg.is_none() {
        eprintln!("SKIP audio-track fixture: FFmpeg unavailable");
        return None;
    }
    Some(tools)
}

fn fixture(path: &Path, tools: &ToolPaths, audio: bool) {
    let mut command = command(tools.ffmpeg.as_ref().unwrap());
    command.args([
        "-hide_banner",
        "-nostdin",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        "testsrc2=size=32x32:rate=10",
    ]);
    if audio {
        command.args(["-f", "lavfi", "-i", "sine=frequency=440:sample_rate=16000"]);
    }
    command.args(["-t", "0.3", "-c:v", "libx264"]);
    if audio {
        command.args(["-c:a", "aac", "-shortest"]);
    } else {
        command.arg("-an");
    }
    let output = command.arg(path).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn probe_variants(tools: &ToolPaths) -> Vec<(&'static str, ToolPaths)> {
    let mut variants = Vec::new();
    if tools.ffprobe.is_some() {
        variants.push(("available ffprobe", tools.clone()));
    }
    let mut unavailable = tools.clone();
    unavailable.ffprobe = None;
    variants.push(("unavailable ffprobe", unavailable));
    let mut failing = tools.clone();
    // FFmpeg is an existing native executable, but rejects ffprobe-only options.
    // This exercises the unknown-result fallback rather than confirming absence.
    failing.ffprobe = tools.ffmpeg.clone();
    variants.push(("failed ffprobe", failing));
    variants
}

#[test]
fn video_without_audio_has_the_same_italian_error_with_available_missing_and_failed_probes() {
    let Some(tools) = tools() else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("video muto è.mp4");
    fixture(&input, &tools, false);
    let original = fs::read(&input).unwrap();
    for (label, tools) in probe_variants(&tools) {
        for format in [OutputFormat::Wav, OutputFormat::Mp3] {
            let settings = ConversionSettings {
                format,
                ..Default::default()
            };
            let error = engine::convert(&input, &settings, &tools, &AtomicBool::new(false), |_| {})
                .unwrap_err();
            assert_eq!(error.to_string(), NO_AUDIO_ERROR, "{label}: {error:#}");
            assert_eq!(
                fs::read_dir(directory.path()).unwrap().count(),
                1,
                "{label}"
            );
            assert_eq!(fs::read(&input).unwrap(), original);
        }
    }
}

#[test]
fn video_with_audio_still_extracts_when_the_stream_probe_succeeds_is_missing_or_fails() {
    let Some(tools) = tools() else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("video con audio.mp4");
    fixture(&input, &tools, true);
    let original = fs::read(&input).unwrap();
    let variants = probe_variants(&tools);
    let count = variants.len();
    for (label, tools) in variants {
        let settings = ConversionSettings {
            format: OutputFormat::Wav,
            ..Default::default()
        };
        let output = engine::convert(&input, &settings, &tools, &AtomicBool::new(false), |_| {})
            .unwrap_or_else(|error| panic!("{label}: {error:#}"));
        assert_eq!(output.len(), 1);
        let wave = fs::read(&output[0]).unwrap();
        assert!(wave.len() > 44);
        assert_eq!(&wave[..4], b"RIFF");
        assert_eq!(&wave[8..12], b"WAVE");
        assert_eq!(fs::read(&input).unwrap(), original);
    }
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1 + count);
}

#[test]
fn cancellation_before_audio_preflight_never_falls_through_to_encoding() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("cancelled.mp4");
    fs::write(&input, b"untouched video fixture").unwrap();
    let settings = ConversionSettings {
        format: OutputFormat::Wav,
        ..Default::default()
    };
    let tools = ToolPaths {
        ffmpeg: Some(std::env::current_exe().unwrap()),
        ..Default::default()
    };
    let cancel = AtomicBool::new(false);
    let result = engine::convert(&input, &settings, &tools, &cancel, |_| {
        cancel.store(true, Ordering::Relaxed);
    });
    assert!(result.unwrap_err().to_string().contains("annullata"));
    assert_eq!(fs::read(input).unwrap(), b"untouched video fixture");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}
