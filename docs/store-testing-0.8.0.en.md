# 0.8.0 candidate / Store 1.0.8.0 testing

Application **0.8.0**, planned Store package **1.0.8.0**, publisher **Salvatore Corsi**. **In preparation: no publication or submission of this candidate is attested.** A clean commit, final artifacts, digests, screenshots and new-MSIX audit must be recorded after the build. [Italiano](store-testing-0.8.0.md).

The latest Store status referenced here is historical only: the [0.7.1 / 1.0.7.0 record](store-testing-0.7.1.md) reports **In certification on 9 November 2026**. This document does not update that status. WACK, installation of the new MSIX and testing on a physical Windows 10 host have not been performed.

## Additional Testing Information — draft, not yet saved in the portal

```text
Convert Here 0.8.0 / MSIX 1.0.8.0 candidate. Publisher: Salvatore Corsi. Windows 10 build 19041 or later, x64, including Windows 11. No in-app account or payment is required. Conversion, OCR and speech transcription run locally. Optional engines and language models are not bundled. Requested downloads require confirmation, HTTPS and SHA-256 verification; no selected images, documents, recordings or transcripts are uploaded. Linked HTML resources may access files or the network through LibreOffice.

After installing the candidate, use non-sensitive test samples. Open each main tab: Files, History, Settings and Profiles. Defaults remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Enable TXT under Settings > Output formats > Raw for OCR or transcription. Recognized inputs total 94, including static SVG; outputs remain 44.

Convert a simple local SVG to PNG and PDF without external engines. SVG is rasterized, not exported as vectors. External references, scripts, DTDs, embedded raster images and oversized documents are rejected.

In Settings > Conversion engines, request Tesseract OCR installation and confirm its sources, versions, size and licenses. Italian and English data are included. Recognize text in a test image. Scanned-PDF OCR additionally needs Poppler and remains opt-in. Recognition can contain errors.

Request libheif installation, then encode and decode HEIC/AVIF samples. For audio/video, request FFmpeg and open Edit. Use timeline handles or numeric times to trim, then apply gain/mute to a compatible media output. Waveforms and frames show the source before effects. Positive gain can clip; this is not a playback or multitrack editor.

Request Whisper with Tiny multilingual. The model is 77,691,713 bytes; runtime plus model downloads total 87,939,016 bytes, excluding FFmpeg. The GUI includes FFmpeg if FFmpeg or ffprobe is missing. Choose TXT for a short recording, select language or Auto and trim the interval. TXT uses trimming without gain/mute. Verify the local transcript and review recognition errors.

In File Explorer, check Convert to in the Windows 10 classic menu or Windows 11 modern menu/Show more options. With TXT enabled, audio/video offers TXT · Transcription. Test a native conversion, queue progress, cancellation and a conflicting destination name: preserve originals and existing results. Verify a profile, JSON import/export, PDF Bulk/Merge and history after restart.

The MSIX is intended to contain GUI and Explorer DLL with the native runtime downloader, notices and GPL license; no separate CLI, conversion engine or model. Final payload audit is pending. No new-MSIX installation, physical Windows 10 test or WACK result is claimed. Original code is GPL-3.0-only; engine and model licenses are retained. This candidate has not yet been submitted or published.
```

## Completed local checks

- **393 application tests + 18 DLL tests = 411 passed**. Application all-target Clippy, formatting, DLL all-target Clippy and DLL build passed. The pre-existing optional cached-ZIP test remains outside the passed count.
- Real DLL COM checks without registration: default/custom/empty/invalid preferences, live Italian/English language changes, batches, profiles and selection validation. Seven Invoke routes, including audio/video TXT, were tested against an executable that only captures arguments.
- **94 input extensions** match across application, DLL, script and installer; both inconsistent fixtures are rejected. Classic registration uses the same DLL exercised by COM.
- PowerShell fixtures: **8 PS7 suites and 6 PS5.1 suites** passed; **25 files** parsed. These include **51** Store checks, **37** branding checks, **26** sparse-menu checks, **21** corresponding-source checks, **12** runtime-wrapper checks, **7** literal roundtrips and **6** installer-policy cases. Fixtures do not install packages or change real registrations. PS5.1 uses the CI process parameter without changing persistent execution policy.
- Actual isolated preparation and repeated-install verification of catalog-pinned runtimes. Tesseract **5.5.4** recognized Italian/English; libheif **1.23.7** encoded and decoded HEIC/AVIF without extra plugin environment settings; Whisper **1.9.5.r258** with Tiny produced TXT. Whisper was also tested with only the generic x64 CPU backend present and no AVX backend DLLs.

These are source and local-runtime checks. They do not replace final release builds, artifact audits, candidate installation or Microsoft certification.

## Provenance, pins and privacy

The public [`portable-packages.json`](../src/runtime_install/portable-packages.json) catalog lists every archive/model with version, bytes, immutable or version-pinned URL, SHA-256 and selected files. Runtime closures contain **34 Tesseract packages**, **27 libheif packages** and **6 Whisper packages**, plus models and their notices. Download totals are **38,276,135**, **28,215,281** and **87,939,016 bytes** respectively; FFmpeg is separate.

Multilingual Tiny comes from Hugging Face revision **`5359861c739e955e79d9a303bcbc70fb988958b1`**, SHA-256 **`be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`**. OCR models come from tessdata_fast **`87416418657359cb625c412a48b6e1d6d41c29bd`**. Individual component pins remain in the catalog rather than being replaced by one bundle hash.

The **17 original license supplements**, **158,039 bytes**, retain copyright and provenance for giflib, libidn2, lz4, libx264 and x265. They are embedded through [`license_data.rs`](../src/runtime_install/license_data.rs) and verified before copying. `RUNTIME-SOURCES.json` and `LICENSE-SOURCES.json` document the installation. Third-party licenses and sources remain separate from GPL-3.0-only original code. See [engine inventory](release-licenses/EXTERNAL-ENGINES.md), [notices](../THIRD-PARTY-NOTICES.md) and the [privacy statement updated on 10 November 2026](privacy.md).

## Pending after the build

- [ ] Record the clean commit, PE/manifest versions, identities, hashes and contents of setup, portable, source and MSIX artifacts.
- [ ] Check final vendored sources offline with an initially empty cache.
- [x] Captured [four native 0.8.0 screenshots](store-assets/0.8.0/README.md), 2562 × 1441 PNGs, with original JPEGs, 150% UI zoom, hashes and provenance, without the pointer or retouching.
- [ ] Test the installed new MSIX, COM activation and Explorer menus, with separate results per operating system.
- [ ] Run on a physical Windows 10 host and run WACK, or keep these checks explicitly marked as not performed.
- [ ] After validation, record GitHub publication, Store upload/submission and observed status without reusing the 0.7.1 status.
