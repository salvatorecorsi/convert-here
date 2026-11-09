//! Cross-process checks: preference loading, explicit CLI override, real native
//! conversion and diagnostics must agree without rewriting a user's settings.
use std::{
    fs,
    process::{Command, Output},
};

fn invoke(config: &std::path::Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_the-converter-cli"))
        .env("THE_CONVERTER_CONFIG_DIR", config)
        .args(arguments)
        .output()
        .expect("run localized CLI")
}

#[test]
fn help_and_errors_follow_saved_language_and_invocation_override() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("preferences.json");
    let preferences =
        br#"{"ui_language":"it","settings":{"format":"Flac","ocr":{"language":"ita+eng"}}}"#;
    fs::write(&path, preferences).unwrap();
    for (arguments, expected, absent) in [
        (
            vec!["--help"],
            "File da aprire",
            "Files to open in the window",
        ),
        (
            vec!["--language=en", "--help"],
            "Files to open in the window",
            "File da aprire",
        ),
        (
            vec!["--language", "it", "--help"],
            "File da aprire",
            "Files to open in the window",
        ),
    ] {
        let output = invoke(directory.path(), &arguments);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains(expected), "{text}");
        assert!(!text.contains(absent), "{text}");
    }
    for (language, expected) in [
        ("en", "Unsupported format: invalid"),
        ("it", "Formato non supportato: invalid"),
    ] {
        let output = invoke(
            directory.path(),
            &["--language", language, "--convert", "--target", "invalid"],
        );
        assert!(!output.status.success());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains(expected), "{text}");
    }
    assert_eq!(fs::read(path).unwrap(), preferences);
}

#[test]
fn native_conversion_keeps_outputs_and_settings_independent_of_ui_language() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config");
    fs::create_dir(&config).unwrap();
    let settings =
        br#"{"ui_language":"it","settings":{"format":"Flac","ocr":{"language":"ita+eng"}}}"#;
    fs::write(config.join("preferences.json"), settings).unwrap();
    let input = directory.path().join("source.png");
    fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ocr.png"),
        &input,
    )
    .unwrap();
    for (language, expected) in [
        ("en", "Output files created: 1"),
        ("it", "Completato: 1 file"),
    ] {
        let output_folder = directory.path().join(language);
        let output = invoke(
            &config,
            &[
                "--language",
                language,
                "--convert",
                "--target",
                "jpg",
                "--output-dir",
                output_folder.to_str().unwrap(),
                "--files",
                input.to_str().unwrap(),
            ],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result = output_folder.join("source.jpg");
        assert!(image::open(&result).is_ok());
        assert!(String::from_utf8(output.stderr).unwrap().contains(expected));
    }
    assert_eq!(fs::read(config.join("preferences.json")).unwrap(), settings);
    assert_eq!(
        fs::read(directory.path().join("en/source.jpg")).unwrap(),
        fs::read(directory.path().join("it/source.jpg")).unwrap()
    );
}
