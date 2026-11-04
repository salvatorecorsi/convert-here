# Motori esterni: provenienza e scelta di distribuzione

L'installer pubblico e il pacchetto portable di questa release contengono l'app Rust, la CLI e l'estensione Explorer. **Non contengono binari FFmpeg, Poppler, LibreOffice, Tesseract o libheif, ne modelli OCR.** Le conversioni dei formati immagine nativi, immagini in PDF e le operazioni di compressione/divisione/unione PDF sono implementate nell'app. Le altre funzioni usano programmi separati installati dall'utente o scaricati direttamente dai rispettivi distributori.

Il download facoltativo di FFmpeg e Poppler usa versioni fissate e verifica gli SHA-256 prima dell'estrazione. Conserva la documentazione presente negli archivi di origine. LibreOffice, Tesseract, i modelli OCR e libheif vengono configurati separatamente e non sono scaricati automaticamente.

| Motore | Archivio di origine | SHA-256 |
| --- | --- | --- |
| FFmpeg 9.0.2 essentials, Gyan Doshi | [Archivio upstream](https://github.com/GyanD/codexffmpeg/releases/download/9.0.2/ffmpeg-9.0.2-essentials_build.zip) | `60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba` |
| Poppler 26.09.0-0, oschwartz10612/conda-forge | [Archivio upstream](https://github.com/oschwartz10612/poppler-windows/releases/download/v26.09.0-0/Release-26.09.0-0.zip) | `7a6f256a0ddf7536182246a5733331bf4677cbcc34f4663774947ad34556c8d0` |

La build FFmpeg indicata dichiara GPL v3 nel proprio `README.txt` e include il testo in `LICENSE`. Il distributore collega il [commit FFmpeg corrispondente](https://github.com/FFmpeg/FFmpeg/commit/946fcce07b) e descrive le librerie abilitate nella [pagina delle build](https://www.gyan.dev/ffmpeg/builds/).

L'archivio Poppler è un pacchetto indipendente che raccoglie Poppler e DLL dell'ambiente conda-forge risolto. Il [suo script alla release esatta](https://github.com/oschwartz10612/poppler-windows/blob/v26.09.0-0/package.sh) copia tutte le DLL dell'ambiente: le componenti mantengono le rispettive licenze. I riferimenti di origine sono il [progetto Poppler](https://poppler.freedesktop.org/), [poppler-feedstock](https://github.com/conda-forge/poppler-feedstock) e la [release del distributore](https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0).

Per pubblicare in futuro un bundle che ospiti questi binari insieme all'app, la distribuzione deve essere preparata anche con i testi di licenza, gli avvisi e il sorgente corrispondente richiesti per le build e le loro dipendenze, comprese eventuali patch e istruzioni di compilazione. Un solo collegamento al repository FFmpeg non costituisce un inventario completo di tutte le componenti del bundle. La release corrente evita questo ri-impacchettamento e lascia il download ai distributori originali.

Fonti: [condizioni FFmpeg](https://ffmpeg.org/legal.html), [GPL v3, sezione 6](https://www.gnu.org/licenses/gpl-3.0.html#section6), [FAQ GNU sulla redistribuzione di binari invariati](https://www.gnu.org/licenses/gpl-faq.html#UnchangedJustBinary), [MPL 2.0, sezione 3.2](https://www.mozilla.org/en-US/MPL/2.0/).

LibreOffice è facoltativo e non è incluso né scaricato automaticamente da questa release. [Licenze LibreOffice](https://www.libreoffice.org/about-us/licenses/).

## Runtime di verifica locali

Le cartelle `.tools/` sono escluse da Git e dal pacchetto pubblico. I runtime seguenti sono stati estratti nel workspace per provare le funzioni effettive; non sono motori distribuiti insieme all'app e non cambiano le versioni scaricate dagli script pubblici.

### Tesseract e tessdata_fast

- Motore: `tesseract-ocr-w64-setup-5.5.3.20260724.exe` dalla [release ufficiale Tesseract 5.5.3](https://github.com/tesseract-ocr/tesseract/releases/tag/5.5.3), collegata dalla [pagina Windows di UB Mannheim](https://github.com/UB-Mannheim/tesseract/wiki), a sua volta indicata nella [documentazione Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html).
- Estrazione locale: `.tools/tesseract-validation/runtime/`, senza installazione globale o modifica del PATH persistente.
- Modelli: `eng.traineddata` e `ita.traineddata` dal repository ufficiale [tessdata_fast](https://github.com/tesseract-ocr/tessdata_fast), nella cartella `tessdata` del runtime.
- Licenze: [Tesseract Apache-2.0](https://github.com/tesseract-ocr/tesseract/blob/main/LICENSE) e [tessdata_fast Apache-2.0](https://github.com/tesseract-ocr/tessdata_fast/blob/main/LICENSE). Le dipendenze della build, comprese le librerie per immagini, mantengono le licenze della distribuzione di origine.

### libheif

L'app usa la coppia CLI `heif-dec.exe`/`heif-enc.exe` con DLL e plugin della stessa installazione. Le copie MSYS2 estratte sotto `.tools/libheif-validation/` sono soltanto runtime di test; non vengono ri-ospitate in un artefatto pubblico.

La verifica usa **MSYS2 UCRT64 libheif 1.23.7-1** dall'[archivio del distributore](https://mirror.msys2.org/mingw/ucrt64/mingw-w64-ucrt-x86_64-libheif-1.23.7-1-any.pkg.tar.zst), SHA-256 `394f86529164f878b78e505f0585ddfb12b20c55e965467551c3264a54416cc5`. Il runtime comprende 35 dipendenze risolte da MSYS2; database e archivi sono conservati nelle sottocartelle locali `db/` e `packages/`, con la verifica degli archivi eseguita dallo script di preparazione `fetch.py` del workspace. Questa e provenienza dei test, non una nuova distribuzione pubblica.

Riferimenti: [sorgenti libheif](https://github.com/strukturag/libheif), [licenze upstream](https://github.com/strukturag/libheif/blob/master/COPYING), [codec e plugin](https://github.com/strukturag/libheif#supported-features). La libreria e sotto GNU LGPL v3; le applicazioni di esempio e i wrapper sono sotto MIT. Ogni codec e dipendenza conserva la propria licenza; il contenuto effettivo dipende dalla build selezionata.
