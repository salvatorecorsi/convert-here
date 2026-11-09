# Configuration, engines and local data

[Italiano](configuration.md) · **English**

This guide describes Convert Here's settings. Start with the [README](../README.en.md); see the [format guide](conversions.en.md) for available conversions.

## Interface language

In **Settings → Language**, choose **System**, **Italiano** or **English**. System follows the Windows display language: Italian for Italian Windows, English otherwise. The choice is stored in the `ui_language` field of `preferences.json` as `system`, `it` or `en`. Existing settings without that field use System. It does not change OCR language, conversion formats, output filenames or other preferences.

The 0.5.0 installer and portable packages include the bilingual interface ready to use. The setup is also available in Italian and English.

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

## Optional FFmpeg and Poppler downloads

Public packages do not contain the engines. Request FFmpeg, Poppler or both in **Settings → Conversion engines**. The confirmation displays the source, version, size, licenses and destination. Installation uses only the personal folder `%LOCALAPPDATA%\ConvertHere\tools`.

The 0.5.0 engine configuration downloads [FFmpeg 9.0.2 from Gyan's distribution](https://github.com/GyanD/codexffmpeg/releases/tag/9.0.2) and [Poppler 26.09.0 for Windows](https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0): 158478032 bytes in total, about 159 MB. About 615 MB of free space is needed during downloading and extraction. SHA-256 is checked before extraction; upstream licenses and documentation are preserved.

HTTPS requests reach the distributors and their CDNs, which receive the IP address and normal HTTP request data, including a User-Agent with the app version. Files to convert are not uploaded. Cancellation and errors attempt to remove temporary staging; an unexpected shutdown may leave residual files.

The setup does not start downloads or add a Start menu shortcut for installing engines. Configure LibreOffice, libheif, Tesseract and OCR models from local installations. Development copies under `.tools/` are excluded from public packages.

### Command-line alternative

```powershell
.\the-converter-cli.exe --install-runtime ffmpeg
.\the-converter-cli.exe --install-runtime poppler
.\scripts\install-runtime-tools.ps1 -InstallDirectory "C:\Programs\ConvertHere" -Engine ffmpeg
```

The script calls the same CLI; omitting `-Engine` requests both engines. `-InstallDirectory` locates the app folder, not the engine destination. Distributors are contacted only when you run the command. [CLI guide](command-line.en.md).

## Visible formats and appearance

In **Settings → Output formats**, choose which destinations appear in the app and Explorer menu, grouped into audio, video, images and documents. The initial eight outputs are MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4.

Choices are saved and the menu rereads them without restarting Explorer. **Custom settings…** stays available even if all outputs are disabled. The filter does not block the CLI, explicit `--quick` requests or already queued jobs. All 44 engine formats and 93 input extensions remain supported.

For appearance, follow the Windows theme or select **Light** or **Dark**.

## Personal profiles

The new **Profiles** tab creates, edits, duplicates, renames and deletes recipes and selects a default. Store up to **32 profiles**, with **8 visible in Convert to** when their output is enabled and every selected input is compatible. Allow all compatible inputs or restrict a recipe to selected extensions; this does not change the app's 93 recognized input extensions. Reopen the Explorer menu after changing a profile. Already queued jobs retain their recipe. PDF Merge profiles require at least two files and open page ordering.

**Import JSON…**, **Export…** and **Export all…** transfer recipes. JSON contains parameters, input filters, visibility and an optional output folder, without engine paths or sample content. Import is atomic, creates new copies and never overwrites profiles. Recipes beyond the eight visible entries are imported hidden; files over 256 KiB or imports exceeding 32 total profiles are rejected. User-defined names are not translated. [Complete guide and JSON example](image-editor-profiles.en.md).

### Size, duration and Advanced

The common **maximum size per file** is checked before publishing each output. Older image limits remain supported; when both exist, the smaller one wins. The JPEG/WebP editor can find a suitable quality and resize only with permission. Other routes fail if their output remains too large; audio/video limits do not promise automatic bitrate adjustment.

**Maximum audio/video duration** applies to audio/video sources, including audio extraction and GIF, after the selected start and combined with the trim end. It does not change images or text.

**Advanced** provides searchable typed codec, filter, metadata and performance controls. Encoder-specific options are discovered from configured local FFmpeg, not an online service. Availability depends on the build, codec and container; incompatible or unverifiable options produce an error. Recipes with FFmpeg options require audio/video inputs, including for GIF output. The catalogue accepts no free-form commands, scripts or executable paths.

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
