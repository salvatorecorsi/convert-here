# Convert Here — informativa sulla privacy / privacy statement

Bozza del 4 novembre 2026, basata sul codice applicativo attuale. **Prima della pubblicazione occorre completare i dati seguenti e verificare i percorsi dati sul MSIX finale.** Questo file non costituisce un URL privacy gia pubblicato.

| Dato | Valore |
| --- | --- |
| Applicazione | Convert Here |
| Brand editore | CORSIVO |
| Autore / sviluppatore | Salvatore Corsi |
| Identita legale del titolare | DA COMPLETARE con i dati reali del proprietario; il brand non identifica automaticamente un soggetto giuridico. |
| Contatto privacy e supporto | DA COMPLETARE con un recapito effettivo del proprietario. |
| URL pubblico della presente informativa | DA COMPLETARE dopo la pubblicazione HTTPS. |

## Italiano

### File e conversioni

Convert Here legge i file che scegli tramite Esplora file, selettore o trascinamento e crea i risultati nella destinazione configurata. Le conversioni di formato avvengono sul PC. La trascrizione audio TXT con OpenAI e invece online, facoltativa e descritta qui sotto. L'app non invia file o percorsi a un server di Corsivo e non contiene servizi di telemetria, pubblicita o registrazione di account propri. Gli originali non vengono modificati dall'app.

Se usi FFmpeg, Poppler, LibreOffice, Tesseract o libheif, gli eseguibili locali ricevono i percorsi e le opzioni necessari alla conversione. Sono programmi separati, installati o selezionati dall'utente. Possono avere proprie impostazioni e informative; Convert Here non controlla tutte le loro funzioni. Non e prevista una trasmissione allo sviluppatore dell'app.

Le conversioni Excel/CSV e HTML in PDF usano LibreOffice con un profilo temporaneo separato. Le macro e l'aggiornamento dei collegamenti Office vengono disabilitati. Un documento HTML puo comunque contenere riferimenti a immagini, CSS o altre risorse locali o remote: il motore puo leggerle durante l'apertura, contattando i rispettivi server per quelle remote. Il separatore CSV scelto viene applicato localmente in ingresso e uscita. La lettura e la creazione di HEIC/HEIF e AVIF usano gli eseguibili locali libheif con le relative DLL e i plugin.

L'OCR di immagini in TXT e l'OCR facoltativo di PDF in TXT, Word DOCX e HTML usano Tesseract sul PC con i modelli delle lingue installati; per i PDF, Poppler crea immagini temporanee delle pagine alla risoluzione DPI scelta. Non vengono inviate immagini o parole riconosciute a OpenAI o a Corsivo. Senza OCR, le destinazioni testuali dei PDF usano il testo incorporato estratto da Poppler. DOCX e HTML vengono ricostruiti come file locali senza risorse collegate. Queste funzioni sono separate dalla trascrizione audio online.

Compressione, divisione e unione dei PDF avvengono nell'app sul PC, senza un servizio remoto. Compressione e copia del risultato non sono operazioni di rimozione dei metadati. Divisione e unione ricostruiscono il documento dalle pagine e non trasferiscono metadati del documento, segnalibri o tag; i dati presenti nei contenuti delle pagine possono restare nel risultato.

**PDF (Bulk)** mantiene un lavoro separato per sorgente nella coda rapida. **PDF (Merge)** e invece una sessione interattiva: l'app copia i PDF e converte le immagini e i documenti compatibili in PDF temporanei, in una cartella temporanea dell'utente nascosta in Esplora file. Questi file possono contenere il contenuto completo delle sorgenti. L'ordine delle singole pagine resta nella sessione e non viene registrato nei lavori `job.json` della coda rapida; soltanto il PDF finale viene salvato nella destinazione scelta. I temporanei possono restare disponibili mentre l'editor e aperto, anche dopo un salvataggio; l'app tenta di rimuoverli all'annullamento o alla chiusura della sessione. Un arresto improvviso puo lasciare residui. LibreOffice e libheif ricevono gli input necessari quando richiesti dal formato; il comportamento delle risorse HTML collegate resta quello descritto sopra.

### Trascrizione audio online facoltativa

La trascrizione non e necessaria per le conversioni locali. Per abilitarla inserisci una tua chiave OpenAI API nella pagina Strumenti, dopo l'avviso sull'invio e sui costi. Scegliendo **TXT con OpenAI** per un audio richiedi l'invio a OpenAI: l'app trasmette via HTTPS l'audio selezionato o i segmenti preparati localmente, i parametri della trascrizione e la chiave necessaria ad autenticare la richiesta. Il file e presentato con un nome generico `audio.<estensione>`, senza il nome o il percorso originale. Il destinatario e `api.openai.com/v1/audio/transcriptions`; Corsivo non fa da intermediario del contenuto. Le voci, le parole pronunciate e gli altri suoni presenti nella registrazione vengono quindi comunicati a OpenAI. Non vengono trasmesse automaticamente le altre conversioni o l'intera coda.

L'invio diretto comprende anche gli eventuali metadati incorporati nel file audio. Il modello usato e `gpt-transcribe`. Puoi scegliere una lingua o lasciarne il riconoscimento automatico. Per formati o dimensioni che richiedono preparazione, FFmpeg crea segmenti audio temporanei sul PC e l'app li invia in sequenza. Dopo il primo segmento, quando disponibile, viene reinviata come contesto una coda di massimo 200 caratteri di parole intere gia trascritte della stessa registrazione, con spazi e caratteri di controllo normalizzati. Il testo viene estratto dalla risposta JSON del servizio e salvato nel TXT di destinazione solo dopo il completamento del lavoro. Puoi annullare l'operazione, ma l'annullamento non ritira audio gia inviato e non annulla eventuali addebiti API gia maturati.

Per trattamento, conservazione e controlli sui dati ricevuti da OpenAI si applicano le condizioni e le impostazioni del tuo account API; l'app non promette cancellazione immediata o una specifica area geografica di elaborazione. Consulta i [controlli dati OpenAI API](https://developers.openai.com/api/docs/guides/your-data). L'accesso e gli eventuali costi API appartengono al tuo account OpenAI e sono separati dall'abbonamento ChatGPT.

### Protezione e rimozione della chiave API

Su Windows la chiave e salvata nel Gestore credenziali dell'utente corrente, con voce `Corsivo.TheConverter.OpenAI`, e protetta da Windows. Non e conservata nei file JSON delle preferenze, nei lavori in coda, nei resoconti o nei log. Viene usata in memoria per autenticare le richieste al servizio. Il pacchetto distribuito non contiene una chiave dello sviluppatore.

**Rimuovi chiave** nelle impostazioni elimina la credenziale salvata e disabilita nuove trascrizioni. Per fermare un lavoro gia in corso occorre anche annullarlo. Questa rimozione non revoca la chiave presso OpenAI e non cancella dati o richieste gia inviati: i controlli dell'account API si gestiscono separatamente presso il fornitore. Rimuovere i soli file di preferenze o la coda non equivale a rimuovere la credenziale.

La voce del Gestore credenziali e indipendente dai file del programma e rimane dopo la disinstallazione EXE o MSIX. Per eliminarla usa **Rimuovi chiave** prima di disinstallare oppure, anche dopo la rimozione dell'app, elimina `Corsivo.TheConverter.OpenAI` in **Gestione credenziali → Credenziali Windows → Credenziali generiche**, nell'account Windows con cui l'hai salvata. Questa operazione riguarda soltanto la copia locale della chiave.

### Informazioni conservate sul dispositivo

L'app salva preferenze di conversione, lingua di trascrizione, lingua e attivazione OCR, separatore CSV, operazione PDF, cartella di destinazione e percorsi degli strumenti nelle proprie cartelle utente. La coda rapida registra nomi e percorsi dei file, destinazioni, formato, impostazioni, motori utilizzati, date, avanzamento ed eventuali errori. L'ultimo resoconto contiene data, formato, risultati e messaggi di errore, che possono includere percorsi o diagnostica dei motori. Nessuno di questi record contiene la chiave API.

Questi dati sono memorizzati come file locali, fra cui `preferences.json`, `report-last.json` e i record `job.json`, nelle cartelle dati dell'app sotto `%APPDATA%` e `%LOCALAPPDATA%`. In un'installazione MSIX Windows puo reindirizzare tali percorsi nei dati privati del pacchetto. I file di preferenze, stato e risultato non sono cifrati dall'app: sono soggetti ai permessi e alle protezioni dell'account Windows e del dispositivo. La chiave API e conservata separatamente come descritto sopra.

Nella cartella di destinazione vengono creati temporanei, prenotazioni dei nomi e collegamenti segnaposto con informazioni sul lavoro. I temporanei OCR possono contenere immagini delle scansioni e testo riconosciuto; quelli HEIC/AVIF possono contenere una rappresentazione PNG dell'immagine. L'estensione Explorer usa inoltre un JSON temporaneo per passare la selezione all'app. L'app tenta la pulizia dei temporanei al termine o all'annullamento; un arresto improvviso puo lasciare residui. I risultati convertiti restano nella cartella scelta.

La pulizia automatica della coda, eseguita quando il processo di lavoro riprende o termina la coda, elimina i record completati o annullati dopo 7 giorni e quelli falliti dopo 30 giorni; tenta inoltre di mantenere al massimo i 300 record terminali piu recenti. Non elimina originali, risultati o lavori attivi. I record illeggibili isolati e gli eventuali file estranei vengono conservati. Il comando **Rimuovi segnaposto** per i lavori falliti elimina soltanto i collegamenti ancora appartenenti al lavoro, mantenendo il record e i risultati.

Preferenze e ultimo resoconto non hanno una scadenza automatica. Per rimuovere manualmente i dati locali, termina o annulla i lavori, attendi la fine dei processi e chiudi l'app. La cancellazione dei record della coda attiva puo impedire il recupero dei lavori. I risultati si gestiscono e si eliminano con Esplora file. La rimozione dell'app e la gestione dei suoi dati da parte di Windows dipendono dal metodo di installazione.

### Collegamenti e servizi esterni

La pagina Strumenti contiene collegamenti ai siti di FFmpeg, Poppler, LibreOffice, Tesseract e libheif, e alla gestione delle chiavi API OpenAI. Si aprono solo quando li selezioni, nel browser configurato. Il browser, il sito visitato e gli strumenti esterni applicano le proprie regole. Anche cartelle di rete, sincronizzazione cloud, antivirus e funzioni diagnostiche di Windows dipendono dalle impostazioni del tuo dispositivo: questa informativa riguarda il codice di Convert Here.

L'edizione Store prevista non scarica automaticamente motori o aggiornamenti eseguibili. Installazione e aggiornamenti Store sono gestiti da Microsoft. L'eventuale script per scaricare motori della distribuzione EXE separata contatta i distributori indicati nello script quando viene avviato; non fa parte del pacchetto Store previsto.

### Controlli e richieste

Puoi scegliere file e destinazioni, configurare i motori, rimuovere la chiave API e annullare i lavori. Non esiste un archivio cloud dei file presso Corsivo; per l'audio inviato a OpenAI valgono i controlli del servizio descritti sopra. Per richieste su questa informativa usa il contatto effettivo da completare sopra. La presente informativa deve essere aggiornata se cambiano le funzioni o i dati trattati.

## English

### Files and local processing

Convert Here reads files you select through File Explorer, a file picker or drag and drop, then writes results to your configured destination. Format conversion takes place on your PC. Optional OpenAI audio-to-text transcription is online and is described below. The app does not send files or paths to a Corsivo server and contains no telemetry service, advertising or its own account-registration system. The app does not modify source files.

Optional local FFmpeg, Poppler, LibreOffice, Tesseract and libheif executables receive the paths and options needed for conversion. They are separate programs installed or selected by the user and may have their own settings and privacy statements. Convert Here does not control all their behavior. No transfer to the app developer is part of this workflow.

Excel/CSV and HTML-to-PDF conversion use LibreOffice with a separate temporary profile. Macros and Office link updates are disabled. HTML documents may still reference local or remote images, CSS or other resources; the engine may read these during import, contacting the respective servers for remote resources. The selected CSV delimiter is applied locally to imports and exports. Reading and creating HEIC/HEIF and AVIF files uses local libheif executables with their DLLs and plugins.

Image-to-TXT OCR and optional PDF-to-TXT, Word DOCX or HTML OCR use Tesseract on your PC with installed language models. For PDFs, Poppler creates temporary page images at the selected DPI. No images or recognized words are sent to OpenAI or Corsivo. Without OCR, PDF text outputs use embedded text extracted by Poppler. DOCX and HTML outputs are reconstructed as local files without linked resources. These features are separate from online audio transcription.

PDF compression, splitting and merging run in the app on your PC without a remote service. Compression and output copying do not remove metadata. Splitting and merging rebuild the document from its pages and omit document metadata, bookmarks and tags; data within page content may remain in the result.

**PDF (Bulk)** retains a separate quick-queue job for each source. **PDF (Merge)** is an interactive session: the app copies PDFs and converts compatible images and documents to temporary PDFs in a user temporary folder hidden in File Explorer. These files may contain the sources' full content. Individual page order stays within the session and is not recorded in quick-queue `job.json` records; only the final PDF is saved to the selected destination. Temporary files may remain available while the editor is open, including after a save; the app attempts to remove them when cancelling or closing the session. An unexpected shutdown may leave residual files. LibreOffice and libheif receive the required inputs when needed for a format; linked HTML resources retain the behavior described above.

### Optional online audio transcription

Transcription is not required for local conversions. To enable it, provide your own OpenAI API key in Strumenti after the audio-upload and cost notice. Selecting **TXT con OpenAI** for an audio file requests upload to OpenAI. The app sends the selected audio or locally prepared segments, transcription parameters and the authentication key over HTTPS to `api.openai.com/v1/audio/transcriptions`. Files use a generic `audio.<extension>` name without the original filename or path. Corsivo does not relay the content. Voices, spoken words and other sounds in the recording are therefore disclosed to OpenAI. Other conversions and the entire queue are not automatically uploaded.

Direct uploads also include any metadata embedded in the audio file. The model is `gpt-transcribe`. Choose a language or use automatic detection. When preparation is needed for a file format or size, FFmpeg creates temporary audio segments on your PC; the app uploads them sequentially. After the first segment, when available, a whole-word tail of up to 200 characters already transcribed from that same recording is sent again as context, with whitespace and control characters normalized. Text is extracted from the service's JSON response and saved to the output TXT only after the job completes. Cancelling does not withdraw audio already uploaded or reverse API charges already incurred.

OpenAI's handling, retention and available controls depend on its terms and your API-account settings. The app does not promise immediate deletion or processing in a particular region. See [OpenAI API data controls](https://developers.openai.com/api/docs/guides/your-data). Access and any API charges belong to your OpenAI account and are separate from a ChatGPT subscription.

### API key protection and removal

On Windows, the key is stored in Windows Credential Manager for the current user under `Corsivo.TheConverter.OpenAI`, protected by Windows. It is not saved in preferences JSON, queued jobs, reports or logs. It is used in memory to authenticate service requests. Distributed packages contain no developer API key.

**Rimuovi chiave** removes the saved credential and disables new transcriptions. Also cancel any job already running if you want it to stop. Removing the saved key does not revoke it with OpenAI or delete previously submitted data or requests; manage those controls separately through your API account. Deleting only preferences or queue files does not remove the saved credential.

The Credential Manager entry is independent of the program files and remains after EXE or MSIX uninstall. Use **Rimuovi chiave** before uninstalling, or remove `Corsivo.TheConverter.OpenAI` under **Credential Manager → Windows Credentials → Generic Credentials** in the Windows account that saved it, including after the app has been removed. This deletes only the local copy of the key.

### Information stored on your device

The app stores conversion preferences, transcription language, OCR language and enablement, CSV delimiter, PDF operation, output-folder and tool paths. Its persistent quick-conversion queue stores source and destination paths, formats, settings, tool paths, timestamps, progress and errors. The latest report stores its date, format, results and error messages, which may contain paths or engine diagnostics. None of these records contains the API key.

Local files include `preferences.json`, `report-last.json` and `job.json` records in the app's user-data folders under `%APPDATA%` and `%LOCALAPPDATA%`. In an MSIX installation, Windows may redirect these locations into private package data. The app does not encrypt preferences, state or output files; Windows account permissions and device protections apply. The API key is stored separately as described above.

Temporary conversion files, name reservations and placeholder shortcuts are created in the output folder. OCR temporary files may contain page images and recognized text; HEIC/AVIF temporary files may contain a PNG representation of the image. The Explorer extension also uses a temporary selection JSON file. The app attempts to remove temporary files after completion or cancellation; an unexpected shutdown may leave residual files. Completed results remain in the selected folder.

Automatic queue cleanup runs when the worker resumes or finishes the queue. It removes completed or cancelled records after 7 days and failed records after 30 days, and attempts to retain at most the 300 newest terminal records. Original files, outputs and active jobs are preserved. Quarantined unreadable records and unrelated files are retained. **Rimuovi segnaposto** for a failed job removes only shortcuts that still belong to that job, retaining its record and outputs.

Preferences and the last report have no automatic expiry. To remove local data manually, finish or cancel jobs, wait for processing to stop, and close the app. Removing active queue records can prevent job recovery. You can manage and delete outputs in File Explorer. Windows handling of app data during uninstall depends on the installation method.

### External links and services

The Strumenti page links to the FFmpeg, Poppler, LibreOffice, Tesseract and libheif websites and to OpenAI API-key management. Selecting a link opens your configured browser; the browser and destination website apply their own policies. External engines, network folders, cloud synchronization, antivirus and Windows diagnostics are governed by their own behavior and your device settings. This statement describes Convert Here's application code.

The planned Store edition does not automatically download conversion engines or executable updates. Microsoft manages Store installation and updates. A separate EXE distribution may offer an engine-download script that contacts the distributors named in that script when run; that script is not part of the planned Store package.

### Controls and questions

You choose the files, destination and optional engines, can remove your saved API key, and can cancel jobs. Corsivo does not hold a cloud copy of your files. Audio submitted to OpenAI is subject to the service controls described above. For privacy questions use the real contact details that must be completed above. This statement must be updated when the app's functionality or data practices change.
