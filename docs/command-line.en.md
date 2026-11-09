# Command line

[Italiano](command-line.md) · **English**

Convert Here's CLI converts files, automates processing and installs optional engines. Run the PowerShell examples from the app folder. See the [format guide](conversions.en.md) for conversion requirements.

Use the dedicated console executable for output, diagnostics and reliable exit codes:

```powershell
.\the-converter-cli.exe --language en --diagnose
.\the-converter-cli.exe --install-runtime ffmpeg
.\the-converter-cli.exe --install-runtime poppler
.\the-converter-cli.exe --convert --target mp3 --preset quality --files "C:\Video\clip.mp4"
.\the-converter-cli.exe --convert --target png --output-dir "C:\Results" --files "C:\Documents\file.pdf"
.\the-converter-cli.exe --convert --target csv --files "C:\Documents\table.xlsx"
.\the-converter-cli.exe --convert --target xlsx --csv-separator semicolon --files "C:\Documents\table.csv"
.\the-converter-cli.exe --convert --target docx --files "C:\Documents\file.pdf"
.\the-converter-cli.exe --convert --target docx --ocr --ocr-language ita+eng --files "C:\Documents\scan.pdf"
.\the-converter-cli.exe --convert --target txt --ocr-language ita+eng --files "C:\Documents\scan.png"
.\the-converter-cli.exe --convert --target avif --files "C:\Images\photo.heic"
.\the-converter-cli.exe --convert --target pdf --files "C:\Documents\page.html"
.\the-converter-cli.exe --convert --target pdf --pdf-action compress --files "C:\Documents\file.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action split --files "C:\Documents\file.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Documents\first.pdf" "C:\Documents\second.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Images\photo.png" "C:\Documents\document.docx" "C:\Documents\pages.pdf"
.\the-converter-cli.exe --convert --settings .\settings.json --files "C:\Audio\recording.wav"
```

`--language system|it|en` selects the interface and application-message language for the current invocation without changing saved preferences. Without an override, the app uses the saved language choice. `system` follows the Windows display language, using Italian for Italian Windows and English otherwise. This option is available in the updated source code; the previously published v0.4.0 binaries do not include it. Option names, JSON keys and output filename suffixes remain stable across languages. Windows and external-engine messages may use their own language settings.

`--convert` runs without a UI, continues with other files after an error and exits with a nonzero code if any conversion fails. Ctrl+C cancels processing and cleans temporary files. Paths are passed directly to processes without shell interpolation.

`--install-runtime ffmpeg|poppler` is a separate mode: it explicitly downloads a pinned, verified engine, without conversion options or input files. It writes into the user's engine folder, preserves licenses and documentation and does not execute the downloaded program. Ctrl+C cancels the operation. The optional `install-runtime-tools.ps1 -InstallDirectory <app-folder> -Engine ffmpeg` script calls the same CLI; omit `-Engine` to request both engines. The app folder locates the CLI; it is not the engine destination.

`--pdf-action compress|split|merge` requires PDF output. `compress` and `split` accept only PDFs; `merge` accepts the PDF-compatible files described in the [conversion guide](conversions.en.md#compress-split-and-merge-pdfs). The default is `compress`. CLI merging follows argument order and page order within each source; use the graphical editor to interleave individual pages.

`--ocr` enables recognition of scanned PDFs for text destinations. Images converted to TXT always use OCR. `--ocr-language ita+eng` selects recognition models without enabling PDF OCR by itself; it is independent of `--language`. `--csv-separator comma|semicolon|tab|pipe` selects the separator for import and export; the default is `comma`.

Example settings JSON; omitted fields use their defaults:

```json
{
  "format": "Mp3",
  "audio": { "bitrate_kbps": 256, "sample_rate": 48000, "channels": 2, "normalize": true },
  "video": { "trim_start": 5.0, "trim_end": 35.0 }
}
```

[Back to README](../README.en.md)
