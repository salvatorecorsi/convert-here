# 0.8.0 / Store 1.0.8.0 testing

Application **0.8.0**, Store package **1.0.8.0**, publisher **Salvatore Corsi**, from clean commit **`f1f3f95fceed52507ef715fa4ac6893acc63544a`**. Final artifacts and audits are complete; **GitHub release published on 10 November 2026 at 12:09:39 UTC**. Submission 5 was submitted at about **12:10 UTC**; observed status **Update in certification**. The new MSIX was uploaded, saved and Validated before submission. [Italiano](store-testing-0.8.0.md).

Publication of preceding **Submission 4 / Store 1.0.7.0** was verified on **10 November 2026**. The [0.7.1 record](store-testing-0.7.1.md) retains its historical In certification state from 9 November. New 1.0.8.0 is not declared approved or published. WACK, installation of the new MSIX and testing on a physical Windows 10 host have not been performed.

## Additional Testing Information — saved and reread in the portal

```text
Convert Here 0.8.0 / MSIX 1.0.8.0. Publisher: Salvatore Corsi. Windows 10 build 19041 or later, x64, including Windows 11. No in-app account or payment is required. Conversion, OCR and speech transcription run locally. Optional engines and language models are not bundled. Requested downloads require confirmation, HTTPS and SHA-256 verification; no selected images, documents, recordings or transcripts are uploaded. Linked HTML resources may access files or the network through LibreOffice.

After installing the candidate, use non-sensitive test samples. Open each main tab: Files, History, Profiles and Settings. Defaults remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Enable TXT under Settings > Output formats > Raw for OCR or transcription. Recognized inputs total 94, including static SVG; outputs remain 44.

Convert a simple local SVG to PNG and PDF without external engines. SVG is rasterized, not exported as vectors. External references, scripts, DTDs, embedded raster images and oversized documents are rejected.

In Settings > Conversion engines, request Tesseract OCR installation and confirm its sources, versions, size and licenses. Italian and English data are included. Recognize text in a test image. Scanned-PDF OCR additionally needs Poppler and remains opt-in. Recognition can contain errors.

Request libheif installation, then encode and decode HEIC/AVIF samples. For audio/video, request FFmpeg and open Edit. Use timeline handles or numeric times to trim, then apply gain/mute to a compatible media output. Waveforms and frames show the source before effects. Positive gain can clip; this is not a playback or multitrack editor.

Request Whisper with Tiny multilingual. The model is 77,691,713 bytes; runtime plus model downloads total 87,939,016 bytes, excluding FFmpeg. The GUI includes FFmpeg if FFmpeg or ffprobe is missing. Choose TXT for a short recording, select language or Auto and trim the interval. TXT uses trimming without gain/mute. Verify the local transcript and review recognition errors.

In File Explorer, check Convert to in the Windows 10 classic menu or Windows 11 modern menu/Show more options. With TXT enabled, audio/video offers TXT · Transcription. Test a native conversion, queue progress, cancellation and a conflicting destination name: preserve originals and existing results. Verify a profile, JSON import/export, PDF Bulk/Merge and history after restart.

The audited MSIX contains the GUI and Explorer COM DLL, native runtime downloader, notices and GPL license, with runFullTrust as its only capability. It contains no separate CLI, conversion engine or model. Build source: clean commit f1f3f95fceed52507ef715fa4ac6893acc63544a. MSIX SHA-256: a7a010371015a69b798e255b373b90f6cfddf4fed4b5611bc7cd845f00c841f9. Optional AI use is limited to local speech recognition of recordings. Original code is GPL-3.0-only; engine and model licenses are retained. Corresponding source with vendored dependencies and offline build instructions: https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.0. Offline cargo checks passed with an initially empty cache. No new-MSIX installation, physical Windows 10 host test or WACK result is claimed.
```

## Completed local checks

- **400 application tests + 18 DLL tests = 418 passed**. Application all-target Clippy, formatting, DLL all-target Clippy and DLL build passed. The pre-existing optional cached-ZIP test remains outside the passed count.
- SVG guards before usvg conversion: seven additional regressions cover expanded path, text and attribute costs, `use`/`textPath` references, `href` precedence, ambiguous IDs and CSS or inherited copies (`stroke-dasharray`, font families and inline styles). The conservative budget multiplies attribute/CSS bytes by expanded nodes; `marker` and `tref` are rejected. Ordinary renderable cases remain verified.
- Real DLL COM checks without registration: default/custom/empty/invalid preferences, live Italian/English language changes, batches, profiles and selection validation. Seven Invoke routes, including audio/video TXT, were tested against an executable that only captures arguments.
- **94 input extensions** match across application, DLL, script and installer; both inconsistent fixtures are rejected. Classic registration uses the same DLL exercised by COM.
- PowerShell fixtures: **8 PS7 suites and 6 PS5.1 suites** passed; **25 files** parsed. These include **51** Store checks, **37** branding checks, **26** sparse-menu checks, **21** corresponding-source checks, **12** runtime-wrapper checks, **7** literal roundtrips and **6** installer-policy cases. Fixtures do not install packages or change real registrations. PS5.1 uses the CI process parameter without changing persistent execution policy.
- Actual isolated preparation and repeated-install verification of catalog-pinned runtimes. Tesseract **5.5.4** recognized Italian/English; libheif **1.23.7** encoded and decoded HEIC/AVIF without extra plugin environment settings; Whisper **1.9.5.r258** with Tiny produced TXT. Whisper was also tested with only the generic x64 CPU backend present and no AVX backend DLLs.
- Real downloads through **production native WinHTTP**, without cache injection: Tesseract, libheif and Whisper completed under isolated roots, with observed bytes matching pins, all payload hashes verified and a second installation requiring no network. Cancellation during a real MSYS2 stream after **65,536 bytes** removed the partial workspace, leaving only the expected lock. Local record `.tools/runtime-research-080/native-winhttp-qa.json` (SHA-256 `982cdca8217d549da72d918d46887072824c9ae429a29d7201e66a37dfbd9227`) records byte-for-byte production source copies and a separate QA adapter. This check does not execute downloaded engines or attest GUI or Store installation.

These source and local-runtime checks are separate from the final artifact audits below. They do not attest installation of the new MSIX or Microsoft certification.

## Provenance, pins and privacy

The public [`portable-packages.json`](../src/runtime_install/portable-packages.json) catalog lists every archive/model with version, bytes, immutable or version-pinned URL, SHA-256 and selected files. Runtime closures contain **34 Tesseract packages**, **27 libheif packages** and **6 Whisper packages**, plus models and their notices. Download totals are **38,276,135**, **28,215,281** and **87,939,016 bytes** respectively; FFmpeg is separate.

Multilingual Tiny comes from Hugging Face revision **`5359861c739e955e79d9a303bcbc70fb988958b1`**, SHA-256 **`be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`**. OCR models come from tessdata_fast **`87416418657359cb625c412a48b6e1d6d41c29bd`**. Individual component pins remain in the catalog rather than being replaced by one bundle hash.

The **17 original license supplements**, **158,039 bytes**, retain copyright and provenance for giflib, libidn2, lz4, libx264 and x265. They are embedded through [`license_data.rs`](../src/runtime_install/license_data.rs) and verified before copying. `RUNTIME-SOURCES.json` and `LICENSE-SOURCES.json` document the installation. Third-party licenses and sources remain separate from GPL-3.0-only original code. See [engine inventory](release-licenses/EXTERNAL-ENGINES.md), [notices](../THIRD-PARTY-NOTICES.md) and the [privacy statement updated on 10 November 2026](privacy.md).

## Final artifacts and audits

All artifacts below come from clean commit **`f1f3f95fceed52507ef715fa4ac6893acc63544a`**. The four GitHub files are published in [release v0.8.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.0), without draft status; remote digests match local files.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.8.0-windows-x64-setup.exe` | 15,685,660 | `1959d372be6210cf9107bd2a39ec80c41ab2a6474c213fd2e07065a5e7cb9f73` |
| `ConvertHere-0.8.0-windows-x64-portable.zip` | 20,700,405 | `6b59b12294a8b95c038b3d35fc20184792e333121a36a395f4335fe6cbc23d33` |
| `ConvertHere-0.8.0-source.zip` | 89,286,003 | `ba3e2695a3a53bbdcbaf216c275e3c98f2ee0c138e0d0e90b01aa6cebe64eea9` |
| `SHA256SUMS.txt` | 313 | `8389e4be7d8fcd21e060fc7fa5977446020b2f12ba986861ea2513055dcb92f7` |
| `ConvertHere-1.0.8.0-x64-store.msix` | 9,342,790 | `a7a010371015a69b798e255b373b90f6cfddf4fed4b5611bc7cd845f00c841f9` |

Standalone auditing confirms **98 portable files** byte-identical to staging, three binaries matching `build-info.json`, EXE version **0.8.0**, publisher **Salvatore Corsi**, x64 and static CRT. The allowlist excludes optional engines, personal data and development files. This audit did not execute the setup.

Store auditing confirms **57 payload entries / 59 ZIP entries**, GUI and DLL without the CLI, engines, models or downloader scripts; **277 dependency components**, complete GPL text and the entire license tree matching the commit. Identity **SalvatoreCorsi.ConvertHere**, technical publisher **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17**, MinVersion **10.0.19041.0**, x64 and the sole **runFullTrust** capability are unchanged. Dynamic CRT imports are absent; COM surrogate/STA, CLSID and **0000ConvertHere** verb are checked in the manifest. This check neither installs the package nor loads the COM DLL. Local reports: `.tools/standalone-080-audit.json`, `.tools/standalone-080-payload-audit.json` and `.tools/store-080-audit.json`.

## Offline source checks and final-binary smoke tests

Actual **`cargo check --locked --offline --all-targets`** commands passed for the app and DLL from extracted source, with initially empty Cargo cache and target: **75.56 s** and **4.02 s**, static CRT and a compiler matching provenance. Checked **465 crates**, **24,338 vendor files** and **228 tracked files**; 192 files required only exact CRLF → LF normalization. This is not a full linked build. The two helpers count different sets of license filenames, so those counts are neither combined nor used as the count of 277 Windows dependencies.

Local record: `.tools/source-offline-080-91eb8afd23cd48fd9083810eaa844be3/offline-check.json`, SHA-256 **`d7fe4861f7d70b6f068df3fadaf499e2c53ed99126e667b15be974a359fc6d74`**. The final cache contains only `CACHEDIR.TAG`, without downloaded source or packages.

Final-binary smoke tests: **13/13 passed**, including SVG conversions, audio/video transcription, trimming/gain, source preservation and filename conflicts without overwriting. The audio segment measures **0.75 s** and the measured RMS ratio is **0.5001102921**. Real user preferences were not used and no downloads took place during these smoke tests. Record `.tools/release-smoke-080-529f1ecefd0145c8a08b6bb0fd65a1e6/report.json`, SHA-256 **`4ded74939cc47abb0761f12381416338558daf523d53359425c1d197fa4dd468`**.

## Saved portal state and remaining checks

**Submission 5**, ID **1152921505702365703**: MSIX 1.0.8.0 uploaded and saved as Validated, then submitted on **10 November 2026 at about 12:10 UTC**. The overview shows **Update in certification**. No unobserved processing, certification or publishing sub-stage is inferred. Italian and English listings are saved and reread with **20 features and four PNGs per language**; privacy and license point to public tag **v0.8.0**. Free pricing, **240 markets** and publication as soon as approved are confirmed. The Windows **8.11** agreement entry was already accepted on 4 November, **Action: N/A**; no new acceptance was performed. Local DOM verification records are `.tools/store-listing-080-it-verified.json` and `.tools/store-listing-080-en-verified.json`.

- [x] Audit final commit, versions, identities, checksums and payloads.
- [x] Offline source checks with initially empty cache and target.
- [x] Final-binary smoke tests, 13/13.
- [x] Captured [four native 0.8.0 screenshots](store-assets/0.8.0/README.md), 2562 × 1441 PNGs, with original JPEGs, 150% UI zoom, hashes and provenance, without the pointer or retouching.
- [x] Upload and save Validated MSIX, IT/EN listings and their images.
- [ ] Test the installed new MSIX, COM activation and Explorer menus, with separate results per operating system.
- [ ] Run on a physical Windows 10 host and run WACK, or keep these checks explicitly marked as not performed.
- [x] [Windows checks 42811925135](https://github.com/salvatorecorsi/convert-here/actions/runs/42811925135): **SUCCESS**, verify job **9 min 49 s**; local log `.tools/ci-080-watch.log`.
- [x] Publish GitHub release at **12:09:39 UTC**, submit Submission 5 and record **Update in certification**. New Store 1.0.8.0 is not declared approved or public; preceding 1.0.7.0 remains published.

Final testing notes (**3,207 portal characters**) were saved and reread; local record `.tools/store-notes-080-verified.txt`. Submission evidence: `.tools/store-080-submission-state.txt` and `.tools/store-080-submission-proof.jpg`. These show the certification header; no further timeline stage is claimed.
