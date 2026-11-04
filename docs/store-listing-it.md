# Scheda Microsoft Store — italiano (it-IT)

Testi per Convert Here **0.3.2**, pacchetto Store **1.0.2.0**, caricato e Validated; submission nuovamente inviata il 4 novembre 2026 e **In certification**, non ancora pubblicata. Editore e sviluppatore: **Salvatore Corsi**, nome approvato con account Active e verifica Authorized. Si riutilizzano le tre schermate native 0.3.1 perche la UI e identica. Non copiare questo preambolo nei campi pubblici.

## Descrizione breve

Conversioni di file dal tasto destro di Windows 11, con coda, opzioni e OCR locale. Motori esterni per audio/video e alcune funzioni PDF.

## Descrizione

Le immagini nei formati nativi e le immagini in PDF funzionano subito, come compressione, divisione e unione dei PDF. Audio/video richiedono FFmpeg; esportazione PDF richiede Poppler; Office, Excel/CSV e HTML in PDF richiedono LibreOffice. HEIC/HEIF e AVIF richiedono libheif; OCR richiede Tesseract e i modelli delle lingue, oltre a Poppler per le scansioni PDF. Questi motori opzionali non sono inclusi.

Convert Here porta le conversioni nella cartella in cui lavori. Dal menu contestuale di Windows 11 scegli Converti in e il formato desiderato: il lavoro entra in coda con le impostazioni salvate, senza aprire ogni volta la finestra delle opzioni.

Un segnaposto con icona di attesa mostra subito dove comparira il risultato. Puoi aprirlo per vedere lo stato o annullare il lavoro. Quando la conversione termina, il file pronto prende il suo posto. Per impostazione predefinita i risultati sono accanto agli originali; se un nome esiste gia viene aggiunto un numero.

Sono disponibili 44 formati di destinazione e 93 estensioni di ingresso riconosciute, nelle categorie compatibili. Le conversioni di immagini native comprendono PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR e PNM, oltre alla creazione di un PDF A4 per ciascuna immagine. Puoi regolare qualita JPEG e larghezza delle immagini.

Con libheif puoi leggere HEIC/HEIF e AVIF e crearli dalle immagini supportate. Configura heif-dec.exe e heif-enc.exe in Strumenti, mantenendo insieme DLL e plugin della build. La disponibilita dipende dai decoder e dagli encoder HEVC/AV1 installati. Puoi scegliere qualita e larghezza; viene usata la prima immagine del contenitore, senza conservare sequenze o animazioni.

Con FFmpeg puoi convertire audio in MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A con AAC, WMA, AC3, AU e CAF con ALAC; convertire video in MP4, MKV, MOV, WebM, AVI, MPEG, WMV, FLV, MPEG-TS, M4V, 3GP e GIF animato, oppure estrarne l'audio. La disponibilita dei codec dipende dal motore FFmpeg selezionato. Le opzioni comprendono preset di qualita, bitrate, frequenza, canali, normalizzazione, qualita video, larghezza, FPS e taglio temporale.

Con Poppler puoi esportare i PDF in immagini PNG/JPEG, estrarne il testo incorporato in TXT oppure ricostruirlo in Word DOCX e HTML modificabili. Le esportazioni DOCX e HTML contengono solo testo: immagini, tabelle e impaginazione originali non vengono ricostruite.

Con Tesseract puoi riconoscere il testo delle immagini in TXT e, attivando OCR locale per i PDF, creare TXT, DOCX o HTML dalle scansioni. OCR dei PDF e disattivato per default e usa anche Poppler. Seleziona lingue installate, per esempio ita+eng, e i DPI delle pagine PDF; conserva i modelli nella cartella tessdata del motore. L'elaborazione resta sul PC. Il testo riconosciuto puo contenere errori e va riletto; OCR non crea un PDF ricercabile.

Puoi comprimere i PDF senza perdita o dividerli in un file per pagina. Con piu file convertibili in PDF, PDF (Bulk) crea risultati separati in coda; PDF (Merge) prepara immagini, PDF, Word/Office, Excel/CSV e HTML compatibili per creare un PDF unico. Trascina le singole pagine per intercalarle fra sorgenti, oppure usa le frecce o Alt+su/giu. PDF e immagini native non richiedono motori esterni per l'unione; restano LibreOffice per i documenti e libheif per HEIC/AVIF. Audio e video sono esclusi. La compressione non ricampiona le immagini e non produce un file piu grande dell'originale. Divisione e unione non trasferiscono segnalibri, tag o metadati del documento e rifiutano moduli interattivi. Le operazioni rifiutano PDF cifrati o firmati; la compressione conserva catalogo e moduli e non rimuove i metadati.

Con LibreOffice puoi convertire documenti Word e altri documenti, fogli e presentazioni compatibili in PDF, esportare XLS/XLSX/XLSM/ODS in CSV e creare Excel XLSX da CSV. Il CSV contiene soltanto il primo foglio, in UTF-8 con virgolette: conserva i valori visualizzati, senza formattazione o formule. Puoi scegliere virgola, punto e virgola, tab o pipe sia in ingresso sia in uscita; il default e la virgola. I CSV in ingresso devono essere UTF-8; le espressioni che iniziano con = vengono importate come testo e i campi quotati mantengono gli identificativi con zeri iniziali.

Puoi anche convertire file HTML in PDF con LibreOffice. Sono supportati documenti HTML semplici; CSS complessi possono cambiare impaginazione e JavaScript non viene eseguito. Le risorse collegate nell'HTML possono accedere a file locali o alla rete durante la conversione.

Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. Nessun pagamento richiesto per continuare a usarlo. Le conversioni di formato vengono elaborate sul PC e non richiedono un account nell'app. Preferenze, percorsi dei file, stato della coda e dettagli degli errori vengono salvati localmente.

Limiti: non e disponibile la modifica libera del contenuto delle pagine PDF. OCR riconosce il testo senza ricostruire la grafica o l'impaginazione. Da GIF animate, TIFF multipagina e contenitori HEIC/AVIF viene convertito il primo fotogramma o la prima immagine. PDF (Bulk) mantiene risultati separati; PDF (Merge) include tutte le pagine preparate una volta, fino a 10.000. L'impaginazione dei documenti Office dipende da LibreOffice e dai font disponibili. L'interfaccia e attualmente in italiano. Il menu moderno richiede Windows 11 x64.

## Funzionalita

Inserire ciascuna riga in un campo separato, senza il trattino iniziale:

```text
Menu Converti in direttamente in Esplora file su Windows 11
Conversioni rapide in coda senza aprire ogni volta le opzioni
Segnaposto nella cartella di destinazione e stato del lavoro su richiesta
Immagini in PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld, OpenEXR, HDR e PNM
Creazione di un PDF A4 separato per ciascuna immagine
Qualita JPEG, ridimensionamento e cartella di destinazione personalizzabili
Audio e video con FFmpeg opzionale selezionato dall'utente
PDF in immagini, TXT, Word DOCX e HTML di solo testo con Poppler opzionale
Excel in CSV del primo foglio e CSV UTF-8 in XLSX, con quattro separatori e LibreOffice opzionale
Word e HTML semplici in PDF con LibreOffice opzionale; JavaScript non eseguito
HEIC/HEIF e AVIF in ingresso e HEIC/AVIF in uscita con libheif opzionale
OCR locale di immagini e scansioni PDF con Tesseract e modelli delle lingue opzionali
PDF Bulk separati o PDF Merge unico con trascinamento delle singole pagine
File originali conservati e nomi numerati in caso di conflitto
Conversioni di formato elaborate sul PC senza un account nell'app
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
FFmpeg per audio/video; Poppler per esportare PDF; LibreOffice per Office/HTML in PDF ed Excel/CSV. Motori opzionali non inclusi.
libheif con heif-dec.exe, heif-enc.exe, DLL e plugin per HEIC/AVIF; Tesseract con i modelli delle lingue per OCR, oltre a Poppler per i PDF. Non inclusi.
```

## Novita

Lasciare vuoto per la prima submission Store. Preparare una nota specifica soltanto per gli aggiornamenti successivi.

## Didascalie delle schermate da acquisire

1. Converti in dal menu contestuale di Windows 11.
2. La coda mostra formato, stato e destinazione dei file.
3. Impostazioni di qualita e ridimensionamento per le immagini.
4. Configura gli eseguibili opzionali nella pagina Strumenti.

## Riferimenti submission

Titolare e sviluppatore: **Salvatore Corsi**; editore: **Salvatore Corsi**. Contatto privacy e supporto: [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). [Informativa privacy pubblica](https://github.com/salvatorecorsi/convert-here/blob/v0.3.2/docs/privacy.md). Nome riservato: **Convert Here**. Per licenza, identita e stato operativo vedere la [procedura di submission](store-submission.md).

Le schede nel portale sono Complete e incluse nella submission 1.0.2.0 inviata per certificazione; questo stato non attesta la pubblicazione Store.

Descrizione entro 10.000 caratteri; breve entro 1.000 (qui meno di 270); massimo 20 funzionalita da 200 caratteri; novita massimo 1.500, vuote alla prima submission. [Limiti ufficiali dei campi MSIX](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info).
