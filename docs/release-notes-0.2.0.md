# Convert Here 0.2.0 — Windows x64

Il progetto prosegue come **Convert Here** nel repository [salvatorecorsi/convert-here](https://github.com/salvatorecorsi/convert-here). Questa versione amplia le conversioni a **44 destinazioni e 93 estensioni riconosciute**, rinnova l'interfaccia e include quattordici revisioni di affidabilita.

## Installazione

- **Setup:** `ConvertHere-0.2.0-windows-x64-setup.exe`, per l'utente corrente, con disinstaller e menu contestuale classico facoltativo.
- **Portable:** `ConvertHere-0.2.0-windows-x64-portable.zip`, da estrarre in una cartella stabile.
- **Integrita:** `SHA256SUMS.txt` contiene gli SHA-256 degli artefatti della stessa release.

Richiede **Windows 10 build 19041 o successivo, x64**. Su Windows 11 il menu classico compare in **Mostra altre opzioni**. Il menu moderno richiede separatamente un'identita MSIX con firma attendibile; non e incluso negli artefatti pubblici di questa release. Setup ed eseguibili pubblici **non sono firmati Authenticode**.

Il setup e il portable escludono i motori esterni. L'opzione del setup e lo script incluso possono scaricare FFmpeg e Poppler dai distributori originali, con verifica SHA-256. LibreOffice, Tesseract con i modelli delle lingue e libheif si configurano separatamente nella pagina **Strumenti**. Non servono Python, Node, WebView o Rust sul PC dell'utente.

## Nuove funzioni

- **Excel e CSV:** XLS/XLSX/XLSM/ODS in CSV del primo foglio e CSV UTF-8 in XLSX, con LibreOffice. Separatore selezionabile fra virgola, punto e virgola, tab e pipe; campi quotati e formule importate come testo.
- **Documenti e HTML:** Word e altri documenti compatibili in PDF con LibreOffice; HTML semplice in PDF. PDF in Word DOCX e HTML modificabili mediante ricostruzione del solo testo, con separazione delle pagine.
- **OCR locale:** immagini in TXT e scansioni PDF in TXT, DOCX o HTML con Tesseract; Poppler prepara le pagine PDF. OCR dei PDF e facoltativo e disattivato per default, con lingua e DPI configurabili.
- **Immagini e media:** ulteriori formati nativi, audio e video; HEIC/HEIF/AVIF in ingresso e HEIC/AVIF in uscita mediante libheif. La disponibilita dei codec dipende dal motore installato.
- **Operazioni PDF:** compressione senza perdita, divisione in un file per pagina e unione nell'ordine scelto, senza motori esterni. La compressione non produce un file piu grande dell'originale.
- **Interfaccia Fluent:** navigazione, controlli, icone dei file, temi chiaro/scuro e pagina Strumenti rinnovati. L'app resta basata su egui; le superfici simulano l'aspetto Mica.

## Affidabilita

- Record illeggibili della coda isolati e conservati, senza bloccare gli altri lavori; errori visibili nell'interfaccia.
- Conteggio delle pagine tramite `pdfinfo`, con fallback nativo quando manca; esportazione PNG/JPEG dei PDF con sola password proprietario ed errore italiano per la password di apertura.
- Lettura della coda in background e pulizia della cronologia: 7 giorni per completati/annullati, 30 per falliti e al massimo 300 record terminali, preservando originali e risultati.
- Pubblicazione dei risultati prenotati e delle pagine PDF tramite spostamento, senza ricopiarne il contenuto e senza sovrascrivere file esistenti.
- Aggiornamenti di avanzamento dei segnaposto limitati nel tempo, mantenendo immediati i cambi di stato.
- Parametri VP9/WebM adattati alla qualita scelta e codifica accelerata.
- Messaggio chiaro quando un video non contiene una traccia audio da estrarre.
- Rendering PDF a blocchi con avanzamento fra i blocchi e annullamento.
- Disinstallazione disponibile anche senza menu moderno, con gestione della DLL Explorer ancora caricata.
- Gestione degli apici Unicode nei comandi PowerShell dell'installer.
- CI con FFmpeg/Poppler reali, smoke test COM e confronto delle estensioni fra app, DLL, script e installer.
- Preferenze e percorsi rifiniti, temporanei nascosti e comando **Rimuovi segnaposto** per i lavori falliti, che preserva record, risultati e collegamenti sostituiti.

Le fixture dei probe PDF usano inoltre processi Rust nativi per verificare stream limitati, timeout e cancellazione dell'intero albero, senza dipendere dall'avvio di PowerShell sul runner CI. La verifica locale comprende 195 test dell'app e 5 della DLL, formattazione e Clippy, oltre ai 26 controlli del packaging con PowerShell 5.1 e 7.

## Limiti e compatibilita

PDF in DOCX/HTML e OCR ricostruiscono il testo, senza immagini, tabelle o impaginazione originale. Il riconoscimento puo contenere errori. HTML in PDF supporta documenti semplici e puo leggere risorse collegate locali o remote; non esegue JavaScript. CSV contiene il primo foglio senza formule o formattazione. GIF animate, TIFF multipagina e contenitori HEIC/AVIF usano il primo fotogramma o la prima immagine.

Compressione, divisione e unione rifiutano PDF cifrati o firmati. Divisione e unione rifiutano anche moduli interattivi e omettono segnalibri, tag e metadati del documento. La compressione conserva catalogo e moduli, senza rimuovere i metadati.

Per compatibilita restano `the-converter.exe`, `the-converter-cli.exe`, `the_converter_shell.dll`, gli identificativi COM/MSIX e le cartelle dati. Il percorso di installazione resta `%LOCALAPPDATA%\Programs\The Converter`; nuove installazioni usano il gruppo Start **Convert Here**, mentre gli aggiornamenti conservano quello precedente. I nomi degli artefatti e i checksum della [release storica 0.1.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.1.0) restano invariati.

Gli esiti delle verifiche sono raccolti in [store-validation.md](https://github.com/salvatorecorsi/convert-here/blob/v0.2.0/docs/store-validation.md). Le prove Store precedenti non costituiscono una certificazione della release GitHub. Dettagli sui dati locali: [privacy](https://github.com/salvatorecorsi/convert-here/blob/v0.2.0/docs/privacy.md).
