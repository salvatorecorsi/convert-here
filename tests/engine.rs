use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine::convert,
    model::{ConversionSettings, MediaKind, OutputFormat},
    tools::ToolPaths,
};

fn settings(format: OutputFormat) -> ConversionSettings {
    ConversionSettings {
        format,
        ..Default::default()
    }
}
fn fixture(path: &Path) {
    let image = image::RgbaImage::from_pixel(64, 32, image::Rgba([255, 0, 0, 0]));
    image.save(path).unwrap();
}

#[test]
fn disguised_playlists_cannot_open_network_connections() {
    let tools = ToolPaths::discover();
    if tools.ffmpeg.is_none() {
        return;
    }
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("playlist.mp4");
    fs::write(&input, format!("#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXTINF:1.0,\nhttp://127.0.0.1:{port}/segment.ts\n#EXT-X-ENDLIST\n")).unwrap();
    assert!(
        convert(
            &input,
            &settings(OutputFormat::Mp3),
            &tools,
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn image_conversion_resizes_flattens_alpha_and_never_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("foto è speciale.png");
    fixture(&input);
    let expected = fs::read(&input).unwrap();
    let mut s = settings(OutputFormat::Jpg);
    s.image.width = Some(32);
    let first = convert(
        &input,
        &s,
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let second = convert(
        &input,
        &s,
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_ne!(first, second);
    assert_eq!(fs::read(&input).unwrap(), expected);
    let image = image::open(&first[0]).unwrap().to_rgb8();
    assert_eq!(image.dimensions(), (32, 16));
    assert!(image.get_pixel(0, 0)[0] > 245 && image.get_pixel(0, 0)[1] > 245);
    let same = convert(
        &input,
        &settings(OutputFormat::Png),
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_ne!(same[0], input);
    assert_eq!(fs::read(&input).unwrap(), expected);
}

#[test]
fn native_pdf_is_valid_and_has_a_page() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("foto.png");
    fixture(&input);
    let output = convert(
        &input,
        &settings(OutputFormat::Pdf),
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let doc = lopdf::Document::load(&output[0]).unwrap();
    assert_eq!(doc.get_pages().len(), 1);
    assert!(doc.objects.values().any(|o| o.as_stream().is_ok()));
}

#[test]
fn native_image_pdf_renders_with_source_colours() {
    let tools = ToolPaths::discover();
    if tools.pdftoppm.is_none() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("colore.png");
    image::RgbImage::from_pixel(64, 32, image::Rgb([40, 100, 200]))
        .save(&input)
        .unwrap();
    let pdf = convert(
        &input,
        &settings(OutputFormat::Pdf),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let pages = convert(
        &pdf[0],
        &settings(OutputFormat::Png),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let raster = image::open(&pages[0]).unwrap().to_rgb8();
    let colour = raster.get_pixel(raster.width() / 2, raster.height() / 2);
    for (got, expected) in colour.0.into_iter().zip([40i32, 100, 200]) {
        assert!((got as i32 - expected).abs() < 8);
    }
}

#[test]
fn all_advertised_native_image_targets_encode() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.png");
    fixture(&input);
    for format in [
        OutputFormat::Png,
        OutputFormat::Jpg,
        OutputFormat::Webp,
        OutputFormat::Bmp,
        OutputFormat::Tiff,
        OutputFormat::Gif,
        OutputFormat::Ico,
    ] {
        let result = convert(
            &input,
            &settings(format),
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(image::open(&result[0]).is_ok(), "{}", format.label());
    }
}

#[test]
fn camera_exif_orientation_is_applied_before_resize_and_pdf_layout() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("foto-ruotata.jpg");
    let source = image::RgbImage::from_fn(40, 20, |x, _| {
        if x < 20 {
            image::Rgb([240, 10, 10])
        } else {
            image::Rgb([10, 10, 240])
        }
    });
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 95)
        .encode_image(&source)
        .unwrap();
    // Minimal little-endian TIFF IFD inside a JPEG APP1 EXIF segment: Orientation SHORT = 6.
    let exif: &[u8] = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
    let mut oriented = jpeg[..2].to_vec();
    oriented.extend_from_slice(&[0xff, 0xe1]);
    oriented.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
    oriented.extend_from_slice(exif);
    oriented.extend_from_slice(&jpeg[2..]);
    fs::write(&input, &oriented).unwrap();
    let cancel = AtomicBool::new(false);
    let mut s = settings(OutputFormat::Png);
    s.image.width = Some(10);
    let outputs = convert(&input, &s, &ToolPaths::default(), &cancel, |_| {}).unwrap();
    let image = image::open(&outputs[0]).unwrap().to_rgb8();
    assert_eq!(image.dimensions(), (10, 20));
    assert!(image.get_pixel(5, 3)[0] > 200 && image.get_pixel(5, 3)[2] < 40);
    assert!(image.get_pixel(5, 16)[2] > 200 && image.get_pixel(5, 16)[0] < 40);
    let outputs = convert(
        &input,
        &settings(OutputFormat::Pdf),
        &ToolPaths::default(),
        &cancel,
        |_| {},
    )
    .unwrap();
    let doc = lopdf::Document::load(&outputs[0]).unwrap();
    let page = doc
        .get_dictionary(*doc.get_pages().values().next().unwrap())
        .unwrap();
    let bounds = page.get(b"MediaBox").unwrap().as_array().unwrap();
    assert!(bounds[2].as_float().unwrap() < bounds[3].as_float().unwrap());
    assert_eq!(fs::read(&input).unwrap(), oriented);
}

#[test]
fn cancellation_and_validation_leave_no_partial_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("foto.png");
    fixture(&input);
    assert!(
        convert(
            &input,
            &settings(OutputFormat::Jpg),
            &ToolPaths::default(),
            &AtomicBool::new(true),
            |_| {}
        )
        .is_err()
    );
    let mut invalid = settings(OutputFormat::Jpg);
    invalid.image.width = Some(0);
    assert!(
        convert(
            &input,
            &invalid,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    assert_eq!(
        MediaKind::from_path(Path::new("FILE.MKV")),
        MediaKind::Video
    );
}

#[test]
fn ffmpeg_audio_roundtrip_and_failed_conversion_cleanup() {
    let tools = ToolPaths::discover();
    if tools.ffmpeg.is_none() {
        eprintln!("FFmpeg not available; integration fixture skipped");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("sorgente è (1).wav");
    // One second of a 440Hz mono PCM tone, with a conventional WAV header.
    let rate = 44100u32;
    let bytes = rate * 2;
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(bytes + 36).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&rate.to_le_bytes());
    wav.extend_from_slice(&(rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&bytes.to_le_bytes());
    for n in 0..rate {
        let sample = ((n as f64 * 440. * std::f64::consts::TAU / rate as f64).sin() * 8000.) as i16;
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(&input, &wav).unwrap();
    for format in [
        OutputFormat::Mp3,
        OutputFormat::Flac,
        OutputFormat::Aac,
        OutputFormat::Ogg,
        OutputFormat::Opus,
        OutputFormat::Aiff,
    ] {
        let output = convert(
            &input,
            &settings(format),
            &tools,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(fs::metadata(&output[0]).unwrap().len() > 100);
    }
    assert_eq!(fs::read(&input).unwrap(), wav);
    let broken = dir.path().join("broken.mp3");
    fs::write(&broken, b"not an audio stream").unwrap();
    assert!(
        convert(
            &broken,
            &settings(OutputFormat::Wav),
            &tools,
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert!(!dir.path().join("broken.wav").exists());
    assert!(!fs::read_dir(dir.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".the-converter-")
    }));
}

#[test]
fn pdf_page_export_rolls_back_after_cancellation() {
    use lopdf::{Object, Stream, dictionary};
    let tools = ToolPaths::discover();
    if tools.pdftoppm.is_none() {
        eprintln!("Poppler not available; integration fixture skipped");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("two.pdf");
    let mut doc = lopdf::Document::with_version("1.5");
    let pages = doc.new_object_id();
    let mut kids = Vec::new();
    for _ in 0..2 {
        let content = doc.add_object(Stream::new(
            dictionary! {},
            b"0 0 1 rg 0 0 100 100 re f".to_vec(),
        ));
        let page=doc.add_object(dictionary!{"Type"=>"Page","Parent"=>pages,"MediaBox"=>vec![0.into(),0.into(),100.into(),100.into()],"Resources"=>dictionary!{},"Contents"=>content});
        kids.push(Object::Reference(page));
    }
    doc.objects.insert(
        pages,
        Object::Dictionary(dictionary! {"Type"=>"Pages","Kids"=>kids,"Count"=>2}),
    );
    let root = doc.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages});
    doc.trailer.set("Root", root);
    doc.save(&input).unwrap();
    let cancel = AtomicBool::new(false);
    let result = convert(&input, &settings(OutputFormat::Png), &tools, &cancel, |p| {
        if p.message.starts_with("Pagina 1 ") {
            cancel.store(true, Ordering::Relaxed);
        }
    });
    assert!(result.is_err());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    cancel.store(false, Ordering::Relaxed);
    let outputs = convert(
        &input,
        &settings(OutputFormat::Png),
        &tools,
        &cancel,
        |_| {},
    )
    .unwrap();
    assert_eq!(outputs.len(), 2);
    for output in outputs {
        assert!(image::open(output).is_ok());
    }
}

#[test]
fn ffmpeg_video_targets_h265_and_audio_extraction() {
    use std::process::Command;
    use the_converter::model::VideoCodec;
    let tools = ToolPaths::discover();
    let (Some(ffmpeg), Some(ffprobe)) = (&tools.ffmpeg, &tools.ffprobe) else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("video è speciale.mkv");
    let mut fixture = Command::new(ffmpeg);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        fixture.creation_flags(0x08000000);
    }
    let generated = fixture
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
            "sine=frequency=440:sample_rate=44100",
            "-t",
            "1",
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
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    for format in [
        OutputFormat::Mp4,
        OutputFormat::Mkv,
        OutputFormat::Mov,
        OutputFormat::Webm,
        OutputFormat::Avi,
        OutputFormat::Gif,
        OutputFormat::Mp3,
    ] {
        let mut s = settings(format);
        s.video.width = Some(128);
        s.video.trim_start = Some(0.1);
        s.video.trim_end = Some(0.8);
        let output = convert(&input, &s, &tools, &AtomicBool::new(false), |_| {}).unwrap();
        let mut probe = Command::new(ffprobe);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            probe.creation_flags(0x08000000);
        }
        let info = probe
            .args([
                "-v",
                "error",
                "-show_entries",
                "stream=codec_name,codec_type,width,height:format=duration",
                "-of",
                "json",
            ])
            .arg(&output[0])
            .output()
            .unwrap();
        assert!(info.status.success(), "{}", format.label());
        let json: serde_json::Value = serde_json::from_slice(&info.stdout).unwrap();
        assert!(!json["streams"].as_array().unwrap().is_empty());
        if format == OutputFormat::Webm {
            let streams = json["streams"].as_array().unwrap();
            let video = streams
                .iter()
                .find(|stream| stream["codec_type"] == "video")
                .unwrap();
            let audio = streams
                .iter()
                .find(|stream| stream["codec_type"] == "audio")
                .unwrap();
            assert_eq!(video["codec_name"], "vp9");
            assert_eq!(audio["codec_name"], "opus");
            assert_eq!(video["width"], 128);
            assert_eq!(video["height"], 96);
            let mut decode = Command::new(ffmpeg);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                decode.creation_flags(0x08000000);
            }
            let decoded = decode
                .args(["-v", "error", "-nostdin", "-i"])
                .arg(&output[0])
                .args(["-f", "null", "-"])
                .output()
                .unwrap();
            assert!(
                decoded.status.success(),
                "{}",
                String::from_utf8_lossy(&decoded.stderr)
            );
            assert!(
                decoded.stderr.is_empty(),
                "{}",
                String::from_utf8_lossy(&decoded.stderr)
            );
        }
        let duration = json["format"]["duration"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!(
            (0.4..1.5).contains(&duration),
            "{} duration {}",
            format.label(),
            duration
        );
    }
    let mut s = settings(OutputFormat::Mp4);
    s.video.codec = VideoCodec::H265;
    let output = convert(&input, &s, &tools, &AtomicBool::new(false), |_| {}).unwrap();
    let mut probe = Command::new(ffprobe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        probe.creation_flags(0x08000000);
    }
    let info = probe
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(&output[0])
        .output()
        .unwrap();
    assert!(info.status.success());
    assert_eq!(String::from_utf8_lossy(&info.stdout).trim(), "hevc");
}

#[test]
fn webm_presets_preserve_streams_frames_and_control_quality_size_tradeoff() {
    use std::process::Command;

    fn command(executable: &Path) -> Command {
        let mut command = Command::new(executable);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        command
    }

    let tools = ToolPaths::discover();
    let (Some(ffmpeg), Some(ffprobe)) = (&tools.ffmpeg, &tools.ffprobe) else {
        eprintln!("FFmpeg/FFprobe unavailable; WebM preset fixture skipped");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("webm-presets.mkv");
    let generated = command(ffmpeg)
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=320x180:rate=24",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000",
            "-t",
            "1.5",
            "-c:v",
            "ffv1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "pcm_s16le",
            "-ac",
            "2",
            "-shortest",
        ])
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let original = fs::read(&input).unwrap();
    let mut sizes = Vec::new();
    let mut distortion = Vec::new();
    // These are the app's compact, balanced and high-quality preset values.
    for (quality, bitrate) in [(28, 128), (23, 192), (18, 320)] {
        let mut conversion = settings(OutputFormat::Webm);
        conversion.video.crf = quality;
        conversion.audio.bitrate_kbps = bitrate;
        let output = convert(&input, &conversion, &tools, &AtomicBool::new(false), |_| {}).unwrap();
        let probe = command(ffprobe)
            .args([
                "-v",
                "error",
                "-count_frames",
                "-show_entries",
                "stream=codec_name,codec_type,width,height,nb_read_frames:format=duration",
                "-of",
                "json",
            ])
            .arg(&output[0])
            .output()
            .unwrap();
        assert!(
            probe.status.success(),
            "{}",
            String::from_utf8_lossy(&probe.stderr)
        );
        let info: serde_json::Value = serde_json::from_slice(&probe.stdout).unwrap();
        let streams = info["streams"].as_array().unwrap();
        let video = streams
            .iter()
            .find(|stream| stream["codec_type"] == "video")
            .unwrap();
        let audio = streams
            .iter()
            .find(|stream| stream["codec_type"] == "audio")
            .unwrap();
        assert_eq!(video["codec_name"], "vp9");
        assert_eq!(audio["codec_name"], "opus");
        assert_eq!(video["width"], 320);
        assert_eq!(video["height"], 180);
        assert_eq!(video["nb_read_frames"], "36");
        let duration = info["format"]["duration"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!((duration - 1.5).abs() < 0.08, "duration {duration}");
        let decoded = command(ffmpeg)
            .args(["-v", "error", "-nostdin", "-i"])
            .arg(&output[0])
            .args(["-f", "null", "-"])
            .output()
            .unwrap();
        assert!(
            decoded.status.success(),
            "{}",
            String::from_utf8_lossy(&decoded.stderr)
        );
        assert!(
            decoded.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&decoded.stderr)
        );
        sizes.push(fs::metadata(&output[0]).unwrap().len());
        // Compare decoded pixels with the lossless source rather than merely
        // asserting constructed arguments. No timing assertion depends on CI CPU.
        let psnr = command(ffmpeg)
            .args(["-hide_banner", "-nostdin", "-loglevel", "info", "-i"])
            .arg(&input)
            .arg("-i")
            .arg(&output[0])
            .args([
                "-filter_complex",
                "[0:v]setpts=PTS-STARTPTS[ref];[1:v]setpts=PTS-STARTPTS[dist];[ref][dist]psnr",
                "-an",
                "-f",
                "null",
                "-",
            ])
            .output()
            .unwrap();
        assert!(
            psnr.status.success(),
            "{}",
            String::from_utf8_lossy(&psnr.stderr)
        );
        let diagnostic = String::from_utf8_lossy(&psnr.stderr);
        let average = diagnostic
            .lines()
            .find_map(|line| {
                line.split_once("average:")
                    .and_then(|(_, tail)| tail.split_whitespace().next())
                    .and_then(|value| value.parse::<f64>().ok())
            })
            .unwrap_or_else(|| panic!("PSNR result missing: {diagnostic}"));
        distortion.push(average);
    }
    assert!(
        sizes[0] < sizes[1] && sizes[1] < sizes[2],
        "controlled preset sizes: {sizes:?}"
    );
    assert!(
        distortion[0] < distortion[1] && distortion[1] < distortion[2],
        "controlled preset PSNR: {distortion:?}"
    );
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn pdf_text_extraction_and_scan_feedback() {
    use lopdf::{
        Object, Stream,
        content::{Content, Operation},
        dictionary,
    };
    let tools = ToolPaths::discover();
    if tools.pdftotext.is_none() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("testo.pdf");
    let mut doc = lopdf::Document::with_version("1.5");
    let pages = doc.new_object_id();
    let font =
        doc.add_object(dictionary! {"Type"=>"Font","Subtype"=>"Type1","BaseFont"=>"Courier"});
    let resources = doc.add_object(dictionary! {"Font"=>dictionary!{"F1"=>font}});
    let content = Content {
        operations: vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 12.into()]),
            Operation::new("Td", vec![20.into(), 50.into()]),
            Operation::new("Tj", vec![Object::string_literal("Ciao PDF")]),
            Operation::new("ET", vec![]),
        ],
    };
    let content = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page=doc.add_object(dictionary!{"Type"=>"Page","Parent"=>pages,"MediaBox"=>vec![0.into(),0.into(),200.into(),100.into()],"Resources"=>resources,"Contents"=>content});
    doc.objects.insert(
        pages,
        Object::Dictionary(
            dictionary! {"Type"=>"Pages","Kids"=>vec![Object::Reference(page)],"Count"=>1},
        ),
    );
    let catalog = doc.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages});
    doc.trailer.set("Root", catalog);
    doc.save(&input).unwrap();
    let outputs = convert(
        &input,
        &settings(OutputFormat::Txt),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert!(
        fs::read_to_string(&outputs[0])
            .unwrap()
            .contains("Ciao PDF")
    );
    let image = dir.path().join("scansione.png");
    fixture(&image);
    let scanned = convert(
        &image,
        &settings(OutputFormat::Pdf),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let result = convert(
        &scanned[0],
        &settings(OutputFormat::Txt),
        &tools,
        &AtomicBool::new(false),
        |_| {},
    );
    assert!(result.unwrap_err().to_string().contains("OCR"));
    assert!(!dir.path().join("scansione.txt").exists());
}
