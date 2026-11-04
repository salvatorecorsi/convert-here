# Microsoft Store listing — English (en-US)

Draft for Convert Here **0.3.2**, Store package **1.0.2.0**: the new candidate is in preparation and has not been submitted. Intended publisher: **Salvatore Corsi**; the Microsoft account display-name change still needs verification. Developer: **Salvatore Corsi**. The owner confirmed reservation of **Convert Here** in Partner Center on 4 November 2026. This listing is translated; the application UI is currently Italian. Do not copy this preamble into public listing fields.

## Short description

Convert files from the Windows 11 context menu with a queue, custom settings and local OCR. External engines enable more formats. App UI is Italian.

## Description

Native image formats and image-to-PDF conversion work immediately, as do PDF compression, splitting and merging. Audio/video requires FFmpeg; PDF export requires Poppler; Office, Excel/CSV and HTML-to-PDF requires LibreOffice. HEIC/HEIF and AVIF require libheif; OCR requires Tesseract and language models, plus Poppler for scanned PDFs. These optional engines are not included. The application UI is Italian.

Convert Here brings conversion to the folder where you work. In the Windows 11 context menu, choose Converti in and a compatible format. The job joins a queue with your saved settings, without opening the options window every time.

A placeholder with a pending icon immediately shows where the result will appear. Open it to check progress or cancel the job. When conversion finishes, the completed file replaces the placeholder. Outputs are saved beside their originals by default; numbered names preserve files that already exist.

There are 44 destination formats and 93 recognized input extensions across compatible categories. Native image conversions cover PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR and PNM, plus a separate A4 PDF for each source image. Adjust JPEG quality and image width.

With libheif, read HEIC/HEIF and AVIF files or create HEIC and AVIF from supported images. Configure heif-dec.exe and heif-enc.exe in Strumenti, keeping the build's DLLs and plugins together. Availability depends on installed HEVC/AV1 decoders and encoders. Choose output quality and width. Only the first image in a container is converted; image sequences and animations are not preserved.

With optional FFmpeg, convert audio to MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A with AAC, WMA, AC3, AU and CAF with ALAC; convert video to MP4, MKV, MOV, WebM, AVI, MPEG, WMV, FLV, MPEG-TS, M4V, 3GP and animated GIF; or extract its audio. Codec availability depends on the selected FFmpeg engine. Settings include quality presets, bitrate, sample rate, channels, normalization, video quality, width, frame rate and time trimming.

With Poppler, export PDF pages to PNG/JPEG, extract embedded text as TXT, or reconstruct it as editable Word DOCX and HTML files. DOCX and HTML exports contain text only: original images, tables and page layout are not reconstructed.

With Tesseract, recognize text in images as TXT, or enable local OCR for PDFs to create TXT, DOCX or HTML from scanned pages. PDF OCR is off by default and also uses Poppler. Select installed language models, such as ita+eng, and the PDF page DPI. Keep models in the engine's tessdata directory. Processing stays on your PC. Recognized text may contain errors and should be reviewed; OCR does not create a searchable PDF.

Compress PDFs losslessly or split them into one file per page. For multiple PDF-compatible files, PDF (Bulk) queues separate results; PDF (Merge) prepares supported images, PDFs, Word/Office, Excel/CSV and HTML to create one PDF. Drag individual pages to interleave sources, or use the arrows or Alt+up/down. PDFs and native images need no external merge engine; documents still require LibreOffice and HEIC/AVIF needs libheif. Audio and video are excluded. Compression does not resample images or produce a file larger than its source. Splitting and merging omit bookmarks, tags and document metadata, and reject interactive forms. These operations reject encrypted or signed PDFs; compression preserves the catalog and forms and does not remove metadata.

With LibreOffice, convert compatible Word documents, other documents, spreadsheets and presentations to PDF; export XLS/XLSX/XLSM/ODS to CSV; or create Excel XLSX files from CSV. CSV export contains only the first sheet, using UTF-8 and double quotes. It preserves displayed values without formatting or formulas. Choose commas, semicolons, tabs or pipes for both import and export; commas are the default. Input CSV files must use UTF-8. Expressions beginning with = are imported as text, and quoted fields preserve identifiers with leading zeros.

Convert simple HTML files to PDF with LibreOffice. Complex CSS may change the layout, and JavaScript is not executed. Resources linked from the HTML may access local files or the network during conversion.

Convert Here is free, with no subscriptions, in-app purchases or usage-based charges. No payment is required to keep using it. Format conversion runs on your PC and does not require an in-app account. Preferences, file paths, queue status and error details are stored locally.

Limitations: free editing of PDF page content is not available. OCR recognizes text without reconstructing graphics or layout. Animated GIFs, multipage TIFFs and HEIC/AVIF containers use their first frame or image. PDF (Bulk) keeps separate results; PDF (Merge) includes every prepared page once, up to 10,000 pages. Office document layout depends on LibreOffice and available fonts. The modern context menu requires Windows 11 x64. The application UI is Italian.

## Product features

Enter each line in a separate feature field, without adding bullet characters:

```text
Converti in command directly in the Windows 11 File Explorer context menu
Quick conversions join a queue without opening the options window each time
Output-folder placeholders with job status available on demand
Images in PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR and PNM
Create a separate A4 PDF for each source image
Adjust JPEG quality, image width and output folder
Audio and video conversion with optional user-selected FFmpeg
PDF to images, TXT, text-only Word DOCX and HTML with optional Poppler
Excel to first-sheet CSV and UTF-8 CSV to XLSX, with four delimiters and optional LibreOffice
Word and simple HTML to PDF with optional LibreOffice; JavaScript is not executed
HEIC/HEIF and AVIF input and HEIC/AVIF output with optional libheif
Local OCR for images and scanned PDFs with optional Tesseract and language models
Separate PDF Bulk outputs or one PDF Merge with individual page drag and drop
Preserve original files and use numbered output names for existing files
Format conversion processed on your PC without an in-app account
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
Windows 11 x64 for the modern context menu
FFmpeg for audio/video; Poppler for PDF export; LibreOffice for Office/HTML to PDF and Excel/CSV conversion. Optional engines are not included. The application UI is Italian.
libheif with heif-dec.exe, heif-enc.exe, DLLs and plugins for HEIC/AVIF; Tesseract and language models for OCR, plus Poppler for PDFs. Not included.
```

## What's new

Leave blank for the first Microsoft Store submission. Add release-specific notes for later updates.

## Captions for screenshots to capture

1. Converti in in the Windows 11 context menu. App UI shown in Italian.
2. The queue shows file formats, status and destinations. App UI shown in Italian.
3. Image quality and resizing settings. App UI shown in Italian.
4. Configure optional executables on the Strumenti page. App UI shown in Italian.

## Submission references

Data controller and developer: **Salvatore Corsi**; intended publisher: **Salvatore Corsi**. Privacy and support contact: [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). [Public privacy statement](https://github.com/salvatorecorsi/convert-here/blob/master/docs/privacy.md). Reserved name: **Convert Here**. See the [submission procedure](store-submission.md) for licensing, identity and current status.

These texts are prepared for the new candidate and still need to be applied to the submission. They do not attest its upload or an update to the Partner Center listings.

Description: 10,000 characters maximum; short description: 1,000 maximum (this draft is below 270); up to 20 features of 200 characters each; release notes: 1,500 maximum, empty for the first submission. [Official MSIX listing limits](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info).
