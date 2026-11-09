use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
    process::Command,
    sync::atomic::AtomicBool,
};
use the_converter::{
    engine,
    media_options::{self, OptionKind},
    model::{ConversionSettings, OutputFormat, VideoCodec},
    tools::ToolPaths,
};

fn map(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(key, value)| ((*key).into(), (*value).into()))
        .collect()
}

fn tools() -> Option<ToolPaths> {
    let tools = ToolPaths::discover();
    if tools.ffmpeg.is_none() || tools.ffprobe.is_none() {
        eprintln!("SKIP advanced media fixture: FFmpeg/ffprobe unavailable");
        return None;
    }
    Some(tools)
}

fn command(path: &Path) -> Command {
    let mut command = Command::new(path);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}

fn fixture(input: &Path, video: bool, tools: &ToolPaths) {
    let mut command = command(tools.ffmpeg.as_ref().unwrap());
    command.args(["-hide_banner", "-nostdin", "-loglevel", "error"]);
    if video {
        command.args([
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=80x60:rate=20:duration=1",
        ]);
    }
    command.args([
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:sample_rate=16000:duration=1",
    ]);
    if video {
        command.args(["-c:v", "libx264", "-c:a", "aac", "-shortest"]);
    } else {
        command.args(["-c:a", "pcm_s16le"]);
    }
    let output = command.arg(input).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn probe(input: &Path, tools: &ToolPaths) -> serde_json::Value {
    let output = command(tools.ffprobe.as_ref().unwrap())
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn catalog_is_typed_unique_and_hides_incompatible_categories() {
    let keys: HashSet<_> = media_options::SPECS.iter().map(|spec| spec.key).collect();
    assert_eq!(keys.len(), media_options::SPECS.len());
    assert!(keys.len() >= 50);
    assert!(media_options::options_for(OutputFormat::Pdf).is_empty());
    assert!(
        media_options::options_for(OutputFormat::Mp3)
            .iter()
            .all(|spec| !spec.key.starts_with("video_") && !spec.key.starts_with("crop_"))
    );
    assert!(
        media_options::options_for(OutputFormat::Mp4)
            .iter()
            .any(|spec| spec.key == "audio_volume")
    );
    assert!(media_options::video_encoder_choices(OutputFormat::Webm).contains(&"libvpx-vp9"));
    assert!(!media_options::video_encoder_choices(OutputFormat::Webm).contains(&"libx264"));
    assert!(
        media_options::encoder_specific_choices("preset", "libx264")
            .unwrap()
            .contains(&"slow")
    );
    assert!(
        !media_options::encoder_specific_choices("preset", "libx264")
            .unwrap()
            .contains(&"p7")
    );
    assert!(media_options::encoder_specific_choices("preset", "h264_qsv").is_none());
}

#[test]
fn unknown_paths_raw_filters_options_and_nonfinite_numbers_are_rejected() {
    for (key, value) in [
        ("-i", "input.wav"),
        ("output", "another.mp4"),
        ("filter_complex", "movie=https://example.com/"),
        ("vf", "movie=secret"),
        ("passlogfile", "stats.txt"),
        ("audio_volume", "1,amovie=secret"),
        ("audio_volume", "NaN"),
        ("video_fps", "inf"),
        ("threads", "129"),
        ("codec:libx264:passlogfile", "x"),
        ("codec:libx264:x264opts", "bframes=2:stats=path"),
        ("codec:libx264:i", "file"),
        ("title", "one\ntwo"),
    ] {
        assert!(
            media_options::validate(&map(&[(key, value)])).is_err(),
            "{key}"
        );
    }
    assert!(media_options::validate(&map(&[("crop_x", "2")])).is_err());
    assert!(media_options::validate(&map(&[("crop_width", "20")])).is_err());
    assert!(media_options::validate(&map(&[("audio_fade_out_start", "1")])).is_err());
    // Metadata is one argument, not a raw command line, even when it resembles options.
    assert!(media_options::validate(&map(&[("title", "-y -i https://example.com/other")])).is_ok());
}

#[test]
fn real_encoder_discovery_retains_auto_enum_and_supported_formats_and_excludes_strings() {
    let Some(tools) = tools() else { return };
    let options =
        media_options::discover_encoder_options(&tools, "libx264", &AtomicBool::new(false))
            .unwrap();
    let option = |name: &str| options.iter().find(|option| option.name == name).unwrap();
    assert!(
        option("aq-mode")
            .choices
            .iter()
            .any(|choice| choice == "variance")
    );
    assert_eq!(option("weightb").default.as_deref(), Some("auto"));
    assert_eq!(option("weightb").choices, ["auto", "0", "1"]);
    assert_eq!(option("weightb").kind, OptionKind::Int { min: -1, max: 1 });
    assert!(
        option("pixel_format")
            .choices
            .iter()
            .any(|choice| choice == "yuv420p")
    );
    assert!(
        !options.iter().any(
            |option| ["passlogfile", "x264opts", "x264-params"].contains(&option.name.as_str())
        )
    );
    let audio =
        media_options::discover_encoder_options(&tools, "aac", &AtomicBool::new(false)).unwrap();
    assert_eq!(
        audio
            .iter()
            .find(|option| option.key == "audio_sample_format")
            .unwrap()
            .choices,
        ["fltp"]
    );
    assert!(
        media_options::discover_encoder_options(
            &tools,
            "not_an_encoder_123",
            &AtomicBool::new(false)
        )
        .is_err()
    );
}

#[test]
fn discovery_cancellation_also_applies_to_cached_queries() {
    let Some(tools) = tools() else { return };
    media_options::discover_encoder_options(&tools, "aac", &AtomicBool::new(false)).unwrap();
    assert!(
        media_options::discover_encoder_options(&tools, "aac", &AtomicBool::new(true)).is_err()
    );
    assert!(
        media_options::discover_encoder_options(
            &ToolPaths::default(),
            "aac",
            &AtomicBool::new(true)
        )
        .is_err()
    );
}

#[test]
fn imported_private_keys_and_incompatible_static_values_fail_before_mutating_command() {
    let Some(tools) = tools() else { return };
    for (format, values) in [
        (OutputFormat::Mp4, map(&[("preset", "p7")])),
        (OutputFormat::Aac, map(&[("audio_sample_format", "s16")])),
        (OutputFormat::Webm, map(&[("video_encoder", "libx264")])),
        (OutputFormat::Wav, map(&[("video_fps", "24")])),
        (OutputFormat::Mp4, map(&[("pixel_format", "rgb24")])),
        (
            OutputFormat::Mp4,
            map(&[("codec:libx264:unknown_numeric", "1")]),
        ),
        (OutputFormat::Mp4, map(&[("codec:libx264:x264opts", "1")])),
        (
            OutputFormat::Mp4,
            map(&[("codec:aac:aac_coder", "missing_enum")]),
        ),
        (OutputFormat::Mp4, map(&[("codec:libx265:crf", "20")])),
    ] {
        let settings = ConversionSettings {
            format,
            media_options: values,
            ..Default::default()
        };
        let mut command = Command::new(tools.ffmpeg.as_ref().unwrap());
        command.args(["-i", "input"]);
        let before: Vec<_> = command.get_args().map(|value| value.to_owned()).collect();
        assert!(
            media_options::append_args(&mut command, &settings, &tools, &AtomicBool::new(false))
                .is_err(),
            "{:?}",
            settings.media_options
        );
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            before
                .iter()
                .map(|value| value.as_os_str())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn controlled_filters_preserve_normalization_and_never_add_input_or_output_arguments() {
    let Some(tools) = tools() else { return };
    let mut settings = ConversionSettings {
        format: OutputFormat::Mp4,
        media_options: map(&[
            ("crop_width", "48"),
            ("crop_height", "32"),
            ("crop_x", "2"),
            ("rotation", "90"),
            ("scale_width", "64"),
            ("scale_height", "32"),
            ("audio_volume", "0.5"),
            ("title", "-i https://example.com/output -y"),
        ]),
        ..Default::default()
    };
    settings.audio.normalize = true;
    let mut command = Command::new("ffmpeg");
    media_options::append_args(&mut command, &settings, &tools, &AtomicBool::new(false)).unwrap();
    let args: Vec<_> = command
        .get_args()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    let after = |flag: &str| args[args.iter().position(|value| value == flag).unwrap() + 1].clone();
    assert!(after("-vf").starts_with("crop=48:32:2:0,transpose=clock,scale=64:32"));
    assert!(after("-af").starts_with("loudnorm=I=-16:TP=-1.5:LRA=11,volume=0.5"));
    assert_eq!(after("-metadata"), "title=-i https://example.com/output -y");
    assert!(!args.iter().any(|argument| {
        ["-i", "-y", "-filter_script", "-filter_complex_script"].contains(&argument.as_str())
    }));
}

#[test]
fn real_video_combines_static_and_discovered_options_filters_metadata_and_duration_cap() {
    let Some(tools) = tools() else { return };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.mp4");
    fixture(&input, true, &tools);
    let original = fs::read(&input).unwrap();
    let mut settings = ConversionSettings {
        format: OutputFormat::Mp4,
        output_dir: Some(directory.path().join("out")),
        media_options: map(&[
            ("video_encoder", "libx264"),
            ("preset", "ultrafast"),
            ("pixel_format", "yuv420p"),
            ("gop_size", "10"),
            ("b_frames", "0"),
            ("codec:libx264:aq-mode", "variance"),
            ("codec:libx264:weightb", "auto"),
            ("crop_width", "48"),
            ("crop_height", "32"),
            ("rotation", "90"),
            ("scale_width", "64"),
            ("scale_height", "32"),
            ("audio_channels", "1"),
            ("audio_sample_format", "fltp"),
            ("audio_volume", "0.25"),
            ("audio_fade_in", "0.1"),
            ("strip_metadata", "true"),
            ("title", "Titolo è -i https://example.com/ -y"),
        ]),
        ..Default::default()
    };
    // An encoder override must clear the base HEVC tag when switching back to H.264.
    settings.video.codec = VideoCodec::H265;
    settings.limits.max_duration = Some(0.3);
    let outputs =
        engine::convert(&input, &settings, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(outputs.len(), 1);
    let metadata = probe(&outputs[0], &tools);
    let streams = metadata["streams"].as_array().unwrap();
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .unwrap();
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .unwrap();
    assert_eq!(video["codec_name"], "h264");
    assert_eq!(
        (
            video["width"].as_u64().unwrap(),
            video["height"].as_u64().unwrap()
        ),
        (64, 32)
    );
    assert_eq!(audio["channels"], 1);
    assert!(
        metadata["format"]["duration"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap()
            <= 0.36
    );
    assert_eq!(
        metadata["format"]["tags"]["title"],
        "Titolo è -i https://example.com/ -y"
    );
    assert_eq!(fs::read(input).unwrap(), original);
    assert_eq!(
        fs::read_dir(settings.output_dir.unwrap()).unwrap().count(),
        1
    );
}

#[test]
fn real_audio_uses_discovered_flac_enum_and_typed_audio_controls() {
    let Some(tools) = tools() else { return };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.wav");
    fixture(&input, false, &tools);
    let settings = ConversionSettings {
        format: OutputFormat::Flac,
        output_dir: Some(directory.path().join("out")),
        media_options: map(&[
            ("audio_encoder", "flac"),
            ("audio_compression_level", "8"),
            ("audio_sample_format", "s16"),
            ("audio_sample_rate", "22050"),
            ("audio_volume", "0.5"),
            ("codec:flac:lpc_type", "fixed"),
            ("title", "Local audio"),
        ]),
        ..Default::default()
    };
    let outputs =
        engine::convert(&input, &settings, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    let metadata = probe(&outputs[0], &tools);
    assert_eq!(metadata["streams"][0]["codec_name"], "flac");
    assert_eq!(metadata["streams"][0]["sample_rate"], "22050");
    assert_eq!(metadata["format"]["tags"]["title"], "Local audio");
}

#[test]
fn real_gif_uses_only_discovered_flag_names() {
    let Some(tools) = tools() else { return };
    let options =
        media_options::discover_encoder_options(&tools, "gif", &AtomicBool::new(false)).unwrap();
    let flags = options
        .iter()
        .find(|option| option.name == "gifflags")
        .unwrap();
    assert!(flags.choices.iter().any(|choice| choice == "offsetting"));
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.mp4");
    fixture(&input, true, &tools);
    let mut settings = ConversionSettings {
        format: OutputFormat::Gif,
        output_dir: Some(directory.path().join("out")),
        media_options: map(&[("codec:gif:gifflags", "+offsetting+transdiff")]),
        ..Default::default()
    };
    settings.video.width = Some(64);
    settings.video.fps = Some(8);
    settings.limits.max_duration = Some(0.25);
    let outputs =
        engine::convert(&input, &settings, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    assert!(fs::read(&outputs[0]).unwrap().starts_with(b"GIF89a"));
    settings
        .media_options
        .insert("codec:gif:gifflags".into(), "unknown_flag".into());
    assert!(engine::convert(&input, &settings, &tools, &AtomicBool::new(false), |_| {}).is_err());
    assert_eq!(
        fs::read_dir(settings.output_dir.unwrap()).unwrap().count(),
        1
    );
}
