# Conversioni e formati

**Italiano** · [English](conversions.en.md)

Guida alle destinazioni, alle opzioni e ai limiti di Convert Here. Per iniziare, consulta il [README](../README.md); per impostare gli strumenti esterni, consulta [Motori e preferenze](configuration.md).

Questa guida include le novità della **candidata 0.8.0, non ancora pubblicata**: SVG, timeline audio/video, trascrizione locale e ulteriori motori scaricabili. Le indicazioni «in sviluppo» si riferiscono a questa candidata e non modificano lo stato delle release collegate nel README.

Il ramo di sviluppo riconosce **94 estensioni di ingresso** e mantiene **44 formati di destinazione**. In **Impostazioni → Formati di uscita** scegli le uscite da mostrare, raggruppate per audio, video, immagini, documenti e **Raw**. Il default resta **MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4**; abilita **TXT** in Raw per OCR o trascrizione. Le scelte restano salvate e il menu le rilegge senza riavviare Explorer. Se non abiliti alcuna uscita, resta accessibile **Impostazioni personalizzate…**. Il filtro riguarda soltanto le uscite visibili: non disabilita ingressi, CLI, richieste `--quick` esplicite o lavori già accodati. Le destinazioni effettive dipendono dalla categoria del file e dai codec dei motori selezionati.

| Sorgente | Destinazioni | Motore |
| --- | --- | --- |
| Audio: MP3, MPGA, WAV, FLAC, AAC, M4A/M4B, OGG/OGA, Opus, AIFF/AIF/AIFC, WMA, ALAC, APE, MKA, MP2, WebA, AC3, AU, CAF, AMR, DTS, TTA, VOC, WV, TAK, W64, RA | MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A (AAC), WMA, AC3, AU, CAF (ALAC) | FFmpeg |
| Video: MP4, MKV, MOV, WebM, AVI, WMV, MPEG/MPG, M4V, TS/MTS/M2TS/M2T, 3GP/3G2, FLV, ASF, VOB, OGV, M2V, MXF, F4V, DivX, MPE, MPV | MP4, MKV, MOV, WebM, AVI, GIF animato, MPEG, WMV, FLV, TS, M4V, 3GP; estrazione audio nei dodici formati sopra | FFmpeg |
| Immagini native: PNG, JPEG/JPG, WebP, BMP, TIFF/TIF, GIF, ICO, QOI, TGA, Farbfeld (.ff), OpenEXR, HDR, PNM/PAM/PBM/PGM/PPM | PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, PDF, QOI, TGA, Farbfeld, OpenEXR, HDR, PNM; HEIC, AVIF; TXT con OCR | Rust per i formati nativi e PDF; libheif per creare HEIC/AVIF; Tesseract per TXT |
| HEIC/HEIF, AVIF | Destinazioni immagine sopra, PDF e TXT con OCR | libheif per la lettura; Tesseract per TXT |
| SVG statico UTF-8, in sviluppo | Formati immagine nativi e PDF dopo rasterizzazione; HEIC/AVIF o TXT con gli stessi motori opzionali delle altre immagini | resvg integrato; nessun motore esterno per raster/PDF |
| Audio/video → testo, in sviluppo | TXT dal parlato | FFmpeg + Whisper + modello Tiny multilingue, tutti locali |
| PDF | PNG/JPEG, una immagine per pagina; TXT, Word DOCX e HTML dal testo incorporato o con OCR | Poppler per immagini e testo incorporato; Tesseract + Poppler per OCR; DOCX e HTML ricostruiti dall'app |
| PDF | PDF compresso senza perdita, un PDF per pagina, unione di più PDF | Rust, senza motori esterni |
| DOC/DOCX, ODT, RTF, TXT, PPT/PPTX, ODP | PDF | LibreOffice |
| XLS/XLSX/XLSM, ODS, CSV | CSV, Excel XLSX, PDF | LibreOffice |
| HTML/HTM | PDF | LibreOffice |

L'estensione determina la categoria proposta; il motore verifica il contenuto durante la conversione. File protetti, corrotti o con codec non disponibili producono un errore leggibile.

Impostazioni disponibili: preset leggero/bilanciato/alta qualità, bitrate, frequenza e canali audio, normalizzazione, H.264/H.265, qualità video, larghezza, FPS, taglio temporale, qualità JPEG, ridimensionamento immagini e DPI dei PDF. L'interfaccia mostra solo i controlli pertinenti. Per WebM si usa VP9/Opus, per AVI MPEG-4/MP3, per MPEG MPEG-2/MP2, per WMV WMV2/WMA2, per FLV FLV1/MP3, per 3GP MPEG-4/AAC, per TS H.264/AAC. M4V è un contenitore MP4 con audio; WAV, AIFF e AU sono PCM 16 bit, CAF usa ALAC senza perdita. I PDF creati da immagini sono A4 con proporzioni mantenute e sfondo bianco.

## Editor immagini e impaginazione

L'editor aggiunge ritaglio, scala, tela e qualità JPEG/WebP regolabile, con anteprima e peso effettivo dei byte preparati. Esporta PNG/JPEG/WebP/PDF; per JPEG/WebP può cercare una qualità entro il peso massimo, con riduzione ulteriore delle dimensioni solo se autorizzata. Il limite non raggiungibile impedisce il salvataggio e fa fallire la conversione rapida senza produrre un file oltre il limite.

Immagini, TXT UTF-8 e testo digitato possono essere impaginati nativamente su A4/A5 con margini e orientamento. Questo percorso TXT si attiva con le opzioni pagina; il percorso TXT tradizionale e gli altri documenti Office mantengono LibreOffice. L'editor non modifica il contenuto completo di PDF o Word. [Guida editor e profili](image-editor-profiles.md).

## SVG nativo — in sviluppo

Gli SVG vengono rasterizzati localmente con **resvg 0.49** prima di usare i normali encoder immagine o il percorso PDF. Non vengono esportati SVG né conservati i vettori nel PDF risultante. Il testo usa il font Ubuntu incorporato: font o funzioni non supportati possono cambiare l'aspetto rispetto all'app di origine.

Sono accettati SVG UTF-8 non compressi fino a **4 MiB**, con un'immagine di massimo **40 milioni di pixel** e **32768 pixel per lato**. SVGZ, DTD, script, `foreignObject`, `marker`, `tref`, risorse esterne e immagini raster incorporate vengono rifiutati. Non vengono scaricati font, immagini o CSS. Prima della conversione il renderer limita nodi, tracciati e testo anche dopo le espansioni `use` e i riferimenti `textPath`; per prudenza, il budget di **4 MiB** per attributi e CSS considera ogni valore applicato a tutti i nodi espansi, inclusa l'ereditarietà. Il renderer limita inoltre la memoria di lavoro e rifiuta i documenti troppo complessi.

## Editor audio/video — in sviluppo

Apri **Modifica…** accanto al file. La timeline mostra la forma d'onda e, per i video, una striscia di fotogrammi della sorgente. Le anteprime precedono gli effetti; su file lunghi o non decodificabili possono non essere disponibili, lasciando utilizzabili i tempi numerici.

Trascina inizio/fine oppure usa **←/→** sulla timeline per passi di 0,1 secondi; **Spazio** cambia maniglia, **Maiusc** usa passi di un secondo e **Alt** di 0,01 secondi. **Home/Fine** porta il bordo al limite. Puoi anche inserire i secondi nei controlli. Il gain va da **−60 a +20 dB**, con muto opzionale: è un moltiplicatore di volume, non un limiter, e valori positivi possono saturare il segnale; se la normalizzazione è attiva, viene applicato dopo di essa.

**Salva con nome…** converte il solo file aperto, conserva gli originali e non sovrascrive una destinazione esistente. **Usa queste impostazioni** applica i parametri alle impostazioni attive; il salvataggio di un profilo conserva taglio, gain e opzioni. Non sono presenti riproduzione o montaggio multitraccia.

## Trascrizione locale audio/video → TXT — in sviluppo

Abilita **TXT** in **Impostazioni → Formati di uscita → Raw** e scegli TXT per un audio/video. Servono **FFmpeg/ffprobe**, **Whisper CPU** e il modello **Tiny multilingue**, di **77.691.713 byte**. Runtime e modello richiedono circa **88 MB**, separati da FFmpeg. La richiesta Whisper dall'app include automaticamente FFmpeg se manca FFmpeg o ffprobe; da CLI installa separatamente i motori necessari. Non servono account, chiavi API o invio dei file a un servizio cloud.

Nell'editor TXT puoi scegliere il segmento e la lingua, automatica per default, oppure italiano, inglese o un altro codice supportato; sono configurabili anche i thread. Le ricette JSON conservano `speech.language` e `speech.threads` insieme a `video.trim_start`/`trim_end`. In questa modalità gain, muto e `media_options` non vengono applicati; rimuovi le opzioni encoder da un JSON TXT manuale.

Il riconoscimento divide l'intervallo in segmenti di massimo **dieci minuti**, con WAV mono 16 kHz e TXT temporanei locali rimossi al termine o all'annullamento; un arresto improvviso può lasciare residui. I segmenti vengono riuniti in un TXT, senza trascrizione del contenuto visivo del video. Errori di riconoscimento sono possibili e il risultato va riletto; se non viene riconosciuto parlato, non viene pubblicato un file vuoto. La durata massima del profilo si applica insieme al taglio; resta attivo anche il limite di peso finale.

## Excel, CSV, Word e HTML

Excel e CSV vengono convertiti con un profilo LibreOffice temporaneo separato, che disabilita macro e aggiornamento dei collegamenti Office. L'esportazione CSV produce **il primo foglio**, in UTF-8 con delimitatore virgolette; conserva i valori visualizzati, senza formule, formattazione o altri fogli. Puoi scegliere **virgola, punto e virgola, tab o pipe**, con la stessa impostazione per importazione ed esportazione. La virgola è il default. L'importazione conserva come testo i campi quotati, incluse le cifre iniziali degli identificativi, e non valuta le formule. Le altre codifiche vanno prima convertite in UTF-8; il separatore non viene indovinato automaticamente.

PDF → Word/HTML ricostruisce **solo il testo**, conservando la separazione delle pagine in un singolo file modificabile. Di default usa il testo incorporato; puoi attivare OCR locale per riconoscerlo dalle scansioni. Immagini, tabelle, font e impaginazione originali non vengono ricostruiti. L'HTML esportato contiene il testo escapato e gli stili nel file, senza risorse esterne.

HTML → PDF usa il motore di LibreOffice per documenti HTML: supporta HTML/CSS di base e non esegue JavaScript. Non sostituisce la stampa di una pagina dinamica da browser; l'impaginazione può cambiare. Durante l'apertura di HTML, il motore può leggere risorse collegate locali o remote, ad esempio immagini e CSS.

## Immagini HEIC e AVIF

Puoi leggere HEIC/HEIF e AVIF, convertirli nei formati immagine nativi o in PDF, e crearli dalle immagini supportate. In **Impostazioni → Motori** seleziona `heif-dec.exe` per leggere e `heif-enc.exe` per creare questi formati, oppure usa il download portabile aggiunto nel ramo di sviluppo. Gli strumenti di [libheif](https://github.com/strukturag/libheif) sono facoltativi e separati dall'app: mantieni insieme eseguibili, DLL e plugin. La build deve includere i codec HEVC/AV1 necessari. Puoi regolare larghezza e qualità; dei contenitori multipagina viene usata la prima immagine, senza conservare sequenze o animazioni.

Gli altri formati immagine sono nativi e non richiedono FFmpeg. HDR non conserva la trasparenza, che viene composta su bianco. OpenEXR e HDR possono conservare valori a virgola mobile; convertirli in formati ordinari può ridurne precisione e gamma dinamica. JPEG XL, RAW fotografici e DDS non sono attualmente inclusi. I codec audio/video disponibili dipendono dalla build FFmpeg selezionata.

## OCR locale di immagini e scansioni PDF

Per un'immagine scegli **TXT · OCR locale**. Per un PDF scegli TXT, Word DOCX o HTML e abilita **OCR locale**: il riconoscimento è disattivato per default e, quando attivo, elabora tutte le pagine invece di estrarre il testo incorporato. Sono necessari [Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html) e i [modelli delle lingue](https://github.com/tesseract-ocr/tessdata_fast); i PDF richiedono anche `pdftoppm.exe` di Poppler. In **Impostazioni → Motori** indica `tesseract.exe` e conserva accanto la cartella `tessdata` con i modelli richiesti, per esempio `ita.traineddata` e `eng.traineddata`.

Il ramo di sviluppo aggiunge **Installa Tesseract** con i modelli italiano/inglese già compresi nel download; non occorre un setup manuale. Le altre lingue restano configurabili con i rispettivi modelli locali.

La lingua predefinita è `ita+eng`; puoi scegliere altri codici Tesseract installati. I DPI PDF determinano la risoluzione delle pagine da riconoscere: 300 DPI è un punto di partenza utile. Le immagini vengono orientate secondo EXIF e la trasparenza viene composta su bianco. Il riconoscimento e il testo restano sul PC. Il riconoscimento può sbagliare e va riletto; non ricostruisce l'impaginazione e non aggiunge uno strato ricercabile al PDF originale. Se non viene riconosciuto testo, l'app segnala l'errore senza creare un risultato vuoto.

## Comprimi, dividi e unisci PDF

Per PDF già esistenti, **PDF (Bulk)** comprime senza perdita i flussi senza ricampionare le immagini e crea `nome-compresso.pdf`; se la riscrittura sarebbe più grande, salva una copia dei byte originali. Il risultato non supera le dimensioni del file letto e può restare della stessa dimensione. **Dividi: una pagina per file** produce `nome-pagina-001.pdf`, `nome-pagina-002.pdf` e così via.

Per unire più file compatibili, scegli **PDF (Merge)** dal menu contestuale oppure **PDF (Merge) · ordina pagine** nell'app e avvia **Ordina e unisci PDF**. L'app prepara le sorgenti e mostra tutte le pagine, identificate dal file e dal numero originale. Trascina la maniglia di una pagina, usa le frecce o **Alt+↑ / Alt+↓** per spostarla: puoi intercalare pagine di PDF multipagina, immagini e documenti diversi. L'ordine iniziale segue i file ricevuti da Esplora file; controllalo nell'editor. **Crea PDF unico** salva `primo-unito.pdf`, aggiungendo un numero se esiste già. Tutte le pagine vengono incluse una volta, fino a 10.000.

La preparazione usa copie PDF temporanee in una cartella utente nascosta, senza PDF intermedi nella destinazione. Gli originali restano invariati; il salvataggio usa la versione preparata delle pagine. L'ordine è una sessione locale separata dalla coda rapida. Annullamento e chiusura eliminano gli intermedi; un arresto improvviso può lasciare residui.

Compressione, divisione e unione di PDF esistenti, e l'unione di immagini native, non richiedono motori esterni. Per preparare Office/Excel/CSV/HTML serve LibreOffice; per leggere HEIC/AVIF serve libheif. Poppler non è necessario per questa unione. I PDF cifrati o firmati vengono rifiutati; unione e divisione rifiutano anche moduli interattivi. I contenuti delle pagine vengono mantenuti, mentre segnalibri, struttura accessibile/tag e metadati del documento non vengono trasferiti. La compressione mantiene il catalogo e i moduli; non è una rimozione dei metadati. Non è disponibile la modifica libera del contenuto delle pagine.

## Testo da immagini e PDF

La destinazione TXT è disponibile per le immagini tramite OCR locale e per i PDF tramite estrazione del testo incorporato o OCR locale. I motori richiesti si selezionano nella sezione Motori delle Impostazioni; il riconoscimento e il risultato restano sul PC. Dettagli sui dati locali e sui motori: [privacy](privacy.md).

## Limiti espliciti

- OCR richiede Tesseract e i modelli delle lingue; per i PDF serve anche Poppler. È riconoscimento del testo, senza ricostruzione della grafica o del layout.
- Le immagini GIF animate e i TIFF multipagina sono letti dal primo fotogramma/pagina. La conversione da video a GIF conserva l'animazione.
- La conversione WebP ordinaria è senza perdita; l'editor offre qualità WebP regolabile. La qualità JPEG non influisce sui formati senza perdita.
- PDF (Bulk) crea un PDF separato per ogni immagine; PDF (Merge) può unire direttamente immagini e altri file PDF-compatibili, ordinando le singole pagine. Unione e divisione non trasferiscono segnalibri, tag o metadati e rifiutano moduli interattivi. Compressione, divisione e unione rifiutano PDF cifrati o firmati. L'esportazione PDF in PNG/JPEG accetta la sola password proprietario; una password di apertura richiede di sbloccare prima il documento.
- I documenti Office, i fogli di calcolo, i CSV e HTML in PDF richiedono LibreOffice installato o configurato; l'impaginazione dipende anche dai font disponibili.
- PDF in DOCX/HTML è una ricostruzione del testo, senza immagini o impaginazione originale. Excel in CSV esporta soltanto il primo foglio e perde formule e formattazione.
- La coda interattiva usa un formato comune ai file selezionati. Se le categorie non hanno una destinazione comune, vanno convertite in gruppi separati.
- Le conversioni rapide lavorano in coda in background. La finestra di stato si apre su richiesta dal segnaposto, mentre la finestra principale mostra la coda completa. Il resoconto conserva l'ultima esecuzione rapida; chiudere la finestra di stato non interrompe il lavoro.

[Torna al README](../README.md)
