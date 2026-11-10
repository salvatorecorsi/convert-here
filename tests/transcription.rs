use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine,
    model::{ConversionSettings, MediaKind, OutputFormat, supported_formats},
    tools::ToolPaths,
};

#[test]
fn speech_is_an_output_route_and_old_settings_keep_automatic_language() {
    for input in ["voice.flac", "voice.mp3", "movie.mp4"] {
        assert!(
            supported_formats(MediaKind::from_path(std::path::Path::new(input)))
                .contains(&OutputFormat::Txt)
        );
    }
    let settings: ConversionSettings = serde_json::from_str(r#"{"format":"Txt"}"#).unwrap();
    assert_eq!(settings.speech.language, "auto");
    assert_eq!(settings.speech.threads, 0);
}

// Optional real-model fixtures: set these paths explicitly; tests never download a model.
fn local_fixture() -> Option<(PathBuf, ToolPaths)> {
    let fixture = std::env::var_os("CONVERT_HERE_SPEECH_FIXTURE").map(PathBuf::from)?;
    let mut tools = ToolPaths::discover();
    tools.whisper_cli = std::env::var_os("CONVERT_HERE_WHISPER_CLI")
        .map(PathBuf::from)
        .or(tools.whisper_cli);
    tools.whisper_model = std::env::var_os("CONVERT_HERE_WHISPER_MODEL")
        .map(PathBuf::from)
        .or(tools.whisper_model);
    assert!(fixture.is_file());
    assert!(
        tools
            .missing_for(MediaKind::Audio, OutputFormat::Txt)
            .is_none(),
        "Set all local FFmpeg/Whisper paths for the optional speech fixture"
    );
    Some((fixture, tools))
}

#[test]
fn real_local_model_produces_utf8_and_keeps_existing_output_and_source() {
    let Some((fixture, tools)) = local_fixture() else {
        eprintln!("SKIP real Whisper fixture: CONVERT_HERE_SPEECH_FIXTURE is unset");
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("audio è.wav");
    fs::copy(fixture, &input).unwrap();
    let original = fs::read(&input).unwrap();
    let existing = directory.path().join("audio è.txt");
    fs::write(&existing, b"keep existing").unwrap();
    let mut settings = ConversionSettings {
        format: OutputFormat::Txt,
        ..Default::default()
    };
    settings.speech.language = "en".into();
    let outputs =
        engine::convert(&input, &settings, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    assert_ne!(outputs[0], existing);
    let text = fs::read_to_string(&outputs[0]).unwrap();
    assert!(text.to_lowercase().contains("computer"), "{text}");
    assert_eq!(fs::read(existing).unwrap(), b"keep existing");
    assert_eq!(fs::read(input).unwrap(), original);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 3);
}

#[test]
fn cancelling_after_audio_preparation_does_not_publish_text_or_leave_temporaries() {
    let Some((fixture, tools)) = local_fixture() else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("cancel.wav");
    fs::copy(fixture, &input).unwrap();
    let original = fs::read(&input).unwrap();
    let settings = ConversionSettings {
        format: OutputFormat::Txt,
        ..Default::default()
    };
    let plan = engine::prepare(&input, &settings, &tools).unwrap();
    let cancel = AtomicBool::new(false);
    let result = engine::convert_prepared(&plan, &tools, &cancel, |progress| {
        if progress.fraction >= 0.1 {
            cancel.store(true, Ordering::Relaxed);
        }
    });
    assert!(result.is_err());
    engine::release_prepared(&plan).unwrap();
    assert_eq!(fs::read(&input).unwrap(), original);
    assert!(!directory.path().join("cancel.txt").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}
