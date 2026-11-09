# Convert Here

[Italiano](README.md) · **English**

**Convert audio, video, images and documents directly on your Windows PC.**

Developed by **[Salvatore Corsi](https://github.com/salvatorecorsi)**, Convert Here is a free, open source program written in Rust. Use its window, the File Explorer context menu or the command line. Conversion and OCR run on your computer without uploading your files to an online service. No account, subscription or in-app purchases.

[Download for Windows](https://github.com/salvatorecorsi/convert-here/releases/latest) · [Microsoft Store](https://apps.microsoft.com/detail/9NPKPPCBNBM8) · [What's new in 0.5.0](docs/release-notes-0.5.0.en.md) · [Report a problem](https://github.com/salvatorecorsi/convert-here/issues)

**GitHub release 0.5.0 is bilingual, in Italian and English.** Download the installer or portable package and choose a language in **Settings → Language**. See [Languages](#languages).

![Convert Here in English: two PNG images converted to JPEG, with two completed files and zero errors](docs/screenshots/en-conversion.jpg)

*Two images successfully converted to JPEG in the English interface: two completed files and zero errors. The originals are preserved and each file shows its result.*

## What you can do

- **Convert multiple files together**, with 44 output formats and 93 recognized input extensions.
- **Use the right-click menu** to queue quick conversions with your saved preferences or open custom settings.
- **Convert audio and video**, extract audio from video, create animated GIFs and adjust quality, dimensions and time range.
- **Convert and resize images**, create PDFs from images and work with HEIC/HEIF, AVIF, OpenEXR and HDR.
- **Compress, split and merge PDFs**, including arranging individual pages from different images and documents.
- **Extract text from images to TXT and from PDFs to TXT, Word DOCX or HTML**, with local OCR for scans.
- **Convert Office documents, Excel, CSV and HTML**, including choosing the CSV separator.
- **Customize visible formats and appearance**, with system, light and dark themes.

## Installation

The GitHub distribution requires **Windows 10 build 19041 or later, x64**. The [0.5.0 release](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.5.0) offers:

| Package | How to use it |
| --- | --- |
| [ConvertHere-0.5.0-windows-x64-setup.exe](https://github.com/salvatorecorsi/convert-here/releases/download/v0.5.0/ConvertHere-0.5.0-windows-x64-setup.exe) | Run the setup. It installs for your user without administrator privileges; optional context-menu integration is selected by default. |
| [ConvertHere-0.5.0-windows-x64-portable.zip](https://github.com/salvatorecorsi/convert-here/releases/download/v0.5.0/ConvertHere-0.5.0-windows-x64-portable.zip) | Extract it to a stable folder and run `the-converter.exe`. It also includes the CLI and context-menu scripts. |

Check the packages against `SHA256SUMS.txt` from the same release. The executable names remain `the-converter.exe` and `the-converter-cli.exe`; some earlier releases display the name The Converter.

**Native images, images to PDF and operations on existing PDFs work immediately.** Configure the [optional engines](#optional-engines) for other conversions: the setup neither includes nor downloads them. Python, Node and Rust are not required to run the app.

In the GitHub distribution, the Windows 11 menu is under **Show more options**. The GitHub 0.5.0 packages are not Authenticode-signed. See the [Windows integration guide](docs/windows-integration.en.md) for classic and modern menu details.

To uninstall, use **Windows Settings → Apps → Installed apps → Convert Here → Uninstall**. Preferences and converted files are preserved.

## How to use it

### From the app window

1. Open Convert Here and drag files into the window, or choose **Add files**.
2. Select the output **format** and a preset: **Compact**, **Balanced** or **High quality**.
3. Choose to save **next to the original** or in a folder of your choice. Open **Custom settings** for format-specific controls.
4. Start the conversion and follow each file's progress. You can cancel the job.

Preferences are saved between sessions. Originals and existing results are preserved: the app adds a number when a name is already taken. The interactive queue uses one format shared by the selection; convert categories in separate groups if they have no common destination.

### From the File Explorer context menu

Select one or more files and choose **Convert to → MP3**, **WAV**, **PDF** or another compatible format. The conversion enters the background quick queue using your saved settings. **Custom settings…** opens the app window instead.

A placeholder with a **blue hourglass** and the planned output name appears in the destination. It is a temporary shortcut: double-click to see progress, details and cancellation. The converted file replaces it when the job finishes; an error gives it a **red X**. Closing the status window lets the job continue. The main window shows the **Quick queue** and **History**. [How the queue works](docs/queued-conversions.en.md).

With multiple files that can become PDFs:

- **PDF (Bulk)** creates separate results, one per source. Existing PDFs are compressed without loss.
- **PDF (Merge)** opens the page editor to create a single document in your chosen order.

### Choosing which formats to show

Open **Settings → Output formats** and enable the destinations you use. Your choices apply to both the app selector and File Explorer, without restarting Explorer. **MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4** are enabled initially. Each selection shows only compatible destinations.

![Convert Here settings in English: output formats grouped by category and the System, Italiano and English language choices](docs/screenshots/en-settings.jpg)

*Enable every format, restore the common formats or choose individual outputs. This is a display filter: all inputs remain recognized and the CLI keeps every destination. The Language section lets you choose System, Italiano or English.*

## Formats and conversions

| File type | Available conversions | Required engine |
| --- | --- | --- |
| Audio | MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A, WMA, AC3, AU, CAF | FFmpeg |
| Video | MP4, MKV, MOV, WebM, AVI, animated GIF, MPEG, WMV, FLV, TS, M4V, 3GP; audio extraction to the formats above | FFmpeg |
| Images | PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR, PNM; PDF; HEIC and AVIF; TXT with OCR | None for native images and PDF; libheif for HEIC/AVIF; Tesseract for TXT |
| PDF | PNG/JPEG per page; TXT, DOCX and HTML from text; compression, splitting and merging | Poppler for images and text; Tesseract + Poppler for OCR; none for compression, splitting and merging of existing PDFs |
| DOC/DOCX, ODT, RTF, TXT, PPT/PPTX, ODP | PDF | LibreOffice |
| XLS/XLSX/XLSM, ODS, CSV | CSV, Excel XLSX, PDF | LibreOffice |
| HTML/HTM | PDF | LibreOffice |

Adjust audio bitrate, sample rate, channels and normalization; video H.264/H.265 codec, quality, width, FPS and trimming; image resizing, JPEG quality and PDF DPI. The app shows the relevant controls for each format. Available audio/video codecs depend on the configured FFmpeg build.

The [complete conversion guide](docs/conversions.en.md) lists input extensions, codecs, options and the limits of each family.

### Compressing, splitting and merging PDFs

For a PDF, select **PDF** and the desired operation in its options:

- **Lossless compression** creates `name-compresso.pdf` without resampling images. If rewriting would produce a larger file, the app keeps a copy of the original bytes: the result may be the same size as the source.
- **Split: one page per file** creates `name-pagina-001.pdf`, `name-pagina-002.pdf` and so on.
- **PDF (Merge) · arrange pages** prepares the sources and opens the editor. Move pages using the handle, arrows or **Alt+↑ / Alt+↓**, then choose **Create merged PDF**.

![PDF Merge editor in English: four pages from two images and a two-page PDF, with Illustration.png placed between the PDF pages](docs/screenshots/en-merge-pdf.jpg)

*Four pages from Cover.png, Document.pdf and Illustration.png. The illustration is placed between the PDF's two pages; every row identifies the file and original page number.*

The merged result is named after the first source with the `-unito.pdf` suffix, with a number added if needed. Every page is included once, up to 10,000 pages; check the order in the editor. The originals are unchanged and intermediate PDFs are prepared in a temporary user folder. Output suffixes remain the same in both interface languages.

These operations on existing PDFs and merging native images work without external engines. Office/CSV/HTML sources require LibreOffice; HEIC/AVIF require libheif. Encrypted or signed PDFs are rejected; merging and splitting also reject interactive forms and do not transfer document bookmarks, tags or metadata.

### Extracting text and using OCR

For an image, choose **TXT · Local OCR**. For a PDF, choose **TXT**, **Word DOCX** or **HTML**: embedded text is normally extracted; enable **Recognize scanned pages with local OCR** for a scanned document.

OCR uses **Tesseract** with installed language models; PDFs also require **Poppler**. The default recognition language is **Italian + English** (`ita+eng`), but you can select other languages or enter your models' codes. For scanned PDFs, 300 DPI is a useful starting point.

Recognized text stays on your PC. Review it for errors: OCR does not recreate the layout or add a searchable layer to the original PDF. PDF-to-DOCX/HTML exports reconstruct text and page separation without original images, tables or formatting.

### Excel, CSV, HEIC and AVIF

With LibreOffice, convert spreadsheets to XLSX, CSV or PDF. For CSV, choose **comma, semicolon, tab or pipe**: the same separator is used for import and export. CSV is UTF-8; export keeps the displayed values of the **first sheet**, without formulas, formatting or other sheets. Import keeps quoted fields as text and does not evaluate formulas.

Configure `heif-dec.exe` to read HEIC/HEIF and AVIF; configure `heif-enc.exe` to create them. Both belong to libheif and must be kept with compatible DLLs and plugins. Width and quality are adjustable.

GIF, TIFF and image containers with multiple frames or pages are read from the first image. Video-to-GIF conversion preserves animation. WebP is encoded losslessly; JPEG quality does not affect lossless formats. [All format details and limitations](docs/conversions.en.md).

## Optional engines

Open **Settings → Conversion engines** to detect installed tools or choose their paths.

| Engine | Purpose | Configuration |
| --- | --- | --- |
| **FFmpeg** | Audio, video, audio extraction and GIFs from video | Download in the app or select `ffmpeg.exe` |
| **Poppler** | PDF images, embedded text and page preparation for OCR | Download in the app or select `pdftoppm.exe`, `pdftotext.exe` and `pdfinfo.exe` |
| **LibreOffice** | Office documents, Excel/CSV and HTML | Local installation and `soffice.exe` path |
| **Tesseract** | OCR | `tesseract.exe` path and `tessdata` folder with the required models |
| **libheif** | Reading and creating HEIC/HEIF and AVIF | `heif-dec.exe` and `heif-enc.exe` paths, with DLLs and plugins |

**Download FFmpeg and Poppler directly from the app**, including the portable edition. Before confirmation, you see the source, version, size, licenses and installation folder. In 0.5.0, both archives require about **159 MB of downloads** and about **615 MB of free space** during download and extraction. SHA-256 checksums are verified and licenses and documentation are retained.

![English confirmation dialog for optional FFmpeg and Poppler downloads, showing versions, sizes, licenses and the personal installation folder](docs/screenshots/en-download-engines.jpg)

*Downloading starts only after confirmation. This capture shows the confirmation dialog, which was cancelled without downloading engines. Engines use your user folder; files to convert stay on your PC.*

Configure LibreOffice, libheif, Tesseract and OCR models from local installations. See the [configuration guide](docs/configuration.en.md) for paths, preferences, manual scripts and queue recovery.

## Languages

The app and File Explorer menu support **Italian and English** from version 0.5.0. In **Settings → Language**, choose **System**, **Italiano** or **English**. System follows the Windows display language: Italian is used for Italian Windows, English for other languages. The choice is saved separately from OCR settings.

The installer supports Italian and English. The CLI uses stable English option names; `--language system|it|en` selects the language for that invocation without changing saved preferences. Messages from Windows and external engines may follow their own language settings.

The installer and portable packages in [GitHub release 0.5.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.5.0) include the bilingual interface ready to use.

**OCR supports the languages of the installed Tesseract models**: Italian and English are the default; the app also offers French, German, Spanish and a field for other codes. Recognition language is independent of interface language.

## Data and privacy

Conversion and OCR run locally. Preferences, queue records and reports are stored on the device and may contain paths and errors. There is no telemetry, advertising or app account registration.

Optional engine downloads require Internet access and contact the distributors; opening HTML may load linked images or CSS, including remote resources, through LibreOffice. The app preserves originals and does not send files to a developer server. [Privacy statement (Italian and English)](docs/privacy.md).

## Command line

The GitHub distribution includes `the-converter-cli.exe`. For example, from its folder:

```powershell
.\the-converter-cli.exe --language en --diagnose
.\the-converter-cli.exe --language en --convert --target mp3 --preset quality --files "C:\Video\clip.mp4"
.\the-converter-cli.exe --language en --convert --target docx --ocr --ocr-language ita+eng --files "C:\Documents\scan.pdf"
.\the-converter-cli.exe --language en --convert --target pdf --pdf-action merge --files "C:\Documents\first.pdf" "C:\Documents\second.pdf"
```

The CLI continues with other files after an error and exits with a nonzero code if any conversion fails. **Ctrl+C** cancels the process. [Complete CLI guide, options and JSON examples](docs/command-line.en.md).

## Development and documentation

Building requires Windows x64, Rust stable MSVC and Visual Studio C++ Build Tools with the Windows SDK:

```powershell
cargo build --release --locked
cargo build --release --locked --manifest-path .\shell-extension\Cargo.toml
```

The [developer guide](docs/development.en.md) describes tests, distribution builds, the installer and the source archive with dependencies for offline builds.

- [Conversions, options and limitations](docs/conversions.en.md)
- [Configuration, engines and local data](docs/configuration.en.md)
- [Quick queue and placeholders](docs/queued-conversions.en.md)
- [File Explorer integration](docs/windows-integration.en.md)
- [Screenshot sources and versions (Italian)](docs/screenshots/README.md)

## Author and license

**Salvatore Corsi** · [GitHub profile](https://github.com/salvatorecorsi) · [Support](mailto:salvo@salvatorecorsi.com)

Copyright (C) 2026 Salvatore Corsi. The original code is licensed under **GNU GPL version 3 only** (`GPL-3.0-only`), without warranty under the terms of the [complete license](LICENSE). The project continues the work of [Final Converter](https://github.com/salvatorecorsi/final-converter).

Sources are public; the release includes `ConvertHere-0.5.0-source.zip` with both lockfiles, vendored Cargo dependencies and offline build instructions. Conversion does not change the rights to your content or impose the GPL on results solely through use of the program. Dependencies, fonts and engines retain their own licenses: [third-party notices](THIRD-PARTY-NOTICES.md) and [license texts](docs/release-licenses/README.md).
