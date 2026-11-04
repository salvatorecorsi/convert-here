# Convert Here

Sviluppato da **Salvatore Corsi**: versione Rust per Windows di [Final Converter](https://github.com/salvatorecorsi/final-converter). Conversioni di formato locali, OCR sul PC, selezioni multiple e integrazione con Esplora file. Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. Nessun pagamento richiesto per continuare a usarlo.

Il nome pubblico e **Convert Here**, riservato anche in Microsoft Partner Center; il repository canonico e [salvatorecorsi/convert-here](https://github.com/salvatorecorsi/convert-here). Per compatibilita restano invariati i nomi tecnici degli eseguibili e dei crate, le cartelle dati e gli identificativi COM/MSIX. Le release precedenti possono ancora mostrare il nome The Converter. [Novita della versione 0.3.2](docs/release-notes-0.3.2.md).

## Preparazione Microsoft Store

La distribuzione Store usa un **MSIX completo** con il menu di Windows 11 dichiarato nel pacchetto. Editore pubblico e autore del codice: **Salvatore Corsi**. Il nome editore e approvato da Microsoft; il profilo risulta **Authorized** e l'account **Active**. L'account GitHub che ospita il sorgente resta indipendente dall'identita dell'editore Microsoft.

La build di anteprima e separata dalla release GitHub e non contiene certificati privati o motori esterni. Le immagini nei formati nativi e le immagini in PDF funzionano subito, come compressione, divisione e unione dei PDF. Audio/video richiedono FFmpeg; esportazione PDF in immagini o testo incorporato richiede Poppler; Office, Excel/CSV e HTML in PDF richiedono LibreOffice. HEIC/HEIF e AVIF richiedono libheif, mentre OCR richiede Tesseract con i modelli delle lingue e, per i PDF, anche Poppler. I motori si selezionano in **Strumenti**. Questi requisiti sono dichiarati anche all'inizio delle schede Store.

```powershell
.\scripts\build-store-package.ps1
```

La preview usa un'identita di sviluppo e **non va caricata nello Store**. Per il pacchetto definitivo occorrono i dati esatti dell'Identita prodotto in Partner Center, la versione Store e il nome editore effettivamente confermato nel portale. Il modello e in `packaging/store/identity.partner-center.example.json`; la procedura e in [docs/store-submission.md](docs/store-submission.md).

La [release GitHub 0.3.2](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.2) e pubblicata come latest. Il pacchetto Store **1.0.2.0** e caricato, salvato e **Validated**; ha sostituito il precedente 1.0.1.0. La **Submission 1** e stata nuovamente inviata il **4 novembre 2026** ed e **In certification**, con pre-elaborazione in corso e certificazione/pubblicazione non ancora iniziate. L'app resta gratuita in tutti i 240 mercati selezionati, con pubblicazione automatica appena supera la certificazione. Lo Store non e ancora pubblicato. La [privacy 0.3.2](https://github.com/salvatorecorsi/convert-here/blob/v0.3.2/docs/privacy.md) e verificata e salvata; le tre schermate native 0.3.1 sono riutilizzate perche l'interfaccia e identica.

Gli esiti dei test e i controlli Store ancora da completare sono raccolti in [store-validation.md](docs/store-validation.md).

Su GitHub Actions, **Windows checks** verifica codice e fixture su push e pull request. **Microsoft Store package** si avvia manualmente: senza dati Partner Center produce una preview; con `for_submission` e l'identita reale produce il candidato unsigned da caricare. Il pacchetto e il materiale per la scheda si scaricano dagli artefatti del workflow. Le credenziali Microsoft e il certificato locale non sono richiesti dalla pipeline.

## Scarica e installa

Scarica la [release 0.3.2 per Windows x64](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.2) e avvia `ConvertHere-0.3.2-windows-x64-setup.exe`. Il setup richiede Windows 10 build 19041 o successivo, x64. L'installer aggiunge il programma e il disinstaller per il tuo utente; la voce **Converti in** del menu contestuale e facoltativa e selezionata per default. Non richiede privilegi amministrativi.

Il setup è leggero. L'opzione consigliata scarica FFmpeg e Poppler direttamente dai distributori originali (circa 160 MB, con verifica SHA-256). Servono per audio, video e PDF; le conversioni di immagini e immagini in PDF funzionano anche senza questi download. Per riprovare o installare i motori in seguito, nel menu Start trovi **Installa o aggiorna motori di conversione**.

Su Windows 11 il menu della distribuzione GitHub compare in **Mostra altre opzioni**. Gli artefatti pubblici non sono firmati Authenticode e non includono un'identita MSIX firmata attendibile per il menu moderno. Quel percorso resta separato ed e descritto in [windows-integration.md](docs/windows-integration.md).

Per rimuovere il programma usa **Impostazioni Windows → App → App installate → Convert Here → Disinstalla**, oppure il collegamento nel menu Start. Le preferenze e i file convertiti restano conservati. Se Windows mantiene caricata la DLL del menu, la pulizia viene completata al successivo accesso senza riavviare Esplora file durante il lavoro.

È disponibile anche `ConvertHere-0.3.2-windows-x64-portable.zip` nella [release 0.3.2](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.2): estrailo in una cartella stabile, esegui `scripts/install-runtime-tools.ps1 -InstallDirectory <cartella>` per i motori e, se desideri il menu, `scripts/install-context-menu.ps1 -Mode Classic -InstallDirectory <cartella>`. Verifica i download con il file `SHA256SUMS.txt` della stessa release.

## Uso quotidiano

Dal tasto destro scegli **Converti in → MP3**, **WAV**, **PDF**, ecc. La conversione rapida entra in una coda condivisa, con le preferenze salvate, e **non apre la finestra delle opzioni**. Nella cartella di destinazione compare subito un segnaposto con il nome previsto e una clessidra blu. Il menu propone i formati compatibili con la selezione.

Con piu file convertibili in PDF, **PDF (Bulk)** accoda una conversione per ogni file e mantiene risultati separati. **PDF (Merge)** apre invece l'editor delle singole pagine per creare un PDF unico. Sono compatibili immagini, PDF, documenti Word e altri documenti Office supportati, Excel/CSV e HTML; audio e video non hanno queste azioni.

Il segnaposto è un collegamento Windows temporaneo: Esplora file normalmente ne nasconde l'estensione `.lnk`. Un doppio clic apre solo lo stato del lavoro, con avanzamento e annullamento. A conversione conclusa lascia il posto al file effettivo; in caso di errore mostra una X rossa e permette di aprire i dettagli. La clessidra in Esplora file è un'icona statica. [Dettagli della coda](docs/queued-conversions.md).

**Converti in → Impostazioni personalizzate…** apre la finestra con coda, opzioni, avanzamento, annullamento e ultimo resoconto rapido. Puoi anche avviare `the-converter.exe` e trascinare i file nella finestra.

Per default i risultati sono accanto agli originali. Se il nome esiste già, viene aggiunto un numero: gli originali e i risultati precedenti non vengono sovrascritti. Le preferenze, inclusa la cartella di destinazione, restano salvate tra gli avvii.

## Conversioni

Sono disponibili **44 formati di destinazione** e **93 estensioni di ingresso riconosciute**. Le destinazioni proposte dipendono dalla categoria del file e i codec disponibili dai motori selezionati.

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

Impostazioni disponibili: preset leggero/bilanciato/alta qualità, bitrate, frequenza e canali audio, normalizzazione, H.264/H.265, qualità video, larghezza, FPS, taglio temporale, qualità JPEG, ridimensionamento immagini e DPI dei PDF. L'interfaccia mostra solo i controlli pertinenti. Per WebM si usa VP9/Opus, per AVI MPEG-4/MP3, per MPEG MPEG-2/MP2, per WMV WMV2/WMA2, per FLV FLV1/MP3, per 3GP MPEG-4/AAC, per TS H.264/AAC. M4V e un contenitore MP4 con audio; WAV, AIFF e AU sono PCM 16 bit, CAF usa ALAC senza perdita. I PDF creati da immagini sono A4 con proporzioni mantenute e sfondo bianco.

### Excel, CSV, Word e HTML

Excel e CSV vengono convertiti con un profilo LibreOffice temporaneo separato, che disabilita macro e aggiornamento dei collegamenti Office. L'esportazione CSV produce **il primo foglio**, in UTF-8 con delimitatore virgolette; conserva i valori visualizzati, senza formule, formattazione o altri fogli. Puoi scegliere **virgola, punto e virgola, tab o pipe**, con la stessa impostazione per importazione ed esportazione. La virgola e il default. L'importazione conserva come testo i campi quotati, incluse le cifre iniziali degli identificativi, e non valuta le formule. Le altre codifiche vanno prima convertite in UTF-8; il separatore non viene indovinato automaticamente.

PDF → Word/HTML ricostruisce **solo il testo**, conservando la separazione delle pagine in un singolo file modificabile. Di default usa il testo incorporato; puoi attivare OCR locale per riconoscerlo dalle scansioni. Immagini, tabelle, font e impaginazione originali non vengono ricostruiti. L'HTML esportato contiene il testo escapato e gli stili nel file, senza risorse esterne.

HTML → PDF usa il motore di LibreOffice per documenti HTML: supporta HTML/CSS di base e non esegue JavaScript. Non sostituisce la stampa di una pagina dinamica da browser; l'impaginazione puo cambiare. Durante l'apertura di HTML, il motore puo leggere risorse collegate locali o remote, ad esempio immagini e CSS.

### Immagini HEIC e AVIF

Puoi leggere HEIC/HEIF e AVIF, convertirli nei formati immagine nativi o in PDF, e creare HEIC o AVIF dalle immagini supportate. In **Strumenti** seleziona `heif-dec.exe` per leggere e `heif-enc.exe` per creare questi formati. Sono strumenti di [libheif](https://github.com/strukturag/libheif), facoltativi e non inclusi: mantieni insieme eseguibili, DLL e plugin. La build deve includere i decoder e gli encoder HEVC/AV1 richiesti; se mancano, l'app mostra l'errore del motore. Per HEIC/AVIF puoi regolare larghezza e qualita. Dei contenitori con piu immagini viene usata la prima; sequenze e animazioni non vengono conservate.

Gli altri formati immagine sono nativi e non richiedono FFmpeg. HDR non conserva la trasparenza, che viene composta su bianco. OpenEXR e HDR possono conservare valori a virgola mobile; convertirli in formati ordinari puo ridurne precisione e gamma dinamica. JPEG XL, RAW fotografici e DDS non sono attualmente inclusi. I codec audio/video disponibili dipendono dalla build FFmpeg selezionata.

### OCR locale di immagini e scansioni PDF

Per un'immagine scegli **TXT · OCR locale**. Per un PDF scegli TXT, Word DOCX o HTML e abilita **OCR locale**: il riconoscimento e disattivato per default e, quando attivo, elabora tutte le pagine invece di estrarre il testo incorporato. Sono necessari [Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html) e i [modelli delle lingue](https://github.com/tesseract-ocr/tessdata_fast); i PDF richiedono anche `pdftoppm.exe` di Poppler. In **Strumenti** indica `tesseract.exe` e conserva accanto la cartella `tessdata` con i modelli richiesti, per esempio `ita.traineddata` e `eng.traineddata`.

La lingua predefinita e `ita+eng`; puoi scegliere altri codici Tesseract installati. I DPI PDF determinano la risoluzione delle pagine da riconoscere: 300 DPI e un punto di partenza utile. Le immagini vengono orientate secondo EXIF e la trasparenza viene composta su bianco. Il riconoscimento e il testo restano sul PC. Il riconoscimento puo sbagliare e va riletto; non ricostruisce l'impaginazione e non aggiunge uno strato ricercabile al PDF originale. Se non viene riconosciuto testo, l'app segnala l'errore senza creare un risultato vuoto.

### Comprimi, dividi e unisci PDF

Per PDF gia esistenti, **PDF (Bulk)** comprime senza perdita i flussi senza ricampionare le immagini e crea `nome-compresso.pdf`; se la riscrittura sarebbe piu grande, salva una copia dei byte originali. Il risultato non supera le dimensioni del file letto e puo restare della stessa dimensione. **Dividi: una pagina per file** produce `nome-pagina-001.pdf`, `nome-pagina-002.pdf` e cosi via.

Per unire piu file compatibili, scegli **PDF (Merge)** dal menu contestuale oppure **PDF (Merge) · ordina pagine** nell'app e avvia **Ordina e unisci PDF**. L'app prepara le sorgenti e mostra tutte le pagine, identificate dal file e dal numero originale. Trascina la maniglia di una pagina, usa le frecce o **Alt+↑ / Alt+↓** per spostarla: puoi intercalare pagine di PDF multipagina, immagini e documenti diversi. L'ordine iniziale segue i file ricevuti da Esplora file; controllalo nell'editor. **Crea PDF unico** salva `primo-unito.pdf`, aggiungendo un numero se esiste gia. Tutte le pagine vengono incluse una volta, fino a 10.000.

La preparazione usa copie PDF temporanee in una cartella utente nascosta, senza PDF intermedi nella destinazione. Gli originali restano invariati; il salvataggio usa la versione preparata delle pagine. L'ordine e una sessione locale separata dalla coda rapida. Annullamento e chiusura eliminano gli intermedi; un arresto improvviso puo lasciare residui.

Compressione, divisione e unione di PDF esistenti, e l'unione di immagini native, non richiedono motori esterni. Per preparare Office/Excel/CSV/HTML serve LibreOffice; per leggere HEIC/AVIF serve libheif. Poppler non e necessario per questa unione. I PDF cifrati o firmati vengono rifiutati; unione e divisione rifiutano anche moduli interattivi. I contenuti delle pagine vengono mantenuti, mentre segnalibri, struttura accessibile/tag e metadati del documento non vengono trasferiti. La compressione mantiene il catalogo e i moduli; non e una rimozione dei metadati. Non e disponibile la modifica libera del contenuto delle pagine.

### Testo da immagini e PDF

La destinazione TXT e disponibile per le immagini tramite OCR locale e per i PDF tramite estrazione del testo incorporato o OCR locale. I motori richiesti si selezionano nella pagina Strumenti; il riconoscimento e il risultato restano sul PC. Dettagli sui dati locali e sui motori: [privacy](docs/privacy.md).

### Limiti espliciti

- OCR richiede Tesseract e i modelli delle lingue; per i PDF serve anche Poppler. E riconoscimento del testo, senza ricostruzione della grafica o del layout.
- Le immagini GIF animate e i TIFF multipagina sono letti dal primo fotogramma/pagina. La conversione da video a GIF conserva l'animazione.
- WebP è codificato senza perdita. La qualità JPEG non influisce sui formati senza perdita.
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

L'app cerca gli strumenti nella cartella `tools` accanto all'eseguibile, nella cartella utente `%LOCALAPPDATA%\Corsivo\TheConverter\tools`, nel `PATH`, nelle impostazioni salvate e nella cartella indicata dalla variabile `THE_CONVERTER_TOOLS`. I percorsi possono essere scelti nella pagina **Strumenti**. LibreOffice e Tesseract vengono cercati anche nelle rispettive posizioni standard di installazione.

Per Poppler, `pdftoppm.exe` esporta le pagine in immagini e prepara i PDF per OCR, `pdftotext.exe` estrae il testo incorporato e `pdfinfo.exe` conta le pagine. Selezionandone uno, l'app cerca gli altri nella stessa installazione, rispettando eventuali scelte esplicite separate. Se `pdfinfo.exe` non e disponibile, il conteggio usa il lettore PDF interno.

La distribuzione locale di sviluppo può includere FFmpeg e Poppler. L'installer e l'archivio portable pubblici li scaricano separatamente dagli upstream su richiesta. LibreOffice, libheif, Tesseract e i modelli OCR non sono inclusi ne scaricati automaticamente: configura installazioni locali nella pagina Strumenti. Le copie estratte sotto `.tools/` per i test non fanno parte dei pacchetti pubblici. L'app non richiede Python, Node, WebView o un'installazione Rust per essere eseguita.

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

Per preparare gli artefatti pubblici, dopo la build:

```powershell
.\release-support\collect-rust-licenses.ps1
.\scripts\prepare-public-release.ps1 -CompilerPath C:\Percorso\InnoSetup\ISCC.exe
```

Lo staging pubblico usa una lista esplicita di file e una verifica che rifiuta motori esterni o DLL aggiuntive. Per la versione 0.3.2 la cartella e `dist/releases/v0.3.2/`, con `ConvertHere-0.3.2-windows-x64-setup.exe`, `ConvertHere-0.3.2-windows-x64-portable.zip` e `SHA256SUMS.txt`. Gli avvisi delle dipendenze Rust e dei font sono inclusi in entrambi gli artefatti.

Per una distribuzione con il menu moderno, usa `prepare-public-release.ps1 -RequireModernMenu` dopo una build firmata con un certificato di produzione. Il comando include MSIX e loghi esterni, oppure si interrompe se manca una firma attendibile. Il relativo setup abilita per default la voce diretta su Windows 11 e la rimuove durante la disinstallazione. Non importa certificati di prova: la procedura locale e i requisiti di firma sono descritti in [docs/windows-integration.md](docs/windows-integration.md).

## Riga di comando

Usa l'eseguibile console dedicato, che restituisce output, diagnostica e codici di uscita affidabili:

```powershell
.\the-converter-cli.exe --diagnose
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

Vedi anche [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) per provenienza e licenze dei motori locali.
