# Scheda Microsoft Store — italiano (it-IT)

Bozza per Convert Here **0.6.0**, pacchetto Store candidato **1.0.5.0**. Editore e sviluppatore: **Salvatore Corsi**. Testi pronti con 20 funzionalità; catture, URL versionati e caricamento vanno verificati sul candidato finale. Questo documento non attesta pubblicazione, invio o approvazione della 0.6.0. Il record della precedente 0.4.0 resta nello storico. Non copiare questo preambolo nei campi pubblici.

## Descrizione breve

Conversioni locali, editor immagini con peso esatto, fogli A4/A5 e profili dal tasto destro di Windows 11. Motori opzionali per altri formati e OCR.

## Descrizione

Le immagini nei formati nativi e le immagini in PDF funzionano subito, come compressione, divisione e unione dei PDF. Audio/video richiedono FFmpeg; esportazione PDF richiede Poppler; Office, Excel/CSV e HTML in PDF richiedono LibreOffice. HEIC/HEIF e AVIF richiedono libheif; OCR richiede Tesseract e i modelli delle lingue, oltre a Poppler per le scansioni PDF. Questi motori opzionali non sono inclusi. Da Impostazioni → Motori puoi richiedere il download di FFmpeg, Poppler o entrambi, con conferma di origine, versione, dimensioni, licenze e cartella. Internet serve per questo download facoltativo; conversioni e OCR restano locali.

Convert Here porta le conversioni nella cartella in cui lavori. Dal menu contestuale di Windows 11 scegli Converti in e il formato desiderato: il lavoro entra in coda con le impostazioni salvate, senza aprire ogni volta la finestra delle opzioni.

Un segnaposto con icona di attesa mostra subito dove comparira il risultato. Puoi aprirlo per vedere lo stato o annullare il lavoro. Quando la conversione termina, il file pronto prende il suo posto. Per impostazione predefinita i risultati sono accanto agli originali; se un nome esiste gia viene aggiunto un numero.

Sono disponibili 44 formati di destinazione e 93 estensioni di ingresso riconosciute, nelle categorie compatibili. Le conversioni di immagini native comprendono PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR e PNM, oltre alla creazione di un PDF A4 per ciascuna immagine. Puoi regolare qualita JPEG e larghezza delle immagini. Nelle Impostazioni scegli le uscite visibili nella GUI e nel menu Explorer: di default MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4. Puoi abilitare le altre uscite; le 93 estensioni in ingresso restano riconosciute e il menu si aggiorna senza riavviare Explorer.

Apri Modifica… nella riga di un'immagine per ritagliare, ridimensionare o creare una tela con risoluzione, colore, posizione e ancoraggio scelti. Esporta PNG, JPEG, WebP o PDF con anteprima del risultato preparato e peso effettivo dei byte codificati. JPEG e WebP offrono qualità regolabile: con un peso massimo il motore cerca una qualità adatta; 300 kB sono 300000 byte. Ridurre anche le dimensioni richiede una scelta esplicita. Nei PDF regola qualità, dimensioni e DPI per cambiare il peso. Se il limite è impossibile, il salvataggio resta disattivato e la conversione rapida non produce un file oltre il limite. Salva con nome scrive gli stessi byte preparati e conserva i file già esistenti.

Impagina immagini, testo scritto in Nuovo foglio… o file TXT UTF-8 su A4/A5, in verticale o orizzontale, con margini scelti. Il testo semplice offre dimensione, interlinea, allineamento e pagine automatiche, con Ubuntu Light incorporato e testo selezionabile nel PDF. I caratteri non supportati producono un errore esplicito. Questo percorso è nativo e non richiede LibreOffice; i documenti Word mantengono la conversione Office esistente.

Salva le mie impostazioni crea profili con nome: fino a 32 salvati e 8 visibili in Converti in. Riusa ritaglio relativo, dimensioni, tela, qualità, peso e opzioni pagina senza salvare l'immagine di prova o il testo digitato. I miei profili nelle Impostazioni permette di applicare e aggiornare le ricette. Nel menu compaiono soltanto i profili abilitati con formato attivo e ingressi compatibili; la coda conserva le opzioni con cui ogni lavoro è stato creato.

Con libheif puoi leggere HEIC/HEIF e AVIF e crearli dalle immagini supportate. Configura heif-dec.exe e heif-enc.exe in Impostazioni → Motori, mantenendo insieme DLL e plugin della build. La disponibilita dipende dai decoder e dagli encoder HEVC/AV1 installati. Puoi scegliere qualita e larghezza; viene usata la prima immagine del contenitore, senza conservare sequenze o animazioni.

Con FFmpeg puoi convertire audio in MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A con AAC, WMA, AC3, AU e CAF con ALAC; convertire video in MP4, MKV, MOV, WebM, AVI, MPEG, WMV, FLV, MPEG-TS, M4V, 3GP e GIF animato, oppure estrarne l'audio. La disponibilita dei codec dipende dal motore FFmpeg selezionato. Le opzioni comprendono preset di qualita, bitrate, frequenza, canali, normalizzazione, qualita video, larghezza, FPS e taglio temporale.

Con Poppler puoi esportare i PDF in immagini PNG/JPEG, estrarne il testo incorporato in TXT oppure ricostruirlo in Word DOCX e HTML modificabili. Le esportazioni DOCX e HTML contengono solo testo: immagini, tabelle e impaginazione originali non vengono ricostruite.

Con Tesseract puoi riconoscere il testo delle immagini in TXT e, attivando OCR locale per i PDF, creare TXT, DOCX o HTML dalle scansioni. OCR dei PDF e disattivato per default e usa anche Poppler. Seleziona lingue installate, per esempio ita+eng, e i DPI delle pagine PDF; conserva i modelli nella cartella tessdata del motore. L'elaborazione resta sul PC. Il testo riconosciuto puo contenere errori e va riletto; OCR non crea un PDF ricercabile.

Puoi comprimere i PDF senza perdita o dividerli in un file per pagina. Con piu file convertibili in PDF, PDF (Bulk) crea risultati separati in coda; PDF (Merge) prepara immagini, PDF, Word/Office, Excel/CSV e HTML compatibili per creare un PDF unico. Trascina le singole pagine per intercalarle fra sorgenti, oppure usa le frecce o Alt+su/giu. PDF e immagini native non richiedono motori esterni per l'unione; restano LibreOffice per i documenti e libheif per HEIC/AVIF. Audio e video sono esclusi. La compressione non ricampiona le immagini e non produce un file piu grande dell'originale. Divisione e unione non trasferiscono segnalibri, tag o metadati del documento e rifiutano moduli interattivi. Le operazioni rifiutano PDF cifrati o firmati; la compressione conserva catalogo e moduli e non rimuove i metadati.

Con LibreOffice puoi convertire documenti Word e altri documenti, fogli e presentazioni compatibili in PDF, esportare XLS/XLSX/XLSM/ODS in CSV e creare Excel XLSX da CSV. Il CSV contiene soltanto il primo foglio, in UTF-8 con virgolette: conserva i valori visualizzati, senza formattazione o formule. Puoi scegliere virgola, punto e virgola, tab o pipe sia in ingresso sia in uscita; il default e la virgola. I CSV in ingresso devono essere UTF-8; le espressioni che iniziano con = vengono importate come testo e i campi quotati mantengono gli identificativi con zeri iniziali.

Puoi anche convertire file HTML in PDF con LibreOffice. Sono supportati documenti HTML semplici; CSS complessi possono cambiare impaginazione e JavaScript non viene eseguito. Le risorse collegate nell'HTML possono accedere a file locali o alla rete durante la conversione.

Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. Nessun pagamento richiesto per continuare a usarlo. Le conversioni di formato vengono elaborate sul PC e non richiedono un account nell'app. Preferenze, percorsi dei file, stato della coda e dettagli degli errori vengono salvati localmente.

Il codice originale e software libero con licenza GPL-3.0-only; i sorgenti sono disponibili su https://github.com/salvatorecorsi/convert-here. Dipendenze, font e motori mantengono le rispettive licenze. La conversione non cambia i diritti sui contenuti dei tuoi file.

Limiti: non e disponibile la modifica libera del contenuto delle pagine PDF. OCR riconosce il testo senza ricostruire la grafica o l'impaginazione. Da GIF animate, TIFF multipagina e contenitori HEIC/AVIF viene convertito il primo fotogramma o la prima immagine. PDF (Bulk) mantiene risultati separati; PDF (Merge) include tutte le pagine preparate una volta, fino a 10.000. L'impaginazione dei documenti Office dipende da LibreOffice e dai font disponibili. Interfaccia e menu sono disponibili in italiano e inglese; la lingua OCR si sceglie separatamente. Il menu moderno richiede Windows 11 x64.

## Funzionalita

Inserire ciascuna riga in un campo separato, senza il trattino iniziale:

```text
Menu Converti in direttamente in Esplora file su Windows 11
Conversioni rapide in coda senza aprire ogni volta le opzioni
Segnaposto nella cartella di destinazione e stato del lavoro su richiesta
Immagini in PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR e PNM
Immagini e testo semplice su PDF A4/A5 con orientamento e margini scelti
Editor locale: ritaglio, scala e tela con risoluzione, sfondo e ancoraggio
Audio e video con FFmpeg opzionale selezionato dall'utente
PDF in immagini, TXT, Word DOCX e HTML di solo testo con Poppler opzionale
Excel in CSV del primo foglio e CSV UTF-8 in XLSX, con quattro separatori e LibreOffice opzionale
Word e HTML semplici in PDF con LibreOffice opzionale; JavaScript non eseguito
HEIC/HEIF e AVIF in ingresso e HEIC/AVIF in uscita con libheif opzionale
OCR locale di immagini e scansioni PDF con Tesseract e modelli delle lingue opzionali
PDF Bulk separati o PDF Merge unico con trascinamento delle singole pagine
File originali conservati e nomi numerati in caso di conflitto
Conversioni di formato elaborate sul PC senza un account nell'app
Scegli le uscite visibili: otto formati predefiniti e 93 estensioni in ingresso riconosciute
Download facoltativo di FFmpeg e Poppler con conferma, verifica SHA-256 e licenze conservate
Anteprima e peso esatto per PNG/JPEG/WebP/PDF; salvataggio degli stessi byte preparati
Fino a 32 profili personali, con 8 scorciatoie nel menu Converti in per file compatibili
Testo scritto nell'app o TXT UTF-8 in PDF con font incorporato e testo selezionabile
```

## Termini di ricerca

Un termine per campo; non usare nomi di prodotti terzi:

```text
convertitore file
convertire immagini
immagini PDF
menu contestuale
conversione audio
conversione video
coda conversioni
```

## Requisiti aggiuntivi

```text
Windows 11 x64 per il menu contestuale moderno
FFmpeg per audio/video; Poppler per esportare PDF; LibreOffice per Office/HTML in PDF ed Excel/CSV. Motori opzionali non inclusi. Download facoltativo di FFmpeg/Poppler da Impostazioni → Motori; richiede Internet.
libheif con heif-dec.exe, heif-enc.exe, DLL e plugin per HEIC/AVIF; Tesseract con i modelli delle lingue per OCR, oltre a Poppler per i PDF. Non inclusi.
```

## Novita

```text
Modifica immagini con ritaglio, scala e tela personalizzata; esporta PNG, JPEG, WebP o PDF con anteprima e peso effettivo. Regola la qualità e imposta un peso massimo, con riduzione delle dimensioni soltanto se autorizzata. Impagina immagini, testo scritto nell'app o TXT UTF-8 su A4/A5. Salva fino a 32 profili, con 8 scorciatoie nel menu Converti in. Elaborazione sul PC, 44 destinazioni e 93 estensioni riconosciute. Gratuito: nessun pagamento richiesto per continuare a usarlo.
```

## Didascalie delle schermate

1. Conversione locale di due immagini in JPEG completata.
2. Riordina le singole pagine in PDF Merge per creare un documento unico.
3. Scegli le uscite visibili nelle Impostazioni, con otto formati predefiniti e categorie dedicate.
4. Conferma fonti, versioni, dimensioni e cartella prima di scaricare FFmpeg e Poppler.

Catture 1, 3 e 4 della 0.4.0; la 2 riusa PDF Merge della 0.3.1 con interfaccia invariata. [Percorsi delle catture](store-assets/README.md).

Per la candidata 0.6.0 sono da aggiungere catture native dell'editor e dei profili, con didascalie da abbinare soltanto dopo la verifica dell'immagine: «Ritaglia e regola qualità e peso sul risultato preparato»; «Impagina un testo su A4 o A5»; «Salva le opzioni e riusale nei tuoi profili». Non attribuire queste nuove funzioni alle catture storiche.

## Copyright e termini di licenza aggiuntivi

Copyright:

```text
Copyright (C) 2026 Salvatore Corsi.
```

URL previsto per Additional license terms, da verificare dopo la pubblicazione del tag 0.6.0 e prima del salvataggio nel portale; collega il testo integrale della GPLv3:

```text
https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/LICENSE
```

La scelta del progetto è GPL-3.0-only. Per la candidata è previsto `ConvertHere-0.6.0-source.zip` con sorgenti, dipendenze Cargo vendorizzate e istruzioni di build offline. Queste istruzioni dei campi non fanno parte della descrizione pubblica.

## Riferimenti submission

Titolare e sviluppatore: **Salvatore Corsi**; editore: **Salvatore Corsi**. Contatto privacy e supporto: [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). [Privacy della candidata 0.6.0](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/privacy.md), URL da verificare dopo il tag. Nome riservato: **Convert Here**. Per licenza, identità e stato operativo vedere la [procedura di submission](store-submission.md).

Queste sono bozze locali 0.6.0, non campi già salvati nel portale. Le evidenze della 0.4.0 e delle altre versioni restano nello storico. [Istruzioni di test della candidata](store-testing-0.6.0.md).

Descrizione entro 10.000 caratteri; breve entro 1.000 (qui meno di 270); massimo 20 funzionalita da 200 caratteri; novita massimo 1.500, vuote alla prima submission. [Limiti ufficiali dei campi MSIX](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info).
