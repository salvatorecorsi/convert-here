# Image editor, pages and profiles

[Italiano](image-editor-profiles.md) · **English**

Available from **Convert Here 0.6.0**.

## Open the editor

Add an image and choose **Edit…** in its row, or **Edit image** in its options. For a TXT file, **Edit…** opens plain-text layout, also available as **Lay out text**. **New sheet…** lets you write text directly in the app. Image editing exports PNG, JPEG, WebP and PDF; the image output selector respects formats enabled in Settings. Interface language is independent of these settings.

The original is preserved. **Save as…** writes the prepared result and refuses an existing destination, including the original or a hard link to it. Choose a free output name. An earlier preview may remain visible while options change; wait for the new result before saving.

## Crop, resize and canvas

- Select an area by dragging in the preview or entering crop percentages. Reset to the whole image when needed. Profiles reuse a relative crop after image orientation.
- Resize to a width and/or height, optionally preserving aspect ratio.
- Choose canvas dimensions in pixels, background, anchor and offsets. Keep the image's dimensions, fit it inside the canvas or fill the canvas with cropping.

Animated images and multi-image containers use the first frame or image. HEIC/HEIF and AVIF still need the configured local libheif decoder. Transparency is composited onto white for JPEG and images on PDF pages; PNG and WebP can retain alpha.

## Quality, exact size and preview

JPEG and WebP have adjustable quality; WebP is encoded locally by the built-in codec. PNG preserves encoded pixels losslessly and does not use JPEG quality. Fit the full preview, zoom it or inspect the available 1:1 detail view, with one image pixel per screen pixel.

The displayed size is calculated from actual encoded bytes. Saving reuses those same bytes. Lossy image previews are decoded from the compressed result; PDF previews use the prepared geometry, embedded font and JPEG image.

The maximum-size option uses decimal kB: **300 kB = 300000 bytes**. For JPEG and WebP the engine searches for a quality within the selected limit. Further dimension reduction is opt-in. A smaller PNG may require this reduction because its compression is lossless. For PDFs, adjust image quality, dimensions and DPI to change size.

Some limits cannot be met. Saving then remains disabled; a quick job using the same profile fails without publishing an oversized result. Increase the limit or change dimensions, quality or content.

## A4/A5 images and plain text

For PDF output, choose A4 or A5, portrait or landscape, and margins in millimetres. Fit preserves the entire image; fill crops centrally to cover the area inside the margins. DPI limits the inserted image's resolution without changing the physical paper size.

Typed text and **UTF-8 TXT** are laid out locally with font size, line spacing, alignment, wrapping and automatic pages. The PDF embeds Ubuntu Light and keeps selectable text. Unsupported characters produce a clear error rather than disappearing. Limits are 8 MiB of text, one million drawn characters and 1000 pages.

This is a plain-text layout tool. Word and other Office documents keep the existing LibreOffice conversion; the editor does not open the full content of an existing PDF or DOCX as an editable document. TXT conversion without page settings also retains its traditional LibreOffice path.

## Save and reuse profiles

Save named settings in the editor and enable their visibility in **Convert to**. Up to **32 profiles** can be saved, with **8 visible in the menu**. The profiles section in Settings lets you apply, edit, rename, update, delete or select a default profile.

Profiles store parameters, a name, menu visibility and an optional destination folder. User-defined names are not translated when the interface language changes. Profiles do not contain the sample image, typed text or previews. Image-edit recipes apply to images; plain-text page profiles apply to the compatible inputs defined by their settings.

The **Save for web** example creates a WebP recipe capped at 300 kB, maximum quality 90 and explicitly permitted downsizing. You can change it or create alternatives such as JPEG at 1200 pixels wide or PDF A5 with your own margins and DPI.

In File Explorer, select compatible files and choose a profile from **Convert to**. The quick queue uses that recipe. A profile appears only if enabled in the menu, its output format is active and every selected input is compatible. Reopen the menu after changing a profile. Already queued jobs keep their original recipe snapshot.

Profiles add recipes to the existing **44 destinations**; they do not add formats or change the **93 recognized input extensions**. The eight default outputs remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Other outputs can be enabled in Settings.

Working images, text and previews remain in the editor session. Explicit saving creates the requested result; preferences and profiles stay local. See the bilingual [privacy statement](privacy.md) for queue records, data folders and codec temporary files.
