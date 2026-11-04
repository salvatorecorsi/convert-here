# Microsoft Store: preparazione della submission

Stato al 4 novembre 2026: **Convert Here 0.3.1** e [pubblicata su GitHub](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.1), dal commit `884cb2c610a9197d8b4853f0fa741dbe9035b5ed`. Il candidato Store **1.0.1.0** e stato caricato e risulta **Validated**, con **Packages: Complete**. Il prodotto resta **9NPKPPCBNBM8**, [pagina Store](https://apps.microsoft.com/detail/9NPKPPCBNBM8), `Name` **SalvatoreCorsi.ConvertHere**, `Publisher` **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17**, `PublisherDisplayName` **Corsivo**. Titolare, autore e sviluppatore: **Salvatore Corsi**; contatto pubblico: **salvo@salvatorecorsi.com**, dalla [pagina contatti](https://www.salvatorecorsi.com/contatti/). IARC e stato accettato su consenso esplicito del proprietario; Age ratings risulta **Complete**, con classificazioni IARC/Microsoft/PEGI 3+. La submission e stata inviata il **4 novembre 2026** e risulta **In certification**. **La pubblicazione Store non e ancora iniziata.**

## Stato corrente 0.3.1 / Store 1.0.1.0

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
| Tipo account | **Individual** nell'ultima verifica documentata di Legal info. |
| Nome prodotto riservato | **Convert Here**, prenotazione confermata dal proprietario il 4 novembre 2026. |
| `Package/Identity/@Name` | `SalvatoreCorsi.ConvertHere`, dalla schermata Product identity fornita dal proprietario. |
| `Package/Identity/@Publisher` | `CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17`, dalla stessa schermata. Non sostituirlo con un `CN=CORSIVO` presunto. |
| Publisher display name | **Corsivo**, confermato nel prodotto in Partner Center. |
| Store ID | `9NPKPPCBNBM8`; l'identificativo non attesta la pubblicazione dell'app. |
| Titolare e contatto | **Salvatore Corsi**, **salvo@salvatorecorsi.com**. |
| Informativa privacy | [URL della versione 0.3.1](https://github.com/salvatorecorsi/convert-here/blob/v0.3.1/docs/privacy.md), pubblico verificato e salvato nelle Properties; titolare e contatto reali nell'informativa IT/EN. |
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

- Versione Store del nuovo candidato: `1.0.1.0`, distinta dalla versione applicazione Rust `0.3.1` e maggiore del precedente upload `1.0.0.0`. Il primo segmento Store deve essere maggiore di zero; il quarto rimane zero. Ogni segmento e al massimo 65535. [Versioni dei pacchetti](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements#version-numbering-for-windows-10-and-11-packages).
- Nome e Publisher devono corrispondere esattamente ai dati Partner Center, comprese maiuscole e punteggiatura. Il builder ricompila gli eseguibili con il manifest dedicato al pacchetto completo, senza l'elemento di identita sparse dell'installer EXE.
- La UI e attualmente in italiano. La scheda inglese e una traduzione della descrizione, non una traduzione dell'interfaccia. Non dichiarare supporto UI inglese finche non implementato e verificato.
- Target previsto: Windows 11 desktop x64 per il menu moderno. Il limite OS e quello effettivamente impostato e collaudato nel manifest Store; non promettere ARM64 nativo.

## Funzioni incluse e motori opzionali

Il pacchetto leggero offre subito conversioni immagini native e immagini in PDF, compressione PDF senza perdita, divisione e unione di PDF, coda, impostazioni e menu contestuale. Il modello espone 44 destinazioni per le categorie compatibili e riconosce 93 estensioni in ingresso. FFmpeg serve per audio/video; Poppler per PDF in immagini e testo (TXT, DOCX o HTML); LibreOffice per documenti Office/HTML in PDF ed Excel/CSV; Tesseract con dati delle lingue per OCR locale; libheif (`heif-dec` e `heif-enc`, con i codec necessari) per leggere e scrivere HEIC/AVIF. OCR dei PDF richiede anche Poppler per rasterizzare le pagine. Sono programmi esterni gia disponibili sul PC o selezionati dall'utente nella pagina Strumenti. La build Store non deve includere il downloader PowerShell del setup EXE.

PDF in DOCX/HTML ricostruisce solo il testo: le scansioni richiedono l'opzione OCR locale; immagini, tabelle e impaginazione non vengono ricostruite. CSV esporta il primo foglio senza formule o formattazione; il separatore di importazione/esportazione e selezionabile. Le risorse collegate nei documenti HTML possono accedere a file locali o alla rete. Compressione PDF non garantisce una riduzione delle dimensioni: se il candidato e piu grande, viene conservata una copia identica dell'originale. Divisione e unione conservano le pagine ma omettono metadati del documento, segnalibri e struttura accessibile; PDF firmati o con moduli interattivi richiedono una copia statica senza firme.

Per un PDF singolo, **PDF · comprimi** forza la compressione indipendentemente dall'operazione salvata. Per selezioni multiple PDF-compatibili, **PDF (Bulk)** mantiene lavori e risultati individuali; **PDF (Merge)** apre l'editor delle singole pagine per creare un PDF unico da immagini, PDF, Office, Excel/CSV e HTML. Le pagine possono essere intercalate con trascinamento o frecce. La sessione Merge usa snapshot temporanei nascosti e non crea lavori persistenti nella coda rapida. Divisione e unione restano disponibili nella GUI e con `--pdf-action split`/`--pdf-action merge` nella CLI. I titoli **TXT · OCR**, **TXT** e **TXT · OCR / testo PDF** distinguono il riconoscimento locale delle immagini dall'estrazione PDF. Audio e video non hanno una destinazione TXT. La DLL usa maschere a 64 bit; i GUID dei comandi esistenti e della voce personalizzate restano stabili. Whitelist dell'installer, registrazione classica, DLL e modello dell'app vengono confrontati nei test per tutte le 93 estensioni e le 44 destinazioni.

Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. Le conversioni e l'OCR vengono elaborati sul PC con le funzioni native e i motori locali selezionati dall'utente. Il comportamento di risorse HTML collegate, browser e cartelle di rete e descritto nell'[informativa privacy](privacy.md).


La dipendenza va dichiarata all'inizio delle descrizioni: la policy 10.2.4 ammette software non integrato con questa trasparenza. E un percorso ragionevole da sottoporre alla certificazione, non un'approvazione garantita. Mantenere le funzioni native utilizzabili senza motori e messaggi chiari quando un motore manca. La policy 10.2.2 vieta l'introduzione dinamica di funzionalita incoerenti con quanto dichiarato. [Store policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies#102-security).

## Capability e motivazione per la certificazione

`runFullTrust` e dichiarato per l'app desktop `mediumIL` e il COM packaged. La motivazione corrente di 424 caratteri e stata salvata nelle Submission options. Scopo tecnico:

> Convert Here is a native Rust desktop file conversion application. It reads files explicitly selected by the user, writes converted outputs to the selected folder, and provides an IExplorerCommand shell extension through packaged COM. Optional conversion engines already installed or explicitly selected by the user run as child processes with the user's permissions. The app does not require administrator privileges, a service, a driver, or elevation.

Non aggiungere `unvirtualizedResources` per abitudine ereditandolo dal pacchetto sparse. Serve a disabilitare la virtualizzazione di AppData/HKCU: conservarlo solo se un requisito verificato lo rende indispensabile, documentandone lo scope e chiedendo l'approvazione necessaria. Le capability ristrette richiedono una spiegazione nella submission. [Dichiarazioni delle capability](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations#restricted-capabilities), [virtualizzazione flessibile](https://learn.microsoft.com/en-us/windows/msix/desktop/flexible-virtualization).

## Stato dei materiali e delle verifiche

Il candidato 1.0.1.0 e le schede della 0.3.1 sono validati e salvati nel portale. Lo [stato delle verifiche](store-validation.md) distingue release, prove native e report storici dal collaudo d'installazione del MSIX.

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

## Note per il certificatore: riferimento per 0.3.1

Nel portale sono state salvate e rilette istruzioni di test di 1977 caratteri. Il riferimento esteso seguente descrive le procedure dell'app e del manifest; non attesta un collaudo d'installazione del MSIX.

> Convert Here 0.3.1 / Store package 1.0.1.0. Publisher: Corsivo; developer: Salvatore Corsi. The UI is Italian. The software is free, with no subscriptions, in-app purchases or usage-based charges. Conversion and OCR run locally and need no app account or login. Target: Windows 11 x64.
>
> Native test: launch Convert Here from Start, use Aggiungi file or drop a PNG/JPEG, select another image format or PDF, then Converti file. Outputs use the selected folder or the source folder; existing files are preserved. PDF compression, splitting and merging need no external engine. Use unsigned, unencrypted PDFs without interactive forms.
>
> Explorer test: select two PNG/JPEG files, right-click > Converti in > PDF (Bulk). Each input gets a separate queued result. A temporary shortcut opens job status. With the same selection choose PDF (Merge): wait for preparation, drag individual page handles or use arrows/Alt+Up/Down, then Crea PDF unico. Multipage PDFs can be mixed with images and pages interleaved across sources. Cambia cartella changes the destination. Merge also opens from the main app. Annulla cancels active work; closing the session attempts to remove its temporary snapshots.
>
> External engines are not bundled or automatically downloaded. Strumenti > Seleziona programma selects existing local executables. FFmpeg handles audio/video; Poppler handles PDF image/text export; LibreOffice handles Office/HTML and Excel/CSV; Tesseract and language data handle OCR; libheif tools/codecs handle HEIC/AVIF. PDF OCR also needs Poppler. Missing dependencies are reported. HTML linked resources may access local files or the network.
>
> PDF-to-DOCX/HTML reconstructs text only. OCR does not reconstruct original page layout. Animated images and multipage image containers use the first frame/image. Audio and video do not offer TXT output.
>
> runFullTrust supports the medium-integrity desktop app, user-selected file I/O, child conversion engines and packaged COM Explorer integration. No administrator elevation, service or driver is required.
