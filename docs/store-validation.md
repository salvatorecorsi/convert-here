# Verifiche tecniche — 4 novembre 2026

Queste verifiche accompagnano la preparazione Store di Corsivo. Non equivalgono alla certificazione Microsoft e non sostituiscono l'identita reale di Partner Center.

## Verifiche della revisione corrente

Il modello espone 44 destinazioni e 93 estensioni riconosciute. Sul codice dell'interfaccia integrata sono passati tutti i sei controlli richiesti: formattazione, Clippy con `--all-targets -- -D warnings` e test per app e DLL. I test sono **194 per l'app e 5 per la DLL**, senza test ignorati. Sono compilati entrambi gli eseguibili release; una conversione reale dalla CLI release verifica anche l'alias `jpeg`.

La suite comprende Excel/CSV, HTML in PDF, ricostruzione del testo PDF in Word/HTML, OCR locale, HEIC/AVIF e operazioni PDF. Le prove locali usano FFmpeg/FFprobe e Poppler del progetto, LibreOffice 26.8.2, Tesseract con i modelli ita/eng e libheif estratti nelle cartelle di validazione, senza installazione nel sistema. Le copie di validazione non fanno parte dei pacchetti pubblici; FFmpeg e Poppler restano esclusi dal pacchetto Store.

Le prove verificano CSV UTF-8 con virgole, punti e virgola, tab e pipe, campi quotati, Unicode, righe multiple e identificatori con zeri iniziali; formule mantenute come testo durante l'importazione; primo foglio Excel; HTML/RTF in PDF e PDF → DOCX → PDF → TXT con testo conservato. Senza OCR, i PDF privi di testo incorporato vengono rifiutati con l'indicazione di attivarlo. Con OCR locale attivo, Tesseract riconosce le scansioni e crea TXT/DOCX/HTML; il default dei PDF resta senza OCR. DOCX e HTML sono ricostruzioni del testo, senza immagini o impaginazione originale. Un profilo temporaneo LibreOffice separa ogni conversione e disabilita macro/OLE e aggiornamento dei collegamenti Office; HTML puo comunque leggere le proprie risorse collegate, come dichiarato in interfaccia e privacy.

I fixture media verificano 78 abbinamenti dei sei nuovi formati immagine con i 13 formati immagine nativi, profondita colore, trasparenza e valori HDR superiori a 1. Verificano inoltre codec, decodifica completa e tracce audio dei nuovi contenitori, equivalenza PCM per CAF/AU e tutte le frequenze proposte per Opus/WebM, AC3, WMA/WMV e FLV. Frequenze incompatibili vengono rifiutate prima di creare risultati o prenotazioni.

Sono presenti fixture per roundtrip HEIC/AVIF con la coppia libheif, lingue e riconoscimento OCR, ordine delle pagine e compressione/divisione/unione PDF. Le verifiche dei PDF coprono anche conteggio con `pdfinfo` e fallback nativo quando manca, esportazione diretta e in coda dei documenti con sola password proprietario, errore italiano per password di apertura e avanzamento del rendering a blocchi. Compressione, divisione e unione continuano a rifiutare i PDF cifrati o firmati.

I test della coda coprono isolamento dei record illeggibili, recupero dei lavori, limiti di cronologia 7/30 giorni e 300 record terminali, lettura in background, pubblicazione senza una seconda copia e preservazione dei file sostituiti. Le prove dei segnaposto coprono aggiornamenti limitati durante l'avanzamento e rimozione dei soli collegamenti posseduti dai lavori falliti, preservando record, originali e risultati. Ulteriori fixture verificano qualita VP9/WebM e diagnostica per i video privi di traccia audio.

I 20 test del modulo di trascrizione usano fixture locali e server HTTP simulati: controllano GPT-Transcribe/JSON, `languages[]`, riconoscimento automatico, UTF-8, risposte malformate, errori redatti e cancellazione. FLAC e OGG restano nel percorso diretto; i test verificano byte originali, MIME e assenza del requisito FFmpeg. I segmenti successivi possono inviare un `prompt` con massimo 200 caratteri Unicode a parole intere gia trascritte; primo invio e file singoli non lo includono. Le prove coprono continuita dopo silenzio e segmenti brevi, normalizzazione dei controlli e nessuna pubblicazione di testo parziale. Non sono state effettuate richieste OpenAI con chiavi reali o a pagamento.

La verifica completa comprende test e Clippy con `--all-targets` per app e DLL, formattazione Rust, sintassi PowerShell e fixture dell'installer con PowerShell 5.1 e 7. Il workflow Windows checks prepara FFmpeg e Poppler con archivi e hash fissati prima dei test, compila la DLL Explorer ed esegue smoke test COM e parita delle estensioni. LibreOffice, Tesseract e libheif restano motori aggiuntivi delle prove locali. La whitelist e confrontata fra core, DLL, script di registrazione e installer; i GUID dei vecchi comandi restano stabili. Queste verifiche non registrano il menu installato e non costituiscono una certificazione Store.

La verifica finale PowerShell ha superato il parser di tutti i 20 script, i 6 casi della policy di disinstallazione e i 7 roundtrip degli apici sia con PowerShell 5.1 sia con PowerShell 7. Sono passati la parita delle 93 estensioni con due fixture negative, i 25 controlli del menu moderno e i 44 controlli Store, inclusa la validazione semantica MakeAppx. La DLL release supera lo smoke COM reale senza registrazione nel sistema; nessun installer e stato eseguito.

Il restyle mantiene egui con superfici che simulano l'aspetto Mica: il bonus Mica nativo non e applicato. Sono state verificate a schermo la finestra principale, le opzioni, la pagina Strumenti, i temi chiaro/scuro e una conversione WebM reale che prosegue dopo un errore su un altro file. Le [catture native](store-assets/README.md) usano preferenze e coda isolate; non attestano la certificazione o l'installazione MSIX del candidato finale.

## Verifiche storiche: migrazione GPT-Transcribe

Il backend corrente usa `gpt-transcribe` sullo stesso endpoint HTTPS di trascrizione. Richiede `response_format=json`, legge il campo `text` e conserva l'output finale TXT UTF-8. Il suggerimento di lingua gia salvato resta interoperabile: viene inviato come una sola voce `languages[]`; in modalita automatica il parametro e omesso. [Contratto ufficiale di migrazione](https://developers.openai.com/cookbook/examples/migrating_from_whisper_to_gpt_transcribe).

Nella revisione della migrazione sono superati `cargo test --locked --all-targets` con **75 test** e `cargo clippy --locked --lib --bins -- -D warnings`. I test della trascrizione erano 16 e usavano un server HTTP locale: verificavano richiesta GPT-Transcribe/JSON, `languages[]` senza parametro singolare, riconoscimento automatico, Unicode e righe del TXT, metadati lingua assenti o vuoti, compatibilita delle preferenze, risposte malformate e protezione della chiave. Non sono state effettuate richieste con chiavi reali o a pagamento.

Erano verificati anche annullamento, errori redatti, assenza di fallback a Whisper e mancata pubblicazione di TXT parziali quando una parte fallisce. I segmenti silenziosi venivano saltati senza perdere il parlato delle altre parti; una trascrizione complessivamente vuota non creava un file. Quella revisione non inviava ancora il contesto dei segmenti precedenti. Le prove GUI e i conteggi nella sezione seguente documentano una revisione ancora precedente, non una prova del servizio GPT-Transcribe reale.

## Verifiche storiche della revisione con Whisper

Eseguiti su Windows 11 x64, con Rust 1.99.1:

- `cargo test --locked --all-targets`: 69 test superati.
- `cargo test --locked --manifest-path shell-extension/Cargo.toml`: 3 test superati.
- `cargo clippy --locked --lib --bins -- -D warnings`: superato.
- Formattazione Rust applicazione/DLL e sintassi PowerShell: superate.
- `scripts/test-modern-menu-package.ps1`: 25 controlli isolati superati.
- `scripts/test-store-package.ps1`: 44 controlli superati, compresa la validazione semantica MakeAppx di fixture senza `/nv`.
- `scripts/test-shell-extension.ps1`: ABI COM reale, selezioni `IShellItemArray`, enumerazione, intersezioni dei formati e whitelist di 48 estensioni coerente fra DLL, script e installer.

I test di compatibilita verificano anche tutti gli abbinamenti non supportati delle categorie di ingresso: vengono rifiutati prima di creare cartelle di uscita o prenotazioni. I documenti TXT possono essere esportati in PDF; non possono diventare MP4. Le selezioni miste propongono soltanto destinazioni comuni. Il TXT da PDF resta disponibile senza OpenAI; il TXT da audio richiede la chiave facoltativa.

La trascrizione e stata provata con un server HTTP locale e una credenziale sintetica in una voce casuale separata del Gestore credenziali: nessuna chiave utente letta nei test, nessuna registrazione inviata a OpenAI e nessuna richiesta API a pagamento. Le prove coprono multipart Whisper, lingua, nome file generico, risultati UTF-8, unione di segmenti, quota, errori redatti, cancellazione durante la connessione e prima del salvataggio, dimensioni delle risposte e assenza di TXT parziali. Il mock e stato ripetuto anche sotto esecuzione parallela. Una prova con FFmpeg reale verifica la preparazione di un audio lungo in segmenti ordinati sotto la soglia di caricamento.

La build locale installata e stata controllata anche tramite la GUI, senza inserire una chiave: pannello OpenAI, avviso di invio/costi, campo password e assenza dei formati finche non si aggiunge un file. Ulteriori prove sugli eseguibili installati verificano il rifiuto TXT → MP4, immagine → PDF e conversione rapida immagine → JPEG, con rimozione del segnaposto e della cartella delle prenotazioni. Il menu Windows 11 sparse e stato ripristinato; la DLL installata supera lo smoke test COM. Nessuna modifica agli archivi dei certificati durante questo aggiornamento.

L'autenticazione, l'accesso al modello e la qualita della trascrizione sul servizio OpenAI reale restano da verificare con una chiave e un audio scelti dal proprietario. Non usare una chiave dello sviluppatore nel pacchetto o nei materiali pubblici.

## Pacchetti e certificazione

Il workflow [Windows checks](https://github.com/salvatorecorsi/the-converter/actions/workflows/windows-ci.yml) ripete i controlli sui sorgenti pubblicati. Il workflow [Microsoft Store package](https://github.com/salvatorecorsi/the-converter/actions/workflows/microsoft-store.yml) produce separatamente il MSIX unsigned e il report con commit, stato della working tree e checksum. Verificare il run relativo al commit che si intende distribuire.

La preview completa 1.0.2.0 precedente a Whisper e stata installata localmente con una copia firmata esclusivamente per il test, dopo un aggiornamento dalla preview 1.0.1.0. Al termine e stata rimossa ed e stata ripristinata l'integrazione sparse dell'installazione locale. Quella prova non certifica la successiva integrazione HTTP/Whisper.

Il report WACK locale della precedente preview **1.0.1.0**, eseguito con kit 10.0.26100.8249 su Windows 10.0.26200.0, ha risultato complessivo **WARNING**: 21 test PASS, due FAIL opzionali e un WARNING obbligatorio. I rilievi riguardavano il numero di applicazioni dichiarate, il rilevamento di eseguibili/avvio processi e la verifica DPI. Il pacchetto successivo dichiara una sola applicazione, esclude la CLI e usa API native per aprire cartelle e siti. L'uso legittimo di processi per i motori esterni resta parte del prodotto.

**Il precedente report WACK non verifica il codice migrato a GPT-Transcribe e non viene dichiarato superato.** Prima della submission occorrono il report del candidato effettivo, la prova dell'interfaccia e del menu moderno, installazione/aggiornamento/disinstallazione su un ambiente pulito e le verifiche con i motori opzionali. Microsoft considera il WACK un controllo locale deprecato: il suo esito non garantisce l'approvazione Store. [Validazione locale MSIX](https://learn.microsoft.com/en-us/windows/msix/package/packaging-uwp-apps#validate-your-app-package-locally).

I dati dell'account Corsivo, l'identita del prodotto, gli URL privacy/supporto, i contatti reali e l'invio a Microsoft restano da completare secondo la [procedura di submission](store-submission.md).
