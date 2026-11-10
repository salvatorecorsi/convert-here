# Configuration, engines and local data

[Italiano](configuration.md) · **English**

This guide describes Convert Here's settings. Start with the [README](../README.en.md); see the [format guide](conversions.en.md) for available conversions.

**Status on 10 November 2026:** [0.8.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.0) published on GitHub at **12:09:39 UTC**, commit `f1f3f95fceed52507ef715fa4ac6893acc63544a`. Store **1.0.8.0**, Submission 5 (`1152921505702365703`): **Update in certification**, not yet approved or published in the Store.

## Interface language

In **Settings → Language**, choose **System**, **Italiano** or **English**. System follows the Windows display language: Italian for Italian Windows, English otherwise. The choice is stored in the `ui_language` field of `preferences.json` as `system`, `it` or `en`. Existing settings without that field use System. It does not change OCR or speech language, conversion formats, output filenames or other preferences.

The interface and setup are available in Italian and English.

For one CLI invocation, pass `--language system|it|en`. This overrides the saved choice without rewriting it. Messages produced by Windows and external engines may follow their own language settings. [CLI guide](command-line.en.md).

## Conversion engines

In **Settings → Conversion engines**, choose executable paths or detect installed tools. The app searches saved settings, the `tools` folder next to the executable, `%LOCALAPPDATA%\ConvertHere\tools`, `PATH` and the folder specified by `THE_CONVERTER_TOOLS`. Older engine folders are still read for compatibility. LibreOffice and Tesseract are also searched in standard installation locations.

For Poppler:

- `pdftoppm.exe` exports pages to images and prepares PDFs for OCR.
- `pdftotext.exe` extracts embedded text.
- `pdfinfo.exe` counts pages. The internal PDF reader is used for counting if it is missing.

Selecting a Poppler executable makes the app look for the others in the same installation, while preserving any separate explicit paths.

For OCR, configure `tesseract.exe` and keep `tessdata` beside it with the required models, for example `ita.traineddata` and `eng.traineddata`. Other codes work when their models are installed. OCR language does not change interface language.

For libheif, keep `heif-dec.exe` and `heif-enc.exe` with DLLs and plugins from the same distribution. The build must include the necessary HEVC/AV1 decoders and encoders.

For transcription, configure `whisper-cli.exe` and the `ggml-tiny.bin` model. FFmpeg and ffprobe are also required: the Whisper runtime alone does not read every audio/video container directly. Speech language and CPU threads are configured in transcription controls, separately from interface and OCR language. Automatic language detection is the default; profiles store `speech.language` and `speech.threads`.

## Optional engine downloads

App packages do not contain external engines or the speech model. Request only those needed in **Settings → Conversion engines**. Version 0.8.0 adds Tesseract, libheif and Whisper + Tiny to FFmpeg and Poppler downloads. Confirmation displays the source, version, download size, extracted space, licenses and destination. Installation uses only the personal folder `%LOCALAPPDATA%\ConvertHere\tools`, without UAC or a system installer.

| Engine | Pinned distribution | Download |
| --- | --- | --- |
| FFmpeg + Poppler | [FFmpeg 9.0.2 Gyan](https://github.com/GyanD/codexffmpeg/releases/tag/9.0.2) and [Poppler 26.09.0](https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0) | 158,478,032 bytes combined; about 615 MB free during downloading/extraction |
| Tesseract + ita/eng | [Tesseract 5.5.4 from MSYS2](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-tesseract-ocr) and OCR models | 38,276,135 bytes; about 60 MB extracted |
| libheif | [libheif 1.23.7 from MSYS2](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-libheif) | 28,215,281 bytes; about 71.4 MB extracted |
| Whisper + Tiny | [Whisper 1.9.5.r258 from MSYS2](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-whisper.cpp) and [Tiny multilingual](https://huggingface.co/ggerganov/whisper.cpp) | 87,939,016 bytes, including the 77,691,713-byte model; about 101.5 MB extracted |

The [portable package catalogue](../src/runtime_install/portable-packages.json) records pinned versions, sources and SHA-256 values for Tesseract, libheif and Whisper/Tiny. SHA-256 is checked before extraction; licenses and documentation are preserved. Downloading does not execute the tools. Portable distributions include the required dependencies; Whisper provides a generic CPU backend without requiring manual Visual C++ runtime installation.

Selecting Whisper in the app also adds FFmpeg to the confirmation request when FFmpeg or ffprobe is missing. Its size is separate from Whisper + Tiny's 87,939,016 bytes. No engine or model downloads automatically when the app starts.

HTTPS requests reach distributors and their CDNs: GitHub/Gyan for existing engines, MSYS2 and its mirrors, GitHub for licenses/OCR models, and Hugging Face and CDNs for Tiny. They receive the IP address and normal HTTP request data, including a User-Agent with the app version. Files to convert are not uploaded. Cancellation and errors attempt to remove temporary staging; an unexpected shutdown may leave residual files.

The setup does not start downloads or add a Start menu shortcut for installing engines. Install and select LibreOffice separately; existing local copies of the other engines remain supported. Development copies under `.tools/` are excluded from public packages.

### Command-line alternative

```powershell
.\the-converter-cli.exe --install-runtime ffmpeg
.\the-converter-cli.exe --install-runtime poppler
.\the-converter-cli.exe --install-runtime tesseract
.\the-converter-cli.exe --install-runtime libheif
.\the-converter-cli.exe --install-runtime whisper
.\scripts\install-runtime-tools.ps1 -InstallDirectory "C:\Programs\ConvertHere" -Engine ffmpeg
```

The three additional commands are available in version 0.8.0. The CLI installs only the requested engine: install FFmpeg separately for transcription. The script calls the same CLI; omitting `-Engine` requests **FFmpeg and Poppler**, not every engine. `-InstallDirectory` locates the app folder, not the engine destination. Distributors are contacted only when you run the command. [CLI guide](command-line.en.md).

## Visible formats and appearance

In **Settings → Output formats**, choose which destinations appear in the app and Explorer menu, grouped into audio, video, images, documents and **Raw**. The initial eight outputs remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Enable **TXT** in Raw to show OCR and transcription.

Choices are saved and the menu rereads them without restarting Explorer. **Custom settings…** stays available even if all outputs are disabled. The filter does not disable inputs, the CLI, explicit `--quick` requests or already queued jobs. Version 0.8.0 retains 44 output formats and recognizes 94 input extensions, including SVG. **Files**, **History** and **Profiles** sit on the left of navigation; **Settings** is on the right.

For appearance, follow the Windows theme or select **Light** or **Dark**.

## Personal profiles

The **Profiles** tab creates, edits, duplicates, renames and deletes recipes and selects a default. Store up to **32 profiles**, with **8 visible in Convert to** when their output is enabled and every selected input is compatible. Allow all compatible inputs or restrict a recipe to selected extensions; this does not change version 0.8.0's 94 recognized input extensions. Reopen the Explorer menu after changing a profile. Already queued jobs retain their recipe. PDF Merge profiles require at least two files and open page ordering.

**Import JSON…**, **Export…** and **Export all…** transfer recipes. JSON contains parameters, input filters, visibility and an optional output folder, without engine paths or sample content. Import is atomic, creates new copies and never overwrites profiles. Recipes beyond the eight visible entries are imported hidden; files over 256 KiB or imports exceeding 32 total profiles are rejected. User-defined names are not translated. [Complete guide and JSON example](image-editor-profiles.en.md).

### Size, duration and Advanced

The common **maximum size per file** is checked before publishing each output. Older image limits remain supported; when both exist, the smaller one wins. The JPEG/WebP editor can find a suitable quality and resize only with permission. Other routes fail if their output remains too large; audio/video limits do not promise automatic bitrate adjustment.

**Maximum audio/video duration** applies to audio/video sources, including audio extraction, GIF and transcription, after the selected start and combined with the trim end. It does not truncate image or TXT inputs.

**Advanced** provides searchable typed codec, filter, metadata and performance controls. Encoder-specific options are discovered from configured local FFmpeg, not an online service. Availability depends on the build, codec and container; incompatible or unverifiable options produce an error. Recipes with FFmpeg options require audio/video inputs, including for GIF output. The catalogue accepts no free-form commands, scripts or executable paths.

In the audio/video editor, gain and mute become `media_options.audio_volume`; trimming uses `video.trim_start` and `video.trim_end` for audio too. TXT applies only trimming, limits and speech settings: gain, mute and encoding options are excluded. **Use these settings** updates active settings; **Save profile…** stores parameters without source content or the full transcript.

## Preferences and reports

Preferences and the last quick report are in `%APPDATA%\SalvatoreCorsi\TheConverter\config`, in `preferences.json` and `report-last.json`. `history.json` retains up to 300 completed window, editor and PDF Merge attempts within 1 MiB; **History** combines them with quick-queue jobs. These local files contain paths and errors and are not transmitted. History writes run in the background; unreadable history is reported and preserved.

With MSIX installations, Windows may redirect paths into private package data. The app does not encrypt preferences and state: Windows account and device protections apply. [Privacy statement](privacy.md).

## Quick queue and recovery

The quick queue is saved under the app's `%LOCALAPPDATA%` data. A single process handles jobs even when they come from multiple context-menu commands. After a forced interruption, running jobs are marked as interrupted; waiting jobs resume when the app is started or another request is submitted.

Names are reserved before work starts and remain as shown in the placeholders. PDF-to-image exports reserve a name for each page. Temporary folders in the destination are hidden. The placeholder's Explorer hourglass is a static icon.

History cleanup runs when the worker resumes or finishes the queue. It removes completed or cancelled records after 7 days and failed records after 30 days, and attempts to keep at most the 300 newest terminal records. Originals, outputs and active jobs are preserved. Unreadable records are quarantined and retained, as are unrelated files.

For a failed job, **Remove placeholder** deletes only shortcuts that still belong to that job, preserving records and results. Before manually deleting local data, finish or cancel jobs and close the app. [Queue details](queued-conversions.en.md).

## Portable edition context menu

Extract the archive to a stable folder. Run this from the extracted folder to install the classic menu for the current user:

```powershell
.\scripts\install-context-menu.ps1 -Mode Classic
```

To remove it:

```powershell
.\scripts\uninstall-context-menu.ps1 -Mode Classic
```

On Windows 11, it appears under **Show more options**. The [Windows integration guide](windows-integration.en.md) also covers the modern-menu extension and its requirements.

[Back to README](../README.en.md)
