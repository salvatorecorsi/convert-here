# Convert Here 0.6.0 — image editing, page layout and saved profiles

Candidate release documentation for application **0.6.0** and Store package **1.0.5.0**. Publication and Store submission are pending. Publisher and developer: **Salvatore Corsi**.

The candidate retains the bilingual interface and menu introduced in 0.5.0: select Italian or English in **Settings → Language**, independently from OCR language. The [0.5.0 release](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.5.0) and its artifacts remain unchanged.

Open **Edit…** beside an image to crop, resize or place it on a custom pixel canvas. Choose its background, anchor and position, then export PNG, JPEG, WebP or PDF. The editor shows the prepared result and its actual encoded byte size; saving writes those same bytes without overwriting an existing file or the original.

JPEG and WebP have adjustable quality. An optional size limit uses decimal units: **300 kB = 300,000 bytes**. Reducing dimensions to meet the limit requires explicit permission. If the requested limit cannot be met, saving is disabled and quick conversion fails without creating an oversized result. WebP encoding runs locally with the built-in codec.

Create **A4 or A5** PDFs from an image, a UTF-8 TXT file or text written directly in **New sheet…**. Choose portrait or landscape, margins, image fit and DPI, or text size, line spacing and alignment. Text wraps across pages and remains selectable through the embedded Ubuntu Light font. Unsupported characters produce an explicit error. This plain-text layout tool does not edit Word documents or the complete contents of an existing PDF; Office conversion retains its existing LibreOffice workflow.

Save up to **32 named profiles**, with up to **8 visible in Convert to** in File Explorer. Profiles retain conversion parameters, their menu visibility and an optional output folder; they do not contain the sample image, typed text or previews. Menu entries depend on the enabled output format and compatible selected inputs. Quick jobs keep the profile settings with which they were queued. The **Save for web** example provides a WebP recipe with a 300 kB limit and explicitly permitted dimension reduction.

The engine still supports **44 destination formats** and **93 input extensions**. The default visible outputs remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Editing, page layout, conversion and OCR run on your PC. Optional FFmpeg, Poppler, LibreOffice, libheif and Tesseract requirements remain applicable to their respective formats. Requested FFmpeg/Poppler downloads use HTTPS, confirm their source and destination, verify SHA-256 and preserve upstream licenses; downloading requires Internet and does not upload files to convert. Linked HTML resources may also access the network.

Convert Here is free, with no subscriptions, in-app purchases or usage-based charges. **No payment is required to keep using it.** The original code is licensed under **GPL-3.0-only**; dependencies, fonts and engines retain their own licenses. Conversion does not change rights in your files.

Planned distribution files:

- `ConvertHere-0.6.0-windows-x64-setup.exe`
- `ConvertHere-0.6.0-windows-x64-portable.zip`
- `ConvertHere-0.6.0-source.zip`, including source, vendored Cargo dependencies and offline build instructions
- `SHA256SUMS.txt`

Details: [editor and profiles guide](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/image-editor-profiles.en.md), [README](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/README.en.md), [privacy](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/privacy.md), [GPLv3 license](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/LICENSE), and [validation status](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/store-validation.md). Earlier release tags and artifacts remain unchanged.
