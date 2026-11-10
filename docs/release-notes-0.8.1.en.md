# Convert Here 0.8.1 — engine setup wizard

**Draft for the 0.8.1 candidate**, planned Store package **1.0.9.0**. Final builds, artifacts, verification and publication are not yet attested. The preceding **0.8.0 / Store 1.0.8.0 is published**. Developer and publisher: **Salvatore Corsi**. [Italiano](release-notes-0.8.1.md).

In **Settings → Output formats**, a **Select all** checkbox below each category replaces its check and cross buttons. Checking it selects every format in that category; clearing it deselects them. Individual extensions remain editable: clearing one also clears **Select all**. This filters outputs; supported inputs are unchanged. The eight default outputs remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4.

The first normal main-window launch without files or a preselected target offers an **engine setup wizard**. Choose **FFmpeg**, **Poppler**, **Tesseract with Italian/English**, **libheif** and **Whisper with multilingual Tiny**; missing engines are preselected. **LibreOffice** links to its official website and remains a separate installation. After confirming sources, versions, sizes, licenses and destination, selected engines download **one at a time**, using the existing SHA-256 verification and personal-folder installation.

Skipping or finishing the wizard creates the local **`engine-setup-v1.done`** marker beside preferences. Acknowledgement does not rewrite `preferences.json` and cannot be undone by another open window saving stale preferences. Successful installations save engine paths through the normal preferences flow. Reopen the wizard from **Settings → Conversion engines → Setup wizard**. CLI, quick conversions and custom-settings launches do not open it automatically.

Downloads begin only after confirmation. Engines, models, licenses and conversion routes remain those of 0.8.0. [Configuration guide](configuration.en.md), [verification still to complete](store-testing-0.8.1.en.md).
