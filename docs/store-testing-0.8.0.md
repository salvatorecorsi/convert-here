# Verifiche 0.8.0 / Store 1.0.8.0

Applicazione **0.8.0**, pacchetto Store **1.0.8.0**, publisher **Salvatore Corsi**, dal commit pulito **`f1f3f95fceed52507ef715fa4ac6893acc63544a`**. Artefatti finali e audit completati; **release GitHub pubblicata il 10 novembre 2026 alle 12:09:39 UTC**. Submission 5 inviata intorno alle **12:10 UTC**; stato osservato **Update in certification**. Il nuovo MSIX era caricato, salvato e Validated prima di inviarlo. [English](store-testing-0.8.0.en.md).

Il **10 novembre 2026** è stata verificata la pubblicazione della precedente **Submission 4 / Store 1.0.7.0**. Il [record 0.7.1](store-testing-0.7.1.md) conserva lo stato storico In certification del 9 novembre. La nuova 1.0.8.0 non è dichiarata approvata o pubblicata. Non sono stati eseguiti WACK, installazione del nuovo MSIX o test su un host Windows 10 reale.

## Additional Testing Information — salvate e rilette nel portale

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

## Verifiche locali già completate

- **400 test applicazione + 18 test DLL = 418 passati**; Clippy applicazione all-targets, formattazione, Clippy DLL all-targets e build DLL passati. Il precedente test facoltativo degli archivi ZIP in cache resta escluso dal conteggio dei passati.
- Guardie SVG prima della conversione usvg: sette regressioni aggiuntive coprono costi espansi di tracciati, testo e attributi, riferimenti `use`/`textPath`, precedenza `href`, ID ambigui e copie CSS o ereditate (`stroke-dasharray`, famiglie font e stili inline). Il budget conservativo moltiplica byte degli attributi/CSS per i nodi espansi; `marker` e `tref` sono rifiutati. Restano verificati casi ordinari renderizzabili.
- COM della DLL reale senza registrazione: preferenze predefinite, personalizzate, vuote e non valide; aggiornamento lingua IT/EN; batch, profili e controlli delle selezioni. Sette percorsi Invoke, compreso TXT audio/video, verificati con un eseguibile che cattura soltanto argomenti.
- Parità di **94 estensioni** fra applicazione, DLL, script e installer; entrambe le fixture incoerenti vengono rifiutate. Il menu classico registra la stessa DLL usata dal test COM.
- Fixture PowerShell: **8 suite PS7 e 6 PS5.1** passate; parser di **25 file** passato. Comprendono **51** verifiche Store, **37** branding, **26** menu sparse, **21** corresponding source, **12** wrapper runtime, **7** roundtrip literal e **6** casi policy installer. Le fixture non installano pacchetti o modificano registrazioni reali; PS5.1 usa il parametro di processo previsto dalla CI, senza cambiare la policy persistente.
- Preparazione reale isolata dei runtime fissati nel catalogo e verifica dell'installazione ripetuta. Tesseract **5.5.4** riconosce ita/eng; libheif **1.23.7** codifica e decodifica HEIC/AVIF senza variabili plugin aggiuntive; Whisper **1.9.5.r258** e Tiny producono TXT. Provata anche una cartella Whisper contenente soltanto il backend CPU generico x64, senza i backend AVX.
- Download reali con **WinHTTP nativo di produzione**, senza iniezione della cache: Tesseract, libheif e Whisper completati in radici isolate, con byte osservati uguali ai pin, tutti gli hash del payload verificati e seconda installazione senza rete. Annullamento durante uno stream MSYS2 dopo **65.536 byte**: workspace parziale rimosso, resta soltanto il lock previsto. Il record locale `.tools/runtime-research-080/native-winhttp-qa.json` (SHA-256 `982cdca8217d549da72d918d46887072824c9ae429a29d7201e66a37dfbd9227`) registra sorgenti di produzione copiate byte per byte e adattatore QA separato; questa prova non esegue i motori scaricati e non attesta installazione GUI o Store.

Queste prove riguardano sorgenti e runtime locali e sono distinte dagli audit finali descritti sotto. Non attestano installazione del nuovo MSIX o certificazione Microsoft.

## Provenienza, pin e privacy

Il catalogo pubblico [`portable-packages.json`](../src/runtime_install/portable-packages.json) elenca ogni archivio/modello con versione, byte, URL immutabile o versione fissata, SHA-256 e file selezionati. Le chiusure runtime sono **34 pacchetti Tesseract**, **27 libheif**, **6 Whisper**, oltre ai modelli e relativi avvisi. Totali download: **38.276.135**, **28.215.281** e **87.939.016 byte** rispettivamente; FFmpeg resta separato.

Tiny multilingue proviene dalla revisione Hugging Face **`5359861c739e955e79d9a303bcbc70fb988958b1`**, SHA-256 **`be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`**. I modelli OCR derivano da tessdata_fast **`87416418657359cb625c412a48b6e1d6d41c29bd`**. I pin di tutti i componenti restano nel catalogo, senza sostituirli con un hash unico dell'insieme.

I **17 supplementi originali di licenza**, **158.039 byte**, conservano copyright e provenienza di giflib, libidn2, lz4, libx264 e x265. Sono incorporati tramite [`license_data.rs`](../src/runtime_install/license_data.rs) e verificati prima della copia; `RUNTIME-SOURCES.json` e `LICENSE-SOURCES.json` documentano l'installazione. Licenze e sorgenti dei terzi restano distinti dalla GPL-3.0-only del codice originale. [Inventario motori](release-licenses/EXTERNAL-ENGINES.md), [avvisi](../THIRD-PARTY-NOTICES.md), [privacy aggiornata al 10 novembre 2026](privacy.md).

## Artefatti finali e audit

Tutti gli artefatti seguenti derivano dal commit pulito **`f1f3f95fceed52507ef715fa4ac6893acc63544a`**. I quattro file GitHub sono pubblicati nella [release v0.8.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.0), senza stato draft; i digest remoti coincidono con quelli locali.

| File | Byte | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.8.0-windows-x64-setup.exe` | 15.685.660 | `1959d372be6210cf9107bd2a39ec80c41ab2a6474c213fd2e07065a5e7cb9f73` |
| `ConvertHere-0.8.0-windows-x64-portable.zip` | 20.700.405 | `6b59b12294a8b95c038b3d35fc20184792e333121a36a395f4335fe6cbc23d33` |
| `ConvertHere-0.8.0-source.zip` | 89.286.003 | `ba3e2695a3a53bbdcbaf216c275e3c98f2ee0c138e0d0e90b01aa6cebe64eea9` |
| `SHA256SUMS.txt` | 313 | `8389e4be7d8fcd21e060fc7fa5977446020b2f12ba986861ea2513055dcb92f7` |
| `ConvertHere-1.0.8.0-x64-store.msix` | 9.342.790 | `a7a010371015a69b798e255b373b90f6cfddf4fed4b5611bc7cd845f00c841f9` |

L'audit standalone conferma **98 file** del portable byte per byte uguali allo staging, tre binari corrispondenti a `build-info.json`, EXE **0.8.0**, publisher **Salvatore Corsi**, x64 e CRT statica. L'allowlist esclude motori opzionali, dati personali e file di sviluppo. Il setup non è stato eseguito da questo audit.

L'audit Store conferma **57 elementi payload / 59 voci ZIP**, GUI e DLL senza CLI, motori, modelli o script downloader; **277 componenti di dipendenze**, GPL completa e intero albero delle licenze corrispondente al commit. Identità **SalvatoreCorsi.ConvertHere**, publisher tecnico **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17**, MinVersion **10.0.19041.0**, x64 e sola capability **runFullTrust** sono invariati. Importazioni CRT dinamiche assenti; manifest COM surrogate/STA, CLSID e verbo **0000ConvertHere** verificati. Questa verifica non installa il pacchetto e non carica la DLL COM. Report locali: `.tools/standalone-080-audit.json`, `.tools/standalone-080-payload-audit.json` e `.tools/store-080-audit.json`.

## Sorgenti offline e prove dei binari finali

Sono passati veri **`cargo check --locked --offline --all-targets`** per app e DLL dai sorgenti estratti, con cache Cargo e target inizialmente vuoti: **75,56 s** e **4,02 s**, CRT statica e compilatore uguale alla provenienza. Verificati **465 crate**, **24.338 file vendor** e **228 file tracciati**; per 192 file è stata necessaria soltanto la normalizzazione esatta CRLF → LF. Non è una build completa con linking. I contatori di file licenza dei due helper includono insiemi diversi, perciò non vengono sommati o usati come conteggio delle 277 dipendenze Windows.

Record locale: `.tools/source-offline-080-91eb8afd23cd48fd9083810eaa844be3/offline-check.json`, SHA-256 **`d7fe4861f7d70b6f068df3fadaf499e2c53ed99126e667b15be974a359fc6d74`**. La cache finale contiene soltanto `CACHEDIR.TAG`, senza sorgenti o pacchetti scaricati.

Smoke test dei binari finali: **13/13 passati**, comprese conversioni SVG, trascrizione audio/video, taglio/gain, preservazione degli originali e conflitti di nome senza sovrascrittura. Il segmento audio misura **0,75 s** e il rapporto RMS misurato è **0,5001102921**. Preferenze utente reali non usate e nessun download durante questi smoke test. Record `.tools/release-smoke-080-529f1ecefd0145c8a08b6bb0fd65a1e6/report.json`, SHA-256 **`4ded74939cc47abb0761f12381416338558daf523d53359425c1d197fa4dd468`**.

## Stato salvato nel portale e verifiche residue

**Submission 5**, ID **1152921505702365703**: MSIX 1.0.8.0 caricato e salvato come Validated, poi inviato il **10 novembre 2026 intorno alle 12:10 UTC**. Il riepilogo mostra **Update in certification**. Non vengono dedotte sottofasi di elaborazione, certificazione o pubblicazione non osservate. Schede IT/EN salvate e rilette con **20 funzionalità e quattro PNG per lingua**; privacy e licenza puntano al tag pubblico **v0.8.0**. Confermati prezzo gratuito, **240 mercati** e pubblicazione appena approvata. La voce dell'accordo Windows **8.11** risultava già accettata il 4 novembre, **Action: N/A**; nessuna nuova accettazione eseguita. Le verifiche DOM locali sono `.tools/store-listing-080-it-verified.json` e `.tools/store-listing-080-en-verified.json`.

- [x] Audit di commit, versioni, identità, checksum e payload finali.
- [x] Controlli offline dei sorgenti con cache e target inizialmente vuoti.
- [x] Smoke test dei binari finali, 13/13.
- [x] Acquisite [quattro schermate native 0.8.0](store-assets/0.8.0/README.md), PNG 2562 × 1441, con originali JPEG, zoom 150%, hash e provenienza, senza puntatore o ritocchi.
- [x] Caricati e salvati MSIX Validated, schede IT/EN e relative immagini.
- [ ] Verificare il nuovo MSIX installato, attivazione COM e menu Explorer; separare i risultati per ciascun sistema operativo.
- [ ] Eseguire test su host Windows 10 reale e WACK, oppure mantenere esplicitamente queste verifiche non eseguite.
- [x] [Windows checks 42811925135](https://github.com/salvatorecorsi/convert-here/actions/runs/42811925135): **SUCCESS**, job verify **9 min 49 s**; log locale `.tools/ci-080-watch.log`.
- [x] Pubblicata la release GitHub alle **12:09:39 UTC**, inviata la Submission 5 e registrato **Update in certification**. La nuova Store 1.0.8.0 non è dichiarata approvata o pubblica; resta pubblicata la precedente 1.0.7.0.

Note finali di test (**3.207 caratteri nel portale**) salvate e rilette; record locale `.tools/store-notes-080-verified.txt`. Evidenza del comando di invio: `.tools/store-080-submission-state.txt` e `.tools/store-080-submission-proof.jpg`. Mostrano intestazione di certificazione; non vengono attestate altre fasi della timeline.
