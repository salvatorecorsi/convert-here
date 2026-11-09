# Convert Here — informativa sulla privacy / privacy statement

Informativa aggiornata al 9 novembre 2026, relativa alle funzioni di Convert Here 0.5.0.

| Dato | Valore |
| --- | --- |
| Applicazione / Application | Convert Here |
| Editore / Publisher | Salvatore Corsi |
| Autore / sviluppatore / Developer | Salvatore Corsi |
| Titolare / Data controller | Salvatore Corsi |
| Contatto privacy e supporto / Privacy and support contact | [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com) |
| Informativa pubblica / Public privacy statement | [Informativa privacy di Convert Here](https://github.com/salvatorecorsi/convert-here/blob/master/docs/privacy.md) |

## Italiano

### File e conversioni

Convert Here legge i file che scegli tramite Esplora file, selettore o trascinamento e crea i risultati nella destinazione configurata. Le conversioni di formato e l'OCR avvengono sul PC. Il software e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. L'app non invia file o percorsi a un server dello sviluppatore e non contiene servizi di telemetria, pubblicita o registrazione di account propri. Gli originali non vengono modificati dall'app.

Se usi FFmpeg, Poppler, LibreOffice, Tesseract o libheif, gli eseguibili locali ricevono i percorsi e le opzioni necessari alla conversione. Sono programmi separati, installati o selezionati dall'utente. Possono avere proprie impostazioni e informative; Convert Here non controlla tutte le loro funzioni. Non e prevista una trasmissione allo sviluppatore dell'app.

Le conversioni Excel/CSV e HTML in PDF usano LibreOffice con un profilo temporaneo separato. Le macro e l'aggiornamento dei collegamenti Office vengono disabilitati. Un documento HTML puo comunque contenere riferimenti a immagini, CSS o altre risorse locali o remote: il motore puo leggerle durante l'apertura, contattando i rispettivi server per quelle remote. Il separatore CSV scelto viene applicato localmente in ingresso e uscita. La lettura e la creazione di HEIC/HEIF e AVIF usano gli eseguibili locali libheif con le relative DLL e i plugin.

L'OCR di immagini in TXT e l'OCR facoltativo di PDF in TXT, Word DOCX e HTML usano Tesseract sul PC con i modelli delle lingue installati; per i PDF, Poppler crea immagini temporanee delle pagine alla risoluzione DPI scelta. Non vengono inviate immagini o parole riconosciute a un servizio di riconoscimento remoto. Senza OCR, le destinazioni testuali dei PDF usano il testo incorporato estratto da Poppler. DOCX e HTML vengono ricostruiti come file locali senza risorse collegate.

Compressione, divisione e unione dei PDF avvengono nell'app sul PC, senza un servizio remoto. Compressione e copia del risultato non sono operazioni di rimozione dei metadati. Divisione e unione ricostruiscono il documento dalle pagine e non trasferiscono metadati del documento, segnalibri o tag; i dati presenti nei contenuti delle pagine possono restare nel risultato.

**PDF (Bulk)** mantiene un lavoro separato per sorgente nella coda rapida. **PDF (Merge)** e invece una sessione interattiva: l'app copia i PDF e converte le immagini e i documenti compatibili in PDF temporanei, in una cartella temporanea dell'utente nascosta in Esplora file. Questi file possono contenere il contenuto completo delle sorgenti. L'ordine delle singole pagine resta nella sessione e non viene registrato nei lavori `job.json` della coda rapida; soltanto il PDF finale viene salvato nella destinazione scelta. I temporanei possono restare disponibili mentre l'editor e aperto, anche dopo un salvataggio; l'app tenta di rimuoverli all'annullamento o alla chiusura della sessione. Un arresto improvviso puo lasciare residui. LibreOffice e libheif ricevono gli input necessari quando richiesti dal formato; il comportamento delle risorse HTML collegate resta quello descritto sopra.


### Informazioni conservate sul dispositivo

L'app salva preferenze di conversione, elenco dei formati di uscita visibili, lingua dell'interfaccia, lingua e attivazione OCR, separatore CSV, operazione PDF, cartella di destinazione e percorsi degli strumenti nelle proprie cartelle utente. La coda rapida registra nomi e percorsi dei file, destinazioni, formato, impostazioni, motori utilizzati, date, avanzamento ed eventuali errori. L'ultimo resoconto contiene data, formato, risultati e messaggi di errore, che possono includere percorsi o diagnostica dei motori.

Questi dati sono memorizzati come file locali, fra cui `preferences.json`, `report-last.json` e i record `job.json`, nelle cartelle dati dell'app sotto `%APPDATA%` e `%LOCALAPPDATA%`. In un'installazione MSIX Windows puo reindirizzare tali percorsi nei dati privati del pacchetto. I file di preferenze, stato e risultato non sono cifrati dall'app: sono soggetti ai permessi e alle protezioni dell'account Windows e del dispositivo.

Nella cartella di destinazione vengono creati temporanei, prenotazioni dei nomi e collegamenti segnaposto con informazioni sul lavoro. I temporanei OCR possono contenere immagini delle scansioni e testo riconosciuto; quelli HEIC/AVIF possono contenere una rappresentazione PNG dell'immagine. L'estensione Explorer usa inoltre un JSON temporaneo per passare la selezione all'app. L'app tenta la pulizia dei temporanei al termine o all'annullamento; un arresto improvviso puo lasciare residui. I risultati convertiti restano nella cartella scelta.

La pulizia automatica della coda, eseguita quando il processo di lavoro riprende o termina la coda, elimina i record completati o annullati dopo 7 giorni e quelli falliti dopo 30 giorni; tenta inoltre di mantenere al massimo i 300 record terminali piu recenti. Non elimina originali, risultati o lavori attivi. I record illeggibili isolati e gli eventuali file estranei vengono conservati. Il comando **Rimuovi segnaposto** per i lavori falliti elimina soltanto i collegamenti ancora appartenenti al lavoro, mantenendo il record e i risultati.

Preferenze e ultimo resoconto non hanno una scadenza automatica. Per rimuovere manualmente i dati locali, termina o annulla i lavori, attendi la fine dei processi e chiudi l'app. La cancellazione dei record della coda attiva puo impedire il recupero dei lavori. I risultati si gestiscono e si eliminano con Esplora file. La rimozione dell'app e la gestione dei suoi dati da parte di Windows dipendono dal metodo di installazione.

### Collegamenti e servizi esterni

La sezione **Motori** delle **Impostazioni** contiene collegamenti ai siti di FFmpeg, Poppler, LibreOffice, Tesseract e libheif. Selezionare un collegamento apre il browser configurato, cui si applicano le regole del sito visitato e del browser.

Dalla 0.4.0 puoi richiedere esplicitamente il download di FFmpeg e Poppler nelle Impostazioni, anche nell'edizione Store. La conferma mostra origine, versione, dimensioni, licenze e destinazione. Il download usa HTTPS verso le distribuzioni Gyan e Poppler per Windows ospitate su GitHub e i relativi CDN. I destinatari ricevono l'indirizzo IP e i normali dati della richiesta HTTP, fra cui User-Agent e versione dell'app; **non vengono caricati i file scelti per la conversione**. I distributori applicano le proprie regole ai dati delle richieste.

Gli archivi sono verificati con SHA-256 prima dell'estrazione nella cartella utente `%LOCALAPPDATA%\ConvertHere\tools`, mantenendo licenze e documentazione upstream. I motori restano disponibili per gli avvii successivi; i temporanei di download ed estrazione vengono rimossi normalmente dopo completamento, annullamento o errore, ma un arresto improvviso puo lasciare residui. I percorsi possono essere reindirizzati da Windows nelle installazioni MSIX. Le conversioni e l'OCR continuano a essere locali. Il download non e un aggiornamento automatico dell'app: installazione e aggiornamenti Store sono gestiti da Microsoft. Gli eventuali script della distribuzione EXE separata contattano i distributori soltanto quando avviati.

Anche cartelle di rete, sincronizzazione cloud, antivirus e funzioni diagnostiche di Windows dipendono dalle impostazioni del dispositivo e dagli strumenti esterni; questa informativa descrive il codice di Convert Here.

### Controlli e richieste

Puoi scegliere file e destinazioni, configurare i motori e annullare i lavori. Non esiste un archivio cloud dei file presso lo sviluppatore. Per richieste su questa informativa scrivi a [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). La presente informativa descrive il comportamento della versione indicata sopra.

## English

### Files and local processing

Convert Here reads files you select through File Explorer, a file picker or drag and drop, then writes results to your configured destination. Format conversion and OCR take place on your PC. The software is free, with no subscriptions, in-app purchases or usage-based charges. The app does not send files or paths to a server operated by the developer and contains no telemetry service, advertising or its own account-registration system. The app does not modify source files.

Optional local FFmpeg, Poppler, LibreOffice, Tesseract and libheif executables receive the paths and options needed for conversion. They are separate programs installed or selected by the user and may have their own settings and privacy statements. Convert Here does not control all their behavior. No transfer to the app developer is part of this workflow.

Excel/CSV and HTML-to-PDF conversion use LibreOffice with a separate temporary profile. Macros and Office link updates are disabled. HTML documents may still reference local or remote images, CSS or other resources; the engine may read these during import, contacting the respective servers for remote resources. The selected CSV delimiter is applied locally to imports and exports. Reading and creating HEIC/HEIF and AVIF files uses local libheif executables with their DLLs and plugins.

Image-to-TXT OCR and optional PDF-to-TXT, Word DOCX or HTML OCR use Tesseract on your PC with installed language models. For PDFs, Poppler creates temporary page images at the selected DPI. No images or recognized words are sent to a remote recognition service. Without OCR, PDF text outputs use embedded text extracted by Poppler. DOCX and HTML outputs are reconstructed as local files without linked resources.

PDF compression, splitting and merging run in the app on your PC without a remote service. Compression and output copying do not remove metadata. Splitting and merging rebuild the document from its pages and omit document metadata, bookmarks and tags; data within page content may remain in the result.

**PDF (Bulk)** retains a separate quick-queue job for each source. **PDF (Merge)** is an interactive session: the app copies PDFs and converts compatible images and documents to temporary PDFs in a user temporary folder hidden in File Explorer. These files may contain the sources' full content. Individual page order stays within the session and is not recorded in quick-queue `job.json` records; only the final PDF is saved to the selected destination. Temporary files may remain available while the editor is open, including after a save; the app attempts to remove them when cancelling or closing the session. An unexpected shutdown may leave residual files. LibreOffice and libheif receive the required inputs when needed for a format; linked HTML resources retain the behavior described above.


### Information stored on your device

The app stores conversion preferences, the list of visible output formats, interface language, OCR language and enablement, CSV delimiter, PDF operation, output-folder and tool paths. Its persistent quick-conversion queue stores source and destination paths, formats, settings, tool paths, timestamps, progress and errors. The latest report stores its date, format, results and error messages, which may contain paths or engine diagnostics.

Local files include `preferences.json`, `report-last.json` and `job.json` records in the app's user-data folders under `%APPDATA%` and `%LOCALAPPDATA%`. In an MSIX installation, Windows may redirect these locations into private package data. The app does not encrypt preferences, state or output files; Windows account permissions and device protections apply.

Temporary conversion files, name reservations and placeholder shortcuts are created in the output folder. OCR temporary files may contain page images and recognized text; HEIC/AVIF temporary files may contain a PNG representation of the image. The Explorer extension also uses a temporary selection JSON file. The app attempts to remove temporary files after completion or cancellation; an unexpected shutdown may leave residual files. Completed results remain in the selected folder.

Automatic queue cleanup runs when the worker resumes or finishes the queue. It removes completed or cancelled records after 7 days and failed records after 30 days, and attempts to retain at most the 300 newest terminal records. Original files, outputs and active jobs are preserved. Quarantined unreadable records and unrelated files are retained. **Rimuovi segnaposto** for a failed job removes only shortcuts that still belong to that job, retaining its record and outputs.

Preferences and the last report have no automatic expiry. To remove local data manually, finish or cancel jobs, wait for processing to stop, and close the app. Removing active queue records can prevent job recovery. You can manage and delete outputs in File Explorer. Windows handling of app data during uninstall depends on the installation method.

### External links and services

The **Motori** section of **Impostazioni** links to the FFmpeg, Poppler, LibreOffice, Tesseract and libheif websites. Selecting a link opens the configured browser, subject to the browser's and destination site's own policies.

From 0.4.0 you can explicitly request FFmpeg and Poppler downloads in Impostazioni, including in the Store edition. The confirmation shows the source, version, size, licenses and destination. Downloads use HTTPS to the Gyan and Poppler Windows distributions hosted on GitHub and their CDNs. Recipients receive your IP address and normal HTTP request data, including the User-Agent and app version; **files selected for conversion are not uploaded**. Download providers apply their own policies to request data.

Archives are verified using SHA-256 before extraction into the user's `%LOCALAPPDATA%\ConvertHere\tools` folder, retaining upstream licenses and documentation. Engines remain available for later use. Download and extraction temporary files are normally removed after completion, cancellation or error; an unexpected shutdown may leave residual files. Windows may redirect paths in MSIX installations. Conversion and OCR remain local. Engine downloads are not automatic application updates: Microsoft manages Store installation and app updates. Separate EXE-distribution scripts contact the providers only when run.

Network folders, cloud synchronization, antivirus and Windows diagnostics depend on your device settings and external tools; this statement describes Convert Here's application code.

### Controls and questions

You choose the files, destination and optional engines, and can cancel jobs. The developer does not hold a cloud copy of your files. For privacy questions, contact [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). This statement describes the behavior of the version identified above.
