# Verifiche tecniche — aggiornamento 9 novembre 2026

Queste verifiche accompagnano la preparazione Store di Convert Here, sviluppato da Salvatore Corsi. Non equivalgono alla certificazione Microsoft e non sostituiscono l'identita reale di Partner Center.

## Release 0.7.0 pubblicata / Store 1.0.6.0 in certificazione

La [release GitHub 0.7.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.0) è pubblicata dal **9 novembre 2026, 19:36:38 UTC**, commit `e4b41f66ce2d99c9c7bd528aa29733a71db2330d`. [Windows checks 42683376497](https://github.com/salvatorecorsi/convert-here/actions/runs/42683376497) è **SUCCESS**. L'aggiornamento Store **1.0.6.0** è stato caricato, validato e inviato per certificazione; non è ancora approvato o pubblicato.

La suite completa ha superato **348 test app**, con **un test già ignorato**, e **18 test DLL**. Sono passate anche **due ulteriori regressioni dei controlli numerici e degli alias UI**, oltre a **Clippy con `--all-targets -- -D warnings`**. Le fixture di packaging Store hanno superato **50 controlli** e quelle di branding **37**, sia con PowerShell 7 sia con Windows PowerShell 5.1. Le prove COM e di avvio isolato verificano anche filtri di ingresso, profili Merge e profili FFmpeg per sorgenti audio/video, senza registrare la DLL nel sistema.

La QA della finestra Windows ha verificato le quattro schede **File, Cronologia, Impostazioni e Profili**, le colonne dei formati, la griglia dei motori e il manager. Un'importazione JSON non valida è stata rifiutata senza creare profili (**0**); quella valida ha importato **4 profili**. Queste prove sono distinte dal semplice parsing del JSON e dalle regressioni automatiche.

Un profilo rapido ha convertito un **FLAC di 35 secondi in MP3** con durata massima di **30 secondi**. Il risultato misura **721649 byte**; FFprobe riporta **30.000000 secondi**. La cronologia è stata aggiornata e osservata anche in un'altra finestra dell'app.

Le [due catture italiane della 0.7.0](store-assets/0.7.0/README.md), **JPEG 1722 × 959**, documentano Impostazioni e Profili dalla build debug nativa degli stessi sorgenti destinati alla release. Provenienza, nomi e digest sono nel registro della cartella; gli indici [Store](store-assets/README.md) e [screenshot](screenshots/README.md) mantengono separate le catture precedenti.

Il requisito del manifest Store è **Windows 10 build 19041 o successivo, x64**, verificato dalle fixture e dal pacchetto finale. Le importazioni PE dell'app (**298 simboli**) e della DLL (**110**) non aggiungono simboli rispetto alla precedente review Windows 10; non risultano dipendenze CRT dinamiche o importazioni esclusive di Windows 11. **Non è stato eseguito un test di installazione su un host Windows 10 reale, né il WACK o l'installazione del nuovo MSIX.**

| Artefatto | Byte | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.7.0-windows-x64-setup.exe` | 12758928 | `6f4840b70bafb9a6fcfa1ed3baf35aefd70b3145d573fbcf1818eefd34046238` |
| `ConvertHere-0.7.0-windows-x64-portable.zip` | 16996599 | `4da97260440031e982c315aee13f484fb0f32cb3ca8c9af6dcb8dce308c6d90e` |
| `ConvertHere-0.7.0-source.zip` | 84725416 | `fa5c8f4ec5cab1758116954020add7915ef02a4282da59171e815dab93c3da51` |
| `SHA256SUMS.txt` | 313 | `bdf038ab19ddf3dcf9a9dc81e478a7deed2baf3ee17086f0e619dc66724fc258` |

L'audit standalone è **PASS**: provenienza dal commit pulito, CRT statica, metadati e manifest, **95 voci** portable e **243 componenti Windows** con avvisi. Report locali: `.tools/standalone-070-audit.json` e `.tools/standalone-070-payload-audit.json`. I sorgenti includono **431 crate vendorizzati**, **22801 file verificati** e **196 file tracciati** corrispondenti al commit, ammettendo soltanto la normalizzazione esatta CRLF → LF per **169 file**. Il controllo offline conta **707 file di licenza/avviso**, includendo `PATENTS`; l'auditor pubblico ne conta **706** con il precedente filtro dei nomi. Veri `cargo check --locked --offline --all-targets` sono passati per app (**66.98 s**) e DLL (**3.91 s**), con Cargo home inizialmente vuota, target isolato, CRT statica e Rust **1.99.1** identico alla provenienza. Non si tratta di una build completa con linking. Dopo il controllo il registro Cargo contiene soltanto `CACHEDIR.TAG`, senza dipendenze scaricate; hash dell'archivio invariato. Report: `.tools/source-offline-070-9e939ce0cc78475b84b6285146e10fda/offline-check.json`.

Il MSIX `ConvertHere-1.0.6.0-x64-store.msix` misura **7437834 byte**, SHA-256 `84f2e3561e73f425393312139629edab8d149747380f369580bbe3babbed52d1`. L'audit `.tools/store-070-audit.json` verifica **57 elementi payload**, **59 voci ZIP**, identità, versione, hash, licenze e provenienza pulita dal medesimo commit. Il payload contiene GUI e DLL Explorer; la CLI è nella distribuzione GitHub. Il pacchetto di upload è unsigned; nessuna installazione o modifica dei certificati è stata effettuata dall'audit.

La **Submission 3**, ID **1152921505702351165** del prodotto **9NPKPPCBNBM8**, è stata inviata il **9 novembre 2026 alle 19:45 UTC**. Il portale mostra **Update in certification / Product update: In certification**: **Submission Complete**, **Pre-processing In progress**, **Certification Not started**, **Publishing Not started**. È confermata la pubblicazione automatica dopo il superamento della certificazione; prezzo gratuito e 240 mercati restano invariati. Le schede IT/EN sono state salvate e rilette con **20 funzionalità e quattro schermate ciascuna**; privacy fissata alla 0.7.0 riletta, note di test e motivazione `runFullTrust` di **457 caratteri** salvate. Le [istruzioni e prove della 0.7.0](store-testing-0.7.0.md) distinguono il contenuto verificato del pacchetto dai test ancora non eseguiti. Prove locali: `.tools/store-submission-070-final-state.txt` e `.tools/store-submission-070-final.jpg`; il sondaggio facoltativo è stato chiuso senza inviarlo prima della cattura finale. Nessun artefatto, tag o report precedente è modificato.

## Storico 0.6.0 / Store 1.0.5.0 non caricato

Il record seguente precede l'aggiornamento della stessa Submission 3 con la 0.7.0 descritto sopra.

La [release GitHub 0.6.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.6.0) è pubblicata dal **9 novembre 2026, 18:28:13 UTC**, senza stato draft, dal commit `9b959405e64e62f1f05f49a821bb9444747b3620`. [Windows checks 42672724894](https://github.com/salvatorecorsi/convert-here/actions/runs/42672724894) è **SUCCESS**. Dimensioni e digest dei quattro asset GitHub corrispondono esattamente ai file locali verificati; il report di consegna è `.tools/delivery-060.json`.

| Artefatto | Byte | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.6.0-windows-x64-setup.exe` | 12248695 | `f0ae3e98f85c0cad3483b9e3fd69107fbe65b24c39648f1087a053abaf86e1b0` |
| `ConvertHere-0.6.0-windows-x64-portable.zip` | 16238290 | `e76fc9849515d8c40868dfc49f0148224741f6f2590e993ae38d230df3b48fda` |
| `ConvertHere-0.6.0-source.zip` | 84543591 | `7e0d71a2d9cf5ce8f0df07bd6478243831d651ab7b228ea0119b9e5821336fa0` |
| `SHA256SUMS.txt` | 313 | `d698c7c4ea840da6a5316b02fa5e88dc8daab73f1324c75a50d6e3f2dfa4a579` |

I sorgenti estratti contengono **431 crate vendorizzati**, **22801 file verificati** e **706 file di licenza**. Veri `cargo check --offline --locked --all-targets` con cache Cargo inizialmente vuota sono passati per app (**70.45 s**) e DLL (**3.90 s**); non sono una build completa con linking: `.tools/source-offline-060-9631ad4ff73941e399d0f4931a24bcdd/offline-check.json`.

L'audit del MSIX **1.0.5.0** è superato: **7123294 byte**, SHA-256 `185212fc3118f0d499763e415fcad33ce1eec1999bce5c678de892a8a27d9e5e`, **57 elementi payload**, **59 voci ZIP**, **243 componenti Windows** con avvisi; report `.tools/store-060-audit.json`. Il codec integrato `libwebp-sys 0.14.4` conserva `COPYING` e `PATENTS` originali della libreria C e il testo MIT dichiarato dal wrapper con provenienza esplicita. Questo inventario è distinto dai 236 componenti storici della 0.4.0.

Il pacchetto Store **non è stato caricato o inviato**: il selettore file automatico non ha completato l'operazione. La **Submission 3**, ID **1152921505702351165**, resta **in bozza**. Scheda IT salvata e riletta **Updated**; Save eseguito per EN, senza rilettura successiva, quindi persistenza EN non attestata. Nessuna certificazione o disponibilità Store della 0.6.0 è dichiarata.

Le prove locali sul ramo editor, **prima dell'integrazione con la 0.5.0 bilingue**, hanno superato formattazione, Clippy, **265 test app** e **12 test DLL**, con un test upstream in cache ignorato. La CLI conserva la ricetta del profilo dopo aggiornamento/eliminazione: risultato **220068 byte, 640 × 498 pixel**, originale invariato. La GUI ha salvato esattamente **297678 byte** per un WebP **900 × 700**, qualità **44**, entro il limite di **300000 byte**, e disabilitato il salvataggio con limite **100 byte**. Sono provati ritaglio, scala e tela bianca. Il profilo **«Quadrato per il sito»**, rinominato e scelto come predefinito, è ricaricato al riavvio con la stessa tela da **1024 pixel** e risultato di **47454 byte**; verificato anche il riavvolgimento del controllo **Trasparente** senza area nera. Un TXT UTF-8 ha prodotto un PDF A5 di **4 pagine**, **176031 byte**, con font **18 pt**, misure **419.528 × 595.276 pt**, accenti e **€** corretti nell'estrazione; rendering Poppler ispezionato. Sostituire il contenuto con testo digitato rigenera **una pagina A5 di 173633 byte**, senza salvare quel testo nei profili.

La [checklist dettagliata](store-testing-0.6.0.md) distingue le prove del ramo editor dagli audit e dal CI del commit finale. Non si dichiara eseguito il WACK, né installato o approvato il MSIX 1.0.5.0, né registrato realmente il nuovo menu Explorer. I record successivi conservano data, versioni, checksum e stato del portale osservati per le release precedenti; tag, archivi e report immutabili restano invariati.

## Stato registrato 0.4.0 / Store 1.0.3.0 — storico al 4 novembre 2026

La [release GitHub 0.4.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.4.0) e pubblicata dal **4 novembre 2026, 18:56:51 UTC**, commit `b4d5d0866a59b821125e35053566c454dd598471`, senza flag draft o prerelease. [Windows checks 41890745922](https://github.com/salvatorecorsi/convert-here/actions/runs/41890745922) e **SUCCESS**, inclusi i nuovi controlli dei sorgenti e del wrapper motori. I quattro digest remoti corrispondono agli artefatti locali verificati.

| Artefatto | Byte | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.4.0-windows-x64-setup.exe` | 11004425 | `eb413012f5e8f8708b3ca44645b8b1afe85119e70023b93059765963a4a06076` |
| `ConvertHere-0.4.0-windows-x64-portable.zip` | 13579654 | `b5d73c9ba708bee7d6f6bf80caef52e7977d334a9482251b6d918e382baa5609` |
| `ConvertHere-0.4.0-source.zip` | 82525036 | `6b58cab25c466d967aa45c25c63121978456d8b801ca5840f246f3f15658530c` |
| `SHA256SUMS.txt` | 313 | `633771e2b0ac2223d1ec9e402e6af6e03c5878196add8bfa7b4fbfd9bd04353d` |

L'audit pubblico confronta tutti i **144 file tracciati** con il commit, entrambe le lockfile e la provenienza dei binari. Il pacchetto sorgente contiene **429 crate vendorizzati**, **22578 file verificati** e **703 file di licenza upstream**; l'inventario Windows resta di **236 componenti**. Una verifica separata ha eseguito realmente `cargo check --locked --offline --all-targets` per app e DLL dai sorgenti estratti, con Cargo home inizialmente vuota, target isolato e Rust **1.99.1** identico alla provenienza. Entrambi i comandi sono passati; il registro Cargo finale contiene solo `CACHEDIR.TAG`. L'audit del pacchetto e questa prova di compilazione hanno report separati e non riscrivono i precedenti.

Il MSIX `ConvertHere-1.0.3.0-x64-store.msix`, **6348284 byte**, SHA-256 `2b042f23f96e5d12d2cd1ca104bf6c5252214b83df4718a8eed11e1a2e4fbbd9`, deriva dal medesimo commit pulito. Sono verificati **57 elementi payload**, **59 voci ZIP**, licenze, manifest e import PE. Il report schema 2 dichiara installer nativo presente, nessun motore e nessuno script downloader inclusi. Un successivo smoke test carica la DLL estratta dal MSIX e verifica COM, preferenze e selezioni reali senza registrarla nel sistema. Il nuovo MSIX non e stato installato e il WACK non e stato eseguito.

Il pacchetto e caricato, salvato e **Validated**. La **Submission 2**, ID **1152921505702316905**, e stata inviata il **4 novembre 2026** e il portale mostra **Update in certification / Product update: In certification**. Timeline: **Submission Complete**, **Pre-processing In progress**, **Certification Not started**, **Publishing Not started**. La 0.4.0 non e ancora approvata o pubblicata nello Store; e confermata la pubblicazione automatica dopo il superamento della certificazione. Schede IT/EN rilette con 17 funzionalita, quattro catture e URL GPL 0.4.0 ciascuna; privacy 0.4.0 riletta nelle Properties. Le note di test di **3446 caratteri** e la motivazione `runFullTrust` di **433 caratteri** persistono. Prezzo gratuito, 240 mercati, telefono e indirizzo vuoti restano confermati. [Privacy](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/docs/privacy.md) e [licenza](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/LICENSE) rispondono HTTP 200 con contenuto verificato. Prova locale dello stato: `.tools/store-submission-040-final-state.txt`; nessun tag, artefatto o report precedente e riscritto.

La nuova versione offre uscite visibili configurabili e download esplicito di FFmpeg/Poppler in Impostazioni → Motori. Le 93 estensioni in ingresso e le 44 destinazioni del motore restano supportate, con otto uscite mostrate per default. Il download HTTPS verifica lo SHA-256 e conserva licenze e documentazione nella cartella dei motori dell'utente.

Il codice originale adotta GPL-3.0-only, Copyright (C) 2026 Salvatore Corsi, con testo canonico completo in `LICENSE`; licenze di dipendenze, font e motori restano separate. I pacchetti contengono licenza e avvisi; il pacchetto sorgente pubblico include dipendenze vendorizzate e istruzioni offline.

Le prove native e le fixture della 0.4.0 seguenti sono distinte dallo storico delle precedenti revisioni. Tag, binari e report della release restano immutati dopo questo aggiornamento documentale.

Le fixture isolate hanno superato **21 controlli del pacchetto sorgente** e **11 controlli del wrapper motori**, sia con PowerShell 7 sia con Windows PowerShell 5.1. Verificano commit e lockfile esatti, vendor offline per entrambi i workspace, provenienza dei binari, conservazione delle licenze dei terzi, argomenti CLI, percorsi con spazi e propagazione degli errori. Usano repository e CLI fittizi: non scaricano o eseguono motori e non compilano Rust. Windows checks include ora entrambi gli script.

La verifica nativa dell'interfaccia 0.4.0 ha completato due conversioni PNG in JPEG senza errori, controllato le cinque uscite compatibili mostrate per default alle immagini, l'abilitazione FLAC salvata in JSON e il ripristino delle otto uscite iniziali. Il dialogo di download mostra fonti, versioni, dimensioni e nuova cartella personale; Annulla chiude la conferma senza avviare il download. Un fixture FLAC inerte di 53 byte e stato accettato e il selettore mostra esattamente MP3 e WAV. Sono stati verificati percorso dei motori e testo GPL integrale nel dialogo licenza; nessun motore e stato installato dalla GUI. I sei controlli Rust sono superati: 225 test dell'app, il fixture separato degli archivi upstream in cache e 10 test DLL. Queste prove non attestano un'installazione del nuovo MSIX o un nuovo invio Store. L'inventario Windows resta di **236 componenti**, con le rispettive licenze e avvisi.

## Stato precedente verificato 0.3.2 / Store 1.0.2.0

La [release GitHub 0.3.2](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.2) e pubblicata come latest dal **4 novembre 2026, 16:25:31 UTC**, commit `8f7532bf9d16f80ef65b5229dba93929e11c1d82`. Sono superati formattazione Rust, Clippy rigoroso e test `--all-targets` per app e DLL, oltre alle fixture PowerShell 7 e 5.1. Il job `verify` del [run Windows checks 41866571328](https://github.com/salvatorecorsi/convert-here/actions/runs/41866571328) e **SUCCESS**; questo riferimento attesta il job, non uno stato complessivo del workflow non ancora registrato.

L'audit degli artefatti pubblici e superato; i digest GitHub corrispondono ai file verificati. L'inventario Windows contiene **236 avvisi di dipendenze**.

| Artefatto | Byte | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.3.2-windows-x64-setup.exe` | 10163126 | `dec0ede6e386ad456788af2ec6a6ed8800e337ff1247efffa09fad1fb19a2381` |
| `ConvertHere-0.3.2-windows-x64-portable.zip` | 12531010 | `f3da01da13f49145539207c278e6ba9f97591aa6ebbb233e1dabf3a1eea6a097` |
| `SHA256SUMS.txt` | 217 | `a7292610164549002626c5a57cfbfcac1d00c6c9b3cbcd9d6f193bc0cc720fcf` |

Il MSIX `ConvertHere-1.0.2.0-x64-store.msix` e di **6165973 byte**, SHA-256 `c6d655b58156e4a14e13653a0f5115f3bffc2ba5ff086dbe435bc34440a053b7`, generato dal medesimo commit con working tree pulita. L'audit Store verifica **236 avvisi**, **56 elementi payload**, **58 voci ZIP**, hash, manifest/layout, architettura x64, compatibilita CRT statica e ABI COM reale senza registrazione. I report immutabili descrivono lo stato al momento dell'audit, prima dell'upload.

Il nome **Salvatore Corsi** e approvato: Verification Summary **Authorized**, account **Active**, Public Info Publisher Name **Salvatore Corsi**. Nel portale resta soltanto il nuovo 1.0.2.0 **Validated** dopo il salvataggio e il ricaricamento; il precedente 1.0.1.0 e stato sostituito automaticamente. La privacy fissata alla 0.3.2 e pubblica verificata e salvata; le istruzioni di test di **1977 caratteri** riportano 0.3.2 / 1.0.2.0 e persistono al ricaricamento. Telefono e indirizzo restano vuoti. Le tre schermate native 0.3.1 sono riutilizzate perche l'interfaccia e identica.

La **Submission 1**, ID **1152921505702314630** del prodotto **9NPKPPCBNBM8**, ha completato il percorso di certificazione: la 0.3.2 / Store 1.0.2.0 risulta ora **In Microsoft Store**. E stata osservata l'installazione del pacchetto completo **1.0.2.0**, con **SignatureKind: Store**. La precedente timeline di certificazione e storica; i report immutabili conservano lo stato precedente all'upload. Il prodotto e gratuito nei 240 mercati selezionati. Il WACK non e stato eseguito; questa installazione non attesta un collaudo del futuro MSIX 1.0.3.0.

## Verifiche della 0.3.1 prima del ritiro della certificazione

Convert Here e gratuito, con conversioni di formato e OCR locali, senza abbonamenti, acquisti in-app o costi a consumo. Nessun pagamento richiesto per continuare a usarlo. Restano 44 destinazioni e 93 estensioni riconosciute, con motori locali opzionali. La [release GitHub 0.3.1](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.1) e pubblicata come latest dal commit `884cb2c610a9197d8b4853f0fa741dbe9035b5ed`; [Windows checks 41852511198](https://github.com/salvatorecorsi/convert-here/actions/runs/41852511198) e **SUCCESS**. L'inventario degli avvisi delle dipendenze Windows comprende **236 componenti**, senza avvisi mancanti.

Il MSIX `ConvertHere-1.0.1.0-x64-store.msix` e di **6165956 byte**, SHA-256 `e5f63b0c59983ebde7209a1a2813a63c4df8941c1dc7d77f813bea91f5dfdfdd`, generato dal medesimo commit con working tree pulita. Partner Center mostra **Packages: Complete** e file **Validated**, Desktop x64, minimo Windows `10.0.22000.0`. Il salvataggio ha sostituito automaticamente il precedente 1.0.0.0.

Le schede IT (59) ed EN (4) sono **Complete**, ciascuna con tre catture 0.3.1, didascalie, logo 300 x 300, 15 funzionalita e sette parole chiave. Properties e Submission options sono **Complete**: dichiarazione AI generative non selezionata, [privacy fissata alla 0.3.1](https://github.com/salvatorecorsi/convert-here/blob/v0.3.1/docs/privacy.md), istruzioni di test di 1977 caratteri e motivazione `runFullTrust` di 424 caratteri salvate con persistenza verificata. Prezzo gratuito, 240 mercati e pubblicazione appena approvata restano confermati.

La QA nativa 0.3.1 ha completato due PNG in JPEG con zero errori e PDF Merge da tre sorgenti per cinque pagine, con riordino tramite trascinamento e risultato verificato tramite `pdfinfo` e `pdftotext`. Le [catture e il registro](store-assets/0.3.1/README.md) documentano la sessione isolata. **Il MSIX Store non e stato installato e il WACK non e stato eseguito**; questi esiti non vengono dedotti dalle prove native o dalla validazione del portale.

**Age ratings e Complete**, con classificazioni IARC/Microsoft/PEGI 3+, dopo accettazione e salvataggio su consenso esplicito del proprietario. La **Submission 1**, ID **1152921505702314630** del prodotto **9NPKPPCBNBM8**, e stata inviata il **4 novembre 2026** e risulta **In certification**. Timeline verificata: **Submission Complete**, **Pre-processing In progress**, **Certification Not started**, **Publishing Not started**. Il prodotto resta gratuito in tutti i 240 mercati selezionati, con pubblicazione automatica appena supera la certificazione. **Non e ancora pubblicato nello Store.** I risultati storici seguenti appartengono soltanto alle revisioni e agli artefatti indicati.

## Verifiche della 0.3.0 (storico)

Il modello espone 44 destinazioni e 93 estensioni riconosciute. Sul codice dell'interfaccia integrata sono passati tutti i sei controlli richiesti: formattazione, Clippy con `--all-targets -- -D warnings` e test per app e DLL. I test sono **226 per l'app e 5 per la DLL**, senza test ignorati. Sono compilati entrambi gli eseguibili release; una conversione reale dalla CLI release verifica anche l'alias `jpeg`.

Le prove PDF (Bulk) e PDF (Merge) verificano la selezione multipla compatibile, la distinzione fra coda per file ed editor interattivo, i GUID storici e gli argomenti di avvio dei comandi Explorer. I test dell'editor simulano trascinamenti su piu frame, frecce da tastiera, intercalazione fra sorgenti e visualizzazione di 10.000 pagine. I test del motore controllano snapshot privati, ordine completo senza duplicati, immagini e PDF multipagina mescolati, documenti/HTML con LibreOffice, collisioni dei nomi, cancellazione e rifiuto di cifratura, firme e campi modulo. Nessun PDF intermedio viene pubblicato nella destinazione.

La suite comprende Excel/CSV, HTML in PDF, ricostruzione del testo PDF in Word/HTML, OCR locale, HEIC/AVIF e operazioni PDF. Le prove locali usano FFmpeg/FFprobe e Poppler del progetto, LibreOffice 26.8.2, Tesseract con i modelli ita/eng e libheif estratti nelle cartelle di validazione, senza installazione nel sistema. Le copie di validazione non fanno parte dei pacchetti pubblici; FFmpeg e Poppler restano esclusi dal pacchetto Store.

Le prove verificano CSV UTF-8 con virgole, punti e virgola, tab e pipe, campi quotati, Unicode, righe multiple e identificatori con zeri iniziali; formule mantenute come testo durante l'importazione; primo foglio Excel; HTML/RTF in PDF e PDF → DOCX → PDF → TXT con testo conservato. Senza OCR, i PDF privi di testo incorporato vengono rifiutati con l'indicazione di attivarlo. Con OCR locale attivo, Tesseract riconosce le scansioni e crea TXT/DOCX/HTML; il default dei PDF resta senza OCR. DOCX e HTML sono ricostruzioni del testo, senza immagini o impaginazione originale. Un profilo temporaneo LibreOffice separa ogni conversione e disabilita macro/OLE e aggiornamento dei collegamenti Office; HTML puo comunque leggere le proprie risorse collegate, come dichiarato in interfaccia e privacy.

I fixture media verificano 78 abbinamenti dei sei nuovi formati immagine con i 13 formati immagine nativi, profondita colore, trasparenza e valori HDR superiori a 1. Verificano inoltre codec, decodifica completa e tracce audio dei nuovi contenitori, equivalenza PCM per CAF/AU e tutte le frequenze proposte per Opus/WebM, AC3, WMA/WMV e FLV. Frequenze incompatibili vengono rifiutate prima di creare risultati o prenotazioni.

Sono presenti fixture per roundtrip HEIC/AVIF con la coppia libheif, lingue e riconoscimento OCR, ordine delle pagine e compressione/divisione/unione PDF. Le verifiche dei PDF coprono anche conteggio con `pdfinfo` e fallback nativo quando manca, esportazione diretta e in coda dei documenti con sola password proprietario, errore italiano per password di apertura e avanzamento del rendering a blocchi. Compressione, divisione e unione continuano a rifiutare i PDF cifrati o firmati.

I test della coda coprono isolamento dei record illeggibili, recupero dei lavori, limiti di cronologia 7/30 giorni e 300 record terminali, lettura in background, pubblicazione senza una seconda copia e preservazione dei file sostituiti. Le prove dei segnaposto coprono aggiornamenti limitati durante l'avanzamento e rimozione dei soli collegamenti posseduti dai lavori falliti, preservando record, originali e risultati. Ulteriori fixture verificano qualita VP9/WebM e diagnostica per i video privi di traccia audio.


La verifica completa comprende test e Clippy con `--all-targets` per app e DLL, formattazione Rust, sintassi PowerShell e fixture dell'installer con PowerShell 5.1 e 7. Il workflow Windows checks prepara FFmpeg e Poppler con archivi e hash fissati prima dei test, compila la DLL Explorer ed esegue smoke test COM e parita delle estensioni. LibreOffice, Tesseract e libheif restano motori aggiuntivi delle prove locali. La whitelist e confrontata fra core, DLL, script di registrazione e installer; i GUID dei vecchi comandi restano stabili. Queste verifiche non registrano il menu installato e non costituiscono una certificazione Store.

La verifica finale PowerShell ha superato il parser di tutti i 21 script, i 6 casi della policy di disinstallazione e i 7 roundtrip degli apici sia con PowerShell 5.1 sia con PowerShell 7. Sono passati la parita delle 93 estensioni con due fixture negative, i 25 controlli del menu moderno e i 44 controlli Store, inclusa la validazione semantica MakeAppx. I 26 controlli del packaging Convert Here verificano nomi, struttura portable, checksum e compatibilita degli aggiornamenti in entrambe le versioni PowerShell. La DLL release supera lo smoke COM reale senza registrazione nel sistema; nessun installer e stato eseguito.

I test dei probe PDF usano processi fixture Rust nativi: l'avvio di PowerShell sul runner CI non consuma piu il budget del probe. La cancellazione viene misurata dopo aver verificato vivo un discendente, del quale si trattiene l'handle fino alla verifica della terminazione. Limiti, timeout e comportamento del motore distribuito restano invariati.

Il restyle mantiene egui con superfici che simulano l'aspetto Mica: il bonus Mica nativo non e applicato. Sono state verificate a schermo la finestra principale, le opzioni, la pagina Strumenti, i temi chiaro/scuro e una conversione WebM reale che prosegue dopo un errore su un altro file. Le [catture native](store-assets/README.md) usano preferenze e coda isolate; non attestano la certificazione o l'installazione MSIX del candidato finale.

## Pacchetti GitHub Convert Here 0.3.0 (storico)

La build 0.3.0 di app, CLI e DLL Explorer e stata completata, compresa la validazione MakeAppx dell'identita locale unsigned. La DLL release supera lo smoke COM e la cattura degli avvii Bulk/Merge con helper isolato, senza registrazione. La CLI release unisce PNG, PDF multipagina e PNG in un solo PDF di quattro pagine: `pdfinfo` e `pdftoppm` verificano numero e ordine, un secondo salvataggio conserva il primo risultato con un nome distinto e gli hash degli originali restano invariati. Non sono state usate la coda o le preferenze personali.

Una prova della finestra nativa, con preferenze e coda isolate, prepara cinque pagine da due immagini e un PDF di tre pagine. Trascinamenti in entrambe le direzioni intercalano immagini e pagine PDF; frecce e Alt+freccia cambiano lo stesso ordine. Il PDF unico salvato contiene cinque pagine nell'ordine scelto, verificato con `pdfinfo` e `pdftotext`. I test di regressione coprono anche pressione e primo movimento ricevuti nello stesso frame, senza agganciare per errore la maniglia sotto il puntatore finale.

## Candidato Microsoft Store 1.0.0.0 (storico)

Il candidato `ConvertHere-1.0.0.0-x64-store.msix`, applicazione 0.3.0, deriva dal commit `d6ad0171dd002d01136e58b511f5e9e5e6dbfffb` con working tree pulita al momento della build. SHA-256: `e130564d34e88b3e9f60ba6de76b06536adaa109957e1673a0043886ad2fe1de`. Il report attesta MSIX completo x64, minimo Windows `10.0.22000.0`, validazione MakeAppx superata senza `/nv`, nessun motore o downloader incluso e nessuna installazione o modifica agli archivi dei certificati.

La verifica diretta in Partner Center del prodotto **9NPKPPCBNBM8**, submission **1152921505702314630**, mostra **Packages: Complete** e il file **Validated**, con Desktop selezionato. Sono **Complete** entrambe le schede italiana e inglese, ciascuna con due PNG della 0.3.0, didascalie e logo 300 x 300; anche Pricing, Properties e Submission options sono **Complete**. Sono salvate disponibilita gratuita in tutti i 240 mercati, pubblicazione appena approvata, motivazione `runFullTrust` e istruzioni per il certificatore.

Il questionario IARC e rimasto **In Progress**, con anteprima 3+ non accettata. **La submission non e stata inviata per certificazione.** Il candidato 1.0.0.0 non e stato installato e non ha un report WACK. Le prove di preview e degli eseguibili separati non ne attestano installazione, menu registrato o disinstallazione. I report di build immutabili precedono l'upload; i loro campi di stato non vengono riscritti a posteriori. Il portale ha poi sostituito questo pacchetto con il nuovo 1.0.1.0; lo stato operativo e nella [procedura di submission](store-submission.md).

## Pacchetti GitHub Convert Here 0.2.0 (storico)

Versione Cargo e metadati nativi sono aggiornati alla 0.2.0; i due eseguibili mostrano ProductName `Convert Here`. Sono stati generati `ConvertHere-0.2.0-windows-x64-setup.exe`, `ConvertHere-0.2.0-windows-x64-portable.zip` e `SHA256SUMS.txt`. Il packaging pubblico applica la lista esplicita dei binari consentiti e include gli avvisi delle dipendenze, senza motori esterni o certificati. Il setup pubblico offre il menu classico; l'identita MSIX unsigned usata nella build locale non e distribuita. AppId, CLSID, binari interni e percorsi dei dati restano compatibili con gli aggiornamenti. Il setup e stato compilato e ispezionato, senza installarlo sul PC di lavoro.


## Pacchetti e certificazione

Il workflow [Windows checks](https://github.com/salvatorecorsi/convert-here/actions/workflows/windows-ci.yml) ripete i controlli sui sorgenti pubblicati. Il workflow [Microsoft Store package](https://github.com/salvatorecorsi/convert-here/actions/workflows/microsoft-store.yml) produce separatamente il MSIX unsigned e il report con commit, stato della working tree e checksum. Verificare il run relativo al commit che si intende distribuire.

La preview completa 1.0.2.0 e stata installata localmente con una copia firmata esclusivamente per il test, dopo un aggiornamento dalla preview 1.0.1.0. Al termine e stata rimossa ed e stata ripristinata l'integrazione sparse dell'installazione locale. Quella prova non certifica il candidato attuale.

Il report WACK locale della precedente preview **1.0.1.0**, eseguito con kit 10.0.26100.8249 su Windows 10.0.26200.0, ha risultato complessivo **WARNING**: 21 test PASS, due FAIL opzionali e un WARNING obbligatorio. I rilievi riguardavano il numero di applicazioni dichiarate, il rilevamento di eseguibili/avvio processi e la verifica DPI. Il pacchetto successivo dichiara una sola applicazione, esclude la CLI e usa API native per aprire cartelle e siti. L'uso legittimo di processi per i motori esterni resta parte del prodotto.

**Il precedente report WACK riguarda una preview distinta e non viene dichiarato superato.** Microsoft considera il WACK un controllo locale deprecato e facoltativo: non e stato eseguito sul nuovo candidato e il suo esito non garantirebbe l'approvazione Store. Installazione/aggiornamento/disinstallazione, menu registrato e motori opzionali del MSIX finale restano senza un collaudo su ambiente pulito. [Validazione locale MSIX](https://learn.microsoft.com/en-us/windows/msix/package/packaging-uwp-apps#validate-your-app-package-locally).

Il precedente pacchetto 1.0.1.0 è stato sostituito dal 1.0.2.0. Il nuovo nome **Salvatore Corsi** e approvato, il pacchetto 1.0.2.0 e Validated e la submission e **In certification** dal 4 novembre 2026; la pubblicazione Store non e ancora iniziata. Vedi la [procedura di submission](store-submission.md).

## 0.7.1 / Store 1.0.7.0 — candidata del 9 novembre 2026

La modifica del menu packaged usa `0000ConvertHere`, mantenendo identità, CLSID e capability. I controlli mirati passano: branding 37, Store 51 più MakeAppx semantico, sparse 26, installer 7 e cleanup 6, sorgenti corrispondenti 21, parità delle 93 estensioni con due fixture incoerenti respinte. Il nuovo probe COM/menu classico è passato sul pacchetto **installato 1.0.6.0** e Windows 11 build 26200: questo dato non collauda il candidato 1.0.7.0 né Windows 10. Il gate richiede `-ExpectedVersion 1.0.7.0` dopo l'installazione. Non è disponibile un host Windows 10 reale.

Le due [nuove catture](store-assets/0.7.1/README.md) sono PNG 2562 × 1430, senza puntatore. Provengono dalla UI debug 0.7.0 invariata in 0.7.1, zoom nativo 150%, senza ingrandimento successivo; la CLI ha convertito i JPEG nativi in PNG. I 20 screenshot precedenti con puntatore/alone sono ritirati dal ramo corrente; gli archivi rimandano al tag storico v0.7.0.

Build, audit finali, push, release e stato della nuova submission saranno registrati dopo l'esecuzione; non trasferire checksum o prove della 0.7.0 al candidato.