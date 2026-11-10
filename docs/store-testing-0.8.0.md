# Verifiche candidata 0.8.0 / Store 1.0.8.0

Applicazione **0.8.0**, pacchetto Store previsto **1.0.8.0**, publisher **Salvatore Corsi**. **In preparazione: nessuna pubblicazione o submission della candidata è attestata.** Commit pulito, artefatti finali, digest, schermate e audit del nuovo MSIX vanno aggiunti dopo la build. [English](store-testing-0.8.0.en.md).

L'ultimo stato Store qui richiamato è soltanto storico: il [record 0.7.1 / 1.0.7.0](store-testing-0.7.1.md) riporta **In certification il 9 novembre 2026**. Questo documento non ne aggiorna lo stato. Non sono stati eseguiti WACK, installazione del nuovo MSIX o test su un host Windows 10 reale.

## Additional Testing Information — bozza, non ancora salvata nel portale

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

## Verifiche locali già completate

- **393 test applicazione + 18 test DLL = 411 passati**; Clippy applicazione all-targets, formattazione, Clippy DLL all-targets e build DLL passati. Il precedente test facoltativo degli archivi ZIP in cache resta escluso dal conteggio dei passati.
- COM della DLL reale senza registrazione: preferenze predefinite, personalizzate, vuote e non valide; aggiornamento lingua IT/EN; batch, profili e controlli delle selezioni. Sette percorsi Invoke, compreso TXT audio/video, verificati con un eseguibile che cattura soltanto argomenti.
- Parità di **94 estensioni** fra applicazione, DLL, script e installer; entrambe le fixture incoerenti vengono rifiutate. Il menu classico registra la stessa DLL usata dal test COM.
- Fixture PowerShell: **8 suite PS7 e 6 PS5.1** passate; parser di **25 file** passato. Comprendono **51** verifiche Store, **37** branding, **26** menu sparse, **21** corresponding source, **12** wrapper runtime, **7** roundtrip literal e **6** casi policy installer. Le fixture non installano pacchetti o modificano registrazioni reali; PS5.1 usa il parametro di processo previsto dalla CI, senza cambiare la policy persistente.
- Preparazione reale isolata dei runtime fissati nel catalogo e verifica dell'installazione ripetuta. Tesseract **5.5.4** riconosce ita/eng; libheif **1.23.7** codifica e decodifica HEIC/AVIF senza variabili plugin aggiuntive; Whisper **1.9.5.r258** e Tiny producono TXT. Provata anche una cartella Whisper contenente soltanto il backend CPU generico x64, senza i backend AVX.

Queste prove riguardano sorgenti e runtime locali. Non sostituiscono build e audit degli artefatti finali, installazione del candidato o certificazione Microsoft.

## Provenienza, pin e privacy

Il catalogo pubblico [`portable-packages.json`](../src/runtime_install/portable-packages.json) elenca ogni archivio/modello con versione, byte, URL immutabile o versione fissata, SHA-256 e file selezionati. Le chiusure runtime sono **34 pacchetti Tesseract**, **27 libheif**, **6 Whisper**, oltre ai modelli e relativi avvisi. Totali download: **38.276.135**, **28.215.281** e **87.939.016 byte** rispettivamente; FFmpeg resta separato.

Tiny multilingue proviene dalla revisione Hugging Face **`5359861c739e955e79d9a303bcbc70fb988958b1`**, SHA-256 **`be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`**. I modelli OCR derivano da tessdata_fast **`87416418657359cb625c412a48b6e1d6d41c29bd`**. I pin di tutti i componenti restano nel catalogo, senza sostituirli con un hash unico dell'insieme.

I **17 supplementi originali di licenza**, **158.039 byte**, conservano copyright e provenienza di giflib, libidn2, lz4, libx264 e x265. Sono incorporati tramite [`license_data.rs`](../src/runtime_install/license_data.rs) e verificati prima della copia; `RUNTIME-SOURCES.json` e `LICENSE-SOURCES.json` documentano l'installazione. Licenze e sorgenti dei terzi restano distinti dalla GPL-3.0-only del codice originale. [Inventario motori](release-licenses/EXTERNAL-ENGINES.md), [avvisi](../THIRD-PARTY-NOTICES.md), [privacy aggiornata al 10 novembre 2026](privacy.md).

## Da completare dopo la build

- [ ] Registrare commit pulito, versioni PE/manifest, identità, hash e contenuti di setup, portable, sorgenti e MSIX.
- [ ] Eseguire il controllo offline dei sorgenti vendorizzati finali con cache inizialmente vuota.
- [x] Acquisite [quattro schermate native 0.8.0](store-assets/0.8.0/README.md), PNG 2562 × 1441, con originali JPEG, zoom 150%, hash e provenienza, senza puntatore o ritocchi.
- [ ] Verificare il nuovo MSIX installato, attivazione COM e menu Explorer; separare i risultati per ciascun sistema operativo.
- [ ] Eseguire test su host Windows 10 reale e WACK, oppure mantenere esplicitamente queste verifiche non eseguite.
- [ ] Dopo le verifiche, registrare pubblicazione GitHub, upload/invio Store e stato osservato, senza riutilizzare lo stato della 0.7.1.
