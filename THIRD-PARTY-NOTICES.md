# Licenza del progetto e componenti di terze parti

Il codice originale di Convert Here e distribuito sotto **GPL-3.0-only**, Copyright (C) 2026 Salvatore Corsi. Il testo integrale e in [LICENSE](LICENSE), i [sorgenti del progetto](https://github.com/salvatorecorsi/convert-here) sono pubblici. Le release includono un archivio sorgente con il commit corrispondente, entrambi i lockfile, dipendenze Cargo vendorizzate e istruzioni di build offline. Dipendenze, font e motori esterni mantengono i propri copyright e le proprie licenze; la GPL del codice originale non sostituisce questi avvisi.

## Codec WebP integrato

L'editor usa [libwebp-sys 0.14.4](https://crates.io/crates/libwebp-sys/0.14.4), collegato staticamente alla libreria C **libwebp 1.6.0** contenuta nel pacchetto, per la codifica WebP con qualita regolabile. Non e un motore da scaricare: la codifica avviene nel processo locale e non invia immagini a un servizio remoto.

Il pacchetto Cargo dichiara **MIT** per il wrapper Rust. L'archivio pubblicato non include un avviso MIT autonomo del wrapper: il collector conserva gli autori dichiarati, l'URL del pacchetto esatto e il commit VCS, e aggiunge il [testo standard SPDX della licenza dichiarata](release-support/licenses/README.md), identificato esplicitamente come tale. Non attribuisce un anno o un titolare di copyright ai segnaposto del testo standard.

La libreria C mantiene i file originali `vendor/COPYING` (BSD a tre clausole, copyright Google) e `vendor/PATENTS` (concessione aggiuntiva dei diritti sui brevetti); entrambi sono inclusi negli avvisi raccolti. Anche `glob`, dipendenza di compilazione, conserva la propria licenza. L'inventario e generato dai lockfile effettivi, senza dedurre il numero dei componenti da questa descrizione.

## Motori esterni

L'installer e l'archivio portable pubblici contengono l'app Rust e l'estensione Windows, con gli [avvisi delle dipendenze e dei font](docs/release-licenses/README.md). Non contengono binari FFmpeg, Poppler, LibreOffice, Tesseract, libheif o Whisper, ne modelli OCR o di trascrizione.

L'applicazione esegue i motori come processi separati. L'utente puo richiedere FFmpeg, Poppler, Tesseract con italiano e inglese, libheif oppure Whisper con il modello Tiny multilingue da **Impostazioni → Motori**, dopo conferma di fonte, versione, dimensioni, licenze e cartella. Il downloader nativo Windows usa HTTPS e verifica dimensioni e SHA-256 di tutti i componenti prima dell'estrazione nella cartella personale `%LOCALAPPDATA%\ConvertHere\tools`. Conserva licenze, documentazione e DLL delle distribuzioni originali, con eventuali avvisi supplementari indicati nel catalogo. Non carica i file da convertire. Non viene avviato dal setup e non esegue i programmi scaricati durante l'installazione.

La CLI separata offre `--install-runtime ffmpeg|poppler|tesseract|libheif|whisper`. Lo script facoltativo `install-runtime-tools.ps1` richiama questa CLI soltanto quando avviato esplicitamente, usando la stessa cartella personale; non e un secondo downloader. LibreOffice continua a essere selezionato da un'installazione locale. Le cartelle locali di sviluppo `tools/` e `.tools/` non sono tracciate in Git; i runtime estratti per i test e il bundle locale creato da `build-release.ps1` non sono quelli pubblicati su GitHub.

Il [catalogo dei pacchetti portabili](src/runtime_install/portable-packages.json) conserva URL, versioni, dimensioni, SHA-256 e licenze dei singoli componenti di Tesseract, libheif e Whisper. Non equivale a un archivio sorgente dei motori: le distribuzioni e i relativi sorgenti restano presso i distributori indicati.

## Rendering SVG integrato

Gli SVG vengono rasterizzati localmente con `resvg`/`usvg` e `tiny-skia`, usando il font Ubuntu gia incorporato da egui. Non vengono scaricati font o risorse del documento. Versioni, licenze MIT/Apache/BSD e avvisi originali sono nell'inventario Rust generato dai lockfile. Non sono inclusi programmi SVG esterni.

## FFmpeg 9.0.2 essentials, Windows x64

Build di Gyan Doshi, uno dei distributori Windows collegati dalla [pagina ufficiale FFmpeg](https://ffmpeg.org/download.html).

- [Archivio esatto](https://github.com/GyanD/codexffmpeg/releases/download/9.0.2/ffmpeg-9.0.2-essentials_build.zip).
- SHA-256 verificato: `60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba`.
- Licenza della build: GPL v3, copia nella cartella installata `ffmpeg/LICENSE`.
- Configurazione, librerie e riferimenti sorgenti: `ffmpeg/README.txt`, conservato nell'installazione.
- [Sorgente FFmpeg della build](https://github.com/FFmpeg/FFmpeg/commit/946fcce07b).
- [Distribuzione e informazioni sui sorgenti della build](https://www.gyan.dev/ffmpeg/builds/).

## Poppler 26.09.0, Windows x64

Distribuzione binaria indipendente di oschwartz10612, basata su pacchetti conda-forge; non è una build ufficiale del progetto Poppler.

- [Release e archivio esatto](https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0).
- SHA-256 verificato: `7a6f256a0ddf7536182246a5733331bf4677cbcc34f4663774947ad34556c8d0`.
- [Progetto e sorgenti Poppler](https://poppler.freedesktop.org/), distribuito sotto GPL.
- [Ricetta, dipendenze e licenze della build conda-forge](https://github.com/conda-forge/poppler-feedstock).
- Le DLL scaricate hanno anche le rispettive licenze. Le release pubbliche di Convert Here non ri-ospitano questo bundle; i riferimenti upstream e le modalità di acquisizione sono in [EXTERNAL-ENGINES.md](docs/release-licenses/EXTERNAL-ENGINES.md).

## LibreOffice

Non incluso. L'app usa un'installazione disponibile sul computer. [Sorgenti e licenze](https://www.libreoffice.org/about-us/licenses/).

## Tesseract e modelli OCR

Tesseract e i modelli delle lingue sono facoltativi e non inclusi nei pacchetti dell'app. Il pulsante **Installa** nella relativa card scarica la distribuzione portabile MSYS2 UCRT64, le dipendenze e i modelli ufficiali `ita`/`eng` di `tessdata_fast`. L'app usa `tesseract.exe` come processo locale per immagini in TXT e scansioni PDF in TXT/DOCX/HTML; Poppler rasterizza le pagine PDF. I modelli nella cartella `tessdata` restano file separati dal programma.

- [Progetto e sorgenti Tesseract](https://github.com/tesseract-ocr/tesseract), [licenza Apache-2.0](https://github.com/tesseract-ocr/tesseract/blob/main/LICENSE).
- [Documentazione di installazione](https://tesseract-ocr.github.io/tessdoc/Installation.html), con il riferimento ai binari Windows di UB Mannheim.
- [Modelli ufficiali tessdata_fast](https://github.com/tesseract-ocr/tessdata_fast), [licenza Apache-2.0 dei modelli](https://github.com/tesseract-ocr/tessdata_fast/blob/main/LICENSE).

Le DLL e le altre dipendenze della build Tesseract mantengono le licenze riportate nella distribuzione di origine. Il catalogo identifica gli archivi esatti e gli avvisi supplementari; le copie estratte in `.tools/` servono ai test e non fanno parte delle release pubbliche.

## libheif, HEIC e AVIF

libheif e facoltativo e non incluso nei pacchetti dell'app. L'app invoca `heif-dec.exe` e `heif-enc.exe` come processi separati. Il pulsante **Installa** scarica la distribuzione MSYS2 UCRT64 con DLL, plugin e codec HEVC/AV1. Si puo anche selezionare un'installazione locale compatibile.

- [Progetto e sorgenti libheif](https://github.com/strukturag/libheif).
- [Licenze upstream](https://github.com/strukturag/libheif/blob/master/COPYING): libreria sotto GNU LGPL v3; applicazioni di esempio, inclusi gli strumenti CLI, e wrapper sotto MIT.
- [Codec e plugin supportati](https://github.com/strukturag/libheif#supported-features): le librerie codec e le altre dipendenze mantengono le rispettive licenze.

La disponibilita di un encoder o decoder dipende dalla build locale. Archivi e hash esatti sono nel catalogo dei pacchetti portabili; i riferimenti ai distributori sono in [EXTERNAL-ENGINES.md](docs/release-licenses/EXTERNAL-ENGINES.md).

## Whisper e modello Tiny multilingue

La trascrizione usa il programma locale [whisper.cpp](https://github.com/ggml-org/whisper.cpp) con il [modello Tiny multilingue](https://huggingface.co/ggerganov/whisper.cpp). Entrambi sono facoltativi e conservano le proprie licenze MIT e gli avvisi originali. Il modello viene scaricato soltanto su richiesta, verificato e conservato come file locale; non viene interrogato un servizio di trascrizione remoto. FFmpeg prepara l'audio sul PC. Il catalogo riporta le revisioni e i file di licenza esatti del programma, del modello e delle dipendenze.

## Dipendenze Rust

Le versioni esatte sono in `Cargo.lock` e `shell-extension/Cargo.lock`. I testi delle licenze, gli avvisi dei font e del runtime, e il sorgente originale del componente MPL-2.0 `option-ext` sono in `docs/release-licenses/`. Il pacchetto non contiene un'installazione Rust separata. I sorgenti Cargo vendorizzati dell'archivio sorgente conservano i rispettivi file di licenza e metadati di checksum. Questi avvisi descrivono i componenti terzi; la licenza del codice originale del progetto e indicata separatamente in [LICENSE](LICENSE).
