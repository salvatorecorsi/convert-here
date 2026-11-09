# Release 0.7.0 / Store 1.0.6.0 — istruzioni e prove

La [release GitHub 0.7.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.0) è pubblicata dal **9 novembre 2026, 19:36:38 UTC**, commit `e4b41f66ce2d99c9c7bd528aa29733a71db2330d`; [CI 42683376497](https://github.com/salvatorecorsi/convert-here/actions/runs/42683376497) **SUCCESS**. Il pacchetto **1.0.6.0** è caricato e validato. La **Submission 3**, ID **1152921505702351165**, è stata inviata alle **19:45 UTC** e risulta **In certification**: Submission Complete, Pre-processing In progress, Certification Not started, Publishing Not started. Pubblicazione automatica appena approvata, prezzo gratuito e 240 mercati. La 0.7.0 non è ancora approvata o pubblicata nello Store.

Le schede IT ed EN sono state salvate e rilette con **20 funzionalità e quattro schermate ciascuna**; privacy 0.7.0 verificata nel portale. Il pacchetto Store include GUI e DLL Explorer: la CLI appartiene alla distribuzione GitHub. Le prove seguenti non attestano installazione del nuovo MSIX, registrazione effettiva del menu o esecuzione su un host Windows 10 reale.

## Additional Testing Information — English

```text
Convert Here 0.7.0 / MSIX 1.0.6.0, publisher Salvatore Corsi. Native Rust/Win32 desktop utility, x64. Minimum OS is now Windows 10 version 2004, build 19041; Windows 11 is also supported. The previous package unnecessarily required build 22000. Package schema, semantic validation and imports have been checked, but we have not run an installation on a separate Windows 10 host or WACK.

No account, subscription, purchase or payment is required. File conversion, image editing and OCR are processed locally. Optional engines are not bundled. Internet is only needed for optional engine downloads or opening official websites; linked resources in input HTML may also be fetched by LibreOffice.

1. Launch Convert Here. The four tabs are Files to convert, History, Settings and Profiles. Italian/English can be selected in Settings independently of OCR language. Settings has five output-format columns and a uniform engine grid.
2. Native smoke test: add PNG/JPEG images and convert to PNG, JPEG, WebP or PDF. Results are created alongside originals by default; existing names receive a number. Input formats remain supported even when their output format is disabled in Settings. The eight default outputs are MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4.
3. Profiles: create, name, edit, duplicate, select defaults, import/export JSON and select allowed input extensions. Up to 32 profiles may be saved, 8 visible in Convert to. A queued job retains its settings snapshot after a profile is changed. Incompatible input selections do not offer a profile. JSON imports validate the whole file before changing saved profiles.
4. Image editor: use Edit on an image to crop, resize and place it on a canvas; export PNG/JPEG/WebP/PDF. A4/A5 pages support images, typed text or UTF-8 TXT. JPEG/WebP previews show the actual encoded size. A 300 kB cap means 300000 bytes. If a cap is impossible, no oversized output is published.
5. Optional FFmpeg: Settings can download FFmpeg/Poppler after showing source, version, size, license and destination for explicit confirmation. Configure an existing executable instead if preferred. An audio/video profile can specify maximum duration and file size. A tested 35-second FLAC with a 30-second maximum produced a 30.000000-second MP3. Other audio/video/PDF size caps are hard limits, not guaranteed automatic compression to the requested size.
6. Advanced audio/video options include typed codec, preset, rate, color, filter and metadata controls. Read installed encoder options to discover additional numeric/Boolean/enumerated AVOptions. Raw commands, paths, scripts and arbitrary filter graphs are not accepted. FFmpeg overrides require audio/video sources, including GIF made from video.
7. File Explorer: select supported files and use Convert to. Windows 10 uses its classic context menu; Windows 11 supports the packaged modern menu. A quick conversion creates a status placeholder and runs in the local queue. History combines running, queued and finished work; opening a placeholder shows status and cancellation. For two or more PDF-compatible files, PDF Bulk produces separate outputs and PDF Merge opens drag-and-drop page ordering before saving one PDF. Merge profiles also require at least two files. Audio/video are excluded from PDF Merge.
8. Poppler is needed for PDF export/text extraction, LibreOffice for Office/Excel/HTML routes, libheif for HEIC/AVIF, and Tesseract plus language models for OCR. Scanned PDF OCR also needs Poppler. Missing tools produce a clear error rather than a false successful conversion. PDF-to-DOCX/HTML reconstruction is text-only. Signed, encrypted and interactive-form PDFs have restrictions described in the user guide.

The MSIX includes the GUI app, packaged Explorer COM DLL, native runtime installer and license notices; the separate CLI is in the GitHub distribution; no external conversion engine executables or downloader scripts are bundled. It runs asInvoker, with no service or driver. GUI history is bounded to 300 terminal attempts within 1 MiB. Preferences, job paths and error details remain local.

GPL-3.0-only source and matching vendored dependencies: https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.0
Privacy: https://github.com/salvatorecorsi/convert-here/blob/v0.7.0/docs/privacy.md
Guide: https://github.com/salvatorecorsi/convert-here/blob/v0.7.0/docs/image-editor-profiles.en.md
```
## Prove effettivamente eseguite

- [x] Suite completa: **348 test app**, un test già ignorato, **18 test DLL**; due ulteriori regressioni dei controlli numerici e degli alias UI superate. Formattazione e Clippy rigoroso superati.
- [x] Fixture: **50 controlli packaging Store** e **37 branding**, sia PowerShell 7 sia Windows PowerShell 5.1. Manifest finale Windows.Desktop x64 con MinVersion **10.0.19041.0**.
- [x] GUI nativa: quattro schede, colonne delle uscite, griglia dei motori e manager. Importazione JSON non valida rifiutata senza creare profili; importazione valida di **quattro profili**.
- [x] Profilo rapido: **FLAC 35 s → MP3**, limite **30 s**, risultato **721649 byte**; FFprobe **30.000000 s**. Cronologia osservata anche in una seconda finestra.
- [x] Audit standalone e sorgenti dal commit pulito: `.tools/standalone-070-audit.json`, `.tools/standalone-070-payload-audit.json`. Tre asset più `SHA256SUMS.txt`, licenza GPL completa e **243 componenti Windows** con avvisi.
- [x] Sorgenti: **431 crate**, **22801 file vendor**, **196 file tracciati** corrispondenti al commit, con sola normalizzazione esatta CRLF → LF dove necessaria. Il controllo offline include **707 file di licenza/avviso**, contando anche `PATENTS`.
- [x] Veri `cargo check --locked --offline --all-targets` dai sorgenti estratti, cache Cargo inizialmente vuota e target isolato: app **66.98 s**, DLL **3.91 s**. Rust **1.99.1** identico alla provenienza e CRT statica. Non è una build completa con linking; nessuna dipendenza scaricata, hash ZIP invariato. Report `.tools/source-offline-070-9e939ce0cc78475b84b6285146e10fda/offline-check.json`.
- [x] MSIX finale **7437834 byte**, SHA-256 `84f2e3561e73f425393312139629edab8d149747380f369580bbe3babbed52d1`: **57 elementi payload**, **59 voci ZIP**, identità/versione, hash, GPL, avvisi e import PE verificati. Report `.tools/store-070-audit.json`.
- [x] Review delle importazioni: app **298 simboli**, DLL **110**, nessuno aggiunto rispetto alla precedente review Windows 10; nessuna dipendenza CRT dinamica o importazione esclusiva di Windows 11 rilevata.
- [x] Partner Center: upload Validated, salvataggio/rilettura schede IT/EN e privacy, invio per certificazione. Prova locale `.tools/store-submission-070-final-state.txt`; il sondaggio facoltativo è stato chiuso senza inviarlo prima della cattura finale.

## Prove non attestate

- [ ] Installazione, aggiornamento e disinstallazione del nuovo MSIX su macchina dedicata.
- [ ] Installazione e menu classico su un host Windows 10 reale; menu moderno del nuovo pacchetto effettivamente registrato su Windows 11.
- [ ] Windows App Certification Kit: **non eseguito**.
- [ ] Approvazione, certificazione completata o pubblicazione Store della 0.7.0: **non ancora ottenute**.

I precedenti record restano storici. Nessun tag, archivio, binario o rapporto di build/audit è riscritto da questo aggiornamento documentale. [Registro tecnico e checksum](store-validation.md).
