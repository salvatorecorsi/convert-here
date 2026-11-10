# Command line

[Italiano](command-line.md) · **English**

Convert Here's CLI converts files, automates processing and installs optional engines. Run the PowerShell examples from the app folder. See the [format guide](conversions.en.md) for conversion requirements.

**Status on 10 November 2026:** [0.8.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.0) published on GitHub at **12:09:39 UTC**, commit `f1f3f95fceed52507ef715fa4ac6893acc63544a`. Store **1.0.8.0**, Submission 5 (`1152921505702365703`): **Update in certification**, not yet approved or published in the Store.

Use the dedicated console executable for output, diagnostics and reliable exit codes:

```powershell
.\the-converter-cli.exe --language en --diagnose
.\the-converter-cli.exe --install-runtime ffmpeg
.\the-converter-cli.exe --install-runtime poppler
.\the-converter-cli.exe --install-runtime tesseract
.\the-converter-cli.exe --install-runtime libheif
.\the-converter-cli.exe --install-runtime whisper
.\the-converter-cli.exe --convert --target mp3 --preset quality --files "C:\Video\clip.mp4"
.\the-converter-cli.exe --convert --target png --output-dir "C:\Results" --files "C:\Documents\file.pdf"
.\the-converter-cli.exe --convert --target csv --files "C:\Documents\table.xlsx"
.\the-converter-cli.exe --convert --target xlsx --csv-separator semicolon --files "C:\Documents\table.csv"
.\the-converter-cli.exe --convert --target docx --files "C:\Documents\file.pdf"
.\the-converter-cli.exe --convert --target docx --ocr --ocr-language ita+eng --files "C:\Documents\scan.pdf"
.\the-converter-cli.exe --convert --target txt --ocr-language ita+eng --files "C:\Documents\scan.png"
.\the-converter-cli.exe --convert --target avif --files "C:\Images\photo.heic"
.\the-converter-cli.exe --convert --target png --files "C:\Images\illustration.svg"
.\the-converter-cli.exe --convert --target pdf --files "C:\Images\illustration.svg"
.\the-converter-cli.exe --convert --target txt --files "C:\Audio\recording.wav"
.\the-converter-cli.exe --custom-settings --target txt --files "C:\Audio\recording.wav"
.\the-converter-cli.exe --convert --target pdf --files "C:\Documents\page.html"
.\the-converter-cli.exe --convert --target pdf --pdf-action compress --files "C:\Documents\file.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action split --files "C:\Documents\file.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Documents\first.pdf" "C:\Documents\second.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Images\photo.png" "C:\Documents\document.docx" "C:\Documents\pages.pdf"
.\the-converter-cli.exe --convert --settings .\settings.json --files "C:\Audio\recording.wav"
```

`--language system|it|en` selects the interface and application-message language for the current invocation without changing saved preferences. Without an override, the app uses the saved language choice. `system` follows the Windows display language, using Italian for Italian Windows and English otherwise. This option is available from version 0.5.0. Option names, JSON keys and output filename suffixes remain stable across languages. Windows and external-engine messages may use their own language settings.

`--convert` runs without a UI, continues with other files after an error and exits with a nonzero code if any conversion fails. Ctrl+C cancels processing and cleans temporary files. Paths are passed directly to processes without shell interpolation.

`--install-runtime ffmpeg|poppler|tesseract|libheif|whisper` is a separate mode: it explicitly downloads a pinned, verified engine, without conversion options or input files. The last three values are available in version 0.8.0. It writes into `%LOCALAPPDATA%\ConvertHere\tools`, preserves licenses and documentation and does not execute the downloaded program. Tesseract includes `ita`/`eng` models; Whisper includes Tiny multilingual. Ctrl+C cancels the operation. The optional `install-runtime-tools.ps1 -InstallDirectory <app-folder> -Engine ffmpeg` script calls the same CLI; omit `-Engine` to request **FFmpeg and Poppler**. The app folder locates the CLI; it is not the engine destination.

For audio/video → TXT, install **both FFmpeg and Whisper**: the CLI Whisper command does not automatically add FFmpeg. Transcription runs on the PC, without an account or API key. Without specific settings, language is detected automatically. `--language` selects the app language and `--ocr-language` selects OCR language; neither selects speech language. Set `speech.language` in JSON for the latter.

`--pdf-action compress|split|merge` requires PDF output. `compress` and `split` accept only PDFs; `merge` accepts the PDF-compatible files described in the [conversion guide](conversions.en.md#compress-split-and-merge-pdfs). The default is `compress`. CLI merging follows argument order and page order within each source; use the graphical editor to interleave individual pages.

`--ocr` enables recognition of scanned PDFs for text destinations. Images converted to TXT always use OCR. `--ocr-language ita+eng` selects recognition models without enabling PDF OCR by itself; it is independent of `--language`. `--csv-separator comma|semicolon|tab|pipe` selects the separator for import and export; the default is `comma`.

Example settings JSON; omitted fields use their defaults:

```json
{
  "format": "Mp3",
  "audio": { "bitrate_kbps": 256, "sample_rate": 48000, "channels": 2, "normalize": true },
  "video": { "trim_start": 5.0, "trim_end": 35.0 },
  "limits": { "max_bytes": 3000000, "max_duration": 30.0 },
  "media_options": { "threads": "1", "audio_volume": "0.5" }
}
```

`max_bytes` limits each output; an oversized result is not published. `max_duration` limits audio/video inputs after `trim_start`, combined with any `trim_end`, including transcription; it does not truncate image or TXT inputs. FFmpeg options are typed catalogue values, not arbitrary arguments. Here `audio_volume: "0.5"` halves the amplitude, about −6 dB; gain is applied after any normalization. Active `media_options` reject non-audio/video inputs, including native image-to-GIF. Browse the available catalogue in **Profiles → Advanced**.

Example `speech-settings.json` transcribing seconds 5–35 in Italian with four CPU threads:

```json
{
  "format": "Txt",
  "video": { "trim_start": 5.0, "trim_end": 35.0 },
  "speech": { "language": "it", "threads": 4 },
  "limits": { "max_duration": 30.0, "max_bytes": 100000 }
}
```

```powershell
.\the-converter-cli.exe --convert --settings .\speech-settings.json --files "C:\Audio\recording.wav"
```

`video.trim_start` and `video.trim_end` apply to audio too. Omit `speech.language` or use `auto` for automatic detection; `it` and `en` select Italian and English. Gain/mute and encoding `media_options` are not accepted for TXT. The job uses temporary mono 16 kHz WAV and TXT in chunks of at most 10 minutes, removed on completion or cancellation. Final output does not overwrite an existing file.

To use a saved recipe, `--quick --profile <id> --profile-revision <revision> --files ...` uses the current profile only when its revision matches and inputs are allowed. Explorer supplies these values automatically. Normal profiles queue a settings snapshot; PDF Merge profiles require at least two files and open page ordering. `--custom-settings` opens the compatible editor or window options without starting a conversion. Profile import/export belongs to the **Profiles** tab; its portable JSON differs from the individual `settings.json` shown above.

[Back to README](../README.en.md)
