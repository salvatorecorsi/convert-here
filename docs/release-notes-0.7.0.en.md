# Convert Here 0.7.0 — profiles, engine options and history

**Release in preparation.** Planned application version: **0.7.0**; Microsoft Store package: **1.0.6.0**. This page does not yet announce published packages. Developer and publisher: **Salvatore Corsi**. [Italiano](release-notes-0.7.0.md).

The main window has four tabs: **Files**, **History**, **Settings** and **Profiles**. Format controls stay beside conversion, while engines, enabled outputs and appearance have their own page.

The new manager creates, edits, duplicates, renames and deletes recipes, selects a default and controls Explorer visibility. Limits remain **32 saved profiles** and **8 menu entries**. Each profile can allow all compatible inputs or selected extensions. These restrictions are checked before execution too. The engine still supports **44 output formats and 93 input extensions**.

**Import JSON**, **Export** and **Export all** transfer profiles. Import creates new copies without overwriting recipes. Invalid files, files over **256 KiB** and imports exceeding 32 total profiles apply no partial changes. Recipes beyond the eight visible slots are imported hidden, with their count reported. JSON includes options, filters and an optional output folder, without sample files or executable paths.

PDF profiles can compress, split or **merge**. A Merge profile requires at least two allowed inputs and opens page ordering before creating one PDF. Image edits and page layout retain their input restrictions.

**Common limits** set maximum size per output and maximum audio/video duration. Actual output bytes are checked before publication; an oversized result is not saved. The JPEG/WebP editor can adapt quality and reduce dimensions only with permission. Automatic audio/video bitrate adjustment is not promised. Older image limits remain effective; when both limits exist, the smaller one wins. The common cap also applies to PDFs made from images and text.

For audio, video and video-to-GIF, **Advanced** offers searchable codec, filter, metadata and performance controls. Encoder-specific controls come from configured local FFmpeg; values and compatibility are checked before use. Arbitrary commands and scripts are not accepted. Profiles with these options require audio/video sources; native image-to-GIF remains available without FFmpeg options.

**History** combines quick jobs, window conversions and editor/PDF Merge save attempts. Finished GUI attempts survive restarts, up to **300 entries within 1 MiB**. Refresh and writes run in the background; unreadable history is reported and preserved. History cleanup does not delete originals or outputs.

The Store package requires **Windows 10 build 19041 or later, x64**. Context-menu integration uses the classic menu on Windows 10 and the modern menu on Windows 11 where supplied by the package. The revised requirement was checked through schema and fixtures; this is not a claim of installation testing on a real Windows 10 host.

Conversion, editing and OCR remain local and free. Optional engines retain their requirements. Requested FFmpeg/Poppler downloads require confirmation and Internet; linked HTML resources may access the network through LibreOffice.

The planned distribution includes setup, portable, corresponding source with vendored Cargo dependencies and checksums. Original code remains **GPL-3.0-only**; dependencies, fonts and engines keep their licenses. No payment is required to continue using Convert Here.

Details: [editor and profile manager](image-editor-profiles.en.md), [configuration](configuration.en.md), [history and queue](queued-conversions.en.md), [CLI](command-line.en.md), [privacy](privacy.md), [validation status](store-validation.md).
