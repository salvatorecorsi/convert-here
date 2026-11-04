# Microsoft Store: preparazione della submission

Stato al 4 novembre 2026: la verifica diretta in Partner Center conferma il prodotto **Convert Here**, `Name` **SalvatoreCorsi.ConvertHere**, `Publisher` **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17** e `PublisherDisplayName` **Corsivo**. Lo Store ID attuale e **9NPKPPCBNBM8**, con URL [apps.microsoft.com/detail/9NPKPPCBNBM8](https://apps.microsoft.com/detail/9NPKPPCBNBM8). L'applicazione e alla versione **0.3.0**; la versione del pacchetto Store e **1.0.0.0**. Titolare, autore e sviluppatore: **Salvatore Corsi**; contatto pubblico: **salvo@salvatorecorsi.com**, riportato nella [pagina contatti](https://www.salvatorecorsi.com/contatti/). Il candidato e stato caricato e validato dal portale, ma **non e ancora stato inviato per certificazione**. Installazione e WACK del candidato finale non sono stati eseguiti; le precedenti preview sono artefatti distinti.

## Stato della submission corrente

Verifica diretta della submission **1152921505702314630** del prodotto **9NPKPPCBNBM8**:

| Sezione | Evidenza osservata |
| --- | --- |
| Packages | **Complete**; `ConvertHere-1.0.0.0-x64-store.msix` risulta **Validated**. Famiglia Desktop selezionata, minimo `10.0.22000.0`, architettura x64. |
| Scheda italiana, lingua 59 | **Complete**; due schermate PNG della 0.3.0 con didascalie e logo 300 x 300. |
| Scheda inglese, lingua 4 | **Complete**; due schermate PNG della 0.3.0 con didascalie e logo 300 x 300. La UI resta italiana. |
| Pricing and availability | **Complete**; gratuita, tutti i 240 mercati disponibili, pubblicazione appena approvata. |
| Properties | **Complete**. |
| Submission options | **Complete**; motivazione `runFullTrust` di 469 caratteri e pubblicazione appena approvata. |
| Additional Testing Information | Istruzioni di 2306 caratteri salvate; persistenza verificata nel portale. |
| Age ratings / IARC | **In Progress**; anteprima 3+, non ancora finalizzata. La conferma del proprietario per l'accettazione IARC e stata richiesta e resta in attesa. |
| Invio e certificazione | **Non eseguiti**. Nessuna approvazione o pubblicazione attestata. |

Il file caricato conserva SHA-256 `e130564d34e88b3e9f60ba6de76b06536adaa109957e1673a0043886ad2fe1de`. Il report di build e i relativi checksum restano immutati: descrivono lo stato al momento della creazione, prima dell'upload. La validazione del portale non equivale alla certificazione o a una prova d'installazione.

## Percorso scelto

Distribuire un **MSIX completo x64**, contenente applicazione, DLL Explorer, risorse e avvisi delle dipendenze. Il Microsoft Store firma il pacchetto dopo la certificazione; non occorre acquistare un certificato per questa distribuzione. Il vecchio `TheConverter.Identity.msix` sparse e il setup EXE restano artefatti diversi e non sono la submission Store. [Requisiti MSIX e firma Store](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements#code-signing-for-microsoft-store-submissions).

Il menu moderno usa la DLL `IExplorerCommand`, COM surrogate e `windows.fileExplorerContextMenus` dichiarati nel manifest del pacchetto completo. Non richiede script di registrazione del menu classico nel pacchetto Store. [Integrazione ufficiale con Esplora file](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer).

## Dati confermati e campi della submission

| Campo | Stato / azione |
| --- | --- |
| Account sviluppatore Partner Center | Gia registrato; eventuali azioni di verifica pendenti sono quelle mostrate dal portale. |
| Tipo account | **Individual** nell'ultima verifica documentata di Legal info. |
| Nome prodotto riservato | **Convert Here**, prenotazione confermata dal proprietario il 4 novembre 2026. |
| `Package/Identity/@Name` | `SalvatoreCorsi.ConvertHere`, dalla schermata Product identity fornita dal proprietario. |
| `Package/Identity/@Publisher` | `CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17`, dalla stessa schermata. Non sostituirlo con un `CN=CORSIVO` presunto. |
| Publisher display name | **Corsivo**, confermato nel prodotto in Partner Center. |
| Store ID | `9NPKPPCBNBM8`; l'identificativo non attesta la pubblicazione dell'app. |
| Titolare e contatto | **Salvatore Corsi**, **salvo@salvatorecorsi.com**. |
| Informativa privacy | [URL pubblico](https://github.com/salvatorecorsi/convert-here/blob/master/docs/privacy.md), con titolare e contatto reali nell'informativa IT/EN. |
| Supporto | **salvo@salvatorecorsi.com** oppure [pagina contatti](https://www.salvatorecorsi.com/contatti/), gia pubblici. Per PC il campo e facoltativo, ma consigliato. |
| Prezzo, mercati e calendario | Scelte confermate dal proprietario e salvate: **Gratuita**, **tutti i 240 mercati disponibili**, **pubblicazione appena approvata**. |
| Licenza applicazione Store | Il campo Additional license terms puo rimanere vuoto per usare gli Standard Application License Terms; termini diversi vanno inseriti esplicitamente. Questo non assegna una licenza open source al repository. |

Riferimenti Microsoft: [privacy e supporto](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/support-info), [termini di licenza aggiuntivi](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-additional-information#additional-license-terms).

## Identita, versione e lingue

### Build locale e GitHub Actions

L'anteprima si genera senza account o certificato:

```powershell
.\scripts\build-store-package.ps1
```

La configurazione locale dell'identita e in `packaging/store/identity.partner-center.json`; prima della build deve riportare **Corsivo** come `PublisherDisplayName`, mantenendo `Name` e `Publisher` indicati sopra. Per una nuova configurazione, copiare `packaging/store/identity.partner-center.example.json` e sostituire i placeholder con i valori reali. Per generare il MSIX unsigned destinato alla submission:

```powershell
.\scripts\build-store-package.ps1 -ForSubmission -IdentityConfig .\packaging\store\identity.partner-center.json
```

Il builder rifiuta identita di prova, placeholder e versioni Store non valide. Crea una nuova cartella in `dist/store/preview/` o `dist/store/submission/`, con MSIX, manifest, log, SHA-256 e `submission-report.json`. Non firma, installa, registra l'app o importa certificati.

In alternativa, aprire [Actions → Microsoft Store package → Run workflow](https://github.com/salvatorecorsi/convert-here/actions/workflows/microsoft-store.yml) nel repository. Il valore predefinito genera una preview; per un candidato attivare `for_submission` e inserire identita, editore, nome riservato e versione. Gli input sono trasferiti allo script tramite variabili d'ambiente, senza interpretarli come codice. Gli artefatti includono il pacchetto unsigned e le bozze di submission. Microsoft esegue la firma dopo l'approvazione Store.

[Windows checks](https://github.com/salvatorecorsi/convert-here/actions/workflows/windows-ci.yml) esegue automaticamente formattazione, test Rust e fixture PowerShell. Una pipeline verde verifica queste operazioni; non sostituisce la certificazione Microsoft o il collaudo dell'installazione su Windows 11.

- Versione Store iniziale predisposta: `1.0.0.0`, distinta dalla versione applicazione Rust `0.3.0`. Il primo segmento Store deve essere maggiore di zero; il quarto rimane zero. Ogni segmento e al massimo 65535. [Versioni dei pacchetti](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements#version-numbering-for-windows-10-and-11-packages).
- Nome e Publisher devono corrispondere esattamente ai dati Partner Center, comprese maiuscole e punteggiatura. Il builder ricompila gli eseguibili con il manifest dedicato al pacchetto completo, senza l'elemento di identita sparse dell'installer EXE.
- La UI e attualmente in italiano. La scheda inglese e una traduzione della descrizione, non una traduzione dell'interfaccia. Non dichiarare supporto UI inglese finche non implementato e verificato.
- Target previsto: Windows 11 desktop x64 per il menu moderno. Il limite OS e quello effettivamente impostato e collaudato nel manifest Store; non promettere ARM64 nativo.

## Funzioni incluse e motori opzionali

Il pacchetto leggero offre subito conversioni immagini native e immagini in PDF, compressione PDF senza perdita, divisione e unione di PDF, coda, impostazioni e menu contestuale. Il modello espone 44 destinazioni per le categorie compatibili e riconosce 93 estensioni in ingresso. FFmpeg serve per audio/video; Poppler per PDF in immagini e testo (TXT, DOCX o HTML); LibreOffice per documenti Office/HTML in PDF ed Excel/CSV; Tesseract con dati delle lingue per OCR locale; libheif (`heif-dec` e `heif-enc`, con i codec necessari) per leggere e scrivere HEIC/AVIF. OCR dei PDF richiede anche Poppler per rasterizzare le pagine. Sono programmi esterni gia disponibili sul PC o selezionati dall'utente nella pagina Strumenti. La build Store non deve includere il downloader PowerShell del setup EXE.

PDF in DOCX/HTML ricostruisce solo il testo: le scansioni richiedono l'opzione OCR locale; immagini, tabelle e impaginazione non vengono ricostruite. CSV esporta il primo foglio senza formule o formattazione; il separatore di importazione/esportazione e selezionabile. Le risorse collegate nei documenti HTML possono accedere a file locali o alla rete. Compressione PDF non garantisce una riduzione delle dimensioni: se il candidato e piu grande, viene conservata una copia identica dell'originale. Divisione e unione conservano le pagine ma omettono metadati del documento, segnalibri e struttura accessibile; PDF firmati o con moduli interattivi richiedono una copia statica senza firme.

Per un PDF singolo, **PDF · comprimi** forza la compressione indipendentemente dall'operazione salvata. Per selezioni multiple PDF-compatibili, **PDF (Bulk)** mantiene lavori e risultati individuali; **PDF (Merge)** apre l'editor delle singole pagine per creare un PDF unico da immagini, PDF, Office, Excel/CSV e HTML. Le pagine possono essere intercalate con trascinamento o frecce. La sessione Merge usa snapshot temporanei nascosti e non crea lavori persistenti nella coda rapida. Divisione e unione restano disponibili nella GUI e con `--pdf-action split`/`--pdf-action merge` nella CLI. I titoli **TXT · OCR**, **TXT**, **TXT con OpenAI** e le varianti per selezioni miste distinguono il riconoscimento locale delle immagini dall'estrazione PDF e dalla trascrizione audio online. La DLL usa maschere a 64 bit; i GUID dei comandi esistenti e della voce personalizzate restano stabili. Whitelist dell'installer, registrazione classica, DLL e modello dell'app vengono confrontati nei test per tutte le 93 estensioni e le 44 destinazioni.

La trascrizione audio TXT aggiunge un servizio online facoltativo, distinto dalle conversioni locali: usa la chiave OpenAI API dell'utente e invia l'audio a `api.openai.com` solo quando viene richiesta. Richiede internet e accesso API con eventuali costi a carico dell'utente, separati dall'abbonamento ChatGPT. Dichiarare invio, dipendenza e costi all'inizio delle schede e nel consenso in app. La chiave resta nel Gestore credenziali Windows; nessuna chiave e inclusa nel pacchetto, nelle preferenze, nei record della coda o nei log.

Il backend attuale e `gpt-transcribe`: richiede JSON e salva il campo `text` in un TXT UTF-8. Una lingua eventualmente gia salvata nelle preferenze viene trasmessa con `languages[]`, senza il vecchio parametro API singolare. [Migrazione ufficiale OpenAI](https://developers.openai.com/cookbook/examples/migrating_from_whisper_to_gpt_transcribe).

La dipendenza va dichiarata all'inizio delle descrizioni: la policy 10.2.4 ammette software non integrato con questa trasparenza. E un percorso ragionevole da sottoporre alla certificazione, non un'approvazione garantita. Mantenere le funzioni native utilizzabili senza motori e messaggi chiari quando un motore manca. La policy 10.2.2 vieta l'introduzione dinamica di funzionalita incoerenti con quanto dichiarato. [Store policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies#102-security).

## Capability e motivazione per la certificazione

`runFullTrust` e dichiarato per l'app desktop `mediumIL` e il COM packaged. La motivazione della capability e stata salvata nelle Submission options (469 caratteri). Scopo tecnico:

> Convert Here is a native Rust desktop file conversion application. It reads files explicitly selected by the user, writes converted outputs to the selected folder, and provides an IExplorerCommand shell extension through packaged COM. Optional conversion engines already installed or explicitly selected by the user run as child processes with the user's permissions. The app does not require administrator privileges, a service, a driver, or elevation.

Non aggiungere `unvirtualizedResources` per abitudine ereditandolo dal pacchetto sparse. Serve a disabilitare la virtualizzazione di AppData/HKCU: conservarlo solo se un requisito verificato lo rende indispensabile, documentandone lo scope e chiedendo l'approvazione necessaria. Le capability ristrette richiedono una spiegazione nella submission. [Dichiarazioni delle capability](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations#restricted-capabilities), [virtualizzazione flessibile](https://learn.microsoft.com/en-us/windows/msix/desktop/flexible-virtualization).

## Stato dei materiali e delle verifiche

Consultare lo [stato delle verifiche](store-validation.md): le prove su una preview precedente non certificano la build con trascrizione online. Nel portale resta **In Progress** soltanto IARC; le caselle non selezionate relative al collaudo documentano prove non eseguite sul candidato finale.

- [x] Identita reale Partner Center configurata; candidato unsigned caricato e validato, senza presentare una firma di sviluppo come firma pubblica.
- [x] Informativa privacy IT/EN completata con titolare, contatto reale e URL HTTPS pubblico riportato sopra. [Requisiti privacy](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies#105-personal-information).
- [x] Schede [italiana](store-listing-it.md) e [inglese](store-listing-en.md) salvate come **Complete**; Pricing, Properties e Submission options anch'esse **Complete**.
- [ ] Questionario IARC finalizzato dopo la conferma del proprietario; l'anteprima 3+ non e ancora una classificazione completata.
- [x] Asset del pacchetto inclusi e risolti dal manifest: Square44x44Logo, Square150x150Logo e StoreLogo con varianti di scala; validazione MakeAppx e del portale completate. [Icone e risorse](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction).
- [x] Due schermate PNG aggiornate della 0.3.0 con didascalie e logo Store 300 x 300 salvati in ciascuna scheda linguistica. [Screenshot e immagini](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images).
- [x] Istruzioni operative Additional Testing Information e motivazione `runFullTrust` salvate; persistenza verificata nel portale.
- [ ] Installazione, aggiornamento, disinstallazione e avvio da Start del candidato finale controllati su ambiente pulito; non ancora eseguiti.
- [ ] Verificati menu moderno, selezione multipla, conversione rapida senza finestra opzioni, segnaposto, annullamento, nomi duplicati e percorsi con spazi/accenti.
- [ ] Provate le funzioni native senza motori; separatamente provati i motori opzionali e il selettore di eseguibili. Verificati percorsi di dati realmente usati sotto MSIX e comportamento della disinstallazione.
- [ ] Verificati consenso e informativa per l'invio audio a OpenAI, salvataggio/rimozione della chiave, selezioni compatibili, errori API e assenza della chiave da preferenze/coda/resoconti. Non incorporare credenziali del proprietario per i certificatori.
- [ ] Per la trascrizione: distinguere test con server simulato da richieste reali autorizzate. Non presentare test simulati come prova di disponibilita del servizio. Verificare il contratto GPT-Transcribe: risposta JSON, estrazione di `text`, suggerimento lingua con `languages[]` e continuita delle preferenze salvate. [Guida ufficiale OpenAI](https://developers.openai.com/api/docs/guides/speech-to-text).
- [ ] Associare ogni report al pacchetto e al commit verificati. Le prove locali o WACK precedenti all'integrazione OpenAI non attestano il funzionamento della nuova trascrizione.
- [x] Report del candidato: nessun motore o downloader incluso, pacchetto unsigned e nessuna modifica agli archivi dei certificati.
- [ ] Submission completa inviata per certificazione, con pubblicazione automatica appena approvata secondo l'autorizzazione del proprietario.

Il Windows App Certification Kit non e stato eseguito sul candidato finale. Microsoft descrive il WACK come controllo locale deprecato e facoltativo; non e un requisito locale da presentare come completato per l'upload. Il suo eventuale esito non garantisce l'approvazione Store. Il precedente report riguarda una preview diversa, come indicato nello [stato delle verifiche](store-validation.md). [Validazione locale MSIX](https://learn.microsoft.com/en-us/windows/msix/package/packaging-uwp-apps#validate-your-app-package-locally).

Per il CSV attendere l'esportazione del prodotto reale da Partner Center: preservare colonne Field, ID e Type; trasferire i testi preparati nei campi della lingua corretta. Non inventare un template con ID presunti. [Importazione/esportazione delle schede](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/import-and-export-store-listings).

## Note per il certificatore: riferimento esteso

Nel portale sono state salvate istruzioni operative sintetiche di 2306 caratteri. Il testo seguente e un riferimento esteso sul comportamento previsto da sorgenti e manifest, non un'attestazione di collaudo del MSIX finale.

> The publisher brand is Corsivo; the application is developed by Salvatore Corsi. The application UI is currently Italian. Local conversions do not require an account or login. Optional online transcription requires the user's own OpenAI API key and separately billed API access; it does not use a ChatGPT subscription.
>
> The package includes native image conversion, image-to-PDF conversion and local PDF compression, splitting and merging. It does not bundle or automatically download conversion engines. Audio/video conversion requires FFmpeg; PDF page/text export requires Poppler; Office/HTML-to-PDF and Excel/CSV conversion require LibreOffice; local OCR requires Tesseract with the selected language data; HEIC/AVIF conversion requires libheif tools and the required codecs. PDF OCR also uses Poppler. PDF-to-DOCX/HTML reconstructs text only, using optional OCR for scans; original images, tables and layout are not reconstructed. CSV exports the first worksheet without formulas or formatting, with a selectable separator. Existing local executables can be selected under Strumenti using Seleziona programma…. Missing engines are reported in the UI. These dependencies and HTML linked-resource behavior are disclosed in the Store description.
>
> Optional audio-to-TXT uses OpenAI's gpt-transcribe model through HTTPS at api.openai.com/v1/audio/transcriptions. The API response is JSON; the app extracts its text field and writes a UTF-8 TXT file. An optional saved language is sent as languages[], with no legacy language request field. The app discloses audio upload and API charges before the user saves a key and enables transcription. Choosing TXT con OpenAI for an audio file initiates upload. Compatible smaller audio files can be sent directly; other audio files require local FFmpeg preparation. No developer-supplied credential is bundled. Native image/PDF tests below do not need an API key or a network request. Key storage uses Windows Credential Manager for the current user and excludes preferences, queue records, reports and logs. There is no video-to-TXT operation.
>
> To test without dependencies: launch Convert Here from Start, add a PNG/JPEG using Aggiungi file or drag and drop, select a different image format or PDF, and press Converti file. Output is written beside the source unless the user selects another folder. Existing files are preserved through numbered output names.
>
> On Windows 11 x64, right-click a supported image in File Explorer and select Converti in, then choose a compatible format. The conversion is queued without opening the settings window. A temporary shortcut with a pending icon appears in the output folder and is replaced by the completed output. Double-clicking the shortcut opens job status. Impostazioni personalizzate opens the main options window. The shell extension is registered declaratively through the MSIX manifest.
>
> For a PDF input, the quick PDF · comprimi command always selects lossless compression. With multiple PDF-compatible inputs, PDF (Bulk) creates separate queued outputs; PDF (Merge) opens a page-order editor for a single PDF, also available from the main app. Drag pages or use the arrow controls to interleave pages from images, PDFs and compatible Office/HTML documents. Splitting and merging are also available through the CLI. Compression never produces a larger result than the input snapshot and may preserve the original bytes. Merging and splitting omit document-level metadata, bookmarks and accessibility tagging; signed PDFs and interactive forms require a static unsigned copy. TXT · OCR identifies local image recognition; TXT con OpenAI identifies online audio transcription, including explicit labels for mixed selections. The shell and application support 44 output formats across compatible categories and 93 input extensions; an offered output still requires its optional engine when applicable.
>
> General PDF content editing is not included. Animated image inputs, multipage TIFF inputs and HEIC/AVIF collections use their first frame/page/image. Normal image-to-PDF conversion creates a separate A4 PDF for each image; PDF (Merge) combines the prepared pages in the selected order. The app contains no advertising, subscription or in-app purchase code.
