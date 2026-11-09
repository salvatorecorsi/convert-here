# Image editor, pages and profiles

[Italiano](image-editor-profiles.md) · **English**

The editor is available from **Convert Here 0.6.0**. The manager with input filters, JSON and advanced options is planned for **0.7.0**, currently being prepared.

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

**Maximum file size** uses decimal kB: **300 kB = 300000 bytes**. The common limit also applies to image and TXT PDFs. Older image limits remain effective; when both limits exist, the smaller one wins. Editing the size control updates the common limit.

For JPEG and WebP, enabling the limit changes the quality control to a maximum quality and the encoder searches for a quality that fits. **Also reduce dimensions if necessary** explicitly allows a smaller resolution. PNG is lossless and may need this reduction for a lower size. For PDFs, adjust content, image quality, dimensions and DPI. An audio/video duration saved in the settings does not limit images or text.

Some limits cannot be met. Saving then remains disabled; a quick job using the same profile fails without publishing an oversized result. Increase the limit or change dimensions, quality or content.

## A4/A5 images and plain text

For PDF output, choose A4 or A5, portrait or landscape, and margins in millimetres. Fit preserves the entire image; fill crops centrally to cover the area inside the margins. DPI limits the inserted image's resolution without changing the physical paper size.

Typed text and **UTF-8 TXT** are laid out locally with font size, line spacing, alignment, wrapping and automatic pages. The PDF embeds Ubuntu Light and keeps selectable text. Unsupported characters produce a clear error rather than disappearing. Limits are 8 MiB of text, one million drawn characters and 1000 pages.

This is a plain-text layout tool. Word and other Office documents keep the existing LibreOffice conversion; the editor does not open the full content of an existing PDF or DOCX as an editable document. TXT conversion without page settings also retains its traditional LibreOffice path.

## Save and reuse profiles

The **Profiles** tab contains the manager: **New**, edit, duplicate, rename, delete, default selection and **Show in Convert to**. Save up to **32 profiles**, with **8 visible in the menu**. Choose an output family and format, configure its options, then use **Save profile**. **Image and page editor…** opens visual tools when the profile's inputs permit them. You can also save a recipe from the editor.

Profiles store parameters, a name, menu visibility and an optional destination folder. User-defined names are not translated when the interface language changes. Profiles do not contain the sample image, typed text or previews. Image-edit recipes apply to images; plain-text page profiles apply to the compatible inputs defined by their settings.

For **Save for web**, choose WebP, enable image edits and set a 300 kB limit and maximum quality 90. Allow dimension reduction if desired. Other examples include JPEG at 1200 pixels wide, MP3 at a chosen bitrate or PDF A5 with custom margins and DPI.

In File Explorer, choose a profile from **Convert to** to apply the recipe without reopening its settings. It appears only if visible in the menu, its output is enabled and every selected input is compatible. Reopen an already open menu after changing a profile. Already queued jobs keep their original recipe snapshot.

A **PDF Merge** profile requires at least two files and always opens page ordering before saving one PDF. **PDF Split** profiles accept PDFs only. Image-edit settings restrict inputs to images; page layout accepts compatible images and TXT files.

## Choose input formats

Each recipe can accept **all compatible inputs** or a set of extensions, such as PNG/JPEG for WebP or WAV/FLAC for MP3. The filter applies to the complete Explorer selection: one excluded file hides the profile. Changing output format removes incompatible extensions from the draft; if none remain, select an input before saving.

The filter does not add conversions or change general app support. Unknown or incompatible extensions are rejected during JSON import too.

## Import and export JSON

Use **Export…** on one recipe or **Export all…**, then **Import JSON…** in the other installation. The file includes a schema version, name, visibility, inputs and settings, including an optional output folder. It contains no engine paths, images, sample text or previews. A saved folder may need changing on the new PC.

Import creates new copies without overwriting existing profiles. Invalid recipes or exceeded limits reject the whole import without partial changes. The limits are **256 KiB** and **32 total profiles**. Additional recipes beyond the **8 visible menu entries** are imported hidden; the result message reports their count.

Minimal portable example:

```json
{
  "schema": "convert-here-profiles",
  "version": 1,
  "profiles": [{
    "name": "Save for web",
    "show_in_menu": true,
    "input_extensions": ["jpeg", "jpg", "png"],
    "settings": {
      "format": "Webp",
      "image": { "quality": 90, "edit": { "allow_downsize": true } },
      "limits": { "max_bytes": 300000 }
    }
  }]
}
```

## Limits and advanced controls

The common size limit checks each output in every format family. Oversized audio/video conversions fail without publishing their result; automatic bitrate adjustment is not promised. Maximum duration applies to audio/video inputs, including audio extraction and video-to-GIF, combined with the trim interval.

For these sources, **Advanced** offers searchable codec, filter, metadata and performance settings. Encoder-specific options are read from local FFmpeg and depend on the selected encoder. Values are checked; unsupported or incompatible options produce an explicit error. Arbitrary command lines are not accepted. Profiles with FFmpeg options require audio/video sources. Native image-to-GIF uses its normal path without those options.

Profiles add recipes to the existing **44 destinations**; they do not add formats or change the **93 recognized input extensions**. The eight default outputs remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Other outputs can be enabled in Settings.

Working images, text and previews remain in the editor session. Explicit saving creates the requested result; preferences and profiles stay local. See the bilingual [privacy statement](privacy.md) for queue records, data folders and codec temporary files.
