# Conversions and formats

[Italiano](conversions.md) · **English**

This guide describes Convert Here's destinations, options and limitations. Start with the [README](../README.en.md); see [Configuration and engines](configuration.en.md) to set up external tools.

There are **44 output formats** and **93 recognized input extensions**. In **Settings**, choose which outputs appear in the app selector and Explorer menu, grouped into audio, video, images and documents. **MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4** are enabled by default; other destinations include TXT, DOCX, XLSX, HEIC and AVIF. Preferences are saved and the menu rereads them without restarting Explorer. **Custom settings…** remains available even when every output is hidden. The display filter does not block the CLI, explicit `--quick` requests or existing queued jobs. Compatible destinations depend on the source category and the selected engines' codecs.

## Supported inputs and outputs

| Source | Destinations | Engine |
| --- | --- | --- |
| Audio: MP3, MPGA, WAV, FLAC, AAC, M4A/M4B, OGG/OGA, Opus, AIFF/AIF/AIFC, WMA, ALAC, APE, MKA, MP2, WebA, AC3, AU, CAF, AMR, DTS, TTA, VOC, WV, TAK, W64, RA | MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A (AAC), WMA, AC3, AU, CAF (ALAC) | FFmpeg |
| Video: MP4, MKV, MOV, WebM, AVI, WMV, MPEG/MPG, M4V, TS/MTS/M2TS/M2T, 3GP/3G2, FLV, ASF, VOB, OGV, M2V, MXF, F4V, DivX, MPE, MPV | MP4, MKV, MOV, WebM, AVI, animated GIF, MPEG, WMV, FLV, TS, M4V, 3GP; audio extraction to the twelve audio formats above | FFmpeg |
| Native images: PNG, JPEG/JPG, WebP, BMP, TIFF/TIF, GIF, ICO, QOI, TGA, Farbfeld (.ff), OpenEXR, HDR, PNM/PAM/PBM/PGM/PPM | PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, PDF, QOI, TGA, Farbfeld, OpenEXR, HDR, PNM; HEIC, AVIF; TXT with OCR | Rust for native formats and PDF; libheif to create HEIC/AVIF; Tesseract for TXT |
| HEIC/HEIF, AVIF | Image destinations above, PDF and TXT with OCR | libheif for reading; Tesseract for TXT |
| PDF | PNG/JPEG, one image per page; TXT, Word DOCX and HTML from embedded text or OCR | Poppler for images and embedded text; Tesseract + Poppler for OCR; the app reconstructs DOCX and HTML |
| PDF | Losslessly compressed PDF, one PDF per page, merging multiple PDFs | Rust, without external engines |
| DOC/DOCX, ODT, RTF, TXT, PPT/PPTX, ODP | PDF | LibreOffice |
| XLS/XLSX/XLSM, ODS, CSV | CSV, Excel XLSX, PDF | LibreOffice |
| HTML/HTM | PDF | LibreOffice |

The extension determines the suggested category; the engine checks the content during conversion. Protected or damaged files and unavailable codecs produce a readable error.

## Quality and format options

Options include compact/balanced/high-quality presets; audio bitrate, sample rate, channels and normalization; H.264/H.265, video quality, width, FPS and trimming; JPEG quality, image resizing and PDF DPI. Only relevant controls are shown.

WebM uses VP9/Opus; AVI uses MPEG-4/MP3; MPEG uses MPEG-2/MP2; WMV uses WMV2/WMA2; FLV uses FLV1/MP3; 3GP uses MPEG-4/AAC; TS uses H.264/AAC. M4V is an MP4 container with audio. WAV, AIFF and AU use 16-bit PCM; CAF uses lossless ALAC. PDFs created from images use A4 pages, preserve proportions and have a white background.

## Image editing and page layout — 0.6.0 candidate

The editor adds crop, resize, canvas and adjustable JPEG/WebP quality, with preview and the actual prepared byte size. It exports PNG/JPEG/WebP/PDF. JPEG/WebP can search for a quality within a size limit; further downsizing requires explicit permission. An impossible limit prevents saving and makes quick conversion fail without publishing an oversized file.

Images, UTF-8 TXT and typed text can be laid out natively on A4/A5 with custom margins and orientation. This TXT path is enabled by page settings; traditional TXT conversion and other Office documents retain LibreOffice. The editor does not edit the full contents of PDF or Word documents. [Editor and profiles guide](image-editor-profiles.en.md).

## Excel, CSV, Word and HTML

Excel and CSV conversions use a separate temporary LibreOffice profile that disables macros and Office link updates. CSV export writes the **first sheet**, in UTF-8 with double-quote text delimiters; it preserves displayed values without formulas, formatting or other sheets. Choose **comma, semicolon, tab or pipe**; the same setting applies to import and export. Comma is the default. Import preserves quoted fields as text, including leading zeroes in identifiers, and does not evaluate formulas. Convert other encodings to UTF-8 first; the separator is not guessed automatically.

PDF-to-Word/HTML reconstructs **text only**, keeping page separation in one editable file. Embedded text is used by default; enable local OCR to recognize scanned pages. Original images, tables, fonts and layout are not reconstructed. Exported HTML contains escaped text and inline styles, with no external resources.

HTML-to-PDF uses LibreOffice's HTML document engine: it supports basic HTML/CSS and does not execute JavaScript. It does not replace printing a dynamic page from a browser; layout may change. The engine may read linked local or remote resources, such as images and CSS, while opening HTML.

## HEIC and AVIF images

Read HEIC/HEIF and AVIF, convert them to native image formats or PDF, and create HEIC or AVIF from supported images. In **Settings → Conversion engines**, select `heif-dec.exe` for reading and `heif-enc.exe` for creating these formats. They are optional [libheif](https://github.com/strukturag/libheif) tools, not bundled with the app: keep their executables, DLLs and plugins together. The build must include the required HEVC/AV1 decoders and encoders; missing codecs are reported by the engine. Width and quality are adjustable. Only the first image of a multi-image container is used; sequences and animations are not preserved.

Other image formats are native and do not require FFmpeg. HDR does not preserve transparency; transparent areas are composited on white. OpenEXR and HDR can retain floating-point values; ordinary image formats may reduce their precision and dynamic range. JPEG XL, photographic RAW and DDS are not currently included. Audio/video codec availability depends on the selected FFmpeg build.

## Local OCR for images and scanned PDFs

For an image, choose **TXT · Local OCR**. For a PDF, select TXT, Word DOCX or HTML and enable **Recognize scanned pages with local OCR**: PDF OCR is off by default. When enabled, it processes every page instead of extracting embedded text. Install [Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html) and the [language models](https://github.com/tesseract-ocr/tessdata_fast); PDFs also require Poppler's `pdftoppm.exe`. In **Settings → Conversion engines**, select `tesseract.exe` and keep the `tessdata` folder beside it with the required models, such as `ita.traineddata` and `eng.traineddata`.

The default OCR language is `ita+eng`; other installed Tesseract codes can be selected. This does not change the interface language. PDF DPI sets the resolution of pages sent for recognition; 300 DPI is a useful starting point. Images are oriented using EXIF and transparency is composited on white. Recognition and the resulting text stay on your PC. Review the result for errors: OCR does not reconstruct layout or add a searchable layer to the original PDF. If no text is recognized, the app reports an error rather than creating an empty result.

## Compress, split and merge PDFs

For existing PDFs, **PDF (Bulk)** compresses streams losslessly without resampling images and creates `name-compresso.pdf`. If rewriting would increase the size, it saves a copy of the original bytes. The result cannot exceed the size of the file read and may remain the same size. **Split: one page per file** produces `name-pagina-001.pdf`, `name-pagina-002.pdf` and so on. Filename suffixes are stable across interface languages.

To merge compatible files, choose **PDF (Merge)** in the context menu, or **PDF (Merge) · arrange pages** in the app and start **Arrange and merge PDF**. The app prepares the sources and lists every page with its source filename and original page number. Drag a page's handle, use the arrows or press **Alt+↑ / Alt+↓** to move it. Interleave pages from multipage PDFs, images and different documents. The initial order follows the files received from Explorer; check it in the editor. **Create merged PDF** saves a file named after the first source with the `-unito.pdf` suffix, adding a number if necessary. Every page is included once, up to 10,000 pages.

Preparation uses temporary PDF copies in a hidden user folder, with no intermediate PDFs in the destination. Originals remain unchanged; saving uses the prepared version of the pages. Page order belongs to a local session separate from the quick queue. Cancellation and closing attempt to remove intermediates; an unexpected shutdown may leave residual files.

Compressing, splitting and merging existing PDFs, and merging native images, do not require external engines. Preparing Office/Excel/CSV/HTML requires LibreOffice; reading HEIC/AVIF requires libheif. Poppler is not needed for merging. Encrypted or signed PDFs are rejected; merging and splitting also reject interactive forms. Page content is preserved, while document bookmarks, accessibility structure/tags and metadata are not transferred. Compression preserves the catalog and forms; it does not remove metadata. Free-form editing of page content is not available.

## Text from images and PDFs

Images offer TXT through local OCR; PDFs offer TXT through embedded-text extraction or local OCR. Select the required engines in **Settings → Conversion engines**. Recognition and output stay on the PC. See the bilingual [privacy statement](privacy.md) for local data and engine details.

## Explicit limitations

- OCR requires Tesseract and language models; PDF OCR also needs Poppler. It recognizes text without reconstructing graphics or layout.
- Animated GIFs and multipage TIFFs are read from the first frame/page. Video-to-GIF conversion preserves animation.
- Ordinary WebP conversion is lossless; the 0.6.0 candidate editor adds adjustable WebP quality. JPEG quality does not affect lossless formats.
- PDF (Bulk) creates one PDF per image; PDF (Merge) can combine images and other compatible sources while arranging individual pages. Merging and splitting omit document bookmarks, tags and metadata and reject interactive forms. Compression, splitting and merging reject encrypted or signed PDFs. PDF-to-PNG/JPEG accepts an owner-only password; documents requiring an opening password must be unlocked first.
- Office documents, spreadsheets, CSV and HTML-to-PDF need a configured LibreOffice installation; layout also depends on installed fonts.
- PDF-to-DOCX/HTML reconstructs text without original images or layout. Excel-to-CSV exports only the first sheet and loses formulas and formatting.
- The interactive queue uses one output format common to all selected files. Convert categories separately when they have no common destination.
- Quick conversions run in a background queue. The status window opens on request from the placeholder; the main window shows the complete queue. The report retains the last quick run; closing the status window does not stop conversion.

[Back to README](../README.en.md)
