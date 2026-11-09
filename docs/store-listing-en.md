# Microsoft Store listing — English (en-US)

Draft for Convert Here **0.7.0**, candidate Store package **1.0.6.0**. Publisher and developer: **Salvatore Corsi**. Copy is ready with 20 features; screenshots, versioned URLs and upload must be verified against the final candidate. This document does not attest publication, submission or approval of 0.7.0. Earlier release evidence remains historical. The app supports Italian and English. Do not copy this preamble into public listing fields.

## Short description

Local conversion for Windows 10/11, an image editor, A4/A5 pages and audio/video/PDF profiles. Optional engines add formats and OCR. Italian and English UI.

## Description

Native image formats and image-to-PDF conversion work immediately, as do PDF compression, splitting and merging. Audio/video requires FFmpeg; PDF export requires Poppler; Office, Excel/CSV and HTML-to-PDF requires LibreOffice. HEIC/HEIF and AVIF require libheif; OCR requires Tesseract and language models, plus Poppler for scanned PDFs. These optional engines are not included. The interface and menu support Italian and English. Settings > Conversion engines lets you request FFmpeg, Poppler or both after confirming the source, version, size, licenses and folder. Internet is needed for this optional download; conversion and OCR remain local.

Convert Here brings conversion to the folder where you work, on Windows 10 version 2004 or later (build 19041), x64, and Windows 11. In the Windows 10 classic or Windows 11 modern context menu, choose Convert to and a compatible format. The job joins a queue with your saved settings, without opening the options window every time.

A placeholder with a pending icon immediately shows where the result will appear. Open it to check progress or cancel the job. When conversion finishes, the completed file replaces the placeholder. Outputs are saved beside their originals by default; numbered names preserve files that already exist.

There are 44 destination formats and 93 recognized input extensions across compatible categories. Native image conversions cover PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR and PNM, plus a separate A4 PDF for each source image. Adjust JPEG quality and image width. Choose which outputs appear in the GUI and Explorer menu in Settings: defaults are MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Other outputs can be enabled; all 93 input extensions remain recognized, and the menu refreshes without restarting Explorer.

Open Edit… beside an image to crop, resize or create a canvas with a custom resolution, background, position and anchor. Export PNG, JPEG, WebP or PDF with a preview of the prepared result and its actual encoded byte size. JPEG and WebP offer adjustable quality: a size limit searches for a suitable quality; 300 kB is 300000 bytes. Further dimension reduction requires explicit permission. For PDFs, adjust quality, dimensions and DPI to change the size. If the limit cannot be met, saving is disabled and quick conversion does not publish an oversized file. Save as writes those same prepared bytes and preserves existing files.

Lay out images, text written in New sheet… or UTF-8 TXT files on A4/A5 pages in portrait or landscape with custom margins. Plain text supports font size, line spacing, alignment and automatic pages, with embedded Ubuntu Light and selectable PDF text. Unsupported characters produce an explicit error. This native path does not require LibreOffice; Word documents keep the existing Office conversion workflow.

Save my settings creates named profiles: up to 32 saved, with 8 visible in Convert to. Reuse relative crop, dimensions, canvas, quality, byte limits and page settings without storing the sample image or typed text. The Profiles page lets you apply and update recipes. Menu profiles appear only when enabled, their output format is active and selected inputs are compatible. Queue jobs retain the settings with which they were created.

The Profiles manager has Audio, Video, Images and Documents categories. Create, edit, duplicate, rename or delete recipes, choose a default, import/export JSON and restrict the input extensions of each profile. Relevant settings include bitrate, sample rate, channels, codecs, trimming, page layout and PDF operations. A PDF Merge profile opens the page-order editor; existing queued jobs keep their original recipe snapshot even after the profile is updated or deleted.

Set a maximum size per file and a maximum audio/video duration, combined with the shorter trim interval. Actual size is checked before publication; split PDFs apply the cap to each page. Audio, video and PDF do not promise automatic size fitting: an oversized result is not published. JPEG/WebP retain adaptive quality and opt-in downsizing.

Advanced exposes typed FFmpeg controls for encoders, compatible presets, color, GOP, bitrate, audio samples, numeric filters and metadata. Numeric, boolean and enumerated options are read from installed local encoders and verified before conversion. This is not a free-form command line. FFmpeg options apply to audio/video sources, including video-to-GIF; they do not apply to native image conversions.

With libheif, read HEIC/HEIF and AVIF files or create HEIC and AVIF from supported images. Configure heif-dec.exe and heif-enc.exe in Settings > Conversion engines, keeping the build's DLLs and plugins together. Availability depends on installed HEVC/AV1 decoders and encoders. Choose output quality and width. Only the first image in a container is converted; image sequences and animations are not preserved.

With optional FFmpeg, convert audio to MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A with AAC, WMA, AC3, AU and CAF with ALAC; convert video to MP4, MKV, MOV, WebM, AVI, MPEG, WMV, FLV, MPEG-TS, M4V, 3GP and animated GIF; or extract its audio. Codec availability depends on the selected FFmpeg engine. Settings include quality presets, bitrate, sample rate, channels, normalization, video quality, width, frame rate and time trimming.

With Poppler, export PDF pages to PNG/JPEG, extract embedded text as TXT, or reconstruct it as editable Word DOCX and HTML files. DOCX and HTML exports contain text only: original images, tables and page layout are not reconstructed.

With Tesseract, recognize text in images as TXT, or enable local OCR for PDFs to create TXT, DOCX or HTML from scanned pages. PDF OCR is off by default and also uses Poppler. Select installed language models, such as ita+eng, and the PDF page DPI. Keep models in the engine's tessdata directory. Processing stays on your PC. Recognized text may contain errors and should be reviewed; OCR does not create a searchable PDF.

Compress PDFs losslessly or split them into one file per page. For multiple PDF-compatible files, PDF (Bulk) queues separate results; PDF (Merge) prepares supported images, PDFs, Word/Office, Excel/CSV and HTML to create one PDF. Drag individual pages to interleave sources, or use the arrows or Alt+up/down. PDFs and native images need no external merge engine; documents still require LibreOffice and HEIC/AVIF needs libheif. Audio and video are excluded. Compression does not resample images or produce a file larger than its source. Splitting and merging omit bookmarks, tags and document metadata, and reject interactive forms. These operations reject encrypted or signed PDFs; compression preserves the catalog and forms and does not remove metadata.

With LibreOffice, convert compatible Word documents, other documents, spreadsheets and presentations to PDF; export XLS/XLSX/XLSM/ODS to CSV; or create Excel XLSX files from CSV. CSV export contains only the first sheet, using UTF-8 and double quotes. It preserves displayed values without formatting or formulas. Choose commas, semicolons, tabs or pipes for both import and export; commas are the default. Input CSV files must use UTF-8. Expressions beginning with = are imported as text, and quoted fields preserve identifiers with leading zeros.

Convert simple HTML files to PDF with LibreOffice. Complex CSS may change the layout, and JavaScript is not executed. Resources linked from the HTML may access local files or the network during conversion.

Convert Here is free, with no subscriptions, in-app purchases or usage-based charges. No payment is required to keep using it. Format conversion runs on your PC and does not require an in-app account. Preferences, file paths, queue status and error details are stored locally.

The project's original code is free software licensed under GPL-3.0-only; source code is available at https://github.com/salvatorecorsi/convert-here. Dependencies, fonts and engines retain their own licenses. Conversion does not change rights in the contents of your files.

Limitations: free editing of PDF page content is not available. OCR recognizes text without reconstructing graphics or layout. Animated GIFs, multipage TIFFs and HEIC/AVIF containers use their first frame or image. PDF (Bulk) keeps separate results; PDF (Merge) includes every prepared page once, up to 10,000 pages. Office document layout depends on LibreOffice and available fonts. The modern context menu requires Windows 11 x64. Interface language can be selected separately from OCR language.

## Product features

Enter each line in a separate feature field, without adding bullet characters:

```text
Convert to in the Windows 10 classic and Windows 11 modern menus, from build 19041 x64
Quick queue with output placeholders and job status on demand
44 outputs and 93 input extensions, with eight visible outputs by default
Native images, optional libheif for HEIC/AVIF and optional FFmpeg for audio/video
Image editor with crop, resize, canvas, background and anchor
Preview and exact size for PNG/JPEG/WebP/PDF; save the same prepared bytes
Images, typed text and UTF-8 TXT on A4/A5 PDF pages with an embedded font
Audio, Video, Images and Documents profile manager: 32 saved, 8 menu shortcuts
Import/export profile JSON and select compatible input extensions
Maximum size per file and maximum audio/video duration combined with trimming
Typed advanced FFmpeg controls with installed-encoder options checked in the core
Audio/video bitrate, codecs, sample rate, channels, normalization, FPS and trimming
PDF to images, TXT and text-only DOCX/HTML with optional Poppler
Excel/CSV with four delimiters and Word/HTML to PDF with optional LibreOffice
Local OCR with Tesseract and language models; scanned PDFs also need Poppler
Compress/split PDFs, separate PDF Bulk results or PDF Merge with draggable pages
Preserve originals and use numbered names when files already exist
Queued profiles keep their recipe after the profile is modified or deleted
Local processing; no account, subscription or payment required to keep using it
Optional FFmpeg/Poppler downloads with confirmation, SHA-256 and retained licenses
```

## Search terms

One term per field; no third-party product names:

```text
file converter
image conversion
image to PDF
context menu
audio conversion
video conversion
conversion queue
```

## Additional system requirements

```text
Windows 10 version 2004 (build 19041) or later, x64; classic menu on Windows 10 and modern menu on Windows 11
FFmpeg for audio/video; Poppler for PDF export; LibreOffice for Office/HTML to PDF and Excel/CSV conversion. Optional engines are not included. Optional FFmpeg/Poppler downloads from Settings > Conversion engines require Internet.
libheif with heif-dec.exe, heif-enc.exe, DLLs and plugins for HEIC/AVIF; Tesseract and language models for OCR, plus Poppler for PDFs. Not included.
```

## What's new

```text
New Audio, Video, Images and Documents profile manager: create, duplicate and exchange JSON recipes, select inputs and show up to 8 menu shortcuts. Added a per-file size cap, audio/video duration limit and advanced FFmpeg controls verified against local encoders. The Store package now targets Windows 10 build 19041 or later, x64, with the classic Windows 10 and modern Windows 11 menus. Image editing, A4/A5 text, 44 outputs and 93 input extensions remain available. Local and free; no payment is required to keep using it.
```

## Screenshot captions

1. Two local image-to-JPEG conversions completed. App UI shown in Italian.
2. Reorder individual pages in PDF Merge to create one document. App UI shown in Italian.
3. Choose visible outputs in Settings, with eight defaults and separate categories. App UI shown in Italian.
4. Confirm sources, versions, sizes and folder before downloading FFmpeg and Poppler. App UI shown in Italian.

Screenshots 1, 3 and 4 show 0.4.0; screenshot 2 reuses the unchanged PDF Merge UI from 0.3.1. [Screenshot paths](store-assets/README.md).

Native editor and profile screenshots remain to be captured for the 0.7.0 candidate. Match captions only after verifying the image: “Crop and adjust quality and size on the prepared result”; “Lay out text on A4 or A5”; “Save settings and reuse your profiles”. State the language actually shown in each screenshot. Do not attribute these new functions to historical screenshots.

## Copyright and additional license terms

Copyright:

```text
Copyright (C) 2026 Salvatore Corsi.
```

Planned Additional license terms URL, to be verified after the 0.7.0 tag is published and before saving it in the portal. It links to the complete GPLv3 text:

```text
https://github.com/salvatorecorsi/convert-here/blob/v0.7.0/LICENSE
```

The project uses GPL-3.0-only. The candidate plans `ConvertHere-0.7.0-source.zip` with source code, vendored Cargo dependencies and offline build instructions. These field-entry instructions are not part of the public description.

## Submission references

Data controller and developer: **Salvatore Corsi**; publisher: **Salvatore Corsi**. Privacy and support contact: [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). [Candidate 0.7.0 privacy statement](https://github.com/salvatorecorsi/convert-here/blob/v0.7.0/docs/privacy.md), to be verified after the tag is published. Reserved name: **Convert Here**. See the [submission procedure](store-submission.md) for licensing, identity and current status.

These are local 0.7.0 drafts, not fields already saved in the portal. Evidence for 0.4.0 and other earlier versions remains historical. [Candidate testing instructions](store-testing-0.7.0.md).

Description: 10,000 characters maximum; short description: 1,000 maximum (this draft is below 270); up to 20 features of 200 characters each; release notes: 1,500 maximum, empty for the first submission. [Official MSIX listing limits](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info).
