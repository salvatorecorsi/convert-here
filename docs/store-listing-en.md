# Microsoft Store listing — English (en-US)

**Convert Here 0.8.1 / Store 1.0.9.0**, developer and publisher **Salvatore Corsi**. English listing **saved and reread from the server** in **Submission 6**, ID **1152921505702367000**, with **20 features, four PNGs and four captions**. GitHub release published on 10 November 2026 at **13:42:06 UTC**, from clean commit `07fe5f749327b73e93a92d5101d80b9b4f5317cb`. MSIX uploaded, saved as Validated and submitted: observed state **Update in certification** at **13:44:28 UTC**, with automatic publication after approval. New 1.0.9.0 is not yet claimed approved or available; baseline **0.8.0 / Store 1.0.8.0 is published**. No new-MSIX installation, physical Windows 10 host test or WACK is attested. Do not copy this preamble into public fields.

## Short description

Local conversion for Windows 10/11, editors, profiles and PDFs. An engine setup wizard helps add OCR, audio/video and speech transcription. Italian/English UI.

## Description

Convert Here brings conversion to the folder where you work, on Windows 10 version 2004 or later (build 19041), x64, and Windows 11. Choose Convert to and a compatible format in the context menu to queue a job with saved settings. A placeholder shows where the result will appear and lets you check progress or cancel. Originals and existing outputs are preserved; numbered names resolve conflicts.

There are 44 outputs and 94 recognized input extensions. Default visible outputs remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Enable others in Settings > Output formats; TXT is under Raw. This filters outputs, and the menu refreshes without restarting Explorer. The interface and menu support Italian and English. Select all below each category enables or clears every output in that list. Individual extensions remain clickable: clearing one also clears the collective checkbox.

Native images, static SVG, image-to-PDF and PDF compression/splitting/merging work without external engines. Audio/video requires FFmpeg; PDF export requires Poppler; Office, Excel/CSV and HTML-to-PDF requires LibreOffice. HEIC/HEIF and AVIF use libheif. OCR uses Tesseract and language models, plus Poppler for scanned PDFs. Transcription uses FFmpeg/ffprobe and Whisper with multilingual Tiny.

The first normal main-window launch without files or a preselected target offers an engine setup wizard. Missing engines are preselected: FFmpeg, Poppler, Tesseract with Italian/English, libheif and Whisper with Tiny. Change the selection, skip the wizard or reopen it from Settings > Conversion engines > Setup wizard. LibreOffice links to its official website and remains a separate installation.

Before downloading, confirm sources, versions, sizes, licenses and destination. Selected engines download and install one at a time; Whisper includes FFmpeg if FFmpeg or ffprobe is missing. After cancellation or an error, completed engines are retained and remaining engines can be retried. Assets are checked with SHA-256 before extraction; original licenses and provenance remain in the user folder. Engines and models are neither bundled nor downloaded before confirmation. Internet is needed to acquire engines; files selected for conversion, OCR or transcription are not uploaded to the providers.

Convert images to PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR and PNM, or create PDFs. Integrated resvg rasterizes static UTF-8 SVG locally; vectors are not retained. External resources, scripts, DTDs, foreignObject and embedded images are rejected. SVGZ is unsupported, and size and complexity limits apply.

Open Edit to crop, resize or create a canvas with background, position and anchor. PNG, JPEG, WebP and PDF show the prepared output and its actual byte size; Save as writes those same bytes. JPEG/WebP can search for quality that meets a size cap; further downsizing requires explicit permission. 300 kB means 300000 bytes. An impossible cap does not publish an oversized result.

Lay out images, text written in New sheet and UTF-8 TXT on portrait or landscape A4/A5 pages with chosen margins. Plain text supports size, line spacing, alignment and automatic pages, with embedded Ubuntu Light and selectable PDF text. Unsupported characters produce an error. This path does not require LibreOffice.

The audio/video editor offers a waveform, video source frames, timeline handles and numeric trim times. Apply gain or mute to compatible media outputs; positive gain can clip. Previews show the source before effects. Playback and multitrack editing are not available. TXT output applies trimming without gain or mute.

Enable TXT under Raw to transcribe audio/video speech on your PC. The menu entry is TXT · Transcription. Choose automatic language detection or a supported language, and select the interval. Multilingual Tiny is 77.69 MB; Whisper plus model downloads total about 87.94 MB, excluding FFmpeg. The GUI includes FFmpeg if FFmpeg or ffprobe is missing. Temporary audio/text is normally removed on completion or cancellation; an abrupt shutdown can leave residual files. Recognition can contain errors and should be reviewed.

Install Tesseract with Italian and English using one button to recognize image text as TXT. Opt-in local PDF OCR can produce TXT, DOCX or HTML from scanned pages using Poppler too. PDF OCR is off by default; select installed languages and DPI. Recognition does not reconstruct graphics or layout or create searchable PDFs. Text may need correction.

The libheif button prepares decoders and encoders to read HEIC/HEIF and AVIF and create HEIC/AVIF. Available codecs depend on the selected build. Animated GIFs, multipage TIFFs and HEIC/AVIF containers use their first image; sequences and animations are not retained.

With FFmpeg, convert audio to MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A with AAC, WMA, AC3, AU and CAF with ALAC; video to MP4, MKV, MOV, WebM, AVI, MPEG, WMV, FLV, MPEG-TS, M4V, 3GP and animated GIF; or extract audio. Adjust bitrate, sample rate, channels, normalization, quality, width, FPS and interval. Advanced offers typed controls and local-encoder options checked before use, without arbitrary commands or scripts. FFmpeg controls apply to audio/video sources.

The Profiles manager creates, edits, duplicates, renames and deletes Audio, Video, Images and Documents recipes: up to 32 saved and 8 in the menu. Choose a default, allowed input extensions and visibility. Import/export JSON without overwriting existing recipes. Profiles store options rather than sample images or typed text. Queued jobs keep their recipe snapshot after profile changes or deletion.

Set a maximum size per output and maximum audio/video duration combined with trimming. Actual size is checked before publication; split PDFs apply the cap to each page. Audio/video and PDF do not promise automatic size fitting. History combines window, queue and editor jobs and retains outcomes across restarts without deleting originals or results.

Use Poppler to export PDF pages as PNG/JPEG, embedded text as TXT or text-only DOCX/HTML reconstructions; original images, tables and layout are not recreated. Compress PDFs losslessly, split by page, use PDF Bulk for separate outputs or PDF Merge for one file with draggable pages. Inputs must be PDF-compatible; audio/video is excluded. Compression does not produce a larger file than the original. Operations reject encrypted or signed PDFs; splitting/merging also reject interactive forms and omit bookmarks, tags and metadata. Free editing of PDF page content is not available.

With LibreOffice, convert Word and compatible documents to PDF, XLS/XLSX/XLSM/ODS to CSV and CSV to XLSX. CSV exports the first sheet's displayed values as UTF-8 without formatting or formulas. Choose commas, semicolons, tabs or pipes. Input CSV must be UTF-8; expressions starting with = are imported as text. HTML-to-PDF supports simple documents, with possible CSS differences and no JavaScript execution; linked resources may access files or the network.

Convert Here is free, without subscriptions, in-app purchases or payment to keep using it. No in-app account is required. Preferences, paths, queue state and error details are stored locally. Original code is GPL-3.0-only, with source at https://github.com/salvatorecorsi/convert-here. Dependencies, fonts, engines and models retain their licenses. The modern menu requires Windows 11 x64; Windows 10 uses the classic menu.

## Product features

Enter each line in a separate feature field, without bullet characters:

```text
Convert to in the Windows 10 classic and Windows 11 modern menus, from build 19041 x64
Unified history for work in progress, queued and completed
44 outputs and 94 inputs; Select all by category, eight default outputs and opt-in TXT under Raw
Native images and locally rasterized static SVG without external resources
Image editor with crop, resize, canvas and preview of the bytes to save
Images, typed text and UTF-8 TXT on A4/A5 PDF pages with an embedded font
Audio, Video, Images and Documents profiles: 32 saved, 8 menu shortcuts
Import/export profile JSON and select compatible input extensions
Maximum size per output and maximum audio/video duration combined with trimming
Audio/video editor with timeline, waveform, source frames, trimming and gain
Typed advanced FFmpeg controls with verified local-encoder options
Local audio/video-to-TXT transcription with optional Whisper and multilingual Tiny
One-button Tesseract OCR installation with Italian and English included
One-button libheif installation for reading and creating HEIC/AVIF
FFmpeg audio/video codecs, bitrate, sample rate, channels, normalization and FPS
PDF images/text with Poppler; compression, splitting and merging with draggable pages
Excel/CSV with four delimiters and Word/HTML to PDF with LibreOffice
Preserve originals and existing files; queued jobs retain their recipe
Local processing with no account or payment required to keep using it
First-launch engine setup wizard: sequential downloads, confirmation, SHA-256 and licenses retained
```

## Search terms

One term per field; no third-party product names:

```text
file converter
image conversion
image to PDF
context menu
audio conversion
local transcription
conversion queue
```

## Additional system requirements

```text
Windows 10 version 2004 (build 19041) or later, x64; classic menu on Windows 10 and modern menu on Windows 11.
Optional engines are not included: FFmpeg for audio/video, Poppler for PDF export, LibreOffice for Office/HTML and Excel/CSV, libheif for HEIC/AVIF, Tesseract for OCR; scanned PDFs also need Poppler.
Transcription requires FFmpeg/ffprobe, Whisper CPU and multilingual Tiny. Whisper plus model downloads are about 87.94 MB, plus FFmpeg. Settings downloads require confirmation and Internet; LibreOffice is separate.
```

## What's new

```text
A first-launch setup wizard lets you choose engines, review sources and sizes, then download them one at a time. Skip it or reopen it from Settings. Select all below each format category replaces the check/cross buttons; individual extensions stay editable. The eight default outputs are unchanged.
```

## Screenshot captions

1. Output formats with Select all below each category. Italian interface shown.
2. Engine setup wizard: choices, dependencies and sizes before confirmation. Italian interface shown.
3. Audio/video editor: timeline, interval and gain. Italian interface; reused 0.8.0 capture of the unchanged UI.
4. Local TXT transcription with language and trimming. Italian interface; reused 0.8.0 capture of the unchanged UI.

Gallery order saved and reread in Submission 6: [0.8.1 02-formati-it.png](store-assets/0.8.1/02-formati-it.png), [0.8.1 01-wizard-it.png](store-assets/0.8.1/01-wizard-it.png), [0.8.0 03-timeline-it.png](store-assets/0.8.0/03-timeline-it.png), [0.8.0 04-trascrizione-it.png](store-assets/0.8.0/04-trascrizione-it.png). All are **2562 × 1441 PNGs** from native debug builds at 150% UI zoom, without the pointer or later upscaling. The last two reuse the unchanged editor and retain their historical provenance. [0.8.1 capture record and limits](store-assets/0.8.1/README.md). The gallery is saved and reread in both listings; this does not attest approval or Store publication of new 1.0.9.0.

## Copyright and additional license terms

```text
Copyright (C) 2026 Salvatore Corsi.
```

The project is GPL-3.0-only. The [full text at public tag v0.8.1](https://github.com/salvatorecorsi/convert-here/blob/v0.8.1/LICENSE) is saved and verified in the license field. `ConvertHere-0.8.1-source.zip`, with vendored dependencies and offline instructions, is verified and published in the [GitHub release](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.1). Third parties retain original licenses and copyright.

## Submission references

Data controller, developer and publisher: **Salvatore Corsi**. Support/privacy: [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). [Current privacy statement](privacy.md), [engine pins and licenses](release-licenses/EXTERNAL-ENGINES.md), [0.8.1 testing](store-testing-0.8.1.en.md), [submission procedure and status](store-submission.md).

**0.8.1 / Store 1.0.9.0** listing saved and reread from the server in **Submission 6**, ID **1152921505702367000**, with **20 features and four images**. Privacy and license point to public tag **v0.8.1**, with saving verified. GitHub release is published; Store submission was sent and shows **Update in certification**, with automatic publication after approval. Preceding **1.0.8.0 is published**; new Store 1.0.9.0 availability is not attested. Description: 10,000 characters maximum; short description: 1,000; features: 200 characters each; release notes: 1,500. [MSIX field limits](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info).
