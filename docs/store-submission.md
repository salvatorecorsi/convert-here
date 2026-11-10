# Microsoft Store: stato della submission

## Stato corrente: GitHub 0.8.1 pubblicata / Store 1.0.9.0 in certificazione

La [release GitHub 0.8.1](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.1) è pubblicata il **10 novembre 2026 alle 14:42:06 CET (13:42:06 UTC)**, dal commit pulito **`07fe5f749327b73e93a92d5101d80b9b4f5317cb`**. La versione dell'app è **0.8.1**, quella del pacchetto Store **1.0.9.0**. [Windows checks 42825704990](https://github.com/salvatorecorsi/convert-here/actions/runs/42825704990) è **SUCCESS**, job verify **6 min 31 s**. I quattro asset pubblici sono stati scaricati nuovamente: byte e SHA-256 corrispondono ai locali auditati, ai digest GitHub e al file dei checksum scaricato. Tag e artefatti restano fissati al commit di build.

La **Submission 6**, ID **1152921505702367000**, è stata inviata; stato osservato alle **14:44:28 CET (13:44:28 UTC)** del 10 novembre: l'overview mostra **Update in certification**. Il MSIX **1.0.9.0** era caricato e salvato come **Validated**. Sono confermati prezzo **gratuito**, **240 mercati** e pubblicazione automatica appena approvato. La nuova 1.0.9.0 non è ancora dichiarata approvata o disponibile nello Store. La precedente **Submission 5 / Store 1.0.8.0 è pubblicata**; i record sottostanti conservano l'osservazione storica prima dell'approvazione.

Il nuovo MSIX misura **9.359.517 byte**, SHA-256 **`0b7af68333a5587e0509c65909ebb01cba541e72fcbe4307cc2d2a2e24def772`**. Audit standalone e Store **PASS**: **108 file** portable identici allo stage, tre binari corrispondenti a `build-info.json`, eseguibili 0.8.1 e CRT statico; **57 elementi payload / 59 voci ZIP**, **277 componenti Windows**, licenze complete, identità e CLSID invariati, COM surrogate/STA e verbo **0000ConvertHere**, x64, MinVersion **10.0.19041.0**, sola capability **runFullTrust**. Il MSIX contiene GUI, DLL e installer nativo; CLI separata, motori, modelli e script downloader non sono inclusi.

Sono passati **424 test app + 18 DLL = 442**, con un test facoltativo ignorato, e **13/13 conversioni** con i binari finali. I sorgenti estratti contengono **240 file tracciati** corrispondenti al commit, **465 crate** e **24.338 file vendor** verificati. Veri `cargo check --locked --offline --all-targets` per app (**81,23 s**) e DLL (**4,17 s**) sono passati con cache e target inizialmente vuoti e compilatore fissato; non è una build offline completa con linking.

La QA della **GUI debug nativa 0.8.1 in configurazioni isolate** verifica prima apertura del wizard, annullamento della conferma, ritorno alla scelta, Salta, marker separato persistente, riapertura manuale e riavvio senza wizard; sono provati anche Seleziona tutti, modifica di una singola estensione e ripristino degli otto default. **Il download nel nuovo wizard non è stato premuto**: questa QA non attesta un'installazione reale attraverso la nuova interfaccia. Le prove dei binari finali sono separate. **Installazione del nuovo MSIX, prova su host Windows 10 reale e WACK non eseguiti.**

Schede IT/EN **salvate e rilette**, ciascuna con **20 funzionalità e quattro PNG/didascalie**: due nuove catture 0.8.1 e due storiche 0.8.0 degli editor invariati, tutte native **2562 × 1441**, senza puntatore e con UI italiana dichiarata. Privacy e licenza **v0.8.1** sono verificate. Le note finali di **5.835 caratteri** sono salvate/rilette con normalizzazione degli spazi verificata; la motivazione **runFullTrust di 470 caratteri**, aggiornata ai cinque motori e al consenso, è salvata e riletta. IARC è **Complete**, ereditato; nessun nuovo accordo accettato.

Materiale corrente: note [IT](release-notes-0.8.1.md) / [EN](release-notes-0.8.1.en.md), [audit e digest 0.8.1](store-testing-0.8.1.md), [verifiche tecniche](store-validation.md), [provenienza delle catture](store-assets/0.8.1/README.md) e schede [IT](store-listing-it.md) / [EN](store-listing-en.md). Evidenze locali: `.tools/standalone-081-audit.json`, `.tools/store-081-audit.json`, `.tools/source-offline-081-e77647330ce84fa9b27e64a944e4fe83/offline-check.json`, `.tools/release-smoke-081-324df6a7ee364e5b8cf2784167d14715/report.json`, `.tools/remote-release-081-f0e38951b2c4436396432966aa4fb721/verification.json`, `.tools/ui-081-qa.json` e `.tools/store-081-submission-proof.txt` / `.jpg`. Gli aggiornamenti documentali non ricostruiscono o sostituiscono gli artefatti pubblicati.

## Storico: GitHub 0.8.0 / Store 1.0.8.0 — stato osservato alle 12:10 UTC del 10 novembre

La [release GitHub 0.8.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.0) è pubblicata dal **10 novembre 2026, 12:09:39 UTC**, senza stato draft e come latest, dal commit pulito **`f1f3f95fceed52507ef715fa4ac6893acc63544a`**. [Windows checks 42811925135](https://github.com/salvatorecorsi/convert-here/actions/runs/42811925135) è **SUCCESS**, job verify **9 min 49 s**. I quattro asset pubblici hanno dimensioni e digest remoti uguali a quelli locali verificati. Tag, sorgenti e artefatti finali restano fissati al commit di build; questo aggiornamento documentale non li ricostruisce o sostituisce.

La **Submission 5**, ID **1152921505702365703**, è stata inviata il **10 novembre 2026 intorno alle 12:10 UTC**. L'overview mostra **Update in certification**. Non vengono dedotte ulteriori fasi della timeline non mostrate. La pubblicazione automatica appena approvato è confermata, con prezzo **gratuito** e **240 mercati**. La nuova **1.0.8.0 non è ancora dichiarata approvata o disponibile nello Store**. La precedente **Submission 4 / Store 1.0.7.0** è stata invece verificata **pubblicata** il 10 novembre.

Il MSIX **1.0.8.0** è stato caricato, salvato come **Validated** e sottoposto ad audit **PASS**: **9.342.790 byte**, SHA-256 **`a7a010371015a69b798e255b373b90f6cfddf4fed4b5611bc7cd845f00c841f9`**. Sono verificati **57 elementi payload / 59 voci ZIP**, GUI 0.8.0 e DLL, **277 componenti di dipendenze**, GPL e licenze complete, identità tecnica invariata, x64, MinVersion **10.0.19041.0** e sola capability **runFullTrust**. Il manifest mantiene COM surrogate/STA, CLSID e verbo **0000ConvertHere**. Il pacchetto contiene l'installer nativo dei motori, senza CLI separata, motori, modelli o script downloader. L'audit non installa il pacchetto, non esegue il payload e non modifica registro o certificati. WACK e prova del nuovo MSIX su Windows 10 reale non sono attestati.

Schede italiana e inglese **salvate e rilette**, ciascuna con **20 funzionalità e quattro PNG 2562 × 1441**, con didascalie che dichiarano l'interfaccia italiana. Privacy nelle Properties e licenza nelle due schede puntano ai file pubblici del tag **v0.8.0**, con salvataggio completato. Le Additional Testing Information finali di **3.207 caratteri** sono salvate e rilette. La voce dell'accordo Windows **8.11** risultava già accettata il 4 novembre, **Action: N/A**; nessuna nuova accettazione è stata eseguita.

Audit standalone **PASS**, **98 file** portable identici allo stage e tre binari uguali a `build-info.json`. Dai sorgenti finali estratti sono passati veri `cargo check --locked --offline --all-targets` per app (**75,56 s**) e DLL (**4,02 s**), con cache e target inizialmente vuoti, **465 crate**, **24.338 file vendor** e **228 file tracciati** corrispondenti al commit ammettendo soltanto CRLF → LF. Non è una build completa con linking. Smoke test dei binari finali **13/13 PASS**, con sorgenti preservate e nessuna sovrascrittura.

Materiale corrente: [scheda IT](store-listing-it.md), [scheda EN](store-listing-en.md), note [IT](release-notes-0.8.0.md) / [EN](release-notes-0.8.0.en.md), [audit, digest e prove 0.8.0](store-testing-0.8.0.md), [verifiche tecniche](store-validation.md) e [provenienza delle quattro catture](store-assets/0.8.0/README.md). Evidenze locali: `.tools/github-release-080-verified.json`, `.tools/store-080-submission-state.txt`, `.tools/store-080-submission-proof.jpg`, `.tools/store-notes-080-verified.txt`, `.tools/store-listing-080-it-verified.json` e `.tools/store-listing-080-en-verified.json`. I report degli audit e del controllo offline sono elencati nel record di test.

## Storico: GitHub 0.7.1 / Store 1.0.7.0 — stato osservato il 9 novembre

La [release GitHub 0.7.1](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.1) è pubblicata dal **9 novembre 2026, 21:08:01 UTC**, commit pulito **2c6d6968159a46baef37d8f20aef049e7be4c8c4**. [Windows checks 42698306310](https://github.com/salvatorecorsi/convert-here/actions/runs/42698306310) è **SUCCESS**. Setup, portable, sorgenti vendorizzati e checksum sono verificati anche rispetto ai digest GitHub. Dai sorgenti estratti sono passati veri cargo check offline per app e DLL con cache inizialmente vuota: **68,45 s** e **3,92 s**, senza attestare una build completa con linking.

La **Submission 4**, ID **1152921505702358188**, è stata inviata il **9 novembre 2026 alle 21:09 UTC**. Stato osservato: **Update in certification / Product update: In certification**, Submission Complete, Pre-processing In progress, Certification e Publishing Not started. È selezionata la **pubblicazione automatica appena approvato**. La 1.0.7.0 non è ancora dichiarata approvata o disponibile nello Store. La precedente **Submission 3 / Store 1.0.6.0** risultava invece già pubblicata all'inizio di questo aggiornamento.

Il MSIX **1.0.7.0** misura **7441267 byte**, SHA-256 **60ec178179ade97e1f5f23c40a26c259a25da2ed48cea78c1a1456927ec55d87**. Sono verificati commit, GUI 0.7.1, DLL, **57 payload / 59 voci ZIP**, **243 notice**, CRT statico e sola capability runFullTrust. Restano identità e CLSID, x64 e MinVersion **10.0.19041.0**. Il verbo manifest è **0000ConvertHere**, mitigazione del menu classico da verificare su Windows 10 reale; non è stata installata questa build MSIX né eseguito WACK. GUI, DLL e installer nativo dei motori sono inclusi; CLI e motori non sono inclusi nel MSIX.

IT ed EN sono salvate e rilette: **20 funzionalità e due PNG 2562 × 1430 per lingua**, con didascalie che dichiarano l'interfaccia italiana. Tutte le quattro vecchie immagini di ciascuna scheda sono state rimosse dalla bozza. Nel ramo corrente sono ritirati i 20 screenshot contaminati; i tag storici restano immutati. [Provenienza delle catture](store-assets/0.7.1/README.md).

Privacy e licenza puntano al tag **v0.7.1**. Le nuove Additional Testing Information sono state salvate e ricaricate (**2380 caratteri nel campo**, con normalizzazione dei fine riga). La motivazione runFullTrust resta **457 caratteri**; non sono state aggiunte capability, credenziali o nuove accettazioni contrattuali. Prezzo, mercati e identità pubblica Salvatore Corsi restano invariati.

Materiale corrente: [scheda IT](store-listing-it.md), [scheda EN](store-listing-en.md), note [IT](release-notes-0.7.1.md) / [EN](release-notes-0.7.1.en.md), [test 0.7.1](store-testing-0.7.1.md) e [validazione](store-validation.md). Evidenza locale: .tools/store-submission-071-final-state.txt e .tools/store-submission-071-final.jpg; report .tools/standalone-071-audit.json, .tools/store-071-audit.json e .tools/source-offline-071-fa491735cf384b60ac134a55afc23b9d/offline-check.json.

I paragrafi seguenti conservano gli stati osservati nelle versioni precedenti. Le indicazioni «non ancora pubblicata» nello storico si riferiscono all'ora riportata, precedente all'approvazione della Submission 3.

## Storico: GitHub 0.7.0 / Store 1.0.6.0 — stato osservato alle 19:45 UTC

La [release GitHub 0.7.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.0) è pubblicata dal **9 novembre 2026, 19:36:38 UTC**, commit `e4b41f66ce2d99c9c7bd528aa29733a71db2330d`; [Windows checks 42683376497](https://github.com/salvatorecorsi/convert-here/actions/runs/42683376497) è **SUCCESS**. Setup, portable, sorgenti e checksum sono verificati. I sorgenti includono **431 crate vendorizzati**; veri `cargo check --locked --offline --all-targets` per app e DLL sono passati con cache Cargo inizialmente vuota e CRT statica, senza attestare una build completa con linking.

Il MSIX **1.0.6.0**, **7437834 byte**, SHA-256 `84f2e3561e73f425393312139629edab8d149747380f369580bbe3babbed52d1`, deriva dal medesimo commit pulito. Sono verificati **57 elementi payload**, **59 voci ZIP**, **243 avvisi di dipendenze**, identità e requisito **Windows 10 versione 2004/build 19041 o successivo, x64**, incluso Windows 11. Il pacchetto contiene GUI e DLL Explorer; la CLI è nella distribuzione GitHub. Non sono attestati installazione del nuovo MSIX su Windows 10 reale o WACK.

Il pacchetto è stato caricato e salvato come **Validated**. La **Submission 3**, ID **1152921505702351165**, è stata inviata il **9 novembre 2026 alle 19:45 UTC**: il portale mostra **Update in certification / Product update: In certification**, con **Submission Complete**, **Pre-processing In progress**, **Certification Not started**, **Publishing Not started**. La 0.7.0 non è ancora approvata o pubblicata nello Store. Restano prezzo **gratuito**, **240 mercati** e pubblicazione automatica appena superata la certificazione.

Schede IT ed EN salvate e rilette, ciascuna con **20 funzionalità e quattro schermate**; privacy fissata alla **0.7.0** salvata e riletta. Le Additional Testing Information corrette, di **4406 caratteri**, sono salvate con persistenza verificata; la motivazione `runFullTrust` è di **457 caratteri**. Le note descrivono correttamente GUI e DLL nel MSIX, senza CLI. Nome pubblico **Salvatore Corsi**, identità tecnica, GUID e prezzi restano invariati.

Materiale corrente: [scheda IT](store-listing-it.md), [scheda EN](store-listing-en.md), note [IT](release-notes-0.7.0.md) / [EN](release-notes-0.7.0.en.md), [istruzioni di test 0.7.0](store-testing-0.7.0.md) e [audit, checksum e limiti delle verifiche](store-validation.md). Evidenza locale del portale: `.tools/store-submission-070-final-state.txt`; report `.tools/standalone-070-audit.json`, `.tools/store-070-audit.json` e `.tools/source-offline-070-9e939ce0cc78475b84b6285146e10fda/offline-check.json`. I record seguenti descrivono stati precedenti; tag, artefatti e report restano immutati.

## Storico: release GitHub 0.6.0 / candidato Store 1.0.5.0

Stato registrato il 9 novembre 2026 prima dell'aggiornamento 0.7.0: il candidato 1.0.5.0 non fu caricato o inviato. Lo stato attuale della medesima Submission 3 è quello descritto sopra.

La [release GitHub **0.6.0**](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.6.0) è pubblicata dal **9 novembre 2026, 18:28:13 UTC**, dal commit `9b959405e64e62f1f05f49a821bb9444747b3620`, senza stato draft. [Windows checks 42672724894](https://github.com/salvatorecorsi/convert-here/actions/runs/42672724894) è **SUCCESS**. I quattro asset pubblici corrispondono a dimensioni e digest locali verificati; l'archivio sorgente ha superato veri cargo check --offline --locked --all-targets di app e DLL con cache inizialmente vuota, senza attestare una build completa con linking.

Il MSIX **1.0.5.0** è costruito e verificato: **7123294 byte**, SHA-256 `185212fc3118f0d499763e415fcad33ce1eec1999bce5c678de892a8a27d9e5e`, con **57 elementi payload**, **59 voci ZIP** e **243 avvisi di dipendenze**. **Non è caricato, Validated o inviato**: il tentativo tramite selettore file automatico non è stato completato. La **Submission 3**, ID **1152921505702351165**, resta **in bozza**. La scheda italiana è salvata e riletta come **Updated**; per l'inglese è stato eseguito Save, ma la persistenza dopo il salvataggio **non è stata verificata**. Non è attestata approvazione o pubblicazione Store della 0.6.0.

Materiale storico: note [IT](release-notes-0.6.0.md) / [EN](release-notes-0.6.0.en.md) e [istruzioni di test](store-testing-0.6.0.md). In quel momento restavano da completare caricamento e verifica del pacchetto e dei campi del portale. Gli hash, i report e le evidenze precedenti restano storici; tag e artefatti pubblicati non sono modificati da questo aggiornamento.

## Ultimo stato registrato 0.4.0 / Store 1.0.3.0 — storico

La [release GitHub 0.4.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.4.0) e pubblicata dal **4 novembre 2026, 18:56:51 UTC**, senza flag draft o prerelease, dal commit `b4d5d0866a59b821125e35053566c454dd598471`. Il pacchetto Store **1.0.3.0** e stato generato, verificato, caricato e salvato come **Validated**. La **Submission 2**, ID **1152921505702316905**, e stata inviata il **4 novembre 2026** e risulta **Update in certification / Product update: In certification**. Timeline: **Submission Complete**, **Pre-processing In progress**, **Certification Not started**, **Publishing Not started**. La 0.4.0 non e ancora approvata o pubblicata nello Store; la pubblicazione iniziera automaticamente dopo il superamento della certificazione.

Le schede IT/EN sono salvate e rilette con 17 funzionalita e quattro schermate ciascuna: tre nuove della 0.4.0 e PDF Merge della 0.3.1 riusata con interfaccia invariata. Le istruzioni di test di **3446 caratteri** e la motivazione `runFullTrust` di **433 caratteri** persistono dopo il ricaricamento. Prezzo gratuito, tutti i 240 mercati e pubblicazione appena approvata restano impostati. Privacy 0.4.0 nelle Properties e URL GPL 0.4.0 in entrambe le schede sono salvati e riletti; i campi telefono e indirizzo restano vuoti.

Le Impostazioni mostrano otto uscite per default: MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4. Restano supportate 93 estensioni in ingresso e 44 destinazioni del motore. Il filtro riguarda GUI e menu Explorer, senza bloccare CLI, richieste rapide esplicite o lavori gia accodati.

In **Impostazioni → Motori** puoi richiedere FFmpeg, Poppler o entrambi con conferma di origine, versione, dimensioni, licenze e destinazione. Il download HTTPS verifica lo SHA-256 prima dell'estrazione nel profilo utente, conservando licenze e documentazione. IP e normali dati HTTP raggiungono i distributori; i file da convertire non vengono caricati. Dipendenza e download sono descritti all'inizio delle schede IT/EN e nella [privacy 0.4.0 pubblica verificata](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/docs/privacy.md).

Le [Store Policies 7.20](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies#102-security), efficaci dal 22 ottobre 2026, richiedono consenso e coerenza delle estensioni con la funzione dichiarata; non stabiliscono un divieto generale di acquisire dipendenze. Resta necessaria la certificazione Microsoft.

## Ultimo stato verificato 0.3.2 / Store 1.0.2.0

Stato al 4 novembre 2026: [Convert Here **0.3.2**](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.2) e pubblicata su GitHub dal commit `8f7532bf9d16f80ef65b5229dba93929e11c1d82` e risulta **In Microsoft Store**. Il nome editore **Salvatore Corsi** e approvato, con account Active e verifica Authorized. E stata osservata l'installazione del pacchetto completo **1.0.2.0** con **SignatureKind: Store**. La Submission 1, ID **1152921505702314630**, aveva seguito il percorso di certificazione documentato; questo record e storico rispetto all'aggiornamento 1.0.3.0.

Il prodotto resta **9NPKPPCBNBM8**, `Name` **SalvatoreCorsi.ConvertHere** e `Publisher` **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17**; identita e GUID tecnici non cambiano. Il nuovo `ConvertHere-1.0.2.0-x64-store.msix` e di **6165973 byte**, SHA-256 `c6d655b58156e4a14e13653a0f5115f3bffc2ba5ff086dbe435bc34440a053b7`, generato dal commit pulito `8f7532bf9d16f80ef65b5229dba93929e11c1d82` e verificato prima del caricamento.

Tutte le sei sezioni erano **Complete** prima dell'invio della 1.0.2.0. La precedente timeline **Submission Complete**, **Pre-processing In progress**, **Certification Not started**, **Publishing Not started** e un'evidenza storica anteriore alla pubblicazione ora verificata. Restano prezzo gratuito e tutti i 240 mercati selezionati.

La [privacy 0.3.2](https://github.com/salvatorecorsi/convert-here/blob/v0.3.2/docs/privacy.md) e pubblica verificata e salvata, con persistenza confermata al ricaricamento. Additional Testing Information di **1977 caratteri**, aggiornate alle versioni 0.3.2 / 1.0.2.0, sono salvate e rilette. I campi telefono e indirizzo restano vuoti. Le schede riutilizzano le tre catture native 0.3.1 perche l'interfaccia e identica; non sono nuove catture della 0.3.2.

## Stato della 0.3.1 prima del ritiro della certificazione

Stato al 4 novembre 2026: **Convert Here 0.3.1** e [pubblicata su GitHub](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.1), dal commit `884cb2c610a9197d8b4853f0fa741dbe9035b5ed`. Il candidato Store **1.0.1.0** e stato caricato e risulta **Validated**, con **Packages: Complete**. Il prodotto resta **9NPKPPCBNBM8**, [pagina Store](https://apps.microsoft.com/detail/9NPKPPCBNBM8), `Name` **SalvatoreCorsi.ConvertHere**, `Publisher` **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17**. Titolare, autore e sviluppatore: **Salvatore Corsi**; contatto pubblico: **salvo@salvatorecorsi.com**, dalla [pagina contatti](https://www.salvatorecorsi.com/contatti/). IARC e stato accettato su consenso esplicito del proprietario; Age ratings risulta **Complete**, con classificazioni IARC/Microsoft/PEGI 3+. La submission e stata inviata il **4 novembre 2026** e risulta **In certification**. **La pubblicazione Store non e ancora iniziata.**

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

Il menu usa la DLL `IExplorerCommand`, COM surrogate e `windows.fileExplorerContextMenus` dichiarati nel manifest del pacchetto completo: menu classico su Windows 10, moderno su Windows 11. Non richiede script separati di registrazione nel pacchetto Store. [Integrazione ufficiale con Esplora file](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer).

## Dati confermati e campi della submission

| Campo | Stato / azione |
| --- | --- |
| Account sviluppatore Partner Center | Gia registrato; eventuali azioni di verifica pendenti sono quelle mostrate dal portale. |
| Tipo account | **Individual**, confermato in Legal info nella verifica corrente. |
| Nome prodotto riservato | **Convert Here**, prenotazione confermata dal proprietario il 4 novembre 2026. |
| `Package/Identity/@Name` | `SalvatoreCorsi.ConvertHere`, dalla schermata Product identity fornita dal proprietario. |
| `Package/Identity/@Publisher` | `CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17`, dalla stessa schermata. Non sostituirlo con un `CN=Nome editore` presunto. |
| Publisher display name | **Salvatore Corsi**, approvato; profilo **Authorized**, account **Active** e nome pubblico confermato in Legal info. |
| Store ID | `9NPKPPCBNBM8`; l'identificativo non attesta la pubblicazione dell'app. |
| Titolare e contatto | **Salvatore Corsi**, **salvo@salvatorecorsi.com**. |
| Informativa privacy | [Versione 0.8.1 pubblica](https://github.com/salvatorecorsi/convert-here/blob/v0.8.1/docs/privacy.md); URL aggiornato e salvato nelle Properties della Submission 6. |
| Supporto | **salvo@salvatorecorsi.com** oppure [pagina contatti](https://www.salvatorecorsi.com/contatti/), gia pubblici. Per PC il campo e facoltativo, ma consigliato. |
| Prezzo, mercati e calendario | Scelte confermate dal proprietario e salvate: **Gratuita**, **tutti i 240 mercati disponibili**, **pubblicazione appena approvata**. |
| Licenza applicazione Store | Codice originale **GPL-3.0-only**, Copyright (C) 2026 Salvatore Corsi. [Testo integrale 0.8.1](https://github.com/salvatorecorsi/convert-here/blob/v0.8.1/LICENSE); URL singolo aggiornato e salvato nelle schede IT/EN della Submission 6. Le licenze dei terzi restano invariate. |

Riferimenti Microsoft: [privacy e supporto](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/support-info), [termini di licenza aggiuntivi](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-additional-information#additional-license-terms).

Microsoft accetta un singolo URL come collegamento cliccabile ai termini, oppure fino a 10000 caratteri di testo semplice. Il collegamento alla licenza completa evita di troncare la GPL; le schede IT/EN riportano anche il repository dei sorgenti. `ConvertHere-0.8.1-source.zip` è pubblicato con dipendenze Cargo vendorizzate e ricetta di build offline; i `cargo check` offline sono verificati con cache inizialmente vuota. Licenza e avvisi terzi sono inclusi nei pacchetti; la conversione non cambia i diritti sui file dell'utente.

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

[Windows checks](https://github.com/salvatorecorsi/convert-here/actions/workflows/windows-ci.yml) esegue automaticamente formattazione, test Rust e fixture PowerShell. Una pipeline verde verifica queste operazioni; non sostituisce la certificazione Microsoft o il collaudo dell'installazione su Windows 10 e 11.

- Versione della submission corrente: `1.0.9.0`, distinta dalla versione applicazione `0.8.1`, generata, validata e inviata nella Submission 6; la precedente `1.0.8.0` è pubblicata. Il precedente candidato `1.0.5.0` / `0.6.0` non fu caricato. Il primo segmento Store deve essere maggiore di zero; il quarto rimane zero. Ogni segmento è al massimo 65535. [Versioni dei pacchetti](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements#version-numbering-for-windows-10-and-11-packages).
- Nome e Publisher devono corrispondere esattamente ai dati Partner Center, comprese maiuscole e punteggiatura. Il builder ricompila gli eseguibili con il manifest dedicato al pacchetto completo, senza l'elemento di identita sparse dell'installer EXE.
- Interfaccia e menu supportano italiano e inglese; in Impostazioni si sceglie Sistema, Italiano o English indipendentemente dalla lingua OCR.
- Target: Windows 10 versione 2004/build `10.0.19041.0` o successivo, desktop x64, incluso Windows 11. Menu classico su Windows 10 e moderno su Windows 11. Manifest e import PE sono verificati; nessuna installazione su un host Windows 10 reale è attestata. Nessuna build ARM64 nativa dichiarata.

## Funzioni incluse e motori opzionali

La 0.8.1 comprende editor nativo PNG/JPEG/WebP/PDF con ritaglio, scala, tela e peso effettivo, impaginazione A4/A5 da immagini o testo semplice scritto/TXT UTF-8 e profili personali. Il manager consente creazione, modifica, duplicazione, scelta dei predefiniti, import/export JSON e filtri sulle estensioni di ingresso; limiti di peso per file e durata audio/video, opzioni FFmpeg avanzate tipate e cronologia locale completano il flusso. Restano SVG statico rasterizzato, editor audio/video con timeline e gain e trascrizione locale. Si possono salvare 32 profili e mostrarne 8 nel menu; i profili conservano parametri senza immagini di prova o testo digitato. Le ricette vengono applicate soltanto a ingressi compatibili; sono riconosciute 94 estensioni. La procedura di prova è nella [checklist 0.8.1](store-testing-0.8.1.md).

Il pacchetto leggero offre subito conversioni immagini native e immagini in PDF, SVG statico rasterizzato, compressione PDF senza perdita, divisione e unione di PDF, coda, impostazioni e menu contestuale. Il modello espone 44 destinazioni per le categorie compatibili e riconosce 94 estensioni in ingresso. FFmpeg serve per audio/video; Poppler per PDF in immagini e testo (TXT, DOCX o HTML); LibreOffice per documenti Office/HTML in PDF ed Excel/CSV; Tesseract con dati delle lingue per OCR locale; libheif (`heif-dec` e `heif-enc`, con i codec necessari) per leggere e scrivere HEIC/AVIF. OCR dei PDF richiede anche Poppler per rasterizzare le pagine; la trascrizione audio/video usa FFmpeg/ffprobe e Whisper Tiny. FFmpeg, Poppler, Tesseract con italiano/inglese, libheif e Whisper con Tiny si scaricano su richiesta con conferma nell'app, anche dalla configurazione guidata sequenziale. LibreOffice si installa separatamente; tutti i motori già presenti restano selezionabili da Impostazioni → Motori. Nessun motore, modello o downloader PowerShell è incluso nel MSIX.

PDF in DOCX/HTML ricostruisce solo il testo: le scansioni richiedono l'opzione OCR locale; immagini, tabelle e impaginazione non vengono ricostruite. CSV esporta il primo foglio senza formule o formattazione; il separatore di importazione/esportazione e selezionabile. Le risorse collegate nei documenti HTML possono accedere a file locali o alla rete. Compressione PDF non garantisce una riduzione delle dimensioni: se il candidato e piu grande, viene conservata una copia identica dell'originale. Divisione e unione conservano le pagine ma omettono metadati del documento, segnalibri e struttura accessibile; PDF firmati o con moduli interattivi richiedono una copia statica senza firme.

Per un PDF singolo, **PDF · comprimi** forza la compressione indipendentemente dall'operazione salvata. Per selezioni multiple PDF-compatibili, **PDF (Bulk)** mantiene lavori e risultati individuali; **PDF (Merge)** apre l'editor delle singole pagine per creare un PDF unico da immagini, PDF, Office, Excel/CSV e HTML. Le pagine possono essere intercalate con trascinamento o frecce. La sessione Merge usa snapshot temporanei nascosti e non crea lavori persistenti nella coda rapida. Divisione e unione restano disponibili nella GUI e con `--pdf-action split`/`--pdf-action merge` nella CLI. I titoli **TXT · OCR**, **TXT** e **TXT · OCR / testo PDF** distinguono il riconoscimento locale delle immagini dall'estrazione PDF. Audio e video offrono **TXT · Trascrizione** con i motori locali, quando TXT è abilitato in Raw. La DLL usa maschere a 64 bit; i GUID dei comandi esistenti e della voce personalizzate restano stabili. Whitelist dell'installer, registrazione classica, DLL e modello dell'app vengono confrontati nei test per tutte le 94 estensioni e le 44 destinazioni.

Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. Le conversioni e l'OCR vengono elaborati sul PC con le funzioni native e i motori locali selezionati dall'utente. Il comportamento di risorse HTML collegate, browser e cartelle di rete e descritto nell'[informativa privacy](privacy.md).


La dipendenza va dichiarata all'inizio delle descrizioni: la policy 10.2.4 ammette software non integrato con questa trasparenza. E un percorso ragionevole da sottoporre alla certificazione, non un'approvazione garantita. Mantenere le funzioni native utilizzabili senza motori e messaggi chiari quando un motore manca. La policy 10.2.2 vieta l'introduzione dinamica di funzionalita incoerenti con quanto dichiarato. [Store policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies#102-security).

## Capability e motivazione per la certificazione

`runFullTrust` è dichiarato per l'app desktop `mediumIL` e il COM packaged. Per la **1.0.9.0 / Submission 6** è salvata e riletta una motivazione di **470 caratteri**, aggiornata ai cinque motori facoltativi, alla richiesta esplicita e al consenso prima del download, oltre a conversioni locali e integrazione Explorer, senza elevazione. La precedente 1.0.6.0 aveva una motivazione di 457 caratteri. Il testo seguente, di 424 caratteri, appartiene invece al precedente candidato 1.0.1.0 e resta storico:

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

## Note per il certificatore: 0.4.0 salvata

Il testo seguente e quello salvato e riletto per la 0.4.0 / Store 1.0.3.0 (**3446 caratteri**). La precedente 0.3.2 aveva note di 1977 caratteri. L'invio effettivo della Submission 2 e registrato nel riepilogo corrente sopra.

> Convert Here 0.4.0 / full MSIX 1.0.3.0, Windows 11 x64. Publisher: Salvatore Corsi. Italian application UI; English listing translates the documentation. This is an update to the currently published 1.0.2.0 package with the same product/package identity. No account, login, payment, subscription, in-app purchase, advertising or application telemetry.
>
> All file conversion and OCR run locally. The package contains the Rust/egui app, Explorer COM DLL, assets and licenses. No conversion engines are bundled. Native image conversions and image-to-PDF work immediately, as do PDF compression, splitting and merging. FFmpeg is needed for audio/video; Poppler for PDF page/text export; LibreOffice for Office, Excel/CSV and HTML-to-PDF; Tesseract plus language data for OCR; libheif for HEIC/AVIF. HTML documents may resolve linked local or remote resources; JavaScript is not executed.
>
> NEW OUTPUT PREFERENCES: open Impostazioni at top right, Formati di uscita. Default outputs are MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Expand Audio/Video/Immagini/Documenti e PDF, toggle destinations, or use Piu comuni/Attiva tutti. This changes GUI and Explorer destination visibility only. All 93 input extensions remain recognized. For example, a FLAC selection still shows Converti in and offers MP3/WAV by default. The menu rereads saved preferences each opening; no Explorer restart is required. CLI and already queued jobs keep their conversion capabilities.
>
> NEW OPTIONAL ENGINES: in Impostazioni scroll to Motori di conversione. Use Installa FFmpeg e Poppler or the individual Scarica e installa buttons. A confirmation lists sources, versions, download/expanded sizes, licenses and destination. Annulla performs no download. On explicit confirmation, the app downloads pinned portable ZIPs over HTTPS from Gyan/Poppler Windows releases on GitHub/CDNs, verifies exact length and SHA-256, safely extracts with original licenses and activates the tools. No downloaded program is executed by installation. Progress and cancellation remain available; failure permits retry and preserves existing unrelated tools. Destination is the user's local data ConvertHere/tools folder (Windows may virtualize MSIX data). These tools enhance the already described conversion functions; no services, drivers, elevation or application self-update. Only normal HTTP connection data reaches distributors, never conversion files or their paths.
>
> SMOKE TEST: add two PNG/JPG files, choose JPEG and Converti file; originals remain. Right-click compatible files in Explorer, Converti in: PDF (Bulk) queues separate results; PDF (Merge) opens page ordering. Drag pages or use arrows/Alt+Up/Down, then create one PDF. Audio/video are excluded from PDF merge. PDF/HTML/DOCX text export does not reconstruct layout; OCR is opt-in. Encrypted/signed PDFs and interactive forms have documented operation limits.
>
> GPL-3.0-only applies to original application code; dependencies/fonts/engines retain their licenses. Full LICENSE is packaged and readable under Impostazioni > Informazioni e licenze. Corresponding source with vendored Rust dependencies and offline build instructions is provided at https://github.com/salvatorecorsi/convert-here/releases/tag/v0.4.0 . License: https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/LICENSE . Privacy: https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/docs/privacy.md . Support: https://www.salvatorecorsi.com/contatti/
