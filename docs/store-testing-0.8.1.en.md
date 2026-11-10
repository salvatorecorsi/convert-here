# Verification 0.8.1 / Store 1.0.9.0

Application **0.8.1**, Store package **1.0.9.0**, publisher **Salvatore Corsi**, from clean commit **`07fe5f749327b73e93a92d5101d80b9b4f5317cb`**. [GitHub release](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.1) published on **10 November 2026 at 13:42:06 UTC**; final artifacts verified. **Submission 6**, ID **1152921505702367000**, submitted: observed state **Update in certification** at **13:44:28 UTC**, with automatic publication after approval. New 1.0.9.0 is not yet claimed approved or available; preceding **0.8.0 / Store 1.0.8.0 is published**. [Italiano](store-testing-0.8.1.md).

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

## Verification summary

- [x] Application and DLL tests, focused wizard regressions and all-targets Clippy.
- [x] Italian and English UI QA with isolated preferences and test folders.
- [x] Two new captures with dimensions, hashes and provenance; proposed gallery reuses two historical 0.8.0 images of the unchanged editor.
- [x] Clean commit, PE and manifest versions, checksums and inventories of setup, portable, source and MSIX artifacts.
- [x] Offline checks of final vendored source and final-binary smoke tests.
- [x] GitHub publication; Validated MSIX saved and Submission 6 submitted for certification.

The requirement remains **Windows 10 build 19041 or later, x64**, including Windows 11. Technical identity, CLSID and `runFullTrust` remain unchanged. No new-MSIX installation, physical Windows 10 host test or WACK result is claimed. Verification of this release is separate from historical 0.8.0 results.

## Final artifacts and provenance

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.8.1-windows-x64-setup.exe` | 16400262 | `864aa68a5cab9d653dcedb002df4141b6ae5502f5675bf77cfe89bc605077d9e` |
| `ConvertHere-0.8.1-windows-x64-portable.zip` | 21682023 | `3a0e7a8e31b21f93b59f3e85869fa484fe9603ade09a8e3b456b968e70de0dbd` |
| `ConvertHere-0.8.1-source.zip` | 90238585 | `5049b91425261350e00977a8aa690d6f8ac9997c20431413d1730c9803124326` |
| `SHA256SUMS.txt` | 313 | `a8169e159ff7766e3509419afd9a0a4062cceea31568ea4c1f270ff07af53503` |
| `ConvertHere-1.0.9.0-x64-store.msix` | 9359517 | `0b7af68333a5587e0509c65909ebb01cba541e72fcbe4307cc2d2a2e24def772` |


All artifacts derive from commit **`07fe5f749327b73e93a92d5101d80b9b4f5317cb`**, with `SourceDirty: false`. All four GitHub assets were downloaded again and compared: sizes, SHA-256 and `SHA256SUMS.txt` contents match the audited local files. Local record `.tools/remote-release-081-f0e38951b2c4436396432966aa4fb721/verification.json`.

Standalone audit **PASS**: **108 portable files** byte-identical to staging, three binaries matching `build-info.json`, app/CLI PE **0.8.1**, manifests **0.8.1.0**, publisher Salvatore Corsi and static CRT. **277 Windows components** retain their notices. Records `.tools/standalone-081-audit.json` and `.tools/standalone-081-payload-audit.json`. This audit does not run setup, install or register the program.

Store audit **PASS**: **57 payload files / 59 ZIP entries**, GUI and DLL, native runtime installer, **277 components** and **26 license files** matching the commit. Separate CLI, engines, models and downloader scripts are absent. Identity `SalvatoreCorsi.ConvertHere`, technical publisher `CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17`, x64, MinVersion **10.0.19041.0**, CLSID `67B51BA1-8A78-4B4B-A5F9-08A825BE91F8`, verb `0000ConvertHere` and sole capability `runFullTrust` remain unchanged. PE imports contain no dynamic CRT dependency. The DLL crate version is 0.8.1; the audit does not assign a nonexistent PE file version to the DLL. Record `.tools/store-081-audit.json`.

## Offline source, smoke and CI

Actual **`cargo check --locked --offline --all-targets`** passed for app (**81.23 s**) and DLL (**4.17 s**) from extracted source, with initially empty Cargo cache and target folders and `+crt-static`. Compiler matched to provenance: PASS. Verified **465 crates**, **24,338 vendor files** and **240 tracked files** against the commit; **200** files permit only exact CRLF → LF normalization. Archive hash remained unchanged; the final cache contains only `CACHEDIR.TAG`, without downloaded packages or index. This is not a full offline linked build.

Record `.tools/source-offline-081-e77647330ce84fa9b27e64a944e4fe83/offline-check.json`, SHA-256 **`f2ee8756d0152f8d3df4823f3906b2ba77f4c4982ed94eb35e04f9d0eb5d13cc`**. License-file counters from different helpers describe different sets and are not added together.

Final-binary smoke **13/13 PASS**: SVG PNG/JPG/PDF, audio/video transcription using the local model, trimming/gain, HEIC/AVIF and OCR. Originals and existing files preserved, without personal preferences or engine downloads during these checks. Measured RMS ratio **0.5001102921267032**. Record `.tools/release-smoke-081-324df6a7ee364e5b8cf2784167d14715/report.json`, SHA-256 **`3a0c0c4fbcb663e42cba35e11343e65adfa84ccbcb9f96e59de6bf62256092c5`**.

[Windows checks 42825704990](https://github.com/salvatorecorsi/convert-here/actions/runs/42825704990) is **SUCCESS** for the build commit; verify job **6 min 31 s**. Focused checks already included in the 442 passing tests are not counted twice.

## Store portal and remaining limits

MSIX **1.0.9.0** was uploaded, saved as **Validated** and submitted as **Submission 6**, ID **1152921505702367000**. On **10 November 2026 at 13:44:28 UTC**, the portal shows **Update in certification**; unobserved sub-stages are not inferred. Automatic publication after approval is configured. Preceding **1.0.8.0 is published**; new Store 1.0.9.0 is not yet claimed approved or available.

IT/EN listings were saved and reread from the server with **20 features, four PNGs and four captions per language**. Gallery: two new 0.8.1 captures and two historical 0.8.0 captures of the unchanged editor; every caption identifies the Italian interface. Privacy and license at public tag **v0.8.1** are saved and verified. Final Additional Testing Information of **5,835 characters** was saved and reread with normalized-whitespace comparison. The `runFullTrust` justification was updated for the five engines, **470 characters**, without adding capabilities.

Local evidence: `.tools/store-081-submission-proof.jpg`, `.tools/store-081-submission-proof.txt`, `.tools/store-notes-081-final.txt`. The new GUI's Download button was not pressed in native QA; **new-MSIX installation, a physical Windows 10 host test and WACK were not run**. These limits remain separate from actual 0.8.0 runtime-download checks and new CLI smoke checks.

## Additional Testing Information — saved and reread in the portal

```text
Convert Here 0.8.1 / MSIX 1.0.9.0. Publisher: Salvatore Corsi. Windows 10 build 19041 or later, x64, including Windows 11. No in-app account or payment is required. Conversion, OCR and speech transcription run locally. Optional engines and language models are not bundled. Requested downloads require confirmation, HTTPS and SHA-256 verification; no selected images, documents, recordings or transcripts are uploaded. Linked HTML resources may access files or the network through LibreOffice.

After installing the candidate, use non-sensitive test samples. Open each main tab: Files, History, Profiles and Settings. Defaults remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Enable TXT under Settings > Output formats > Raw for OCR or transcription. Recognized inputs total 94, including static SVG; outputs remain 44. In Settings > Output formats, check Select all below Audio, clear one format and confirm that Select all clears too; individual formats stay clickable. Clearing Select all disables that category. Most common restores the eight defaults. Filtering outputs does not remove supported inputs.

For first-run setup, use a fresh test configuration and open the normal main window without files or a preselected target. The wizard preselects missing engines: FFmpeg, Poppler, Tesseract, libheif and Whisper/Tiny. LibreOffice offers its official website only. Click Continue to review sources, versions, sizes, licenses and destination, then confirm before any download. The selected engines install sequentially; FFmpeg is included once before Whisper when FFmpeg or ffprobe is missing. Cancellation or an error preserves completed engines, and retry plans only what remains missing.

Cancel the download confirmation and verify that the wizard returns to engine selection. Skip or finish the wizard and restart: automatic setup should not return. A separate engine-setup-v1.done marker beside preferences remembers acknowledgement without rewriting preferences.json; successful engine installs save tool paths normally. Reopen setup from Settings > Conversion engines > Setup wizard. Quick conversions, CLI, queue-status windows, PDF Merge and custom-settings launches do not automatically open the wizard.

Convert a simple local SVG to PNG and PDF without external engines. SVG is rasterized, not exported as vectors. External references, scripts, DTDs, embedded raster images and oversized documents are rejected.

In Settings > Conversion engines, request Tesseract OCR installation and confirm its sources, versions, size and licenses. Italian and English data are included. Recognize text in a test image. Scanned-PDF OCR additionally needs Poppler and remains opt-in. Recognition can contain errors.

Request libheif installation, then encode and decode HEIC/AVIF samples. For audio/video, request FFmpeg and open Edit. Use timeline handles or numeric times to trim, then apply gain/mute to a compatible media output. Waveforms and frames show the source before effects. Positive gain can clip; this is not a playback or multitrack editor.

Request Whisper with Tiny multilingual. The model is 77,691,713 bytes; runtime plus model downloads total 87,939,016 bytes, excluding FFmpeg. The GUI includes FFmpeg if FFmpeg or ffprobe is missing. Choose TXT for a short recording, select language or Auto and trim the interval. TXT uses trimming without gain/mute. Verify the local transcript and review recognition errors.

In File Explorer, check Convert to in the Windows 10 classic menu or Windows 11 modern menu/Show more options. With TXT enabled, audio/video offers TXT · Transcription. Test a native conversion, queue progress, cancellation and a conflicting destination name: preserve originals and existing results. Verify a profile, JSON import/export, PDF Bulk/Merge and history after restart.

The final package was built from clean source commit 07fe5f749327b73e93a92d5101d80b9b4f5317cb, https://github.com/salvatorecorsi/convert-here/tree/07fe5f749327b73e93a92d5101d80b9b4f5317cb. Original code is GPL-3.0-only; engine and model licenses are retained. Optional AI use is limited to local speech recognition of recordings. The audited MSIX contains GUI, Explorer COM DLL, native runtime downloader, notices and license, with runFullTrust as its only capability and no separate CLI, conversion engine or model. Package size: 9,359,517 bytes. MSIX SHA-256: 0b7af68333a5587e0509c65909ebb01cba541e72fcbe4307cc2d2a2e24def772. Final source archive SHA-256: 5049b91425261350e00977a8aa690d6f8ac9997c20431413d1730c9803124326. Offline cargo check --locked --offline --all-targets passed for the app (81.23 seconds) and DLL (4.17 seconds), using initially empty Cargo cache and target folders with static CRT flags. This was a check, not a complete linked build. Source audit verified all 240 tracked files against the commit and 465 vendored crates; the archive hash was unchanged afterwards. Final-binary CLI smoke testing passed 13 conversions.

Preparation evidence: 424 application tests plus 18 DLL tests passed (442 total, with one optional ignored test excluded), along with all-targets Clippy and native Italian/English UI checks. The final 24 focused wizard tests are already included in the total. Native GUI QA checked first opening, source confirmation, cancellation, skipping, reopening, restart and format selection; its Download button was not pressed. GitHub Windows checks 42825704990 passed: https://github.com/salvatorecorsi/convert-here/actions/runs/42825704990. GitHub release v0.8.1 was published on 10 November 2026 at 13:42:06 UTC, with corresponding source, vendored dependencies and offline build instructions: https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.1. No new-MSIX installation, physical Windows 10 host test, WACK result or actual GUI download/install is claimed.
```
