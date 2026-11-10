# Convert Here — informativa sulla privacy / privacy statement

Informativa aggiornata al **10 novembre 2026**. Descrive conversione, OCR, trascrizione locale e installazione facoltativa dei motori di Convert Here. Per versioni e disponibilità, consulta lo [stato delle pubblicazioni](store-submission.md).

Updated **10 November 2026**. Covers conversion, OCR, local speech transcription and optional engine installation in Convert Here. See the [publication record](store-submission.md) for versions and availability.

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

Convert Here legge i file che scegli tramite Esplora file, selettore o trascinamento e crea i risultati nella destinazione configurata. Conversioni, OCR e trascrizione del parlato avvengono sul PC. Il software è gratuito, senza abbonamenti, acquisti in-app o costi a consumo. L'app non invia file o percorsi a un server dello sviluppatore e non contiene servizi di telemetria, pubblicità o registrazione di account propri. Gli originali non vengono modificati dall'app.

Se usi FFmpeg, Poppler, LibreOffice, Tesseract, libheif o Whisper, gli eseguibili locali ricevono i percorsi e le opzioni necessari all'elaborazione. Sono programmi separati, scaricati su richiesta, installati o selezionati dall'utente. Possono avere proprie impostazioni e informative; Convert Here non controlla tutte le loro funzioni. Non è prevista una trasmissione allo sviluppatore dell'app.

Le conversioni Excel/CSV e HTML in PDF usano LibreOffice con un profilo temporaneo separato. Le macro e l'aggiornamento dei collegamenti Office vengono disabilitati. Un documento HTML puo comunque contenere riferimenti a immagini, CSS o altre risorse locali o remote: il motore puo leggerle durante l'apertura, contattando i rispettivi server per quelle remote. Il separatore CSV scelto viene applicato localmente in ingresso e uscita. La lettura e la creazione di HEIC/HEIF e AVIF usano gli eseguibili locali libheif con le relative DLL e i plugin.

L'OCR di immagini in TXT e l'OCR facoltativo di PDF in TXT, Word DOCX e HTML usano Tesseract sul PC con i modelli delle lingue installati; per i PDF, Poppler crea immagini temporanee delle pagine alla risoluzione DPI scelta. Non vengono inviate immagini o parole riconosciute a un servizio di riconoscimento remoto. Senza OCR, le destinazioni testuali dei PDF usano il testo incorporato estratto da Poppler. DOCX e HTML vengono ricostruiti come file locali senza risorse collegate.

La trascrizione audio/video → TXT usa FFmpeg/ffprobe e Whisper CPU con il modello Tiny multilingue, tutti locali. Il modello è un download opzionale e non viene inviato il parlato a servizi cloud. L'intervallo selezionato viene suddiviso in blocchi di massimo dieci minuti: WAV mono 16 kHz e TXT intermedi vengono scritti in una cartella temporanea nascosta della destinazione e rimossi normalmente a completamento o annullamento. Possono contenere il parlato e il testo del segmento; un arresto improvviso può lasciare residui. Il TXT finale resta nella destinazione scelta.

L'ingresso SVG viene rasterizzato nell'app con resvg. Non vengono caricate risorse esterne: SVGZ, script, DTD, riferimenti esterni e immagini raster incorporate sono rifiutati. I font necessari al percorso nativo sono incorporati nell'app; questo comportamento è distinto dall'apertura HTML con LibreOffice descritta sopra.

Compressione, divisione e unione dei PDF avvengono nell'app sul PC, senza un servizio remoto. Compressione e copia del risultato non sono operazioni di rimozione dei metadati. Divisione e unione ricostruiscono il documento dalle pagine e non trasferiscono metadati del documento, segnalibri o tag; i dati presenti nei contenuti delle pagine possono restare nel risultato.

**PDF (Bulk)** mantiene un lavoro separato per sorgente nella coda rapida. **PDF (Merge)** e invece una sessione interattiva: l'app copia i PDF e converte le immagini e i documenti compatibili in PDF temporanei, in una cartella temporanea dell'utente nascosta in Esplora file. Questi file possono contenere il contenuto completo delle sorgenti. L'ordine delle singole pagine resta nella sessione e non viene registrato nei lavori `job.json` della coda rapida; soltanto il PDF finale viene salvato nella destinazione scelta. I temporanei possono restare disponibili mentre l'editor e aperto, anche dopo un salvataggio; l'app tenta di rimuoverli all'annullamento o alla chiusura della sessione. Un arresto improvviso puo lasciare residui. LibreOffice e libheif ricevono gli input necessari quando richiesti dal formato; il comportamento delle risorse HTML collegate resta quello descritto sopra.


### Informazioni conservate sul dispositivo

La configurazione guidata propone i motori al primo avvio della finestra principale. Dopo il completamento o la scelta di saltarla, un indicatore locale (`engine-setup-v1.done`) accanto alle preferenze evita di riproporla. Le selezioni del wizard restano nella sessione; i percorsi dei motori installati vengono salvati nelle preferenze. I download iniziano soltanto dopo la conferma delle origini, dimensioni e licenze. Puoi riaprire la guida dalle Impostazioni.

L'app salva preferenze di conversione, elenco dei formati di uscita visibili, lingua dell'interfaccia, lingua e attivazione OCR, lingua del parlato e thread CPU, separatore CSV, operazione PDF, cartella di destinazione e percorsi degli strumenti nelle proprie cartelle utente. Salva inoltre profili con nome, opzioni, visibilità nel menu e scelta del predefinito: possono includere ritaglio relativo, dimensioni, tela, colore, qualità, limite di peso e impostazioni pagina/testo, oltre all'eventuale cartella di destinazione. I profili non includono il file di prova, il suo contenuto, il testo scritto nell'editor o la trascrizione completa. La coda rapida registra nomi e percorsi dei file, destinazioni, formato, impostazioni, motori utilizzati, date, avanzamento ed eventuali errori. L'ultimo resoconto contiene data, formato, risultati e messaggi di errore, che possono includere percorsi o diagnostica dei motori.

I profili possono inoltre conservare filtri sulle estensioni in ingresso, limiti di peso e durata, taglio temporale, gain/muto, opzioni FFmpeg, `speech.language`/`speech.threads` e metadati inseriti manualmente, come titolo o artista. Importazione ed esportazione JSON avvengono su file scelti dall'utente. L'esportazione comprende nome, visibilità, filtro degli ingressi e impostazioni della ricetta, inclusa l'eventuale `output_dir`; non aggiunge nomi dei file di prova, contenuti da convertire o percorsi degli eseguibili dei motori. Il JSON esportato può quindi rivelare la cartella di destinazione e i parametri scelti a chi lo riceve.

Nell'editor, immagine di lavoro, testo digitato e anteprime codificate sono mantenuti in memoria durante la sessione e non sono salvati automaticamente nelle preferenze o nei profili. Il risultato viene scritto con Salva con nome. PNG, JPEG, WebP e l'impaginazione nativa A4/A5 sono elaborati localmente; il codec WebP integrato non carica immagini su un servizio remoto. Per i formati che richiedono libheif possono essere usati i temporanei del decoder descritti sotto. La cronologia conserva esiti e percorsi, senza le immagini di lavoro o il testo integrale digitato.

Nell'editor audio/video, FFmpeg/ffprobe leggono la sorgente per durata, forma d'onda e fotogrammi. La forma d'onda e le anteprime visualizzate restano in memoria; i fotogrammi possono passare da PNG in una cartella temporanea privata, rimossa dopo la preparazione o l'annullamento. Il salvataggio converte il segmento scelto in un nuovo file; applicare le impostazioni o salvare un profilo non salva automaticamente il contenuto multimediale. La cronologia non aggiunge il contenuto audio/video o la trascrizione completa ai record.

La cronologia GUI salva in `history.json`, accanto alle preferenze, fino a 300 tentativi conclusi di conversione, editor e PDF Merge entro un limite di 1 MiB. I record possono contenere date, stato, operazione, impostazioni, percorsi delle sorgenti e dei risultati ed errori, inclusa la diagnostica dei motori. La scrittura avviene in background; un file illeggibile viene conservato e segnalato. Questa cronologia locale è distinta dai record della coda rapida e dalla loro pulizia temporale descritta sotto.

Questi dati sono memorizzati come file locali, fra cui `preferences.json`, `history.json`, `report-last.json` e i record `job.json`, nelle cartelle dati dell'app sotto `%APPDATA%` e `%LOCALAPPDATA%`. In un'installazione MSIX Windows puo reindirizzare tali percorsi nei dati privati del pacchetto. I file di preferenze, stato e risultato non sono cifrati dall'app: sono soggetti ai permessi e alle protezioni dell'account Windows e del dispositivo.

Nella cartella di destinazione vengono creati temporanei, prenotazioni dei nomi e collegamenti segnaposto con informazioni sul lavoro. I temporanei OCR possono contenere immagini delle scansioni e testo riconosciuto; quelli HEIC/AVIF possono contenere una rappresentazione PNG dell'immagine. L'estensione Explorer usa inoltre un JSON temporaneo per passare la selezione all'app. L'app tenta la pulizia dei temporanei al termine o all'annullamento; un arresto improvviso puo lasciare residui. I risultati convertiti restano nella cartella scelta.

La pulizia automatica della coda, eseguita quando il processo di lavoro riprende o termina la coda, elimina i record completati o annullati dopo 7 giorni e quelli falliti dopo 30 giorni; tenta inoltre di mantenere al massimo i 300 record terminali piu recenti. Non elimina originali, risultati o lavori attivi. I record illeggibili isolati e gli eventuali file estranei vengono conservati. Il comando **Rimuovi segnaposto** per i lavori falliti elimina soltanto i collegamenti ancora appartenenti al lavoro, mantenendo il record e i risultati.

Preferenze e ultimo resoconto non hanno una scadenza automatica. Per rimuovere manualmente i dati locali, termina o annulla i lavori, attendi la fine dei processi e chiudi l'app. La cancellazione dei record della coda attiva puo impedire il recupero dei lavori. I risultati si gestiscono e si eliminano con Esplora file. La rimozione dell'app e la gestione dei suoi dati da parte di Windows dipendono dal metodo di installazione.

### Collegamenti e servizi esterni

La sezione **Motori** delle **Impostazioni** contiene collegamenti ai siti di FFmpeg, Poppler, LibreOffice, Tesseract, libheif e Whisper/modello Tiny. Selezionare un collegamento apre il browser configurato, cui si applicano le regole del sito visitato e del browser.

Puoi richiedere esplicitamente il download di FFmpeg e Poppler nelle Impostazioni, anche nell'edizione Store. La versione 0.8.0 aggiunge Tesseract con modelli italiano/inglese, libheif e Whisper con Tiny multilingue. La conferma mostra origine, versione, dimensioni, licenze e destinazione. Il download usa HTTPS verso Gyan/Poppler per Windows su GitHub, MSYS2 e i relativi mirror, GitHub per licenze e modelli OCR, Hugging Face per Tiny e i rispettivi CDN. I destinatari ricevono l'indirizzo IP e i normali dati della richiesta HTTP, fra cui User-Agent e versione dell'app; **non vengono caricati i file scelti per la conversione, immagini, audio o trascrizioni**. I distributori applicano le proprie regole ai dati delle richieste. I motori e il modello non sono inclusi nel pacchetto dell'app né scaricati senza richiesta.

Gli archivi e i singoli asset sono verificati con SHA-256 prima dell'estrazione o pubblicazione nella cartella utente `%LOCALAPPDATA%\ConvertHere\tools`, mantenendo licenze e documentazione upstream. Il download non esegue i programmi scaricati. I motori e il modello restano disponibili per gli avvii successivi; i temporanei di download ed estrazione vengono rimossi normalmente dopo completamento, annullamento o errore, ma un arresto improvviso può lasciare residui. I percorsi possono essere reindirizzati da Windows nelle installazioni MSIX. Conversioni, OCR e trascrizione continuano a essere locali. Il download non è un aggiornamento automatico dell'app: installazione e aggiornamenti Store sono gestiti da Microsoft. Gli eventuali script della distribuzione EXE separata contattano i distributori soltanto quando avviati.

Anche cartelle di rete, sincronizzazione cloud, antivirus e funzioni diagnostiche di Windows dipendono dalle impostazioni del dispositivo e dagli strumenti esterni; questa informativa descrive il codice di Convert Here.

### Controlli e richieste

Puoi scegliere file e destinazioni, configurare i motori e annullare i lavori. Non esiste un archivio cloud dei file presso lo sviluppatore. Per richieste su questa informativa scrivi a [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). Lo stato di pubblicazione GitHub e certificazione Store è indicato nell'apertura dell'informativa.

## English

### Files and local processing

Convert Here reads files you select through File Explorer, a file picker or drag and drop, then writes results to your configured destination. Conversion, OCR and speech transcription take place on your PC. The software is free, with no subscriptions, in-app purchases or usage-based charges. The app does not send files or paths to a server operated by the developer and contains no telemetry service, advertising or its own account-registration system. The app does not modify source files.

Optional local FFmpeg, Poppler, LibreOffice, Tesseract, libheif and Whisper executables receive the paths and options needed for processing. They are separate programs downloaded on request, installed or selected by the user and may have their own settings and privacy statements. Convert Here does not control all their behavior. No transfer to the app developer is part of this workflow.

Excel/CSV and HTML-to-PDF conversion use LibreOffice with a separate temporary profile. Macros and Office link updates are disabled. HTML documents may still reference local or remote images, CSS or other resources; the engine may read these during import, contacting the respective servers for remote resources. The selected CSV delimiter is applied locally to imports and exports. Reading and creating HEIC/HEIF and AVIF files uses local libheif executables with their DLLs and plugins.

Image-to-TXT OCR and optional PDF-to-TXT, Word DOCX or HTML OCR use Tesseract on your PC with installed language models. For PDFs, Poppler creates temporary page images at the selected DPI. No images or recognized words are sent to a remote recognition service. Without OCR, PDF text outputs use embedded text extracted by Poppler. DOCX and HTML outputs are reconstructed as local files without linked resources.

Audio/video → TXT transcription uses local FFmpeg/ffprobe and Whisper CPU with the Tiny multilingual model. The model is an optional download; speech is not sent to cloud services. The selected interval is divided into chunks of at most ten minutes: intermediate mono 16 kHz WAV and TXT files are written into a hidden temporary folder in the destination and normally removed on completion or cancellation. They can contain the segment's speech and text; an abrupt shutdown can leave residual files. Final TXT stays in the selected destination.

SVG input is rasterized within the app using resvg. No external resources are loaded: SVGZ, scripts, DTDs, external references and embedded raster images are rejected. Fonts for the native route are bundled in the app; this differs from LibreOffice HTML import described above.

PDF compression, splitting and merging run in the app on your PC without a remote service. Compression and output copying do not remove metadata. Splitting and merging rebuild the document from its pages and omit document metadata, bookmarks and tags; data within page content may remain in the result.

**PDF (Bulk)** retains a separate quick-queue job for each source. **PDF (Merge)** is an interactive session: the app copies PDFs and converts compatible images and documents to temporary PDFs in a user temporary folder hidden in File Explorer. These files may contain the sources' full content. Individual page order stays within the session and is not recorded in quick-queue `job.json` records; only the final PDF is saved to the selected destination. Temporary files may remain available while the editor is open, including after a save; the app attempts to remove them when cancelling or closing the session. An unexpected shutdown may leave residual files. LibreOffice and libheif receive the required inputs when needed for a format; linked HTML resources retain the behavior described above.


### Information stored on your device

The setup wizard offers engines on the first launch of the main window. Completing or skipping it stores a local marker (`engine-setup-v1.done`) beside preferences so it is not shown again automatically. Wizard selections remain in the session; installed engine paths are saved in preferences. Downloads begin only after confirmation of sources, sizes and licenses. You can reopen the wizard from Settings.

The app stores conversion preferences, the list of visible output formats, interface language, OCR language and enablement, speech language and CPU threads, CSV delimiter, PDF operation, output-folder and tool paths. It also stores named profiles, their options, menu visibility and default selection. Options can include relative crop, dimensions, canvas, color, quality, byte limit and page/text layout, plus a configured output folder. Profiles do not contain the sample file, its content, text written in the editor or the full transcript. The persistent quick-conversion queue stores source and destination paths, formats, settings, tool paths, timestamps, progress and errors. The latest report stores its date, format, results and error messages, which may contain paths or engine diagnostics.

Profiles can also store input-extension filters, size and duration limits, trimming, gain/mute, FFmpeg options, `speech.language`/`speech.threads` and manually entered metadata such as title or artist. JSON import/export uses files chosen by the user. Exports include the profile name, visibility, input filter and recipe settings, including an optional `output_dir`; they do not add sample input filenames, content to convert or engine executable paths. An exported JSON file can therefore disclose the configured destination folder and parameters to its recipients.

In the editor, the working image, typed text and encoded previews remain in memory during the session and are not automatically written to preferences or profiles. Save as writes the requested output. PNG, JPEG, WebP and native A4/A5 layout are processed locally; the built-in WebP codec does not upload images to a remote service. Formats that need libheif may use the decoder temporary files described below. History stores outcomes and paths, not working images or the complete typed text.

In the audio/video editor, FFmpeg/ffprobe read the source for duration, waveform and frames. The waveform and displayed previews remain in memory; frames may pass through PNG files in a private temporary folder removed after preparation or cancellation. Saving converts the selected segment into a new file; applying settings or saving a profile does not automatically save media content. History does not add audio/video content or the full transcript to its records.

GUI history stores up to 300 completed conversion, editor and PDF Merge attempts in `history.json` beside preferences, within a 1 MiB limit. Records may include dates, status, operation, settings, source and result paths, and errors including engine diagnostics. Writes run in the background; an unreadable history file is retained and reported. This local history is separate from quick-queue records and their time-based cleanup described below.

Local files include `preferences.json`, `history.json`, `report-last.json` and `job.json` records in the app's user-data folders under `%APPDATA%` and `%LOCALAPPDATA%`. In an MSIX installation, Windows may redirect these locations into private package data. The app does not encrypt preferences, state or output files; Windows account permissions and device protections apply.

Temporary conversion files, name reservations and placeholder shortcuts are created in the output folder. OCR temporary files may contain page images and recognized text; HEIC/AVIF temporary files may contain a PNG representation of the image. The Explorer extension also uses a temporary selection JSON file. The app attempts to remove temporary files after completion or cancellation; an unexpected shutdown may leave residual files. Completed results remain in the selected folder.

Automatic queue cleanup runs when the worker resumes or finishes the queue. It removes completed or cancelled records after 7 days and failed records after 30 days, and attempts to retain at most the 300 newest terminal records. Original files, outputs and active jobs are preserved. Quarantined unreadable records and unrelated files are retained. **Rimuovi segnaposto** for a failed job removes only shortcuts that still belong to that job, retaining its record and outputs.

Preferences and the last report have no automatic expiry. To remove local data manually, finish or cancel jobs, wait for processing to stop, and close the app. Removing active queue records can prevent job recovery. You can manage and delete outputs in File Explorer. Windows handling of app data during uninstall depends on the installation method.

### External links and services

The **Conversion engines** section of **Settings** links to the FFmpeg, Poppler, LibreOffice, Tesseract, libheif and Whisper/Tiny model websites. Selecting a link opens the configured browser, subject to the browser's and destination site's own policies.

You can explicitly request FFmpeg and Poppler downloads in Settings, including in the Store edition. Version 0.8.0 adds Tesseract with Italian/English models, libheif and Whisper with Tiny multilingual. Confirmation shows the source, version, size, licenses and destination. Downloads use HTTPS to Gyan/Poppler for Windows on GitHub, MSYS2 and its mirrors, GitHub for licenses and OCR models, Hugging Face for Tiny and their CDNs. Recipients receive your IP address and normal HTTP request data, including the User-Agent and app version; **selected conversion files, images, audio and transcripts are not uploaded**. Download providers apply their own policies to request data. Engines and the model are neither included in the app package nor downloaded without a request.

Archives and individual assets are verified using SHA-256 before extraction or publication into the user's `%LOCALAPPDATA%\ConvertHere\tools` folder, retaining upstream licenses and documentation. Downloading does not execute the programs. Engines and the model remain available for later use. Download and extraction temporary files are normally removed after completion, cancellation or error; an unexpected shutdown may leave residual files. Windows may redirect paths in MSIX installations. Conversion, OCR and transcription remain local. Engine downloads are not automatic application updates: Microsoft manages Store installation and app updates. Separate EXE-distribution scripts contact the providers only when run.

Network folders, cloud synchronization, antivirus and Windows diagnostics depend on your device settings and external tools; this statement describes Convert Here's application code.

### Controls and questions

You choose the files, destination and optional engines, and can cancel jobs. The developer does not hold a cloud copy of your files. For privacy questions, contact [salvo@salvatorecorsi.com](mailto:salvo@salvatorecorsi.com). GitHub publication and Store certification status are stated at the beginning of this statement.
