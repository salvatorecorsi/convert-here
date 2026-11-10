use std::{fs, path::Path, process::Command, sync::atomic::AtomicBool};
use the_converter::{
    media_preview,
    model::{ConversionSettings, OutputFormat},
    tools::ToolPaths,
};

fn command(program: &Path) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}
fn tools() -> Option<ToolPaths> {
    let tools = ToolPaths::discover();
    if tools.ffmpeg.is_none() || tools.ffprobe.is_none() {
        eprintln!("SKIP media editor fixture: FFmpeg/ffprobe unavailable");
        None
    } else {
        Some(tools)
    }
}
fn fixture(path: &Path, video: bool, tools: &ToolPaths) {
    let mut command = command(tools.ffmpeg.as_ref().unwrap());
    command.args(["-hide_banner", "-nostdin", "-v", "error"]);
    if video {
        command.args([
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=96x64:rate=10:duration=2",
        ]);
    }
    command.args([
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=1000:sample_rate=16000:duration=2",
    ]);
    if video {
        command.args(["-c:v", "libx264", "-c:a", "aac", "-shortest"]);
    } else {
        command.args(["-c:a", "pcm_s16le"]);
    }
    let output = command.arg(path).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn probe(path: &Path, tools: &ToolPaths) -> serde_json::Value {
    let output = command(tools.ffprobe.as_ref().unwrap())
        .args([
            "-v",
            "error",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    serde_json::from_slice(&output.stdout).unwrap()
}
fn rms(path: &Path, tools: &ToolPaths) -> f64 {
    let output = command(tools.ffmpeg.as_ref().unwrap())
        .args([
            "-hide_banner",
            "-nostdin",
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-i",
        ])
        .arg(path)
        .args([
            "-map",
            "0:a:0",
            "-t",
            "0.1",
            "-ac",
            "1",
            "-c:a",
            "pcm_f32le",
            "-f",
            "f32le",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let samples: Vec<_> = output
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes) as f64)
        .collect();
    assert!(!samples.is_empty());
    (samples.iter().map(|sample| sample * sample).sum::<f64>() / samples.len() as f64).sqrt()
}

#[test]
fn prepares_real_audio_waveform_and_video_strip_without_changing_sources() {
    let Some(tools) = tools() else { return };
    let directory = tempfile::tempdir().unwrap();
    for video in [false, true] {
        let input = directory
            .path()
            .join(if video { "source.mp4" } else { "source.wav" });
        fixture(&input, video, &tools);
        let original = fs::read(&input).unwrap();
        let preview =
            media_preview::prepare(&input, &tools, &AtomicBool::new(false), |_| {}).unwrap();
        assert!((preview.duration - 2.0).abs() < 0.1);
        assert!(preview.has_audio);
        assert_eq!(preview.has_video, video);
        let waveform = preview
            .waveform
            .as_ref()
            .unwrap_or_else(|| panic!("Waveform missing: {:?}", preview.warnings));
        assert_eq!(waveform.dimensions(), (1200, 160));
        assert!(waveform.pixels().any(|pixel| pixel[3] > 0));
        if video {
            assert_eq!(preview.frames.len(), 4);
            assert!(
                preview
                    .frames
                    .windows(2)
                    .all(|frames| frames[0].time < frames[1].time)
            );
            assert!(
                preview
                    .frames
                    .iter()
                    .all(|frame| frame.pixels.dimensions() == (256, 144))
            );
        } else {
            assert!(preview.frames.is_empty());
        }
        assert_eq!(fs::read(&input).unwrap(), original);
    }
}

#[test]
fn saves_actual_trim_gain_and_metadata_and_preserves_original() {
    let Some(tools) = tools() else { return };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("original.wav");
    fixture(&input, false, &tools);
    let original = fs::read(&input).unwrap();
    let stamp = media_preview::SourceStamp::read(&input).unwrap();
    let mut base = ConversionSettings::default();
    base.select_output_format(OutputFormat::Wav);
    base.media_options
        .insert("title".into(), "Media editor title".into());
    let settings =
        media_preview::settings_for_interval(&base, 0.5, 1.25, -6.020599913279624, false).unwrap();
    let output = directory.path().join("edited.wav");
    media_preview::save(
        &input,
        &output,
        &settings,
        &tools,
        &stamp,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let metadata = probe(&output, &tools);
    let duration = metadata["format"]["duration"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!((duration - 0.75).abs() < 0.005);
    assert_eq!(metadata["format"]["tags"]["title"], "Media editor title");
    assert!((rms(&output, &tools) / rms(&input, &tools) - 0.5).abs() < 0.005);
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}

#[test]
fn editor_gain_keeps_normalization_before_volume_and_user_metadata() {
    let Some(tools) = tools() else { return };
    let mut base = ConversionSettings::default();
    base.select_output_format(OutputFormat::Wav);
    base.audio.normalize = true;
    base.media_options
        .insert("title".into(), "Preserved".into());
    let settings =
        media_preview::settings_for_interval(&base, 0.1, 0.2, 6.020599913279624, false).unwrap();
    let mut command = Command::new("ffmpeg");
    the_converter::media_options::append_args(
        &mut command,
        &settings,
        &tools,
        &AtomicBool::new(false),
    )
    .unwrap();
    let args: Vec<_> = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let filters = &args[args.iter().position(|arg| arg == "-af").unwrap() + 1];
    assert!(
        filters.starts_with("loudnorm=I=-16:TP=-1.5:LRA=11,volume=2"),
        "{filters}"
    );
    assert!(args.iter().any(|arg| arg == "title=Preserved"));
}

#[test]
fn cancellation_changes_and_existing_destinations_never_publish_or_replace() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.wav");
    fs::write(&input, b"source content").unwrap();
    let stamp = media_preview::SourceStamp::read(&input).unwrap();
    let mut settings = ConversionSettings::default();
    settings.select_output_format(OutputFormat::Wav);
    let existing = directory.path().join("existing.wav");
    fs::write(&existing, b"keep").unwrap();
    assert!(
        media_preview::save(
            &input,
            &existing,
            &settings,
            &ToolPaths::default(),
            &stamp,
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert!(
        media_preview::save(
            &input,
            &input,
            &settings,
            &ToolPaths::default(),
            &stamp,
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    let output = directory.path().join("new.wav");
    assert!(
        media_preview::save(
            &input,
            &output,
            &settings,
            &ToolPaths::default(),
            &stamp,
            &AtomicBool::new(true),
            |_| {}
        )
        .is_err()
    );
    assert!(
        media_preview::prepare(
            &input,
            &ToolPaths::default(),
            &AtomicBool::new(true),
            |_| {}
        )
        .is_err()
    );
    fs::write(&input, b"changed source length").unwrap();
    assert!(
        media_preview::save(
            &input,
            &output,
            &settings,
            &ToolPaths::default(),
            &stamp,
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert!(!output.exists());
    assert_eq!(fs::read(&existing).unwrap(), b"keep");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}
