use image::{Rgba, RgbaImage};
use lopdf::{Document, Object, Stream, dictionary};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};
use the_converter::{
    engine,
    image_edit::ImageEditSettings,
    model::{ConversionLimits, ConversionSettings, OutputFormat, PdfAction},
    pdf_merge,
    tools::ToolPaths,
};

fn settings(format: OutputFormat, output: &Path) -> ConversionSettings {
    ConversionSettings {
        format,
        output_dir: Some(output.to_owned()),
        ..Default::default()
    }
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

fn tools() -> Option<ToolPaths> {
    let tools = ToolPaths::discover();
    if tools.ffmpeg.is_none() || tools.ffprobe.is_none() {
        eprintln!("SKIP media-limit fixture: real FFmpeg and ffprobe are required");
        return None;
    }
    Some(tools)
}

fn audio_fixture(input: &Path, tools: &ToolPaths) {
    let output = command(tools.ffmpeg.as_ref().unwrap())
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=220:sample_rate=16000:duration=1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=880:sample_rate=16000:duration=2",
            "-filter_complex",
            "[0:a][1:a]concat=n=2:v=0:a=1",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn video_fixture(input: &Path, tools: &ToolPaths) {
    let output = command(tools.ffmpeg.as_ref().unwrap())
        .args([
            "-hide_banner", "-nostdin", "-loglevel", "error", "-f", "lavfi", "-i",
            "color=c=red:s=48x32:r=20:d=1[a];color=c=blue:s=48x32:r=20:d=2[b];[a][b]concat=n=2:v=1:a=0",
            "-f", "lavfi", "-i", "sine=frequency=880:sample_rate=16000:duration=3",
            "-c:v", "libx264", "-c:a", "aac", "-shortest",
        ])
        .arg(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn audio_samples(input: &Path, tools: &ToolPaths) -> Vec<i16> {
    let output = command(tools.ffmpeg.as_ref().unwrap())
        .args(["-hide_banner", "-nostdin", "-loglevel", "error", "-i"])
        .arg(input)
        .args(["-vn", "-f", "s16le", "-ar", "16000", "-ac", "1", "pipe:1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .as_chunks::<2>()
        .0
        .iter()
        .map(|sample| i16::from_le_bytes(*sample))
        .collect()
}

fn duration(input: &Path, tools: &ToolPaths) -> f64 {
    let output = command(tools.ffprobe.as_ref().unwrap())
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=nw=1:nk=1",
        ])
        .arg(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

fn convert(input: &Path, settings: &ConversionSettings, tools: &ToolPaths) -> Vec<PathBuf> {
    engine::convert(input, settings, tools, &AtomicBool::new(false), |_| {}).unwrap()
}

fn image_fixture(input: &Path) {
    let mut state = 1234567u32;
    RgbaImage::from_fn(128, 96, |_, _| {
        let mut channel = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        };
        Rgba([channel(), channel(), channel(), 255])
    })
    .save(input)
    .unwrap();
}

fn pdf_fixture(input: &Path) {
    let mut document = Document::with_version("1.5");
    let pages = document.new_object_id();
    let mut children = Vec::new();
    for index in 0..2 {
        let mut content = b"0 0 1 rg 0 0 20 20 re f\n".to_vec();
        if index == 1 {
            // Incompressible PDF comments make the second page genuinely larger.
            let mut state = 0x12345678u32;
            for _ in 0..4000 {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                content.extend_from_slice(format!("%{state:08x}\n").as_bytes());
            }
        }
        let content = document.add_object(Stream::new(dictionary! {}, content));
        children.push(Object::Reference(document.add_object(dictionary! {
            "Type"=>"Page", "Parent"=>pages, "Contents"=>content,
            "MediaBox"=>vec![0.into(), 0.into(), 100.into(), 100.into()],
        })));
    }
    document.objects.insert(
        pages,
        Object::Dictionary(dictionary! {
            "Type"=>"Pages", "Kids"=>children, "Count"=>2,
        }),
    );
    let root = document.add_object(dictionary! {"Type"=>"Catalog", "Pages"=>pages});
    document.trailer.set("Root", root);
    document.save(input).unwrap();
}

#[test]
fn legacy_json_remains_unlimited_and_limits_roundtrip_without_changing_legacy_recipes() {
    let legacy: ConversionSettings =
        serde_json::from_str(r#"{"format":"Jpg","image":{"edit":{"max_bytes":3000}}}"#).unwrap();
    assert_eq!(legacy.limits, ConversionLimits::default());
    assert!(legacy.media_options.is_empty());
    assert_eq!(legacy.image.edit.as_ref().unwrap().max_bytes, Some(3000));
    let mut configured = legacy;
    configured.limits = ConversionLimits {
        max_bytes: Some(2000),
        max_duration: Some(12.5),
    };
    let restored: ConversionSettings =
        serde_json::from_slice(&serde_json::to_vec(&configured).unwrap()).unwrap();
    assert_eq!(restored.limits, configured.limits);
    assert_eq!(restored.image.edit, configured.image.edit);
}

#[test]
fn limits_reject_zero_nonfinite_and_excessive_values_before_allocating_outputs() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.png");
    image_fixture(&input);
    let output = directory.path().join("output");
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY, 86400.01] {
        let mut settings = settings(OutputFormat::Png, &output);
        settings.limits.max_duration = Some(invalid);
        assert!(
            engine::convert(
                &input,
                &settings,
                &ToolPaths::default(),
                &AtomicBool::new(false),
                |_| {}
            )
            .is_err()
        );
        assert!(!output.exists());
    }
    let mut settings = settings(OutputFormat::Png, &output);
    settings.limits.max_bytes = Some(0);
    assert!(
        engine::convert(
            &input,
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert!(!output.exists());
    settings.limits = ConversionLimits {
        max_bytes: None,
        max_duration: Some(86400.0),
    };
    // A valid saved A/V duration does not block a later image target.
    let result = convert(&input, &settings, &ToolPaths::default());
    assert_eq!(image::open(&result[0]).unwrap().width(), 128);
}

#[test]
fn audio_duration_is_relative_to_the_offset_and_uses_the_shorter_trim_or_limit() {
    let Some(tools) = tools() else { return };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("tones.wav");
    audio_fixture(&input, &tools);
    let original = fs::read(&input).unwrap();
    for (index, start, end, maximum, expected, frequency) in [
        (0, 1.2, 2.5, 0.35, 0.35, 880.0),
        (1, 0.75, 0.9, 0.5, 0.15, 220.0),
    ] {
        let mut settings = settings(
            OutputFormat::Wav,
            &directory.path().join(format!("out-{index}")),
        );
        settings.video.trim_start = Some(start);
        settings.video.trim_end = Some(end);
        settings.limits.max_duration = Some(maximum);
        let output = convert(&input, &settings, &tools);
        let samples = audio_samples(&output[0], &tools);
        let seconds = samples.len() as f64 / 16000.0;
        assert!(
            (seconds - expected).abs() < 0.002,
            "{seconds} != {expected}"
        );
        let crossings = samples
            .windows(2)
            .filter(|pair| pair[0] <= 0 && pair[1] > 0)
            .count();
        assert!((crossings as f64 / seconds - frequency).abs() < 15.0);
    }
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn video_and_extracted_audio_respect_duration_after_seek() {
    let Some(tools) = tools() else { return };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("colors.mp4");
    video_fixture(&input, &tools);
    let original = fs::read(&input).unwrap();
    for format in [OutputFormat::Mp4, OutputFormat::Wav] {
        let mut settings = settings(format, &directory.path().join(format.extension()));
        settings.video.trim_start = Some(1.25);
        settings.video.trim_end = Some(2.0);
        settings.limits.max_duration = Some(0.3);
        let output = convert(&input, &settings, &tools);
        assert!((duration(&output[0], &tools) - 0.3).abs() <= 0.06);
        if format == OutputFormat::Mp4 {
            let frame = command(tools.ffmpeg.as_ref().unwrap())
                .args(["-hide_banner", "-nostdin", "-loglevel", "error", "-i"])
                .arg(&output[0])
                .args([
                    "-frames:v",
                    "1",
                    "-f",
                    "rawvideo",
                    "-pix_fmt",
                    "rgb24",
                    "pipe:1",
                ])
                .output()
                .unwrap();
            assert!(frame.status.success());
            assert!(
                frame.stdout[2] > 200 && frame.stdout[0] < 30,
                "seek did not reach blue frames"
            );
        }
    }
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn audio_and_video_hard_caps_fail_without_public_outputs_or_foreign_deletion() {
    let Some(tools) = tools() else { return };
    let directory = tempfile::tempdir().unwrap();
    let audio = directory.path().join("source.wav");
    let video = directory.path().join("source.mp4");
    audio_fixture(&audio, &tools);
    video_fixture(&video, &tools);
    for (input, format) in [(&audio, OutputFormat::Wav), (&video, OutputFormat::Mp4)] {
        let original = fs::read(input).unwrap();
        let output = directory.path().join(format.extension());
        fs::create_dir(&output).unwrap();
        let foreign = output.join(format!("source.{}", format.extension()));
        fs::write(&foreign, b"external output").unwrap();
        let mut settings = settings(format, &output);
        settings.limits = ConversionLimits {
            max_bytes: Some(64),
            max_duration: Some(0.2),
        };
        assert!(
            engine::convert(input, &settings, &tools, &AtomicBool::new(false), |_| {}).is_err()
        );
        assert_eq!(fs::read(input).unwrap(), original);
        assert_eq!(fs::read(&foreign).unwrap(), b"external output");
        assert_eq!(fs::read_dir(output).unwrap().count(), 1);
    }
}

#[test]
fn a_successful_hard_cap_publishes_the_same_encoded_audio_bytes() {
    let Some(tools) = tools() else { return };
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.wav");
    audio_fixture(&input, &tools);
    let mut settings = settings(OutputFormat::Wav, &directory.path().join("baseline"));
    settings.limits.max_duration = Some(0.25);
    let baseline = fs::read(&convert(&input, &settings, &tools)[0]).unwrap();
    settings.output_dir = Some(directory.path().join("limited"));
    settings.limits.max_bytes = Some(baseline.len() as u64);
    let result = convert(&input, &settings, &tools);
    assert_eq!(fs::read(&result[0]).unwrap(), baseline);
}

#[test]
fn common_image_cap_uses_the_stricter_legacy_limit_and_preserves_existing_width() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("noise.png");
    image_fixture(&input);
    let mut settings = settings(OutputFormat::Jpg, &directory.path().join("images"));
    settings.image.edit = Some(ImageEditSettings {
        max_bytes: Some(3000),
        ..Default::default()
    });
    let legacy = fs::read(&convert(&input, &settings, &ToolPaths::default())[0]).unwrap();
    assert!(legacy.len() <= 3000);
    settings.limits.max_bytes = Some(6000);
    let common = convert(&input, &settings, &ToolPaths::default());
    assert_eq!(fs::read(&common[0]).unwrap(), legacy);
    settings.limits.max_bytes = Some(2000);
    assert!(
        fs::metadata(&convert(&input, &settings, &ToolPaths::default())[0])
            .unwrap()
            .len()
            <= 2000
    );
    settings.image.edit = None;
    settings.image.width = Some(64);
    settings.limits.max_bytes = Some(100000);
    let output = convert(&input, &settings, &ToolPaths::default());
    assert_eq!(image::open(&output[0]).unwrap().width(), 64);
}

#[test]
fn prepared_cap_failure_preserves_reserved_name_and_unrelated_files() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("noise.png");
    image_fixture(&input);
    let output = directory.path().join("out");
    let mut settings = settings(OutputFormat::Bmp, &output);
    settings.limits.max_bytes = Some(100);
    let plan = engine::prepare(&input, &settings, &ToolPaths::default()).unwrap();
    fs::write(output.join("unrelated.txt"), b"keep").unwrap();
    assert!(
        engine::convert_prepared(
            &plan,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert!(!plan.outputs[0].path.exists());
    assert!(plan.outputs[0].marker.is_file());
    assert!(!plan.work_dir.exists());
    assert_eq!(fs::read(output.join("unrelated.txt")).unwrap(), b"keep");
    engine::release_prepared(&plan).unwrap();
}

#[test]
fn split_checks_every_page_before_publication_and_caps_each_file_not_the_batch_sum() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    pdf_fixture(&input);
    let original = fs::read(&input).unwrap();
    let mut settings = settings(OutputFormat::Pdf, &directory.path().join("baseline"));
    settings.pdf.action = PdfAction::Split;
    let baseline = convert(&input, &settings, &ToolPaths::default());
    let sizes: Vec<_> = baseline
        .iter()
        .map(|path| fs::metadata(path).unwrap().len())
        .collect();
    assert!(sizes[1] > sizes[0] * 2);
    settings.output_dir = Some(directory.path().join("too-small"));
    settings.limits.max_bytes = Some(sizes[0]);
    assert!(
        engine::convert(
            &input,
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {}
        )
        .is_err()
    );
    assert_eq!(
        fs::read_dir(settings.output_dir.as_ref().unwrap())
            .unwrap()
            .count(),
        0
    );
    settings.output_dir = Some(directory.path().join("per-file"));
    settings.limits.max_bytes = Some(sizes[1]);
    let outputs = convert(&input, &settings, &ToolPaths::default());
    assert_eq!(outputs.len(), 2);
    assert_eq!(
        outputs
            .iter()
            .map(|path| fs::metadata(path).unwrap().len())
            .sum::<u64>(),
        sizes.iter().sum::<u64>()
    );
    assert!(sizes.iter().sum::<u64>() > settings.limits.max_bytes.unwrap());
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn merge_cap_applies_to_final_pdf_without_restricting_private_preparation() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("image.png");
    image_fixture(&input);
    let original = fs::read(&input).unwrap();
    let output = directory.path().join("out");
    let mut settings = settings(OutputFormat::Pdf, &output);
    settings.pdf.action = PdfAction::Merge;
    settings.limits.max_bytes = Some(100);
    let prepared = pdf_merge::prepare_pdf_merge(
        std::slice::from_ref(&input),
        &settings,
        &ToolPaths::default(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    let result = pdf_merge::merge_prepared(
        &prepared,
        &prepared.pages(),
        &settings,
        &AtomicBool::new(false),
        |_| {},
    );
    assert!(result.is_err());
    assert_eq!(fs::read_dir(&output).unwrap().count(), 0);
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn cancelling_limited_publication_rolls_back_owned_files_but_retains_external_replacement() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.pdf");
    pdf_fixture(&input);
    let original = fs::read(&input).unwrap();
    let output = directory.path().join("out");
    let first = output.join("source-pagina-001.pdf");
    let moved = output.join("moved-by-user.pdf");
    let mut settings = settings(OutputFormat::Pdf, &output);
    settings.pdf.action = PdfAction::Split;
    settings.limits.max_bytes = Some(1000000);
    let cancel = AtomicBool::new(false);
    let mut replaced = false;
    let result = engine::convert(
        &input,
        &settings,
        &ToolPaths::default(),
        &cancel,
        |progress| {
            if !replaced
                && (progress.message.contains("entro il limite")
                    || progress.message.contains("within the size limit"))
            {
                fs::rename(&first, &moved).unwrap();
                fs::write(&first, b"external replacement").unwrap();
                replaced = true;
                cancel.store(true, Ordering::Relaxed);
            }
        },
    );
    assert!(result.is_err());
    assert!(replaced);
    assert_eq!(fs::read(first).unwrap(), b"external replacement");
    assert!(Document::load(moved).is_ok());
    assert_eq!(fs::read_dir(output).unwrap().count(), 2);
    assert_eq!(fs::read(input).unwrap(), original);
}
