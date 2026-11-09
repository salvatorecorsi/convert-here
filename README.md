# Convert Here

Sviluppato da **Salvatore Corsi**: versione Rust per Windows di [Final Converter](https://github.com/salvatorecorsi/final-converter). Conversioni di formato locali, OCR sul PC, selezioni multiple e integrazione con Esplora file. Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. Nessun pagamento richiesto per continuare a usarlo.

Il nome pubblico e **Convert Here**, riservato anche in Microsoft Partner Center; il repository canonico e [salvatorecorsi/convert-here](https://github.com/salvatorecorsi/convert-here). Per compatibilita restano invariati i nomi tecnici degli eseguibili e dei crate, le cartelle dati e gli identificativi COM/MSIX. Le release precedenti possono ancora mostrare il nome The Converter. La **0.6.0 è candidata alla pubblicazione**: [novità in italiano](docs/release-notes-0.6.0.md) e [release notes in English](docs/release-notes-0.6.0-en.md). La release pubblica precedente è la [0.4.0](docs/release-notes-0.4.0.md).

## Preparazione Microsoft Store

La distribuzione Store usa un **MSIX completo** con il menu di Windows 11 dichiarato nel pacchetto. Editore pubblico e autore del codice: **Salvatore Corsi**. Il nome editore e approvato da Microsoft; il profilo risulta **Authorized** e l'account **Active**. L'account GitHub che ospita il sorgente resta indipendente dall'identita dell'editore Microsoft.

La build di anteprima e separata dalla release GitHub e non contiene certificati privati o motori esterni. Le immagini nei formati nativi e le immagini in PDF funzionano subito, come compressione, divisione e unione dei PDF. Audio/video richiedono FFmpeg; esportazione PDF in immagini o testo incorporato richiede Poppler; Office, Excel/CSV e HTML in PDF richiedono LibreOffice. HEIC/HEIF e AVIF richiedono libheif, mentre OCR richiede Tesseract con i modelli delle lingue e, per i PDF, anche Poppler. I motori si selezionano in **Impostazioni → Motori**. Nella 0.4.0 puoi anche richiedere il download di FFmpeg, Poppler oppure entrambi: prima di confermare vedi origine, versione, dimensioni, licenze e destinazione. Il download facoltativo richiede Internet; i file da convertire restano sul PC. Questi requisiti sono dichiarati anche all'inizio delle schede Store.

```powershell
.\scripts\build-store-package.ps1
```

La preview usa un'identita di sviluppo e **non va caricata nello Store**. Per il pacchetto definitivo occorrono i dati esatti dell'Identita prodotto in Partner Center, la versione Store e il nome editore effettivamente confermato nel portale. Il modello e in `packaging/store/identity.partner-center.example.json`; la procedura e in [docs/store-submission.md](docs/store-submission.md).

Convert Here e disponibile nel [Microsoft Store](https://apps.microsoft.com/detail/9NPKPPCBNBM8). Per la versione presente nel catalogo e lo stato dei nuovi aggiornamenti consulta la [procedura Store](docs/store-submission.md) e lo [stato delle verifiche](docs/store-validation.md).

Gli esiti dei test e i controlli Store ancora da completare sono raccolti in [store-validation.md](docs/store-validation.md).

Su GitHub Actions, **Windows checks** verifica codice e fixture su push e pull request. **Microsoft Store package** si avvia manualmente: senza dati Partner Center produce una preview; con `for_submission` e l'identita reale produce il candidato unsigned da caricare. Il pacchetto e il materiale per la scheda si scaricano dagli artefatti del workflow. Le credenziali Microsoft e il certificato locale non sono richiesti dalla pipeline.

## Scarica e installa

Scarica la [release 0.4.0 per Windows x64](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.4.0) e avvia `ConvertHere-0.4.0-windows-x64-setup.exe`. Il setup richiede Windows 10 build 19041 o successivo, x64. L'installer aggiunge il programma e il disinstaller per il tuo utente; la voce **Converti in** del menu contestuale e facoltativa e selezionata per default. Non richiede privilegi amministrativi.

Nella 0.4.0 il setup e leggero e non scarica motori. Apri **Impostazioni → Motori** per richiedere FFmpeg, Poppler o entrambi, poi conferma i dettagli del download. I due archivi richiedono circa 159 MB di rete e, durante download ed estrazione, circa 615 MB liberi. Le conversioni di immagini native e immagini in PDF funzionano anche senza questi motori.

Su Windows 11 il menu della distribuzione GitHub compare in **Mostra altre opzioni**. Gli artefatti pubblici non sono firmati Authenticode e non includono un'identita MSIX firmata attendibile per il menu moderno. Quel percorso resta separato ed e descritto in [windows-integration.md](docs/windows-integration.md).

Per rimuovere il programma usa **Impostazioni Windows → App → App installate → Convert Here → Disinstalla**, oppure il collegamento nel menu Start. Le preferenze e i file convertiti restano conservati. Se Windows mantiene caricata la DLL del menu, la pulizia viene completata al successivo accesso senza riavviare Esplora file durante il lavoro.

È disponibile anche `ConvertHere-0.4.0-windows-x64-portable.zip` nella [release 0.4.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.4.0): estrailo in una cartella stabile e, se desideri il menu, usa `scripts/install-context-menu.ps1 -Mode Classic -InstallDirectory <cartella>`. Nella 0.4.0 anche la versione portable offre il download dei motori da **Impostazioni → Motori**. Lo script `scripts/install-runtime-tools.ps1 -InstallDirectory <cartella>` resta un'alternativa manuale per chi usa la CLI e contatta i distributori soltanto quando lo avvii. Verifica gli artefatti con `SHA256SUMS.txt` della stessa release.

## Uso quotidiano

Dal tasto destro scegli **Converti in → MP3**, **WAV**, **PDF**, ecc. La conversione rapida entra in una coda condivisa, con le preferenze salvate, e **non apre la finestra delle opzioni**. Nella cartella di destinazione compare subito un segnaposto con il nome previsto e una clessidra blu. Il menu propone i formati compatibili con la selezione e abilitati nelle impostazioni.

Con piu file convertibili in PDF, **PDF (Bulk)** accoda una conversione per ogni file e mantiene risultati separati. **PDF (Merge)** apre invece l'editor delle singole pagine per creare un PDF unico. Sono compatibili immagini, PDF, documenti Word e altri documenti Office supportati, Excel/CSV e HTML; audio e video non hanno queste azioni.

Il segnaposto è un collegamento Windows temporaneo: Esplora file normalmente ne nasconde l'estensione `.lnk`. Un doppio clic apre solo lo stato del lavoro, con avanzamento e annullamento. A conversione conclusa lascia il posto al file effettivo; in caso di errore mostra una X rossa e permette di aprire i dettagli. La clessidra in Esplora file è un'icona statica. [Dettagli della coda](docs/queued-conversions.md).

**Converti in → Impostazioni personalizzate…** apre la finestra con coda, opzioni, avanzamento, annullamento e ultimo resoconto rapido. Puoi anche avviare `the-converter.exe` e trascinare i file nella finestra.

Per default i risultati sono accanto agli originali. Se il nome esiste già, viene aggiunto un numero: gli originali e i risultati precedenti non vengono sovrascritti. Le preferenze, inclusa la cartella di destinazione, restano salvate tra gli avvii.

## Editor e profili — candidata 0.6.0

La 0.6.0 aggiunge un editor locale per ritaglio, scala e tela con risoluzione, sfondo e posizione; esporta PNG, JPEG, WebP e PDF con anteprima e peso esatto del risultato. **Modifica…** apre l'editor dalla riga del file. Puoi impaginare immagini o testo semplice scritto nell'app/TXT UTF-8 su A4 o A5 e salvare fino a 32 profili, di cui 8 nel menu **Converti in**. Il limite di peso usa byte reali: 300 kB sono 300000 byte; ridurre anche le dimensioni richiede una scelta esplicita. Il salvataggio riusa il risultato preparato e conserva i file esistenti. È materiale della candidata: i download pubblici indicati sopra restano quelli della 0.4.0 finché la nuova release non viene pubblicata. [Guida all'editor e ai profili](docs/image-editor-profiles.md).

## Conversioni

Sono disponibili **44 formati di destinazione** e **93 estensioni di ingresso riconosciute**. Nella 0.4.0, in **Impostazioni** scegli quali uscite mostrare nel selettore della GUI e nel menu Explorer, raggruppate per audio, video, immagini e documenti. Il default e **MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4**; puoi abilitare le altre destinazioni, comprese TXT, DOCX, XLSX, HEIC e AVIF. Le scelte restano salvate e il menu le rilegge senza riavviare Explorer. Se non abiliti alcuna uscita, resta accessibile **Impostazioni personalizzate…**. Le 93 estensioni di ingresso e i 44 formati gestiti dal motore non cambiano: CLI, richieste `--quick` esplicite e lavori gia accodati non vengono bloccati dal filtro di visualizzazione. Le destinazioni effettive dipendono dalla categoria del file e dai codec dei motori selezionati.

| Sorgente | Destinazioni | Motore |
| --- | --- | --- |
| Audio: MP3, MPGA, WAV, FLAC, AAC, M4A/M4B, OGG/OGA, Opus, AIFF/AIFC, WMA, ALAC, APE, MKA, MP2, WebA, AC3, AU, CAF, AMR, DTS, TTA, VOC, WV, TAK, W64, RA | MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A (AAC), WMA, AC3, AU, CAF (ALAC) | FFmpeg |
| Video: MP4, MKV, MOV, WebM, AVI, WMV, MPEG, M4V, TS/MTS/M2TS/M2T, 3GP/3G2, FLV, ASF, VOB, OGV, M2V, MXF, F4V, DivX, MPE, MPV | MP4, MKV, MOV, WebM, AVI, GIF animato, MPEG, WMV, FLV, TS, M4V, 3GP; estrazione audio nei dodici formati sopra | FFmpeg |
| Immagini native: PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, QOI, TGA, Farbfeld (.ff), OpenEXR, HDR, PNM/PAM/PBM/PGM/PPM | PNG, JPEG, WebP, BMP, TIFF, GIF, ICO, PDF, QOI, TGA, Farbfeld, OpenEXR, HDR, PNM; HEIC, AVIF; TXT con OCR | Rust per i formati nativi e PDF; libheif per creare HEIC/AVIF; Tesseract per TXT |
| HEIC/HEIF, AVIF | Destinazioni immagine sopra, PDF e TXT con OCR | libheif per la lettura; Tesseract per TXT |
| PDF | PNG/JPEG, una immagine per pagina; TXT, Word DOCX e HTML dal testo incorporato o con OCR | Poppler per immagini e testo incorporato; Tesseract + Poppler per OCR; DOCX e HTML ricostruiti dall'app |
| PDF | PDF compresso senza perdita, un PDF per pagina, unione di piu PDF | Rust, senza motori esterni |
| DOC/DOCX, ODT, RTF, TXT, PPT/PPTX, ODP | PDF | LibreOffice opzionale |
| XLS/XLSX/XLSM, ODS, CSV | CSV, Excel XLSX, PDF | LibreOffice opzionale |
| HTML/HTM | PDF | LibreOffice opzionale |

L'estensione determina la categoria proposta; il motore verifica il contenuto durante la conversione. File protetti, corrotti o con codec non disponibili producono un errore leggibile.

Impostazioni disponibili: preset leggero/bilanciato/alta qualità, bitrate, frequenza e canali audio, normalizzazione, H.264/H.265, qualità video, larghezza, FPS, taglio temporale, qualità JPEG, ridimensionamento immagini e DPI dei PDF. L'interfaccia mostra solo i controlli pertinenti. Per WebM si usa VP9/Opus, per AVI MPEG-4/MP3, per MPEG MPEG-2/MP2, per WMV WMV2/WMA2, per FLV FLV1/MP3, per 3GP MPEG-4/AAC, per TS H.264/AAC. M4V e un contenitore MP4 con audio; WAV, AIFF e AU sono PCM 16 bit, CAF usa ALAC senza perdita. Senza una ricetta pagina, i PDF creati da immagini sono A4 con proporzioni mantenute e sfondo bianco; l'editor della 0.6.0 permette anche A5, orientamento e margini personalizzati.

### Excel, CSV, Word e HTML

Excel e CSV vengono convertiti con un profilo LibreOffice temporaneo separato, che disabilita macro e aggiornamento dei collegamenti Office. L'esportazione CSV produce **il primo foglio**, in UTF-8 con delimitatore virgolette; conserva i valori visualizzati, senza formule, formattazione o altri fogli. Puoi scegliere **virgola, punto e virgola, tab o pipe**, con la stessa impostazione per importazione ed esportazione. La virgola e il default. L'importazione conserva come testo i campi quotati, incluse le cifre iniziali degli identificativi, e non valuta le formule. Le altre codifiche vanno prima convertite in UTF-8; il separatore non viene indovinato automaticamente.

PDF → Word/HTML ricostruisce **solo il testo**, conservando la separazione delle pagine in un singolo file modificabile. Di default usa il testo incorporato; puoi attivare OCR locale per riconoscerlo dalle scansioni. Immagini, tabelle, font e impaginazione originali non vengono ricostruiti. L'HTML esportato contiene il testo escapato e gli stili nel file, senza risorse esterne.

HTML → PDF usa il motore di LibreOffice per documenti HTML: supporta HTML/CSS di base e non esegue JavaScript. Non sostituisce la stampa di una pagina dinamica da browser; l'impaginazione puo cambiare. Durante l'apertura di HTML, il motore puo leggere risorse collegate locali o remote, ad esempio immagini e CSS.

### Immagini HEIC e AVIF

Puoi leggere HEIC/HEIF e AVIF, convertirli nei formati immagine nativi o in PDF, e creare HEIC o AVIF dalle immagini supportate. In **Impostazioni → Motori** seleziona `heif-dec.exe` per leggere e `heif-enc.exe` per creare questi formati. Sono strumenti di [libheif](https://github.com/strukturag/libheif), facoltativi e non inclusi: mantieni insieme eseguibili, DLL e plugin. La build deve includere i decoder e gli encoder HEVC/AV1 richiesti; se mancano, l'app mostra l'errore del motore. Per HEIC/AVIF puoi regolare larghezza e qualita. Dei contenitori con piu immagini viene usata la prima; sequenze e animazioni non vengono conservate.

Gli altri formati immagine sono nativi e non richiedono FFmpeg. HDR non conserva la trasparenza, che viene composta su bianco. OpenEXR e HDR possono conservare valori a virgola mobile; convertirli in formati ordinari puo ridurne precisione e gamma dinamica. JPEG XL, RAW fotografici e DDS non sono attualmente inclusi. I codec audio/video disponibili dipendono dalla build FFmpeg selezionata.

### OCR locale di immagini e scansioni PDF

Per un'immagine scegli **TXT · OCR locale**. Per un PDF scegli TXT, Word DOCX o HTML e abilita **OCR locale**: il riconoscimento e disattivato per default e, quando attivo, elabora tutte le pagine invece di estrarre il testo incorporato. Sono necessari [Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html) e i [modelli delle lingue](https://github.com/tesseract-ocr/tessdata_fast); i PDF richiedono anche `pdftoppm.exe` di Poppler. In **Impostazioni → Motori** indica `tesseract.exe` e conserva accanto la cartella `tessdata` con i modelli richiesti, per esempio `ita.traineddata` e `eng.traineddata`.

La lingua predefinita e `ita+eng`; puoi scegliere altri codici Tesseract installati. I DPI PDF determinano la risoluzione delle pagine da riconoscere: 300 DPI e un punto di partenza utile. Le immagini vengono orientate secondo EXIF e la trasparenza viene composta su bianco. Il riconoscimento e il testo restano sul PC. Il riconoscimento puo sbagliare e va riletto; non ricostruisce l'impaginazione e non aggiunge uno strato ricercabile al PDF originale. Se non viene riconosciuto testo, l'app segnala l'errore senza creare un risultato vuoto.

### Comprimi, dividi e unisci PDF

Per PDF gia esistenti, **PDF (Bulk)** comprime senza perdita i flussi senza ricampionare le immagini e crea `nome-compresso.pdf`; se la riscrittura sarebbe piu grande, salva una copia dei byte originali. Il risultato non supera le dimensioni del file letto e puo restare della stessa dimensione. **Dividi: una pagina per file** produce `nome-pagina-001.pdf`, `nome-pagina-002.pdf` e cosi via.

Per unire piu file compatibili, scegli **PDF (Merge)** dal menu contestuale oppure **PDF (Merge) · ordina pagine** nell'app e avvia **Ordina e unisci PDF**. L'app prepara le sorgenti e mostra tutte le pagine, identificate dal file e dal numero originale. Trascina la maniglia di una pagina, usa le frecce o **Alt+↑ / Alt+↓** per spostarla: puoi intercalare pagine di PDF multipagina, immagini e documenti diversi. L'ordine iniziale segue i file ricevuti da Esplora file; controllalo nell'editor. **Crea PDF unico** salva `primo-unito.pdf`, aggiungendo un numero se esiste gia. Tutte le pagine vengono incluse una volta, fino a 10.000.

La preparazione usa copie PDF temporanee in una cartella utente nascosta, senza PDF intermedi nella destinazione. Gli originali restano invariati; il salvataggio usa la versione preparata delle pagine. L'ordine e una sessione locale separata dalla coda rapida. Annullamento e chiusura eliminano gli intermedi; un arresto improvviso puo lasciare residui.

Compressione, divisione e unione di PDF esistenti, e l'unione di immagini native, non richiedono motori esterni. Per preparare Office/Excel/CSV/HTML serve LibreOffice; per leggere HEIC/AVIF serve libheif. Poppler non e necessario per questa unione. I PDF cifrati o firmati vengono rifiutati; unione e divisione rifiutano anche moduli interattivi. I contenuti delle pagine vengono mantenuti, mentre segnalibri, struttura accessibile/tag e metadati del documento non vengono trasferiti. La compressione mantiene il catalogo e i moduli; non e una rimozione dei metadati. Non e disponibile la modifica libera del contenuto delle pagine.

### Testo da immagini e PDF

La destinazione TXT e disponibile per le immagini tramite OCR locale e per i PDF tramite estrazione del testo incorporato o OCR locale. I motori richiesti si selezionano nella sezione Motori delle Impostazioni; il riconoscimento e il risultato restano sul PC. Dettagli sui dati locali e sui motori: [privacy](docs/privacy.md).

### Limiti espliciti

- OCR richiede Tesseract e i modelli delle lingue; per i PDF serve anche Poppler. E riconoscimento del testo, senza ricostruzione della grafica o del layout.
- Le immagini GIF animate e i TIFF multipagina sono letti dal primo fotogramma/pagina. La conversione da video a GIF conserva l'animazione.
- La conversione WebP ordinaria è senza perdita; l'editor della 0.6.0 offre anche WebP con qualità regolabile. La qualità JPEG non influisce sui formati senza perdita.
- PDF (Bulk) crea un PDF separato per ogni immagine; PDF (Merge) puo unire direttamente immagini e altri file PDF-compatibili, ordinando le singole pagine. Unione e divisione non trasferiscono segnalibri, tag o metadati e rifiutano moduli interattivi. Compressione, divisione e unione rifiutano PDF cifrati o firmati. L'esportazione PDF in PNG/JPEG accetta la sola password proprietario; una password di apertura richiede di sbloccare prima il documento.
- I documenti Office, i fogli di calcolo, i CSV e HTML in PDF richiedono LibreOffice installato o configurato; l'impaginazione dipende anche dai font disponibili.
- PDF in DOCX/HTML e una ricostruzione del testo, senza immagini o impaginazione originale. Excel in CSV esporta soltanto il primo foglio e perde formule e formattazione.
- La coda interattiva usa un formato comune ai file selezionati. Se le categorie non hanno una destinazione comune, vanno convertite in gruppi separati.
- Le conversioni rapide lavorano in coda in background. La finestra di stato si apre su richiesta dal segnaposto, mentre la finestra principale mostra la coda completa. Il resoconto conserva l'ultima esecuzione rapida; chiudere la finestra di stato non interrompe il lavoro.

## Eseguibile e menu contestuale

La distribuzione locale pronta è in `dist\ConvertHere\`. Mantieni questa cartella in una posizione stabile: contiene applicazione, CLI, DLL Explorer e gli eventuali motori locali di sviluppo. Gli artefatti pubblici escludono i motori.

Per installare il menu classico per il solo utente corrente, da PowerShell:

```powershell
.\dist\ConvertHere\scripts\install-context-menu.ps1 -Mode Classic
```

Per rimuoverlo:

```powershell
.\dist\ConvertHere\scripts\uninstall-context-menu.ps1 -Mode Classic
```

Su Windows 11 il menu classico compare in **Mostra altre opzioni**. Il progetto include anche la DLL `IExplorerCommand` e un pacchetto MSIX sparse per il **menu moderno di Windows 11**. Per registrarlo serve prima una firma attendibile del pacchetto. La build non registra DLL, non installa pacchetti e non modifica l'archivio dei certificati. La procedura completa e lo stato di verifica sono in [docs/windows-integration.md](docs/windows-integration.md).

Per le prove con un certificato di sviluppo, su Windows 11 build 26200 il trust nel solo archivio utente non e risultato sufficiente. La procedura Microsoft MSIX richiede `LocalMachine\TrustedPeople`: e un'importazione manuale del certificato pubblico, separata dal setup, con privilegi amministrativi, UAC e consenso esplicito perche estende la fiducia a tutti gli utenti del PC. Su questa configurazione sono stati verificati il comando nel menu moderno e la conversione WAV → MP3 nella stessa cartella, senza finestra delle opzioni. La prova usa un certificato di sviluppo; non e una nuova release pubblica. [Dettagli e stato della prova](docs/windows-integration.md#stato-di-verifica).

## Motori e preferenze

L'app cerca gli strumenti nella cartella `tools` accanto all'eseguibile, nella nuova cartella utente `%LOCALAPPDATA%\ConvertHere\tools`, nel `PATH`, nelle impostazioni salvate e nella cartella indicata dalla variabile `THE_CONVERTER_TOOLS`. Resta riconosciuta in lettura la precedente `%LOCALAPPDATA%\Corsivo\TheConverter\tools`. I percorsi si scelgono in **Impostazioni → Motori**. LibreOffice e Tesseract vengono cercati anche nelle rispettive posizioni standard di installazione.

Per Poppler, `pdftoppm.exe` esporta le pagine in immagini e prepara i PDF per OCR, `pdftotext.exe` estrae il testo incorporato e `pdfinfo.exe` conta le pagine. Selezionandone uno, l'app cerca gli altri nella stessa installazione, rispettando eventuali scelte esplicite separate. Se `pdfinfo.exe` non e disponibile, il conteggio usa il lettore PDF interno.

La distribuzione locale di sviluppo puo includere FFmpeg e Poppler. I pacchetti pubblici non contengono i motori; nella 0.4.0 il download facoltativo si avvia in **Impostazioni → Motori**, anche nell'edizione Store, e installa soltanto nella cartella dell'utente. Sono scaricati [FFmpeg 9.0.2 dalla distribuzione Gyan](https://github.com/GyanD/codexffmpeg/releases/tag/9.0.2) e [Poppler 26.09.0 per Windows](https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0), 158478032 byte complessivi (circa 159 MB). Le richieste HTTPS ai distributori e ai loro CDN espongono IP e normali dati HTTP, incluso User-Agent con versione dell'app; non caricano file personali. Lo SHA-256 e verificato prima dell'estrazione e il layout upstream conserva licenze e documentazione. Annullamento o errore tentano di rimuovere lo staging temporaneo; un arresto improvviso puo lasciare residui. LibreOffice, libheif, Tesseract e i modelli OCR si configurano da installazioni locali. Lo script PowerShell facoltativo resta un'alternativa manuale; il setup non avvia download e non aggiunge un collegamento Start per installare i motori. Le copie sotto `.tools/` per i test non fanno parte dei pacchetti pubblici; l'app non richiede Python, Node, WebView o Rust per essere eseguita.

Le preferenze si trovano nella cartella di configurazione utente `SalvatoreCorsi\TheConverter\config` sotto `%APPDATA%`: `preferences.json` e `report-last.json`. Il resoconto include percorsi locali ed errori e non viene trasmesso. La conversione interattiva mantiene i propri risultati nella coda corrente.

La coda rapida è salvata nei dati locali dell'app sotto `%LOCALAPPDATA%`. Un solo processo elabora i lavori, anche quando provengono da più comandi del menu contestuale. Dopo un'interruzione forzata, i lavori già in corso vengono segnalati come interrotti e i lavori ancora in attesa ripartono al successivo avvio dell'app o invio. I nomi sono prenotati prima di iniziare e rimangono quelli mostrati nei segnaposto. Per i PDF esportati in immagini viene prenotato un nome per pagina. Le cartelle temporanee nella destinazione sono nascoste.

La pulizia della cronologia, eseguita quando il processo di lavoro riprende o termina la coda, elimina i record completati o annullati dopo 7 giorni e quelli falliti dopo 30 giorni; tenta inoltre di conservare al massimo i 300 record terminali piu recenti. Originali, risultati e lavori attivi vengono preservati. I record illeggibili sono isolati e conservati, insieme agli eventuali file estranei. Per un lavoro fallito puoi usare **Rimuovi segnaposto**: elimina soltanto i collegamenti ancora appartenenti a quel lavoro, mantenendo record e risultati.

## Compilazione

Windows x64, Rust stable MSVC e Visual Studio C++ Build Tools/Windows SDK:

```powershell
.\scripts\fetch-tools.ps1
cargo build --release --locked
cargo build --release --locked --manifest-path .\shell-extension\Cargo.toml
.\scripts\build-release.ps1 -SkipBuild
```

`fetch-tools.ps1` scarica versioni fissate di FFmpeg e Poppler e ne verifica gli SHA-256 prima di copiarle; non installa nulla nel sistema. È opzionale se usi strumenti già disponibili. `scripts\build-release.ps1` esegue entrambe le compilazioni. `-CertificateThumbprint` permette di firmare eseguibili, DLL e MSIX con un certificato già disponibile, ricavandone il Publisher per entrambi i manifest. MakeAppx è necessario per generare il pacchetto identità; l'applicazione può essere usata anche senza.

Per preparare gli artefatti pubblici, aggiorna prima gli avvisi:

```powershell
.\release-support\collect-rust-licenses.ps1
```

Revisiona e salva in Git sorgenti, lockfile e avvisi, poi esegui la build e la preparazione dallo stesso commit con cartella di lavoro pulita:

```powershell
.\scripts\build-release.ps1
.\scripts\prepare-public-release.ps1 -CompilerPath C:\Percorso\InnoSetup\ISCC.exe
```

Lo staging pubblico usa una lista esplicita di file e una verifica che rifiuta motori esterni o DLL aggiuntive. Per la 0.4.0 prepara `dist/releases/v0.4.0/` con setup, portable, `ConvertHere-0.4.0-source.zip` e `SHA256SUMS.txt` per tutti e tre gli archivi. Licenza del progetto e avvisi di dipendenze e font sono inclusi nei pacchetti.

La preparazione richiama `release-support/build-corresponding-source.ps1`: esporta il commit Git, verifica la provenienza dei binari e usa `cargo vendor --locked --versioned-dirs --sync shell-extension/Cargo.toml` per entrambi i lockfile. L'archivio conserva sorgenti e licenze upstream, configurazione Cargo relativa a `vendor/`, metadati di checksum, toolchain e `BUILDING-OFFLINE.txt`. Dopo aver installato la toolchain indicata, MSVC x64 e Windows SDK, puoi compilare app ed estensione con Cargo offline seguendo quella ricetta. Gli strumenti di sviluppo generali non sono inclusi; Inno Setup serve solo per generare il setup. Motori esterni, certificati e credenziali non fanno parte dell'archivio sorgente.

Per una distribuzione con il menu moderno, usa `prepare-public-release.ps1 -RequireModernMenu` dopo una build firmata con un certificato di produzione. Il comando include MSIX e loghi esterni, oppure si interrompe se manca una firma attendibile. Il relativo setup abilita per default la voce diretta su Windows 11 e la rimuove durante la disinstallazione. Non importa certificati di prova: la procedura locale e i requisiti di firma sono descritti in [docs/windows-integration.md](docs/windows-integration.md).

## Riga di comando

Usa l'eseguibile console dedicato, che restituisce output, diagnostica e codici di uscita affidabili:

```powershell
.\the-converter-cli.exe --diagnose
.\the-converter-cli.exe --install-runtime ffmpeg
.\the-converter-cli.exe --install-runtime poppler
.\the-converter-cli.exe --convert --target mp3 --preset quality --files "C:\Video\clip.mp4"
.\the-converter-cli.exe --convert --target png --output-dir "C:\Risultati" --files "C:\Documenti\file.pdf"
.\the-converter-cli.exe --convert --target csv --files "C:\Documenti\tabella.xlsx"
.\the-converter-cli.exe --convert --target xlsx --csv-separator semicolon --files "C:\Documenti\tabella.csv"
.\the-converter-cli.exe --convert --target docx --files "C:\Documenti\file.pdf"
.\the-converter-cli.exe --convert --target docx --ocr --ocr-language ita+eng --files "C:\Documenti\scansione.pdf"
.\the-converter-cli.exe --convert --target txt --ocr-language ita+eng --files "C:\Documenti\scansione.png"
.\the-converter-cli.exe --convert --target avif --files "C:\Immagini\foto.heic"
.\the-converter-cli.exe --convert --target pdf --files "C:\Documenti\pagina.html"
.\the-converter-cli.exe --convert --target pdf --pdf-action compress --files "C:\Documenti\file.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action split --files "C:\Documenti\file.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Documenti\prima.pdf" "C:\Documenti\seconda.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Immagini\foto.png" "C:\Documenti\documento.docx" "C:\Documenti\pagine.pdf"
.\the-converter-cli.exe --convert --settings .\settings.json --files "C:\Audio\registrazione.wav"
```

`--convert` lavora senza UI, continua sugli altri file dopo un errore e termina con codice non zero se almeno una conversione fallisce. Ctrl+C annulla il processo e pulisce i temporanei. I percorsi sono passati direttamente ai processi senza interpolazione in una shell.

`--install-runtime ffmpeg|poppler` e una modalita separata: richiede un download esplicito del motore fissato e verificato, senza opzioni di conversione o file in ingresso. Scrive nella cartella personale dei motori, conserva licenze e documentazione e non esegue il programma scaricato. Ctrl+C annulla l'operazione. Lo script manuale `install-runtime-tools.ps1 -InstallDirectory <cartella-app> -Engine ffmpeg` richiama la stessa CLI; omettendo `-Engine` richiede entrambi i motori. La cartella dell'app serve a trovare la CLI, non e la destinazione dei motori.

`--pdf-action compress|split|merge` richiede destinazione PDF; `compress` e `split` accettano soltanto PDF, mentre `merge` accetta i file convertibili in PDF descritti sopra. Il default e `compress`. L'unione CLI rispetta l'ordine degli argomenti e delle pagine dentro ogni sorgente; per intercalare singole pagine usa l'editor grafico. `--ocr` attiva il riconoscimento delle scansioni PDF nelle destinazioni testuali; immagini in TXT usano sempre OCR. `--ocr-language ita+eng` sceglie i modelli, senza attivare da solo OCR dei PDF. `--csv-separator comma|semicolon|tab|pipe` sceglie il separatore sia in ingresso sia in uscita; il default e `comma`.

Esempio di impostazioni JSON (i campi omessi usano i default):

```json
{
  "format": "Mp3",
  "audio": { "bitrate_kbps": 256, "sample_rate": 48000, "channels": 2, "normalize": true },
  "video": { "trim_start": 5.0, "trim_end": 35.0 }
}
```

## Verifica

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --manifest-path .\shell-extension\Cargo.toml
.\scripts\test-shell-extension.ps1
```

I test esterni usano i motori trovati localmente; se FFmpeg, Poppler, LibreOffice, Tesseract o libheif mancano, i relativi fixture vengono saltati esplicitamente. I fixture verificano codec e decodifica dei nuovi contenitori, roundtrip HEIC/AVIF e degli altri formati immagine, precisione HDR, i quattro separatori CSV con Unicode/virgolette/righe multiple e formule trattate come testo, primo foglio Excel, ricostruzione PDF in DOCX/HTML con e senza OCR, ordine delle pagine, compressione/unione/divisione PDF, collisioni dei nomi, annullamento e coda. Le verifiche del menu caricano la DLL e usano le API COM di Windows senza registrarla; una verifica di parita confronta tutte le estensioni e destinazioni della DLL con il modello applicativo. La comparsa effettiva nel menu moderno richiede un'installazione firmata e una verifica su Windows 11.

I report di verifica si riferiscono al pacchetto e al codice effettivamente testati. Esiti correnti e prove storiche sono distinti in [store-validation.md](docs/store-validation.md).

## Licenza e sorgenti

Copyright (C) 2026 Salvatore Corsi. Il codice originale di Convert Here e distribuito secondo la **GNU General Public License versione 3 soltanto** (`GPL-3.0-only`). Puoi usarlo, studiarlo, modificarlo e ridistribuirlo alle condizioni del testo integrale in [LICENSE](LICENSE), senza alcuna garanzia nei termini della licenza.

I [sorgenti](https://github.com/salvatorecorsi/convert-here) sono pubblici. `ConvertHere-0.4.0-source.zip` include il commit della release, entrambi i lockfile, dipendenze Cargo vendorizzate, script e istruzioni di build offline; i checksum coprono setup, portable e sorgenti. L'app mostra licenza e collegamento ai sorgenti nelle informazioni sul programma.

La conversione dei tuoi file non cambia i diritti o le licenze dei loro contenuti e non impone la GPL ai risultati per il solo uso del programma. Dipendenze, font e motori esterni conservano le rispettive licenze: vedi [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) e [testi delle licenze](docs/release-licenses/README.md).
