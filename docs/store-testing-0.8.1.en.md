# Verification candidate 0.8.1 / Store 1.0.9.0

**Draft in preparation**, application **0.8.1**, planned Store package **1.0.9.0**, publisher **Salvatore Corsi**. The preceding **0.8.0 / Store 1.0.8.0 is published**. This record does not attest a final 0.8.1 build, GitHub release, upload or Store submission. [Italiano](store-testing-0.8.1.md).

## Intended behavior

- **Select all** below each format category: selecting and clearing the category, individual edits updating the collective control, eight unchanged defaults and output-only filtering.
- Automatic wizard only for a normal main-window launch without files, a preselected target or custom-settings mode. Quick conversions, queue status, CLI and PDF Merge retain their existing routes.
- Missing engines preselected: FFmpeg, Poppler, Tesseract Italian/English, libheif, Whisper/Tiny. LibreOffice offers only its official website. The FFmpeg/ffprobe transcription dependency does not create duplicate requests.
- Confirm sources, versions, sizes, licenses and destination before any download. One engine at a time; verify progress, cancellation, errors and retrying remaining engines in an isolated environment.
- Local `engine-setup-v1.done` marker published atomically without replacing existing files. Skipping or finishing does not rewrite preferences; older windows cannot erase acknowledgement by saving stale snapshots. File collisions and write errors are reported without deleting data.
- Explicit reopening from **Settings → Conversion engines → Setup wizard**, including after completing or skipping initial setup.

## Completed local verification

- **424 application tests and 18 DLL tests passed: 442 total**, with one optional ignored test excluded from the passing count. Local logs: `.tools/tests-081-final.log` and `.tools/tests-dll-081.log`.
- After the final purely visual card compaction, **24 focused wizard tests across GUI and CLI**, already included in the overall suite, and **all-targets Clippy with `-D warnings`** passed. Model and persistence were unchanged after the full suite. Logs: `.tools/tests-081-wizard-final.log`, `.tools/clippy-081-final.log`.
- Regressions cover planning every selection without duplicates, FFmpeg/Whisper dependencies, incomplete tools, ordering and sizes, errors/cancellation retaining successful engines, retrying only missing engines and refusing to report success from incomplete events. Marker tests use isolated folders and cover collisions, stale preferences and eight concurrent acknowledgements.
- **Native Italian and English UI QA passed**, with isolated settings and folders: automatic initial opening; Continue to source confirmation; Cancel returning to selection; Skip creating the marker while preserving preference bytes; manual reopening; Italian restart without the wizard. Record `.tools/ui-081-qa.json`; local English capture `.tools/ui-081-english.jpg`.
- In format QA, **Select all Audio** brings the total enabled outputs to **18**; clearing MP3 brings it to **17** and clears the collective checkbox; clearing Audio leaves **6** outputs in other categories; **Most common** restores the **8** defaults. These are global enabled-output counts, not the number of formats in the Audio family.
- Acquired [two new native 0.8.1 captures](store-assets/0.8.1/README.md), JPEG and PNG **2562 × 1441**, 150% UI zoom, without pointer, halo or retouching. GUI debug SHA-256 `bfba78128e3ee6bf10bf38b71a7deb3a4d31b3af65a2b12bc200d580919bd969`; JPEG → PNG conversion uses the app CLI without resizing or cropping. Digests are in the [capture record](store-assets/0.8.1/captures.json).

**The Download button was not pressed during wizard QA.** No actual download or installation through the new GUI is therefore claimed. Automatic tests and navigating to confirmation do not replace that verification. Captures use a debug build from the working tree; they do not show a newly installed MSIX.

## Checks and artifacts still to record

- [x] Application and DLL tests, focused wizard regressions and all-targets Clippy.
- [x] Italian and English UI QA with isolated preferences and test folders.
- [x] Two new captures with dimensions, hashes and provenance; proposed gallery reuses two historical 0.8.0 images of the unchanged editor.
- [ ] Clean commit, PE and manifest versions, checksums and inventories of setup, portable, source and MSIX artifacts.
- [ ] Offline checks of final vendored source and final-binary smoke tests.
- [ ] GitHub publication and the actual upload, submission or publication state of Store 1.0.9.0.

The planned requirement remains **Windows 10 build 19041 or later, x64**, including Windows 11. Technical identity, CLSID and `runFullTrust` remain unchanged. No new-MSIX installation, physical Windows 10 host test or WACK result is claimed. Previous 0.8.0 results do not replace verification of this candidate.
