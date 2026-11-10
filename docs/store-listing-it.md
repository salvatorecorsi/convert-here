# Scheda Microsoft Store — italiano (it-IT)

**Bozza per la candidata Convert Here 0.8.0 / Store 1.0.8.0**, sviluppatore e publisher **Salvatore Corsi**. Non ancora pubblicata, caricata o inviata; testi e nuove schermate non risultano salvati nel portale. Sono predisposte **20 funzionalità**. Lo stato storico della 0.7.1 resta nel [suo record](store-testing-0.7.1.md); nessuna nuova verifica dello stato Store è attestata qui. Nessun test su host Windows 10 reale o WACK. Non copiare questo preambolo nei campi pubblici.

## Descrizione breve

Conversioni locali per Windows 10 e 11, editor immagini e audio/video, profili e PDF. OCR e trascrizione con motori installabili su richiesta.

## Descrizione

Convert Here porta le conversioni nella cartella in cui lavori, su Windows 10 versione 2004 o successivo (build 19041), x64, e Windows 11. Dal menu contestuale scegli Converti in e un formato compatibile: il lavoro entra in coda con le impostazioni salvate. Un segnaposto indica dove comparirà il risultato e permette di controllare lo stato o annullare. Originali e file già esistenti sono conservati; i conflitti di nome ricevono un numero.

Sono disponibili 44 destinazioni e 94 estensioni di ingresso riconosciute. Le uscite visibili per default restano MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4. Abilita le altre da Impostazioni → Formati di uscita; TXT si trova in Raw. Il filtro riguarda le uscite e il menu si aggiorna senza riavviare Explorer. Interfaccia e menu sono disponibili in italiano e inglese.

Immagini native, SVG statico, immagini in PDF e compressione/divisione/unione PDF funzionano senza motori esterni. Audio/video richiedono FFmpeg; esportazione PDF richiede Poppler; Office, Excel/CSV e HTML in PDF richiedono LibreOffice. HEIC/HEIF e AVIF usano libheif; OCR usa Tesseract e modelli delle lingue, con Poppler per le scansioni PDF. La trascrizione usa FFmpeg/ffprobe e Whisper con Tiny multilingue.

Da Impostazioni → Motori puoi richiedere FFmpeg, Poppler, Tesseract con italiano/inglese, libheif o Whisper con Tiny. Ogni download richiede conferma di origine, versione, dimensioni, licenze e cartella. Gli asset sono verificati con SHA-256 prima dell'estrazione; licenze e provenienza restano nella cartella personale. Motori e modelli non sono inclusi nell'app né scaricati senza richiesta. LibreOffice resta separato. Internet serve per acquisire i motori; i file scelti per conversione, OCR o trascrizione non vengono caricati ai distributori.

Converti immagini in PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR e PNM, oppure crea PDF. Gli SVG UTF-8 statici vengono rasterizzati localmente con resvg integrato: non vengono conservati i vettori. Risorse esterne, script, DTD, foreignObject e immagini incorporate vengono rifiutati. SVGZ non è supportato e valgono limiti di dimensioni e complessità.

Apri Modifica… per ritagliare, ridimensionare e creare una tela con sfondo, posizione e ancoraggio. PNG, JPEG, WebP e PDF mostrano anteprima e peso effettivo dei byte preparati; Salva con nome usa quegli stessi byte. JPEG/WebP possono cercare una qualità compatibile con il peso massimo; ridurre anche le dimensioni richiede una scelta esplicita. 300 kB corrispondono a 300000 byte. Un limite impossibile non pubblica un risultato sovradimensionato.

Impagina immagini, testo scritto in Nuovo foglio… e TXT UTF-8 su A4/A5, in verticale o orizzontale, con margini scelti. Il testo semplice offre dimensione, interlinea, allineamento e pagine automatiche, con Ubuntu Light incorporato e testo selezionabile. I caratteri non supportati producono un errore. Questo percorso non richiede LibreOffice.

L'editor audio/video offre forma d'onda, fotogrammi per i video, maniglie sulla timeline e tempi numerici per il taglio. Puoi impostare gain o muto per le conversioni compatibili: valori positivi possono saturare il segnale. Le anteprime mostrano la sorgente prima degli effetti; non sono presenti riproduzione o montaggio multitraccia. Per TXT viene applicato soltanto il taglio, senza gain o muto.

Abilita TXT in Raw per trascrivere il parlato di audio/video sul PC. La voce del menu è TXT · Trascrizione. Scegli rilevamento automatico o una lingua supportata e il segmento da elaborare. Tiny multilingue richiede 77,69 MB; il download di Whisper con modello è circa 87,94 MB, escluso FFmpeg. La GUI include FFmpeg se manca FFmpeg o ffprobe. I temporanei audio/testo vengono normalmente rimossi a completamento o annullamento; un arresto improvviso può lasciare residui. La trascrizione può contenere errori e va riletta.

Con un pulsante puoi installare Tesseract con italiano e inglese per riconoscere testo da immagini in TXT. Attivando OCR locale per PDF puoi ottenere TXT, DOCX o HTML dalle scansioni, con Poppler. OCR PDF è disattivato per default; scegli lingue installate e DPI. Il riconoscimento non ricostruisce impaginazione e grafica e non crea un PDF ricercabile. Il testo può richiedere correzioni.

Il pulsante libheif prepara decoder ed encoder per leggere HEIC/HEIF e AVIF e creare HEIC/AVIF. Sono supportati i codec presenti nella build selezionata. Da GIF animate, TIFF multipagina e contenitori HEIC/AVIF viene convertita la prima immagine; sequenze e animazioni non sono conservate.

Con FFmpeg converti audio in MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A con AAC, WMA, AC3, AU e CAF con ALAC; video in MP4, MKV, MOV, WebM, AVI, MPEG, WMV, FLV, MPEG-TS, M4V, 3GP e GIF animato, oppure estrai l'audio. Regola bitrate, frequenza, canali, normalizzazione, qualità, larghezza, FPS e intervallo. Avanzate offre opzioni tipizzate e controlli degli encoder locali verificati prima dell'uso, senza comandi o script liberi. Le opzioni FFmpeg riguardano sorgenti audio/video.

Il manager Profili crea, modifica, duplica, rinomina ed elimina ricette per Audio, Video, Immagini e Documenti: fino a 32 salvate e 8 nel menu. Scegli il predefinito, i filtri delle estensioni ammesse e la visibilità; importa/esporta JSON senza sovrascrivere ricette esistenti. I profili conservano opzioni, non immagini o testi di prova. I lavori accodati mantengono il proprio snapshot anche dopo modifica o eliminazione del profilo.

Puoi imporre peso massimo per risultato e durata massima audio/video, combinata con il taglio. Il peso è controllato prima della pubblicazione; per PDF divisi vale su ogni pagina. Audio/video e PDF non promettono adattamento automatico al peso. Cronologia riunisce lavori della finestra, coda ed editor e conserva gli esiti fra gli avvii, senza cancellare originali o risultati.

Con Poppler esporta pagine PDF in PNG/JPEG, testo incorporato in TXT o ricostruzioni DOCX/HTML di solo testo: immagini, tabelle e impaginazione originali non sono ricostruite. Comprimi PDF senza perdita, dividili per pagina o usa PDF Bulk per risultati separati e PDF Merge per un documento unico, con pagine trascinabili. Sono ammessi PDF e sorgenti convertibili in PDF, esclusi audio/video. La compressione non produce un file più grande dell'originale. Le operazioni rifiutano PDF cifrati o firmati; divisione/unione rifiutano moduli interattivi e omettono segnalibri, tag e metadati. Non è disponibile la modifica libera del contenuto delle pagine PDF.

Con LibreOffice converti Word e documenti compatibili in PDF, fogli XLS/XLSX/XLSM/ODS in CSV e CSV in XLSX. Il CSV contiene il primo foglio, valori visualizzati in UTF-8, senza formattazione o formule. Scegli virgola, punto e virgola, tab o pipe. I CSV in ingresso devono essere UTF-8; espressioni iniziate da = sono importate come testo. HTML in PDF supporta documenti semplici, con possibili differenze CSS e senza JavaScript; le risorse collegate possono accedere a file o rete.

Convert Here è gratuito, senza abbonamenti, acquisti in-app o pagamenti per continuare l'uso. Non richiede un account nell'app. Preferenze, percorsi, coda e dettagli degli errori sono salvati localmente. Il codice originale è GPL-3.0-only, con sorgenti su https://github.com/salvatorecorsi/convert-here; dipendenze, font, motori e modelli mantengono le proprie licenze. Il menu moderno richiede Windows 11 x64; Windows 10 usa quello classico.

## Funzionalità

Inserire ciascuna riga in un campo separato, senza il trattino iniziale:

```text
Converti in nel menu classico di Windows 10 e moderno di Windows 11, da build 19041 x64
Cronologia unificata per lavori in corso, in coda e completati
44 destinazioni e 94 ingressi; otto uscite predefinite e TXT attivabile in Raw
Immagini native e SVG statici rasterizzati localmente senza risorse esterne
Editor immagini con ritaglio, scala, tela e anteprima dei byte da salvare
Immagini, testo scritto e TXT UTF-8 su PDF A4/A5 con font incorporato
Profili Audio, Video, Immagini e Documenti: 32 salvati, 8 nel menu
Importa/esporta profili JSON e scegli le estensioni di ingresso compatibili
Peso massimo per risultato e durata massima audio/video combinata con il taglio
Editor audio/video con timeline, forma d'onda, fotogrammi, taglio e gain
Opzioni FFmpeg avanzate tipizzate e parametri degli encoder locali verificati
Trascrizione locale audio/video in TXT con Whisper e Tiny multilingue facoltativi
Tesseract OCR installabile con un pulsante, con italiano e inglese inclusi
libheif installabile con un pulsante per leggere e creare HEIC/AVIF
Audio/video con FFmpeg: codec, bitrate, frequenza, canali, normalizzazione e FPS
PDF in immagini e testo con Poppler; compressione, divisione e unione con pagine trascinabili
Excel/CSV con quattro separatori e Word/HTML in PDF con LibreOffice
Originali e file esistenti conservati; i lavori accodati mantengono la ricetta
Elaborazione locale, nessun account o pagamento richiesto per continuare l'uso
Download motori su richiesta con conferma, SHA-256 e licenze originali conservate
```

## Termini di ricerca

Un termine per campo; non usare nomi di prodotti terzi:

```text
convertitore file
convertire immagini
immagini PDF
menu contestuale
conversione audio
trascrizione locale
coda conversioni
```

## Requisiti aggiuntivi

```text
Windows 10 versione 2004 (build 19041) o successivo, x64; menu classico su Windows 10, moderno su Windows 11.
Motori facoltativi non inclusi: FFmpeg per audio/video, Poppler per esportare PDF, LibreOffice per Office/HTML ed Excel/CSV, libheif per HEIC/AVIF, Tesseract per OCR; scansioni PDF anche con Poppler.
Trascrizione: FFmpeg/ffprobe, Whisper CPU e Tiny multilingue. Download Whisper con modello circa 87,94 MB, oltre a FFmpeg. Download motori da Impostazioni con conferma e Internet; LibreOffice separato.
```

## Novità

```text
SVG statico nativo, editor audio/video con timeline e taglio/gain, trascrizione locale in TXT con Whisper Tiny multilingue. Tesseract con italiano/inglese e libheif si installano da un pulsante con conferma e verifica SHA-256. Le uscite predefinite restano otto; abilita TXT in Raw. Sono riconosciuti 94 ingressi.
```

## Didascalie delle schermate

1. Formati di uscita per categoria e nuova barra delle schede. Interfaccia italiana.
2. Motori locali: installazione di OCR, libheif e Whisper con un pulsante. Interfaccia italiana.
3. Editor audio/video: timeline, intervallo selezionato e gain. Interfaccia italiana.
4. Trascrizione locale in TXT con scelta della lingua e taglio della clip. Interfaccia italiana.

Le [quattro catture native 0.8.0](store-assets/0.8.0/README.md) sono PNG **2562 × 1441**, senza puntatore, da build debug 0.8.0 con zoom 150% e senza ingrandimento successivo. La galleria precedente resta storica. Il salvataggio delle nuove immagini nel portale non è ancora attestato.

## Copyright e termini di licenza aggiuntivi

```text
Copyright (C) 2026 Salvatore Corsi.
```

Il progetto è GPL-3.0-only. Usare l'[attuale testo completo](../LICENSE); dopo la pubblicazione del tag, il riferimento previsto per il campo pubblico è `https://github.com/salvatorecorsi/convert-here/blob/v0.8.0/LICENSE`. Il tag e questo URL non sono ancora attestati come pubblicati. L'asset sorgente previsto è `ConvertHere-0.8.0-source.zip`, con dipendenze vendorizzate e istruzioni offline. Terze parti mantengono licenze e copyright originali.

## Riferimenti submission

Titolare, sviluppatore e publisher: **Salvatore Corsi**. Supporto/privacy: [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). [Privacy della candidata](privacy.md), [pin e licenze dei motori](release-licenses/EXTERNAL-ENGINES.md), [verifiche 0.8.0](store-testing-0.8.0.md), [procedura e stato Store](store-submission.md).

Testi e schermate della candidata non sono ancora attestati come salvati nel portale. Lo stato storico precedente non implica upload, certificazione o pubblicazione della 0.8.0. Descrizione entro 10.000 caratteri, breve entro 1.000, **20 funzionalità** entro 200 caratteri ciascuna e novità entro 1.500. [Limiti dei campi MSIX](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info).
