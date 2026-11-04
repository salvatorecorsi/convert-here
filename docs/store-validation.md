# Verifiche tecniche — 4 novembre 2026

Queste verifiche accompagnano la preparazione Store di Corsivo. Non equivalgono alla certificazione Microsoft e non sostituiscono l'identita reale di Partner Center.

## Verifiche della revisione corrente

Il modello espone 44 destinazioni e 93 estensioni riconosciute. Sul codice dell'interfaccia integrata sono passati tutti i sei controlli richiesti: formattazione, Clippy con `--all-targets -- -D warnings` e test per app e DLL. I test sono **226 per l'app e 5 per la DLL**, senza test ignorati. Sono compilati entrambi gli eseguibili release; una conversione reale dalla CLI release verifica anche l'alias `jpeg`.

Le prove PDF (Bulk) e PDF (Merge) verificano la selezione multipla compatibile, la distinzione fra coda per file ed editor interattivo, i GUID storici e gli argomenti di avvio dei comandi Explorer. I test dell'editor simulano trascinamenti su piu frame, frecce da tastiera, intercalazione fra sorgenti e visualizzazione di 10.000 pagine. I test del motore controllano snapshot privati, ordine completo senza duplicati, immagini e PDF multipagina mescolati, documenti/HTML con LibreOffice, collisioni dei nomi, cancellazione e rifiuto di cifratura, firme e campi modulo. Nessun PDF intermedio viene pubblicato nella destinazione.

La suite comprende Excel/CSV, HTML in PDF, ricostruzione del testo PDF in Word/HTML, OCR locale, HEIC/AVIF e operazioni PDF. Le prove locali usano FFmpeg/FFprobe e Poppler del progetto, LibreOffice 26.8.2, Tesseract con i modelli ita/eng e libheif estratti nelle cartelle di validazione, senza installazione nel sistema. Le copie di validazione non fanno parte dei pacchetti pubblici; FFmpeg e Poppler restano esclusi dal pacchetto Store.

Le prove verificano CSV UTF-8 con virgole, punti e virgola, tab e pipe, campi quotati, Unicode, righe multiple e identificatori con zeri iniziali; formule mantenute come testo durante l'importazione; primo foglio Excel; HTML/RTF in PDF e PDF → DOCX → PDF → TXT con testo conservato. Senza OCR, i PDF privi di testo incorporato vengono rifiutati con l'indicazione di attivarlo. Con OCR locale attivo, Tesseract riconosce le scansioni e crea TXT/DOCX/HTML; il default dei PDF resta senza OCR. DOCX e HTML sono ricostruzioni del testo, senza immagini o impaginazione originale. Un profilo temporaneo LibreOffice separa ogni conversione e disabilita macro/OLE e aggiornamento dei collegamenti Office; HTML puo comunque leggere le proprie risorse collegate, come dichiarato in interfaccia e privacy.

I fixture media verificano 78 abbinamenti dei sei nuovi formati immagine con i 13 formati immagine nativi, profondita colore, trasparenza e valori HDR superiori a 1. Verificano inoltre codec, decodifica completa e tracce audio dei nuovi contenitori, equivalenza PCM per CAF/AU e tutte le frequenze proposte per Opus/WebM, AC3, WMA/WMV e FLV. Frequenze incompatibili vengono rifiutate prima di creare risultati o prenotazioni.

Sono presenti fixture per roundtrip HEIC/AVIF con la coppia libheif, lingue e riconoscimento OCR, ordine delle pagine e compressione/divisione/unione PDF. Le verifiche dei PDF coprono anche conteggio con `pdfinfo` e fallback nativo quando manca, esportazione diretta e in coda dei documenti con sola password proprietario, errore italiano per password di apertura e avanzamento del rendering a blocchi. Compressione, divisione e unione continuano a rifiutare i PDF cifrati o firmati.

I test della coda coprono isolamento dei record illeggibili, recupero dei lavori, limiti di cronologia 7/30 giorni e 300 record terminali, lettura in background, pubblicazione senza una seconda copia e preservazione dei file sostituiti. Le prove dei segnaposto coprono aggiornamenti limitati durante l'avanzamento e rimozione dei soli collegamenti posseduti dai lavori falliti, preservando record, originali e risultati. Ulteriori fixture verificano qualita VP9/WebM e diagnostica per i video privi di traccia audio.

I 20 test del modulo di trascrizione usano fixture locali e server HTTP simulati: controllano GPT-Transcribe/JSON, `languages[]`, riconoscimento automatico, UTF-8, risposte malformate, errori redatti e cancellazione. FLAC e OGG restano nel percorso diretto; i test verificano byte originali, MIME e assenza del requisito FFmpeg. I segmenti successivi possono inviare un `prompt` con massimo 200 caratteri Unicode a parole intere gia trascritte; primo invio e file singoli non lo includono. Le prove coprono continuita dopo silenzio e segmenti brevi, normalizzazione dei controlli e nessuna pubblicazione di testo parziale. Non sono state effettuate richieste OpenAI con chiavi reali o a pagamento.

La verifica completa comprende test e Clippy con `--all-targets` per app e DLL, formattazione Rust, sintassi PowerShell e fixture dell'installer con PowerShell 5.1 e 7. Il workflow Windows checks prepara FFmpeg e Poppler con archivi e hash fissati prima dei test, compila la DLL Explorer ed esegue smoke test COM e parita delle estensioni. LibreOffice, Tesseract e libheif restano motori aggiuntivi delle prove locali. La whitelist e confrontata fra core, DLL, script di registrazione e installer; i GUID dei vecchi comandi restano stabili. Queste verifiche non registrano il menu installato e non costituiscono una certificazione Store.

La verifica finale PowerShell ha superato il parser di tutti i 21 script, i 6 casi della policy di disinstallazione e i 7 roundtrip degli apici sia con PowerShell 5.1 sia con PowerShell 7. Sono passati la parita delle 93 estensioni con due fixture negative, i 25 controlli del menu moderno e i 44 controlli Store, inclusa la validazione semantica MakeAppx. I 26 controlli del packaging Convert Here verificano nomi, struttura portable, checksum e compatibilita degli aggiornamenti in entrambe le versioni PowerShell. La DLL release supera lo smoke COM reale senza registrazione nel sistema; nessun installer e stato eseguito.

I test dei probe PDF usano processi fixture Rust nativi: l'avvio di PowerShell sul runner CI non consuma piu il budget del probe. La cancellazione viene misurata dopo aver verificato vivo un discendente, del quale si trattiene l'handle fino alla verifica della terminazione. Limiti, timeout e comportamento del motore distribuito restano invariati.

Il restyle mantiene egui con superfici che simulano l'aspetto Mica: il bonus Mica nativo non e applicato. Sono state verificate a schermo la finestra principale, le opzioni, la pagina Strumenti, i temi chiaro/scuro e una conversione WebM reale che prosegue dopo un errore su un altro file. Le [catture native](store-assets/README.md) usano preferenze e coda isolate; non attestano la certificazione o l'installazione MSIX del candidato finale.

## Pacchetti GitHub Convert Here 0.3.0

La build 0.3.0 di app, CLI e DLL Explorer e stata completata, compresa la validazione MakeAppx dell'identita locale unsigned. La DLL release supera lo smoke COM e la cattura degli avvii Bulk/Merge con helper isolato, senza registrazione. La CLI release unisce PNG, PDF multipagina e PNG in un solo PDF di quattro pagine: `pdfinfo` e `pdftoppm` verificano numero e ordine, un secondo salvataggio conserva il primo risultato con un nome distinto e gli hash degli originali restano invariati. Non sono state usate la coda o le preferenze personali.

Una prova della finestra nativa, con preferenze e coda isolate, prepara cinque pagine da due immagini e un PDF di tre pagine. Trascinamenti in entrambe le direzioni intercalano immagini e pagine PDF; frecce e Alt+freccia cambiano lo stesso ordine. Il PDF unico salvato contiene cinque pagine nell'ordine scelto, verificato con `pdfinfo` e `pdftotext`. I test di regressione coprono anche pressione e primo movimento ricevuti nello stesso frame, senza agganciare per errore la maniglia sotto il puntatore finale.

## Candidato Microsoft Store 1.0.0.0

Il candidato `ConvertHere-1.0.0.0-x64-store.msix`, applicazione 0.3.0, deriva dal commit `d6ad0171dd002d01136e58b511f5e9e5e6dbfffb` con working tree pulita al momento della build. SHA-256: `e130564d34e88b3e9f60ba6de76b06536adaa109957e1673a0043886ad2fe1de`. Il report attesta MSIX completo x64, minimo Windows `10.0.22000.0`, validazione MakeAppx superata senza `/nv`, nessun motore o downloader incluso e nessuna installazione o modifica agli archivi dei certificati.

La verifica diretta in Partner Center del prodotto **9NPKPPCBNBM8**, submission **1152921505702314630**, mostra **Packages: Complete** e il file **Validated**, con Desktop selezionato. Sono **Complete** entrambe le schede italiana e inglese, ciascuna con due PNG della 0.3.0, didascalie e logo 300 x 300; anche Pricing, Properties e Submission options sono **Complete**. Sono salvate disponibilita gratuita in tutti i 240 mercati, pubblicazione appena approvata, motivazione `runFullTrust` e istruzioni per il certificatore.

IARC resta **In Progress**, con anteprima 3+ e conferma del proprietario ancora in attesa. **La submission non e stata inviata per certificazione.** Il candidato finale non e stato installato e non ha un report WACK. Le prove di preview e degli eseguibili separati non ne attestano installazione, menu registrato o disinstallazione. I report di build immutabili precedono l'upload; i loro campi di stato non vengono riscritti a posteriori. Lo stato operativo aggiornato e nella [procedura di submission](store-submission.md).

## Pacchetti GitHub Convert Here 0.2.0 (storico)

Versione Cargo e metadati nativi sono aggiornati alla 0.2.0; i due eseguibili mostrano ProductName `Convert Here`. Sono stati generati `ConvertHere-0.2.0-windows-x64-setup.exe`, `ConvertHere-0.2.0-windows-x64-portable.zip` e `SHA256SUMS.txt`. Il packaging pubblico applica la lista esplicita dei binari consentiti e include gli avvisi delle dipendenze, senza motori esterni o certificati. Il setup pubblico offre il menu classico; l'identita MSIX unsigned usata nella build locale non e distribuita. AppId, CLSID, binari interni e percorsi dei dati restano compatibili con gli aggiornamenti. Il setup e stato compilato e ispezionato, senza installarlo sul PC di lavoro.

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

Il workflow [Windows checks](https://github.com/salvatorecorsi/convert-here/actions/workflows/windows-ci.yml) ripete i controlli sui sorgenti pubblicati. Il workflow [Microsoft Store package](https://github.com/salvatorecorsi/convert-here/actions/workflows/microsoft-store.yml) produce separatamente il MSIX unsigned e il report con commit, stato della working tree e checksum. Verificare il run relativo al commit che si intende distribuire.

La preview completa 1.0.2.0 precedente a Whisper e stata installata localmente con una copia firmata esclusivamente per il test, dopo un aggiornamento dalla preview 1.0.1.0. Al termine e stata rimossa ed e stata ripristinata l'integrazione sparse dell'installazione locale. Quella prova non certifica la successiva integrazione HTTP/Whisper.

Il report WACK locale della precedente preview **1.0.1.0**, eseguito con kit 10.0.26100.8249 su Windows 10.0.26200.0, ha risultato complessivo **WARNING**: 21 test PASS, due FAIL opzionali e un WARNING obbligatorio. I rilievi riguardavano il numero di applicazioni dichiarate, il rilevamento di eseguibili/avvio processi e la verifica DPI. Il pacchetto successivo dichiara una sola applicazione, esclude la CLI e usa API native per aprire cartelle e siti. L'uso legittimo di processi per i motori esterni resta parte del prodotto.

**Il precedente report WACK non verifica il codice migrato a GPT-Transcribe e non viene dichiarato superato.** Microsoft considera il WACK un controllo locale deprecato e facoltativo: non e stato eseguito sul candidato corrente e il suo esito non garantirebbe l'approvazione Store. Installazione/aggiornamento/disinstallazione, menu registrato e motori opzionali del MSIX finale restano senza un collaudo su ambiente pulito. [Validazione locale MSIX](https://learn.microsoft.com/en-us/windows/msix/package/packaging-uwp-apps#validate-your-app-package-locally).

Identita Corsivo, Store ID e contatto reale sono documentati; il pacchetto e caricato e validato. Finalizzazione IARC e invio per certificazione restano da completare secondo la [procedura di submission](store-submission.md).
