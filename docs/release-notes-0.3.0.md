# Convert Here 0.3.0 — PDF Bulk e Merge, Windows x64

Questa versione aggiunge due azioni distinte per selezioni multiple convertibili in PDF: **PDF (Bulk)** e **PDF (Merge)**. Il modello mantiene **44 formati di destinazione e 93 estensioni riconosciute**.

## Risultati separati oppure PDF unico

**PDF (Bulk)** mantiene il flusso rapido esistente: un lavoro in coda per ciascun file, con risultati separati e segnaposto nella destinazione. Immagini e documenti vengono convertiti in PDF; i PDF gia esistenti vengono compressi senza perdita.

**PDF (Merge)** apre un editor dedicato. Dopo la preparazione delle sorgenti, l'elenco mostra ogni singola pagina con il file e il numero originale. Puoi trascinare le pagine per cambiarne l'ordine, anche intercalando pagine di PDF multipagina con immagini o pagine di altri documenti. Sono disponibili anche frecce e **Alt+↑ / Alt+↓** sulla pagina selezionata.

L'ordine iniziale segue i file ricevuti da Esplora file e le pagine interne di ciascuna sorgente; controllalo nell'editor. **Crea PDF unico** produce un solo `primo-unito.pdf`. Se il nome esiste, viene aggiunto un numero senza sovrascriverlo. Tutte le pagine vengono incluse una volta, fino a 10.000. Il comando Merge nella finestra principale apre lo stesso editor.

## File supportati e preparazione

Sono compatibili **immagini, PDF, Word e altri documenti Office supportati, Excel/CSV e HTML**, anche mescolati. Audio e video non sono sorgenti PDF per queste azioni.

- PDF e immagini native possono essere uniti senza motori esterni; non serve Poppler per il nuovo Merge.
- Office, Excel/CSV e HTML richiedono LibreOffice configurato.
- HEIC/HEIF/AVIF richiedono libheif per la lettura. Per immagini animate o multipagina resta la conversione della prima immagine secondo il comportamento esistente.

La preparazione crea snapshot PDF in una cartella temporanea nascosta dell'utente. Nessun PDF intermedio viene pubblicato nella destinazione e gli originali restano invariati. Il salvataggio usa le pagine preparate, anche se gli originali cambiano nel frattempo. La sessione Merge e il suo ordine restano separati dalla coda persistente. Annullamento e chiusura tentano di eliminare gli intermedi; un arresto improvviso puo lasciare residui.

## Limiti mantenuti

I PDF cifrati, firmati o con moduli interattivi richiedono una copia statica sbloccata prima dell'unione. Il risultato mantiene i contenuti delle pagine, ma non trasferisce segnalibri, tag o metadati del documento. Non e disponibile la modifica libera del contenuto delle pagine. L'impaginazione di Office e HTML dipende da LibreOffice; le risorse HTML collegate possono accedere a file locali o alla rete durante la preparazione.

Le conversioni PDF in Word/HTML restano ricostruzioni del solo testo.

## Download e compatibilita

- `ConvertHere-0.3.0-windows-x64-setup.exe`
- `ConvertHere-0.3.0-windows-x64-portable.zip`
- `SHA256SUMS.txt` per verificare gli artefatti della stessa release.

Richiede **Windows 10 build 19041 o successivo, x64**. Setup ed eseguibili pubblici non sono firmati Authenticode. Il setup include il menu classico, mostrato su Windows 11 in **Mostra altre opzioni**; il menu moderno richiede separatamente un'identita MSIX con firma attendibile.

I motori esterni restano opzionali e non inclusi nei pacchetti pubblici. Setup e script possono scaricare FFmpeg e Poppler su richiesta, con hash verificati; LibreOffice, Tesseract e libheif si configurano separatamente. Restano stabili i nomi tecnici degli eseguibili, gli identificativi e i percorsi dati per gli aggiornamenti.

Dettagli d'uso: [README](https://github.com/salvatorecorsi/convert-here/blob/v0.3.0/README.md). Dati locali e temporanei: [privacy](https://github.com/salvatorecorsi/convert-here/blob/v0.3.0/docs/privacy.md). Esiti delle prove: [verifiche](https://github.com/salvatorecorsi/convert-here/blob/v0.3.0/docs/store-validation.md). Le [note 0.2.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.2.0) rimangono storiche.
