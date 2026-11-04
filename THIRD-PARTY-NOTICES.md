# Motori esterni

L'installer e l'archivio portable pubblici contengono l'app Rust e l'estensione Windows, con gli [avvisi delle dipendenze e dei font](docs/release-licenses/README.md). Non contengono binari FFmpeg, Poppler, LibreOffice, Tesseract o libheif, ne modelli OCR.

L'applicazione esegue i motori come processi separati. L'opzione di installazione e lo script `install-runtime-tools.ps1` scaricano gli archivi FFmpeg e Poppler qui indicati direttamente dai rispettivi distributori e ne verificano gli SHA-256. LibreOffice, Tesseract, i modelli OCR e libheif vengono configurati separatamente dall'utente. Le cartelle locali di sviluppo `tools/` e `.tools/` non sono tracciate in Git; i runtime estratti per i test e il bundle locale creato da `build-release.ps1` non sono quelli pubblicati su GitHub.

## FFmpeg 9.0.2 essentials, Windows x64

Build di Gyan Doshi, uno dei distributori Windows collegati dalla [pagina ufficiale FFmpeg](https://ffmpeg.org/download.html).

- [Archivio esatto](https://github.com/GyanD/codexffmpeg/releases/download/9.0.2/ffmpeg-9.0.2-essentials_build.zip).
- SHA-256 verificato: `60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba`.
- Licenza della build: GPL v3, copia in `tools/ffmpeg/LICENSE`.
- Configurazione, librerie e riferimenti sorgenti: `tools/ffmpeg/README.txt`.
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

Tesseract e i modelli delle lingue sono facoltativi, non inclusi e non scaricati automaticamente. L'app usa `tesseract.exe` come processo locale per immagini in TXT e scansioni PDF in TXT/DOCX/HTML; Poppler rasterizza le pagine PDF. I modelli nella cartella `tessdata` restano file separati dal programma.

- [Progetto e sorgenti Tesseract](https://github.com/tesseract-ocr/tesseract), [licenza Apache-2.0](https://github.com/tesseract-ocr/tesseract/blob/main/LICENSE).
- [Documentazione di installazione](https://tesseract-ocr.github.io/tessdoc/Installation.html), con il riferimento ai binari Windows di UB Mannheim.
- [Modelli ufficiali tessdata_fast](https://github.com/tesseract-ocr/tessdata_fast), [licenza Apache-2.0 dei modelli](https://github.com/tesseract-ocr/tessdata_fast/blob/main/LICENSE).

La verifica locale usa Tesseract `5.5.3.20260724` dalla [release Tesseract 5.5.3](https://github.com/tesseract-ocr/tesseract/releases/tag/5.5.3), estratto in `.tools/tesseract-validation/`, e i modelli `eng` e `ita` di `tessdata_fast`. Non costituisce un nuovo bundle pubblico. Le DLL e le altre dipendenze della build Tesseract mantengono le licenze riportate nella distribuzione di origine.

## libheif, HEIC e AVIF

libheif e facoltativo, non incluso e non scaricato automaticamente. L'app invoca `heif-dec.exe` e `heif-enc.exe` come processi separati. DLL, plugin e codec HEVC/AV1 provengono dall'installazione scelta dall'utente.

- [Progetto e sorgenti libheif](https://github.com/strukturag/libheif).
- [Licenze upstream](https://github.com/strukturag/libheif/blob/master/COPYING): libreria sotto GNU LGPL v3; applicazioni di esempio, inclusi gli strumenti CLI, e wrapper sotto MIT.
- [Codec e plugin supportati](https://github.com/strukturag/libheif#supported-features): le librerie codec e le altre dipendenze mantengono le rispettive licenze.

La disponibilita di un encoder o decoder dipende dalla build locale. La verifica usa il pacchetto MSYS2 UCRT64 `libheif 1.23.7-1`, con 35 dipendenze estratte sotto `.tools/libheif-validation/`: serve soltanto ai test e non viene incluso nelle release pubbliche. Archivio e hash esatti: [EXTERNAL-ENGINES.md](docs/release-licenses/EXTERNAL-ENGINES.md).

## Dipendenze Rust

Le versioni esatte sono in `Cargo.lock` e `shell-extension/Cargo.lock`. I testi delle licenze, gli avvisi dei font e del runtime, e il sorgente originale del componente MPL-2.0 `option-ext` sono in `docs/release-licenses/`. Il pacchetto non contiene un'installazione Rust separata. Questi avvisi non assegnano una nuova licenza al codice originale del progetto.
