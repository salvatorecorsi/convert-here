# Microsoft Store: preparazione della submission

Preparazione della **0.4.0 / Store 1.0.3.0**: uscite visibili configurabili e download facoltativo dei motori su richiesta, editore **Salvatore Corsi**. Non sono ancora attestati build pubblica, checksum, caricamento o invio del nuovo pacchetto.

Le Impostazioni mostrano otto uscite per default: MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4. Restano supportate 93 estensioni in ingresso e 44 destinazioni del motore. Il filtro riguarda GUI e menu Explorer, senza bloccare CLI, richieste rapide esplicite o lavori gia accodati.

In **Impostazioni → Motori** puoi richiedere FFmpeg, Poppler o entrambi con conferma di origine, versione, dimensioni, licenze e destinazione. Il download HTTPS verifica lo SHA-256 prima dell'estrazione nel profilo utente, conservando licenze e documentazione. IP e normali dati HTTP raggiungono i distributori; i file da convertire non vengono caricati. La dipendenza e il download sono descritti all'inizio delle schede IT/EN e nella [bozza privacy 0.4.0](privacy.md).

Le [Store Policies 7.20](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies#102-security), efficaci dal 22 ottobre 2026, richiedono consenso e coerenza delle estensioni con la funzione dichiarata; non stabiliscono un divieto generale di acquisire dipendenze. Resta necessaria la certificazione Microsoft.

## Ultimo stato verificato 0.3.2 / Store 1.0.2.0

Stato al 4 novembre 2026: [Convert Here **0.3.2**](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.2) e pubblicata su GitHub dal commit `8f7532bf9d16f80ef65b5229dba93929e11c1d82` e risulta **In Microsoft Store**. Il nome editore **Salvatore Corsi** e approvato, con account Active e verifica Authorized. E stata osservata l'installazione del pacchetto completo **1.0.2.0** con **SignatureKind: Store**. La Submission 1, ID1152921505702314630, aveva seguito il percorso di certificazione documentato; la nuova **Submission 2**, ID **1152921505702316905**, e **In draft**, senza nuovo pacchetto 1.0.3.0 ancora attestato.

Il prodotto resta **9NPKPPCBNBM8**, `Name` **SalvatoreCorsi.ConvertHere** e `Publisher` **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17**; identita e GUID tecnici non cambiano. Il nuovo `ConvertHere-1.0.2.0-x64-store.msix` e di **6165973 byte**, SHA-256 `c6d655b58156e4a14e13653a0f5115f3bffc2ba5ff086dbe435bc34440a053b7`, generato dal commit pulito `8f7532bf9d16f80ef65b5229dba93929e11c1d82` e verificato prima del caricamento.

Tutte le sei sezioni erano **Complete** prima dell'invio della 1.0.2.0. La precedente timeline **Submission Complete**, **Pre-processing In progress**, **Certification Not started**, **Publishing Not started** e un'evidenza storica anteriore alla pubblicazione ora verificata. Restano prezzo gratuito e tutti i 240 mercati selezionati.

La [privacy 0.3.2](https://github.com/salvatorecorsi/convert-here/blob/v0.3.2/docs/privacy.md) e pubblica verificata e salvata, con persistenza confermata al ricaricamento. Additional Testing Information di **1977 caratteri**, aggiornate alle versioni 0.3.2 / 1.0.2.0, sono salvate e rilette. I campi telefono e indirizzo restano vuoti. Le schede riutilizzano le tre catture native 0.3.1 perche l'interfaccia e identica; non sono nuove catture della 0.3.2.

## Stato della 0.3.1 prima del ritiro della certificazione

Stato al 4 novembre 2026: **Convert Here 0.3.1** e [pubblicata su GitHub](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.1), dal commit `884cb2c610a9197d8b4853f0fa741dbe9035b5ed`. Il candidato Store **1.0.1.0** e stato caricato e risulta **Validated**, con **Packages: Complete**. Il prodotto resta **9NPKPPCBNBM8**, [pagina Store](https://apps.microsoft.com/detail/9NPKPPCBNBM8), `Name` **SalvatoreCorsi.ConvertHere**, `Publisher` **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17**, `PublisherDisplayName` **Corsivo**. Titolare, autore e sviluppatore: **Salvatore Corsi**; contatto pubblico: **salvo@salvatorecorsi.com**, dalla [pagina contatti](https://www.salvatorecorsi.com/contatti/). IARC e stato accettato su consenso esplicito del proprietario; Age ratings risulta **Complete**, con classificazioni IARC/Microsoft/PEGI 3+. La submission e stata inviata il **4 novembre 2026** e risulta **In certification**. **La pubblicazione Store non e ancora iniziata.**

## Dettagli della precedente submission 0.3.1 / Store 1.0.1.0

- **Pacchetto:** `ConvertHere-1.0.1.0-x64-store.msix`, **6165956 byte**, SHA-256 `e5f63b0c59983ebde7209a1a2813a63c4df8941c1dc7d77f813bea91f5dfdfdd`, generato dal commit pulito `884cb2c610a9197d8b4853f0fa741dbe9035b5ed`. Caricato e **Validated**, **Packages: Complete**, Desktop x64, minimo Windows `10.0.22000.0`. Il salvataggio ha sostituito automaticamente il precedente 1.0.0.0 con la versione superiore.
- **Schede:** italiana (59) e inglese (4) **Complete**; ciascuna contiene tre catture PNG della 0.3.1 con didascalie, logo 300 x 300, 15 funzionalita e sette parole chiave. I testi descrivono il prodotto gratuito e locale: nessun pagamento richiesto per continuare a usarlo.
- **Properties:** **Complete**; dichiarazione delle funzionalita AI generative non selezionata e [privacy della 0.3.1](https://github.com/salvatorecorsi/convert-here/blob/v0.3.1/docs/privacy.md) pubblica verificata e salvata.
- **Submission options:** **Complete**; motivazione `runFullTrust` di 424 caratteri e Additional Testing Information di 1977 caratteri salvate, con persistenza verificata.
- **Prezzo e disponibilita:** gratuita, tutti i 240 mercati disponibili, pubblicazione appena approvata.
- **Age ratings / IARC:** accettazione salvata su consenso esplicito del proprietario; **Complete**, classificazioni IARC/Microsoft/PEGI 3+.
- **Invio:** **Submission 1**, ID **1152921505702314630**, inviata per certificazione il **4 novembre 2026**; stato complessivo **In certification**. Timeline: **Submission Complete**, **Pre-processing In progress**, **Certification Not started**, **Publishing Not started**. Il portale conferma l'avvio automatico della pubblicazione appena il prodotto supera la certificazione; non attesta ancora approvazione o disponibilita pubblica.

## Storico: candidato 1.0.0.0, applicazione 0.3.0

Stato osservato per il precedente candidato 1.0.0.0, applicazione 0.3.0, nella submission **1152921505702314630** del prodotto **9NPKPPCBNBM8**:

| Sezione | Evidenza osservata |
| --- | --- |
| Packages | **Complete**; `ConvertHere-1.0.0.0-x64-store.msix` risulta **Validated**. Famiglia Desktop selezionata, minimo `10.0.22000.0`, architettura x64. |
| Scheda italiana, lingua 59 | **Complete**; due schermate PNG della 0.3.0 con didascalie e logo 300 x 300. |
| Scheda inglese, lingua 4 | **Complete**; due schermate PNG della 0.3.0 con didascalie e logo 300 x 300. La UI resta italiana. |
| Pricing and availability | **Complete**; gratuita, tutti i 240 mercati disponibili, pubblicazione appena approvata. |
| Properties | **Complete**. |
| Submission options | **Complete**; motivazione `runFullTrust` di 469 caratteri e pubblicazione appena approvata. |
| Additional Testing Information | Istruzioni di 2306 caratteri salvate; persistenza verificata nel portale. |
| Age ratings / IARC | **In Progress**; anteprima 3+, non finalizzata e non accettata. Il proprietario ha sospeso questo passaggio. |
| Invio e certificazione | **Non eseguiti**. Nessuna approvazione o pubblicazione attestata. |

Il precedente file 1.0.0.0 aveva SHA-256 `e130564d34e88b3e9f60ba6de76b06536adaa109957e1673a0043886ad2fe1de` ed e stato sostituito nel portale dal 1.0.1.0. I report di build e i checksum storici restano immutati: descrivono lo stato al momento della creazione, prima dell'upload.

## Percorso scelto

Distribuire un **MSIX completo x64**, contenente applicazione, DLL Explorer, risorse e avvisi delle dipendenze. Il Microsoft Store firma il pacchetto dopo la certificazione; non occorre acquistare un certificato per questa distribuzione. Il vecchio `TheConverter.Identity.msix` sparse e il setup EXE restano artefatti diversi e non sono la submission Store. [Requisiti MSIX e firma Store](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements#code-signing-for-microsoft-store-submissions).

Il menu moderno usa la DLL `IExplorerCommand`, COM surrogate e `windows.fileExplorerContextMenus` dichiarati nel manifest del pacchetto completo. Non richiede script di registrazione del menu classico nel pacchetto Store. [Integrazione ufficiale con Esplora file](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer).

## Dati confermati e campi della submission

| Campo | Stato / azione |
| --- | --- |
| Account sviluppatore Partner Center | Gia registrato; eventuali azioni di verifica pendenti sono quelle mostrate dal portale. |
| Tipo account | **Individual**, confermato in Legal info nella verifica corrente. |
| Nome prodotto riservato | **Convert Here**, prenotazione confermata dal proprietario il 4 novembre 2026. |
| `Package/Identity/@Name` | `SalvatoreCorsi.ConvertHere`, dalla schermata Product identity fornita dal proprietario. |
| `Package/Identity/@Publisher` | `CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17`, dalla stessa schermata. Non sostituirlo con un `CN=CORSIVO` presunto. |
| Publisher display name | **Salvatore Corsi**, approvato; profilo **Authorized**, account **Active** e nome pubblico confermato in Legal info. |
| Store ID | `9NPKPPCBNBM8`; l'identificativo non attesta la pubblicazione dell'app. |
| Titolare e contatto | **Salvatore Corsi**, **salvo@salvatorecorsi.com**. |
| Informativa privacy | [Bozza 0.4.0](privacy.md) con download facoltativo e formati visibili; la [versione 0.3.2](https://github.com/salvatorecorsi/convert-here/blob/v0.3.2/docs/privacy.md) resta l'URL verificato della precedente submission. |
| Supporto | **salvo@salvatorecorsi.com** oppure [pagina contatti](https://www.salvatorecorsi.com/contatti/), gia pubblici. Per PC il campo e facoltativo, ma consigliato. |
| Prezzo, mercati e calendario | Scelte confermate dal proprietario e salvate: **Gratuita**, **tutti i 240 mercati disponibili**, **pubblicazione appena approvata**. |
| Licenza applicazione Store | Codice originale **GPL-3.0-only**, Copyright (C) 2026 Salvatore Corsi. Per la nuova submission compilare Additional license terms con il singolo URL `https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/LICENSE`, dopo aver pubblicato e verificato il tag. Non lasciare il campo vuoto per affidarsi soltanto ai termini standard. Le licenze dei componenti terzi restano invariate. |

Riferimenti Microsoft: [privacy e supporto](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/support-info), [termini di licenza aggiuntivi](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-additional-information#additional-license-terms).

Microsoft accetta un singolo URL come collegamento cliccabile ai termini, oppure fino a 10000 caratteri di testo semplice. Il collegamento alla licenza completa evita di troncare la GPL; le schede IT/EN riportano anche il repository dei sorgenti. La nuova release prevede `ConvertHere-0.4.0-source.zip` con dipendenze Cargo vendorizzate e ricetta di build offline. Licenza, avvisi terzi e collegamento ai sorgenti sono inclusi nel materiale dell'app; la conversione non cambia i diritti sui file dell'utente.

## Identita, versione e lingue

### Build locale e GitHub Actions

L'anteprima si genera senza account o certificato:

```powershell
.\scripts\build-store-package.ps1
```

La configurazione locale dell'identita e in `packaging/store/identity.partner-center.json`; e allineata a **Salvatore Corsi** come `PublisherDisplayName`, ora confermato in Partner Center, mantenendo `Name` e `Publisher` indicati sopra. Per una nuova configurazione, copiare `packaging/store/identity.partner-center.example.json` e sostituire i placeholder con i valori reali. Per generare il MSIX unsigned destinato alla submission:

```powershell
.\scripts\build-store-package.ps1 -ForSubmission -IdentityConfig .\packaging\store\identity.partner-center.json
```

Il builder rifiuta identita di prova, placeholder e versioni Store non valide. Crea una nuova cartella in `dist/store/preview/` o `dist/store/submission/`, con MSIX, manifest, log, SHA-256 e `submission-report.json`. Non firma, installa, registra l'app o importa certificati.

In alternativa, aprire [Actions → Microsoft Store package → Run workflow](https://github.com/salvatorecorsi/convert-here/actions/workflows/microsoft-store.yml) nel repository. Il valore predefinito genera una preview; per un candidato attivare `for_submission` e inserire identita, editore, nome riservato e versione. Gli input sono trasferiti allo script tramite variabili d'ambiente, senza interpretarli come codice. Gli artefatti includono il pacchetto unsigned e le bozze di submission. Microsoft esegue la firma dopo l'approvazione Store.

[Windows checks](https://github.com/salvatorecorsi/convert-here/actions/workflows/windows-ci.yml) esegue automaticamente formattazione, test Rust e fixture PowerShell. Una pipeline verde verifica queste operazioni; non sostituisce la certificazione Microsoft o il collaudo dell'installazione su Windows 11.

- Versione Store del nuovo candidato in preparazione: `1.0.3.0`, distinta dalla versione applicazione Rust `0.4.0` e maggiore del precedente upload `1.0.2.0`. Il primo segmento Store deve essere maggiore di zero; il quarto rimane zero. Ogni segmento e al massimo 65535. [Versioni dei pacchetti](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements#version-numbering-for-windows-10-and-11-packages).
- Nome e Publisher devono corrispondere esattamente ai dati Partner Center, comprese maiuscole e punteggiatura. Il builder ricompila gli eseguibili con il manifest dedicato al pacchetto completo, senza l'elemento di identita sparse dell'installer EXE.
- La UI e attualmente in italiano. La scheda inglese e una traduzione della descrizione, non una traduzione dell'interfaccia. Non dichiarare supporto UI inglese finche non implementato e verificato.
- Target previsto: Windows 11 desktop x64 per il menu moderno. Il limite OS e quello effettivamente impostato e collaudato nel manifest Store; non promettere ARM64 nativo.

## Funzioni incluse e motori opzionali

Il pacchetto leggero offre subito conversioni immagini native e immagini in PDF, compressione PDF senza perdita, divisione e unione di PDF, coda, impostazioni e menu contestuale. Il modello espone 44 destinazioni per le categorie compatibili e riconosce 93 estensioni in ingresso. FFmpeg serve per audio/video; Poppler per PDF in immagini e testo (TXT, DOCX o HTML); LibreOffice per documenti Office/HTML in PDF ed Excel/CSV; Tesseract con dati delle lingue per OCR locale; libheif (`heif-dec` e `heif-enc`, con i codec necessari) per leggere e scrivere HEIC/AVIF. OCR dei PDF richiede anche Poppler per rasterizzare le pagine. Sono programmi esterni gia disponibili sul PC o selezionati dall'utente in Impostazioni → Motori. La build Store non deve includere il downloader PowerShell del setup EXE. Nella 0.4.0 FFmpeg e Poppler possono essere scaricati su richiesta con conferma nell'app; LibreOffice, Tesseract, modelli OCR e libheif restano da selezionare da installazioni locali. Nessun motore e incluso nel MSIX.

PDF in DOCX/HTML ricostruisce solo il testo: le scansioni richiedono l'opzione OCR locale; immagini, tabelle e impaginazione non vengono ricostruite. CSV esporta il primo foglio senza formule o formattazione; il separatore di importazione/esportazione e selezionabile. Le risorse collegate nei documenti HTML possono accedere a file locali o alla rete. Compressione PDF non garantisce una riduzione delle dimensioni: se il candidato e piu grande, viene conservata una copia identica dell'originale. Divisione e unione conservano le pagine ma omettono metadati del documento, segnalibri e struttura accessibile; PDF firmati o con moduli interattivi richiedono una copia statica senza firme.

Per un PDF singolo, **PDF · comprimi** forza la compressione indipendentemente dall'operazione salvata. Per selezioni multiple PDF-compatibili, **PDF (Bulk)** mantiene lavori e risultati individuali; **PDF (Merge)** apre l'editor delle singole pagine per creare un PDF unico da immagini, PDF, Office, Excel/CSV e HTML. Le pagine possono essere intercalate con trascinamento o frecce. La sessione Merge usa snapshot temporanei nascosti e non crea lavori persistenti nella coda rapida. Divisione e unione restano disponibili nella GUI e con `--pdf-action split`/`--pdf-action merge` nella CLI. I titoli **TXT · OCR**, **TXT** e **TXT · OCR / testo PDF** distinguono il riconoscimento locale delle immagini dall'estrazione PDF. Audio e video non hanno una destinazione TXT. La DLL usa maschere a 64 bit; i GUID dei comandi esistenti e della voce personalizzate restano stabili. Whitelist dell'installer, registrazione classica, DLL e modello dell'app vengono confrontati nei test per tutte le 93 estensioni e le 44 destinazioni.

Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. Le conversioni e l'OCR vengono elaborati sul PC con le funzioni native e i motori locali selezionati dall'utente. Il comportamento di risorse HTML collegate, browser e cartelle di rete e descritto nell'[informativa privacy](privacy.md).


La dipendenza va dichiarata all'inizio delle descrizioni: la policy 10.2.4 ammette software non integrato con questa trasparenza. E un percorso ragionevole da sottoporre alla certificazione, non un'approvazione garantita. Mantenere le funzioni native utilizzabili senza motori e messaggi chiari quando un motore manca. La policy 10.2.2 vieta l'introduzione dinamica di funzionalita incoerenti con quanto dichiarato. [Store policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies#102-security).

## Capability e motivazione per la certificazione

`runFullTrust` e dichiarato per l'app desktop `mediumIL` e il COM packaged. La motivazione di 424 caratteri e stata salvata per il precedente candidato 1.0.1.0. Scopo tecnico:

> Convert Here is a native Rust desktop file conversion application. It reads files explicitly selected by the user, writes converted outputs to the selected folder, and provides an IExplorerCommand shell extension through packaged COM. Optional conversion engines already installed or explicitly selected by the user run as child processes with the user's permissions. The app does not require administrator privileges, a service, a driver, or elevation.

Non aggiungere `unvirtualizedResources` per abitudine ereditandolo dal pacchetto sparse. Serve a disabilitare la virtualizzazione di AppData/HKCU: conservarlo solo se un requisito verificato lo rende indispensabile, documentandone lo scope e chiedendo l'approvazione necessaria. Le capability ristrette richiedono una spiegazione nella submission. [Dichiarazioni delle capability](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations#restricted-capabilities), [virtualizzazione flessibile](https://learn.microsoft.com/en-us/windows/msix/desktop/flexible-virtualization).

## Verifiche documentate della precedente 0.3.1

Le caselle seguenti documentano esclusivamente il precedente candidato 1.0.1.0 e le schede della 0.3.1 salvate nel portale. Per build, validazione e invio del 1.0.2.0 valgono il riepilogo corrente sopra e lo [stato delle verifiche](store-validation.md).

- [x] Identita reale Partner Center e contatto confermati.
- [x] Informativa privacy IT/EN e schede locali aggiornate alla 0.3.1.
- [x] Scelte del proprietario salvate: gratuita, 240 mercati disponibili, pubblicazione appena approvata.
- [x] Build 0.3.1 associata al commit `884cb2c610a9197d8b4853f0fa741dbe9035b5ed`; [Windows checks 41852511198](https://github.com/salvatorecorsi/convert-here/actions/runs/41852511198) **SUCCESS**. MSIX 1.0.1.0 generato, validato e identificato con checksum.
- [x] Candidato 1.0.1.0 verificato e caricato al posto del precedente 1.0.0.0; Packages **Complete**, file **Validated**.
- [x] Schede IT/EN, Properties e Additional Testing Information aggiornate e salvate nel portale per la 0.3.1.
- [x] Tre catture PNG della 0.3.1 con didascalie e logo 300 x 300 salvati in ciascuna scheda; prove native immagini e PDF Merge superate.
- [x] Questionario IARC accettato e salvato su consenso esplicito del proprietario; Age ratings **Complete**, classificazioni 3+.
- [ ] Eventuale collaudo installazione/aggiornamento/disinstallazione, menu moderno, coda e motori su ambiente pulito documentato per il candidato finale.
- [x] Submission 1 inviata per certificazione il 4 novembre 2026; **In certification**, con pre-elaborazione in corso e pubblicazione non ancora iniziata.

Il Windows App Certification Kit non e stato eseguito sul nuovo candidato. Microsoft descrive il WACK come controllo locale deprecato e facoltativo; il suo eventuale esito non garantisce l'approvazione Store. Il precedente report riguarda una preview distinta, come indicato nello [stato delle verifiche](store-validation.md). [Validazione locale MSIX](https://learn.microsoft.com/en-us/windows/msix/package/packaging-uwp-apps#validate-your-app-package-locally).

Per il CSV attendere l'esportazione del prodotto reale da Partner Center: preservare colonne Field, ID e Type; trasferire i testi preparati nei campi della lingua corretta. Non inventare un template con ID presunti. [Importazione/esportazione delle schede](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/import-and-export-store-listings).

## Note per il certificatore: bozza per 0.4.0

La precedente 0.3.2 / 1.0.2.0 aveva istruzioni di test di 1977 caratteri salvate nel portale. Il riferimento seguente prepara la nuova 0.4.0; non attesta salvataggio, invio o collaudo del nuovo MSIX.

> Convert Here 0.4.0 / Store package 1.0.3.0. Publisher and developer: Salvatore Corsi. The UI is Italian. The software is free, with no subscriptions, in-app purchases or usage-based charges. No payment is required to keep using it. Conversion and OCR run locally and need no app account or login. Target: Windows 11 x64.
>
> Native test: launch Convert Here from Start, use Aggiungi file or drop a PNG/JPEG, select another image format or PDF, then Converti file. Outputs use the selected folder or the source folder; existing files are preserved. PDF compression, splitting and merging need no external engine. Use unsigned, unencrypted PDFs without interactive forms. Impostazioni lets users choose visible outputs; defaults are MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Enable a hidden output before testing it in the GUI. Explorer rereads preferences without a restart; all 93 input extensions remain recognized.
>
> Explorer test: select two PNG/JPEG files, right-click > Converti in > PDF (Bulk). Each input gets a separate queued result. A temporary shortcut opens job status. With the same selection choose PDF (Merge): wait for preparation, drag individual page handles or use arrows/Alt+Up/Down, then Crea PDF unico. Multipage PDFs can be mixed with images and pages interleaved across sources. Cambia cartella changes the destination. Merge also opens from the main app. Annulla cancels active work; closing the session attempts to remove its temporary snapshots.
>
> External engines are not bundled. In Impostazioni > Motori, users may select existing local executables or explicitly request FFmpeg, Poppler or both. A confirmation shows the source, version, size, licenses and user-folder destination before downloading. HTTPS requests go to the Gyan/Poppler Windows distributions on GitHub and their CDNs, exposing IP and normal HTTP metadata; conversion files are not uploaded. SHA-256 is checked before extraction into the user's ConvertHere tools folder, retaining upstream licenses and documentation. Other engines remain local user-selected installations. HTML linked resources may access local files or the network.
>
> PDF-to-DOCX/HTML reconstructs text only. OCR does not reconstruct original page layout. Animated images and multipage image containers use the first frame/image. Audio and video do not offer TXT output.
>
> runFullTrust supports the medium-integrity desktop app, user-selected file I/O, child conversion engines and packaged COM Explorer integration. No administrator elevation, service or driver is required.
