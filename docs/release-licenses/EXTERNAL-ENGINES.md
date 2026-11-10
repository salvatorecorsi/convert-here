# Motori esterni: provenienza e scelta di distribuzione

L'installer pubblico e il pacchetto portable contengono l'app Rust, la CLI e l'estensione Explorer; il MSIX completo contiene GUI ed estensione Explorer. **Non contengono binari FFmpeg, Poppler, LibreOffice, Tesseract, libheif o Whisper, né modelli OCR o di trascrizione.** Le conversioni dei formati immagine nativi, il rendering SVG con resvg, immagini in PDF e le operazioni di compressione/divisione/unione PDF sono implementate nell'app. Le altre funzioni usano programmi separati selezionati dall'utente o scaricati direttamente dai rispettivi distributori. Le dipendenze Rust integrate, incluso resvg, sono inventariate separatamente negli avvisi Rust.

I download facoltativi di FFmpeg, Poppler, Tesseract con i modelli OCR italiano/inglese, libheif e Whisper con Tiny multilingue richiedono una scelta e una conferma nell'app. Versioni, dimensioni e SHA-256 sono fissati nel codice; i motori vengono preparati nel profilo dell'utente, senza installazione globale di MSYS2. Non vengono caricati documenti, immagini o registrazioni audio ai distributori. LibreOffice resta da configurare separatamente.

| Motore | Archivio di origine | SHA-256 |
| --- | --- | --- |
| FFmpeg 9.0.2 essentials, Gyan Doshi | [Archivio upstream](https://github.com/GyanD/codexffmpeg/releases/download/9.0.2/ffmpeg-9.0.2-essentials_build.zip) | `60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba` |
| Poppler 26.09.0-0, oschwartz10612/conda-forge | [Archivio upstream](https://github.com/oschwartz10612/poppler-windows/releases/download/v26.09.0-0/Release-26.09.0-0.zip) | `7a6f256a0ddf7536182246a5733331bf4677cbcc34f4663774947ad34556c8d0` |

La build FFmpeg indicata dichiara GPL v3 nel proprio `README.txt` e include il testo in `LICENSE`. Il distributore collega il [commit FFmpeg corrispondente](https://github.com/FFmpeg/FFmpeg/commit/946fcce07b) e descrive le librerie abilitate nella [pagina delle build](https://www.gyan.dev/ffmpeg/builds/).

L'archivio Poppler è un pacchetto indipendente che raccoglie Poppler e DLL dell'ambiente conda-forge risolto. Il [suo script alla release esatta](https://github.com/oschwartz10612/poppler-windows/blob/v26.09.0-0/package.sh) copia tutte le DLL dell'ambiente: le componenti mantengono le rispettive licenze. I riferimenti di origine sono il [progetto Poppler](https://poppler.freedesktop.org/), [poppler-feedstock](https://github.com/conda-forge/poppler-feedstock) e la [release del distributore](https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0).

Per pubblicare in futuro un bundle che ospiti questi binari insieme all'app, la distribuzione deve essere preparata anche con i testi di licenza, gli avvisi e il sorgente corrispondente richiesti per le build e le loro dipendenze, comprese eventuali patch e istruzioni di compilazione. Un solo collegamento al repository FFmpeg non costituisce un inventario completo di tutte le componenti del bundle. La release corrente evita questo ri-impacchettamento e lascia il download ai distributori originali.

Fonti: [condizioni FFmpeg](https://ffmpeg.org/legal.html), [GPL v3, sezione 6](https://www.gnu.org/licenses/gpl-3.0.html#section6), [FAQ GNU sulla redistribuzione di binari invariati](https://www.gnu.org/licenses/gpl-faq.html#UnchangedJustBinary), [MPL 2.0, sezione 3.2](https://www.mozilla.org/en-US/MPL/2.0/).

LibreOffice è facoltativo e non è incluso né scaricato automaticamente da questa release. [Licenze LibreOffice](https://www.libreoffice.org/about-us/licenses/).

## Pacchetti portabili MSYS2 e modelli

La configurazione verificabile è in [`src/runtime_install/portable-packages.json`](../../src/runtime_install/portable-packages.json): riporta per ogni archivio e modello URL di origine, versione, byte, SHA-256 e file da conservare. Si usano le build native Windows **MSYS2 UCRT64**. La selezione conserva gli eseguibili richiesti e la chiusura delle DLL/plugin effettivamente importati: **34 pacchetti per Tesseract**, **27 per libheif**, **6 per Whisper**. Non installa Python, un ambiente MSYS2 completo o programmi ausiliari estranei a questa chiusura.

| Componente principale | Origine fissata | SHA-256 |
| --- | --- | --- |
| Tesseract OCR 5.5.4-1 | [Pacchetto MSYS2](https://mirror.msys2.org/mingw/ucrt64/mingw-w64-ucrt-x86_64-tesseract-ocr-5.5.4-1-any.pkg.tar.zst) | `a38f3f00d23ff9e05f2e6df87a65ec7f10b6735b131548e87aa15016a79538b2` |
| libheif 1.23.7-1 | [Pacchetto MSYS2](https://mirror.msys2.org/mingw/ucrt64/mingw-w64-ucrt-x86_64-libheif-1.23.7-1-any.pkg.tar.zst) | `394f86529164f878b78e505f0585ddfb12b20c55e965467551c3264a54416cc5` |
| whisper.cpp 1.9.5.r258.g4da27c41c-1 | [Pacchetto MSYS2](https://mirror.msys2.org/mingw/ucrt64/mingw-w64-ucrt-x86_64-whisper.cpp-1~1.9.5.r258.g4da27c41c-1-any.pkg.tar.zst) | `5bf4a4fa5b16f396e87b1cc2fb38ffc73b34008db858f3e12942f5364c75535a` |
| Tiny multilingue, 77691713 byte | [Modello GGML, revisione fissata](https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-tiny.bin) | `be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21` |

I modelli OCR `ita.traineddata` e `eng.traineddata` provengono da [tessdata_fast alla revisione `87416418657359cb625c412a48b6e1d6d41c29bd`](https://github.com/tesseract-ocr/tessdata_fast/tree/87416418657359cb625c412a48b6e1d6d41c29bd), con licenza Apache-2.0 originale. Motore Whisper e modello Tiny hanno testi MIT distinti, conservati entrambi. La trascrizione usa la CPU locale; FFmpeg resta un requisito separato per preparare l'audio. Le modalità di acquisizione non attestano il completamento di una specifica installazione o verifica su Windows 10.

### Conservazione dei testi originali

L'estrazione conserva gli avvisi presenti sotto `share/licenses` dei pacchetti selezionati e il testo ICU collocato upstream in `share/icu/78.3/LICENSE`. Alcuni pacchetti non contengono il testo della licenza nell'archivio binario: per **giflib, libidn2, lz4, libx264 e x265** sono inclusi supplementi originali verificati, scritti nella cartella delle licenze del motore. I relativi byte e SHA-256 sono incorporati in [`license_data.rs`](../../src/runtime_install/license_data.rs); la provenienza viene registrata nel file `LICENSE-SOURCES.json` dell'installazione. Non vengono sostituiti con una licenza generica o con copyright inventati.

I supplementi derivano dagli archivi sorgente esatti, il cui SHA-256 corrisponde ai [PKGBUILD MSYS2 fissati alla revisione `d5117673530326a3879e1f5095b3ad81696b56f2`](https://github.com/msys2/MINGW-packages/tree/d5117673530326a3879e1f5095b3ad81696b56f2). Per x264 si conserva `COPYING` del commit upstream `b35605ace3ddf7c1a5d67a2eb553f034aef41d55`, recuperato dal [mirror dello stesso commit](https://github.com/mirror/x264/blob/b35605ace3ddf7c1a5d67a2eb553f034aef41d55/COPYING). I supplementi includono anche avvisi e intestazioni originali ove il solo `COPYING` non nomina gli autori. Le licenze rimangono quelle dei rispettivi componenti: Apache, MIT, BSD, LGPL, GPL e relative eccezioni secondo i pacchetti e i file interessati.

## Runtime di verifica locali precedenti

Le cartelle `.tools/` sono escluse da Git e dal pacchetto pubblico. I runtime seguenti sono stati estratti nel workspace durante le verifiche precedenti: conservano la provenienza di quelle prove e non descrivono la nuova selezione portabile sopra. Non sono motori distribuiti insieme all'app.

### Tesseract e tessdata_fast

- Motore: `tesseract-ocr-w64-setup-5.5.3.20260724.exe` dalla [release ufficiale Tesseract 5.5.3](https://github.com/tesseract-ocr/tesseract/releases/tag/5.5.3), collegata dalla [pagina Windows di UB Mannheim](https://github.com/UB-Mannheim/tesseract/wiki), a sua volta indicata nella [documentazione Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html).
- Estrazione locale: `.tools/tesseract-validation/runtime/`, senza installazione globale o modifica del PATH persistente.
- Modelli: `eng.traineddata` e `ita.traineddata` dal repository ufficiale [tessdata_fast](https://github.com/tesseract-ocr/tessdata_fast), nella cartella `tessdata` del runtime.
- Licenze: [Tesseract Apache-2.0](https://github.com/tesseract-ocr/tesseract/blob/main/LICENSE) e [tessdata_fast Apache-2.0](https://github.com/tesseract-ocr/tessdata_fast/blob/main/LICENSE). Le dipendenze della build, comprese le librerie per immagini, mantengono le licenze della distribuzione di origine.

### libheif

L'app usa la coppia CLI `heif-dec.exe`/`heif-enc.exe` con DLL e plugin della stessa installazione. Le copie MSYS2 estratte sotto `.tools/libheif-validation/` sono soltanto runtime di test; non vengono ri-ospitate in un artefatto pubblico.

La verifica usa **MSYS2 UCRT64 libheif 1.23.7-1** dall'[archivio del distributore](https://mirror.msys2.org/mingw/ucrt64/mingw-w64-ucrt-x86_64-libheif-1.23.7-1-any.pkg.tar.zst), SHA-256 `394f86529164f878b78e505f0585ddfb12b20c55e965467551c3264a54416cc5`. Il runtime comprende 35 dipendenze risolte da MSYS2; database e archivi sono conservati nelle sottocartelle locali `db/` e `packages/`, con la verifica degli archivi eseguita dallo script di preparazione `fetch.py` del workspace. Questa e provenienza dei test, non una nuova distribuzione pubblica.

Riferimenti: [sorgenti libheif](https://github.com/strukturag/libheif), [licenze upstream](https://github.com/strukturag/libheif/blob/master/COPYING), [codec e plugin](https://github.com/strukturag/libheif#supported-features). La libreria e sotto GNU LGPL v3; le applicazioni di esempio e i wrapper sono sotto MIT. Ogni codec e dipendenza conserva la propria licenza; il contenuto effettivo dipende dalla build selezionata.
