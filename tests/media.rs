use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};
use the_converter::{
    engine::convert,
    model::{ConversionSettings, OutputFormat},
    tools::ToolPaths,
};

fn settings(format: OutputFormat) -> ConversionSettings {
    ConversionSettings {
        format,
        ..Default::default()
    }
}

fn convert_one(input: &Path, format: OutputFormat, tools: &ToolPaths) -> PathBuf {
    let outputs = convert(
        input,
        &settings(format),
        tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap_or_else(|error| panic!("{} -> {}: {error:#}", input.display(), format.label()));
    assert_eq!(outputs.len(), 1);
    outputs.into_iter().next().unwrap()
}

fn command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}

#[test]
fn uncommon_native_images_roundtrip_pixels_dimensions_and_transparency() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("colori è (1).png");
    image::RgbaImage::from_pixel(17, 9, image::Rgba([40, 100, 200, 128]))
        .save(&input)
        .unwrap();
    let original = fs::read(&input).unwrap();
    let tools = ToolPaths::default();
    for format in [
        OutputFormat::Qoi,
        OutputFormat::Tga,
        OutputFormat::Farbfeld,
        OutputFormat::OpenExr,
        OutputFormat::Hdr,
        OutputFormat::Pnm,
    ] {
        let output = convert_one(&input, format, &tools);
        let decoded = image::open(&output).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (17, 9));
        let pixel = decoded.to_rgba8().get_pixel(0, 0).0;
        if format == OutputFormat::Hdr {
            assert_eq!(pixel[3], 255);
            // HDR has no alpha; the transparent pixel is composited over white.
            for (actual, expected) in pixel.into_iter().zip([147, 177, 227, 255]) {
                assert!(
                    actual.abs_diff(expected) <= 3,
                    "{}: {pixel:?}",
                    format.label()
                );
            }
        } else {
            for (actual, expected) in pixel.into_iter().zip([40, 100, 200, 128]) {
                assert!(
                    actual.abs_diff(expected) <= 1,
                    "{}: {pixel:?}",
                    format.label()
                );
            }
        }
        // The generated uncommon format must also be accepted as an input by
        // the application, including TGA's extension-based format detection.
        for target in [
            OutputFormat::Png,
            OutputFormat::Jpg,
            OutputFormat::Webp,
            OutputFormat::Bmp,
            OutputFormat::Tiff,
            OutputFormat::Gif,
            OutputFormat::Ico,
            OutputFormat::Qoi,
            OutputFormat::Tga,
            OutputFormat::Farbfeld,
            OutputFormat::OpenExr,
            OutputFormat::Hdr,
            OutputFormat::Pnm,
        ] {
            let mut back = settings(target);
            back.image.width = Some(34);
            let restored = convert(&output, &back, &tools, &AtomicBool::new(false), |_| {})
                .unwrap_or_else(|error| {
                    panic!("{} -> {}: {error:#}", format.label(), target.label())
                });
            assert_eq!(image::open(&restored[0]).unwrap().width(), 34);
        }
    }
    assert_eq!(fs::read(&input).unwrap(), original);
    assert!(!fs::read_dir(dir.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".the-converter-")
    }));
}

#[test]
fn hdr_conversion_retains_values_above_one_and_composites_alpha() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("luci.exr");
    image::DynamicImage::ImageRgba32F(image::Rgba32FImage::from_pixel(
        16,
        8,
        image::Rgba([2.0, 4.0, 8.0, 0.5]),
    ))
    .save(&input)
    .unwrap();
    let output = convert_one(&input, OutputFormat::Hdr, &ToolPaths::default());
    let decoded = image::open(output).unwrap().to_rgb32f();
    for (actual, expected) in decoded.get_pixel(0, 0).0.into_iter().zip([1.5, 2.5, 4.5]) {
        assert!((actual - expected).abs() < 0.05, "{actual} vs {expected}");
    }
}

#[test]
fn added_audio_containers_have_real_decodable_expected_codecs() {
    let tools = ToolPaths::discover();
    let (Some(ffmpeg), Some(ffprobe)) = (&tools.ffmpeg, &tools.ffprobe) else {
        eprintln!("FFmpeg/FFprobe not available; expanded audio fixture skipped");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("suono è speciale.wav");
    let fixture = command(ffmpeg)
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=44100",
            "-t",
            "0.4",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        fixture.status.success(),
        "{}",
        String::from_utf8_lossy(&fixture.stderr)
    );
    for (format, codec) in [
        (OutputFormat::M4a, "aac"),
        (OutputFormat::Wma, "wmav2"),
        (OutputFormat::Ac3, "ac3"),
        (OutputFormat::Au, "pcm_s16be"),
        (OutputFormat::Caf, "alac"),
    ] {
        let output = convert_one(&input, format, &tools);
        let streams = probe(ffprobe, &output);
        assert_eq!(streams[0]["codec_name"], codec, "{}", format.label());
        assert_eq!(streams[0]["codec_type"], "audio");
        let roundtrip = convert_one(&output, OutputFormat::Wav, &tools);
        assert_eq!(probe(ffprobe, &roundtrip)[0]["codec_name"], "pcm_s16le");
        if matches!(format, OutputFormat::Au | OutputFormat::Caf) {
            // Decode both containers to raw samples: lossless paths preserve
            // samples exactly, independently of WAV header and metadata changes.
            assert_eq!(pcm(ffmpeg, &input), pcm(ffmpeg, &roundtrip));
        }
    }
}

fn pcm(ffmpeg: &Path, input: &Path) -> Vec<u8> {
    let output = command(ffmpeg)
        .args(["-v", "error", "-nostdin", "-i"])
        .arg(input)
        .args(["-f", "s16le", "-c:a", "pcm_s16le", "pipe:1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn probe(ffprobe: &Path, input: &Path) -> Vec<serde_json::Value> {
    let output = command(ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_name,codec_type,width,height,sample_rate",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let info: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    info["streams"].as_array().unwrap().clone()
}

#[test]
fn unsupported_sample_rates_fail_before_output_directories_or_reservations() {
    let dir = tempfile::tempdir().unwrap();
    let audio = dir.path().join("input.wav");
    let video = dir.path().join("input.mkv");
    // A real decoder must never be reached: validation precedes tools and I/O.
    fs::write(&audio, b"validation fixture").unwrap();
    fs::write(&video, b"validation fixture").unwrap();
    let output_dir = dir.path().join("must-not-be-created");
    for (format, invalid_rate) in [
        (OutputFormat::Opus, 44100),
        (OutputFormat::Webm, 44100),
        (OutputFormat::Ac3, 22050),
        (OutputFormat::Wma, 96000),
        (OutputFormat::Wmv, 96000),
        (OutputFormat::Flv, 48000),
    ] {
        let offered = format.audio_sample_rates().unwrap();
        assert!(
            offered.contains(&0),
            "{} must offer automatic rate",
            format.label()
        );
        assert!(!offered.contains(&invalid_rate));
        let mut conversion = settings(format);
        conversion.audio.sample_rate = invalid_rate;
        conversion.output_dir = Some(output_dir.clone());
        let input = if format.kind() == the_converter::model::MediaKind::Audio {
            &audio
        } else {
            &video
        };
        let error = convert(
            input,
            &conversion,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| panic!("progress cannot begin for invalid settings"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("frequenze audio supportate"), "{error}");
        assert!(error.contains(format.label()), "{error}");
        let error = the_converter::engine::prepare(input, &conversion, &ToolPaths::default())
            .unwrap_err()
            .to_string();
        assert!(error.contains("frequenze audio supportate"), "{error}");
        assert!(!output_dir.exists());
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    assert!(OutputFormat::Wav.audio_sample_rates().is_none());
}

#[test]
fn offered_sample_rates_are_accepted_by_the_actual_encoders_and_containers() {
    let tools = ToolPaths::discover();
    let (Some(ffmpeg), Some(ffprobe)) = (&tools.ffmpeg, &tools.ffprobe) else {
        eprintln!("FFmpeg/FFprobe not available; offered sample-rate fixture skipped");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("sample-rates.mkv");
    let fixture = command(ffmpeg)
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=32x32:rate=10",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=44100",
            "-t",
            "0.3",
            "-c:v",
            "libx264",
            "-c:a",
            "pcm_s16le",
            "-shortest",
        ])
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        fixture.status.success(),
        "{}",
        String::from_utf8_lossy(&fixture.stderr)
    );
    for format in [
        OutputFormat::Opus,
        OutputFormat::Webm,
        OutputFormat::Ac3,
        OutputFormat::Wma,
        OutputFormat::Wmv,
        OutputFormat::Flv,
    ] {
        for &rate in format.audio_sample_rates().unwrap() {
            let mut conversion = settings(format);
            conversion.audio.sample_rate = rate;
            conversion.audio.bitrate_kbps = 64;
            let outputs = convert(&input, &conversion, &tools, &AtomicBool::new(false), |_| {})
                .unwrap_or_else(|error| panic!("{} {rate}Hz: {error:#}", format.label()));
            let streams = probe(ffprobe, &outputs[0]);
            let audio = streams
                .iter()
                .find(|stream| stream["codec_type"] == "audio")
                .unwrap();
            let actual_rate = audio["sample_rate"]
                .as_str()
                .unwrap()
                .parse::<u32>()
                .unwrap();
            // Opus's coded sample rate is always48kHz; the input rate controls
            // encoding bandwidth, rather than the reported container rate.
            if matches!(format, OutputFormat::Opus | OutputFormat::Webm) {
                assert_eq!(actual_rate, 48000);
            } else if rate != 0 {
                assert_eq!(actual_rate, rate, "{}", format.label());
            }
        }
    }
}

#[test]
fn added_video_containers_keep_picture_and_audio_including_m4v() {
    let tools = ToolPaths::discover();
    let (Some(ffmpeg), Some(ffprobe)) = (&tools.ffmpeg, &tools.ffprobe) else {
        eprintln!("FFmpeg/FFprobe not available; expanded video fixture skipped");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("video àudio.mkv");
    let fixture = command(ffmpeg)
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=128x96:rate=12",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000",
            "-t",
            "0.5",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-shortest",
        ])
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        fixture.status.success(),
        "{}",
        String::from_utf8_lossy(&fixture.stderr)
    );
    for (format, video_codec, audio_codec) in [
        (OutputFormat::Mpeg, "mpeg2video", "mp2"),
        (OutputFormat::Wmv, "wmv2", "wmav2"),
        (OutputFormat::Flv, "flv1", "mp3"),
        (OutputFormat::Ts, "h264", "aac"),
        (OutputFormat::M4v, "h264", "aac"),
        (OutputFormat::ThreeGp, "mpeg4", "aac"),
    ] {
        let output = convert_one(&input, format, &tools);
        let streams = probe(ffprobe, &output);
        let video = streams
            .iter()
            .find(|stream| stream["codec_type"] == "video")
            .unwrap();
        let audio = streams
            .iter()
            .find(|stream| stream["codec_type"] == "audio")
            .unwrap();
        assert_eq!(video["codec_name"], video_codec, "{}", format.label());
        assert_eq!(audio["codec_name"], audio_codec, "{}", format.label());
        assert_eq!(video["width"], 128);
        assert_eq!(video["height"], 96);
        let decoded = command(ffmpeg)
            .args(["-v", "error", "-nostdin", "-i"])
            .arg(&output)
            .args(["-f", "null", "-"])
            .output()
            .unwrap();
        assert!(
            decoded.status.success(),
            "{}: {}",
            format.label(),
            String::from_utf8_lossy(&decoded.stderr)
        );
        assert!(
            decoded.stderr.is_empty(),
            "{}: {}",
            format.label(),
            String::from_utf8_lossy(&decoded.stderr)
        );
    }
}
