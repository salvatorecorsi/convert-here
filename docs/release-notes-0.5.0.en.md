# Convert Here 0.5.0 — Italian and English

[Italiano](release-notes-0.5.0.md) · **English**

Developed by **Salvatore Corsi**. GitHub release 0.5.0 brings the bilingual interface to ready-to-use installer and portable packages.

## Choose your language

Open **Settings → Language** and choose **System**, **Italiano** or **English**. System uses Italian when the Windows display language is Italian and English otherwise. The app window changes immediately and saves your choice for future sessions.

Translations also cover the **Converti in / Convert to** menu, status and PDF Merge windows, engine dialogs, notices, progress and application messages. File Explorer reads the saved choice. Messages produced by Windows or external engines may follow their own language settings.

Interface language is independent of OCR language. Tesseract continues to use your selected models, with `ita+eng` as the default.

## CLI and compatibility

Choose a language for a single invocation without changing the saved preference:

```powershell
.\the-converter-cli.exe --language en --help
.\the-converter-cli.exe --language it --convert --target jpg --files "C:\Images\photo.png"
```

`--language system|it|en` also applies to diagnostics and quick conversions. Each quick job retains an explicit CLI language choice; other jobs follow saved preferences. Preferences and queued jobs from earlier versions remain readable. Formats, JSON keys, CLI options and output filenames retain their identifiers.

The release offers **44 output formats** and **93 recognized input extensions**, batch conversion, local OCR, PDF operations and File Explorer integration. The guides and [English README](../README.en.md) cover the same functions as the Italian documentation and include four [real English screenshots](screenshots/README.md).

## Download and install

From [GitHub release 0.5.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.5.0):

- `ConvertHere-0.5.0-windows-x64-setup.exe`: per-user installation without administrator privileges; optional context-menu integration.
- `ConvertHere-0.5.0-windows-x64-portable.zip`: extract to a stable folder and run `the-converter.exe`; includes the CLI and menu scripts.
- `ConvertHere-0.5.0-source.zip`: corresponding sources, both lockfiles, vendored Cargo dependencies and offline build instructions.
- `SHA256SUMS.txt`: SHA-256 checksums for the installer, portable and source archives.

Requirements: **Windows 10 build 19041 or later, x64**. On Windows 11, the GitHub distribution's classic menu is under **Show more options**. GitHub packages are not Authenticode-signed.

Packages do not include external engines. Native images, images to PDF and operations on existing PDFs work immediately. Configure the [required engines](configuration.en.md) for audio/video, PDF export, Office documents, OCR and HEIC/AVIF. Download FFmpeg and Poppler from the app after reviewing the source, version, size, licenses and destination. Conversion and OCR run on your PC; engine downloads do not upload the files you convert.

## License

Convert Here is free, with no account, subscription or in-app purchases. Copyright (C) 2026 Salvatore Corsi. Original code is licensed under **GPL-3.0-only**, without warranty under the [complete license](../LICENSE). Dependencies, fonts and engines keep their own licenses. See the bilingual [privacy statement](privacy.md).

[Italian README](../README.md) · [English README](../README.en.md)
