# Convert Here

Progetto **Corsivo**, sviluppato da **Salvatore Corsi**: versione Rust per Windows di [Final Converter](https://github.com/salvatorecorsi/final-converter). Conversioni di formato locali, selezioni multiple e integrazione con Esplora file. La trascrizione audio in TXT e facoltativa: usa OpenAI online con la chiave API dell'utente e invia l'audio solo quando viene richiesta.

Il nome **Convert Here** e stato riservato in Microsoft Partner Center. Repository, nomi degli eseguibili e identificativi tecnici restano quelli esistenti per compatibilita. Le release precedenti possono ancora mostrare il nome The Converter.

## Preparazione Microsoft Store

La distribuzione Store usa un **MSIX completo** con il menu di Windows 11 dichiarato nel pacchetto. Editore pubblico: **Corsivo**; autore del codice: Salvatore Corsi. L'account GitHub che ospita il sorgente resta indipendente dall'identita dell'editore Microsoft.

La build di anteprima e separata dalla release GitHub e non contiene certificati privati o motori esterni. Le immagini nei formati nativi e le immagini in PDF funzionano subito, come compressione, divisione e unione dei PDF. Audio/video richiedono FFmpeg; esportazione PDF in immagini o testo incorporato richiede Poppler; Office, Excel/CSV e HTML in PDF richiedono LibreOffice. HEIC/HEIF e AVIF richiedono libheif, mentre OCR richiede Tesseract con i modelli delle lingue e, per i PDF, anche Poppler. I motori si selezionano in **Strumenti**. La trascrizione audio TXT richiede internet, una propria chiave API OpenAI e, quando occorre preparare l'audio, FFmpeg. Questi requisiti sono dichiarati anche all'inizio delle schede Store.

```powershell
.\scripts\build-store-package.ps1
```

La preview usa un'identita di sviluppo e **non va caricata nello Store**. Per il pacchetto definitivo occorrono i dati esatti dell'Identita prodotto in Partner Center, la versione Store e l'editore assegnato all'account di Corsivo. Il modello e in `packaging/store/identity.partner-center.example.json`; la procedura e in [docs/store-submission.md](docs/store-submission.md).

Sono preparate le [schede italiana](docs/store-listing-it.md) e [inglese](docs/store-listing-en.md), con interfaccia attualmente in italiano, e una [bozza dell'informativa privacy](docs/privacy.md). I campi del titolare, i contatti e gli URL mancanti vanno completati prima dell'invio. Microsoft firma il MSIX dopo la certificazione: la build Store non importa certificati e non invia automaticamente l'app a Partner Center.

Gli esiti dei test e i controlli Store ancora da completare sono raccolti in [store-validation.md](docs/store-validation.md).

Su GitHub Actions, **Windows checks** verifica codice e fixture su push e pull request. **Microsoft Store package** si avvia manualmente: senza dati Partner Center produce una preview; con `for_submission` e l'identita reale produce il candidato unsigned da caricare. Il pacchetto e il materiale per la scheda si scaricano dagli artefatti del workflow. Le credenziali Microsoft e il certificato locale non sono richiesti dalla pipeline.

## Scarica e installa

Scarica l'[ultima release Windows x64](https://github.com/salvatorecorsi/the-converter/releases/latest) e avvia il file `TheConverter-<versione>-windows-x64-setup.exe`. L'installer aggiunge il programma, il disinstaller e la voce **Converti in** del tasto destro per il tuo utente, senza richiedere privilegi amministrativi.

Il setup è leggero. L'opzione consigliata scarica FFmpeg e Poppler direttamente dai distributori originali (circa 160 MB, con verifica SHA-256). Servono per audio, video e PDF; le conversioni di immagini e immagini in PDF funzionano anche senza questi download. Per riprovare o installare i motori in seguito, nel menu Start trovi **Installa o aggiorna motori di conversione**.

Su Windows 11 il menu incluso in questa release compare in **Mostra altre opzioni**. Il menu moderno richiede un'identità MSIX firmata attendibile ed è descritto in [windows-integration.md](docs/windows-integration.md); la release iniziale non include un pacchetto firmato.

Per rimuovere il programma usa **Impostazioni Windows → App → App installate → Convert Here → Disinstalla**, oppure il collegamento nel menu Start. Le preferenze e i file convertiti restano conservati. Se Windows mantiene caricata la DLL del menu, la pulizia viene completata al successivo accesso senza riavviare Esplora file durante il lavoro.

È disponibile anche l'archivio portable: estrailo in una cartella stabile, esegui `scripts/install-runtime-tools.ps1 -InstallDirectory <cartella>` per i motori e, se desideri il menu, `scripts/install-context-menu.ps1 -Mode Classic -InstallDirectory <cartella>`.

## Uso quotidiano

Dal tasto destro scegli **Converti in → MP3**, **WAV**, **PDF**, ecc. La conversione rapida entra in una coda condivisa, con le preferenze salvate, e **non apre la finestra delle opzioni**. Nella cartella di destinazione compare subito un segnaposto con il nome previsto e una clessidra blu. Il menu propone i formati compatibili con la selezione.

Il segnaposto è un collegamento Windows temporaneo: Esplora file normalmente ne nasconde l'estensione `.lnk`. Un doppio clic apre solo lo stato del lavoro, con avanzamento e annullamento. A conversione conclusa lascia il posto al file effettivo; in caso di errore mostra una X rossa e permette di aprire i dettagli. La clessidra in Esplora file è un'icona statica. [Dettagli della coda](docs/queued-conversions.md).

**Converti in → Impostazioni personalizzate…** apre la finestra con coda, opzioni, avanzamento, annullamento e ultimo resoconto rapido. Puoi anche avviare `the-converter.exe` e trascinare i file nella finestra.

Per default i risultati sono accanto agli originali. Se il nome esiste già, viene aggiunto un numero: gli originali e i risultati precedenti non vengono sovrascritti. Le preferenze, inclusa la cartella di destinazione, restano salvate tra gli avvii.

## Conversioni

Sono disponibili **44 formati di destinazione** e **93 estensioni di ingresso riconosciute**. Le destinazioni proposte dipendono dalla categoria del file e i codec disponibili dai motori selezionati.

| Sorgente | Destinazioni | Motore |
| --- | --- | --- |
| Audio: MP3, MPGA, WAV, FLAC, AAC, M4A/M4B, OGG/OGA, Opus, AIFF/AIFC, WMA, ALAC, APE, MKA, MP2, WebA, AC3, AU, CAF, AMR, DTS, TTA, VOC, WV, TAK, W64, RA | MP3, WAV, FLAC, AAC, OGG, Opus, AIFF, M4A (AAC), WMA, AC3, AU, CAF (ALAC) | FFmpeg |
| Audio supportati | TXT, trascrizione del parlato | OpenAI GPT-Transcribe facoltativo; FFmpeg per preparare formati non accettati direttamente o file grandi |
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

La lingua predefinita e `ita+eng`; puoi scegliere altri codici Tesseract installati. I DPI PDF determinano la risoluzione delle pagine da riconoscere: 300 DPI e un punto di partenza utile. Le immagini vengono orientate secondo EXIF e la trasparenza viene composta su bianco. Il testo resta sul PC, senza OpenAI o costi API. Il riconoscimento puo sbagliare e va riletto; non ricostruisce l'impaginazione e non aggiunge uno strato ricercabile al PDF originale. Se non viene riconosciuto testo, l'app segnala l'errore senza creare un risultato vuoto.

### Comprimi, dividi e unisci PDF

Nella finestra delle opzioni, seleziona soltanto PDF e scegli **PDF · comprimi / dividi / unisci**. **Comprimi senza perdita** ottimizza i flussi senza ricampionare le immagini e crea `nome-compresso.pdf`; se la riscrittura sarebbe piu grande, salva una copia dei byte originali. Il risultato non supera le dimensioni del file letto e puo restare della stessa dimensione. **Dividi: una pagina per file** produce `nome-pagina-001.pdf`, `nome-pagina-002.pdf` e cosi via. **Unisci i PDF della lista** crea un solo `primo-unito.pdf` nell'ordine mostrato: usa le frecce della lista per cambiarlo. I nomi esistenti vengono preservati con la consueta numerazione.

Queste operazioni sono locali e non richiedono motori esterni. Rifiutano PDF cifrati o con firme digitali. Unione e divisione rifiutano anche moduli interattivi: conservano i contenuti delle pagine, ma non trasferiscono segnalibri, struttura accessibile/tag o metadati del documento. La compressione mantiene il catalogo e i moduli; non e una rimozione dei metadati. Non e disponibile la modifica libera del contenuto delle pagine.

### Trascrizione audio TXT con OpenAI, facoltativa

In **Strumenti → OpenAI · audio in testo**, inserisci una tua chiave API e scegli **Salva e abilita trascrizione**. Windows la conserva nel Gestore credenziali dell'utente corrente, con voce `Corsivo.TheConverter.OpenAI`: non viene scritta in preferenze, coda, resoconti o log. **Rimuovi chiave** disabilita le nuove trascrizioni. Le altre conversioni funzionano anche senza chiave.

La credenziale Windows e separata dai file dell'app e non viene eliminata automaticamente disinstallando il setup EXE o il pacchetto MSIX. Prima di disinstallare usa **Rimuovi chiave**; se l'app e gia stata rimossa, apri **Gestione credenziali → Credenziali Windows → Credenziali generiche** e rimuovi la voce `Corsivo.TheConverter.OpenAI` del tuo utente. Questo elimina la copia locale, senza revocare la chiave nel tuo account OpenAI.

Dopo l'abilitazione, **TXT con OpenAI** e disponibile per gli audio compatibili con l'app. Sceglierlo avvia una trascrizione online: l'audio viene inviato via HTTPS a `api.openai.com/v1/audio/transcriptions`, usando il modello `gpt-transcribe`. Puoi lasciare il riconoscimento automatico della lingua o indicarne una. I video non hanno questa destinazione: prima estrai l'audio in un formato supportato. Il TXT da un PDF rimane invece estrazione locale con Poppler, senza OpenAI.

Occorre un account OpenAI API con accesso e fatturazione disponibili. Gli eventuali costi sono a carico di quell'account, separati dall'abbonamento ChatGPT; Convert Here non acquista credito e non include chiamate API. Verifica le [tariffe OpenAI](https://developers.openai.com/api/docs/pricing) prima dell'uso.

Gli audio in un formato accettato direttamente e fino alla soglia prudenziale di 24 MB (24.000.000 byte) vengono inviati senza FFmpeg. Negli altri casi FFmpeg prepara localmente MP3 mono a 16 kHz/64 kbit/s, suddivisi in segmenti di circa 20 minuti; l'app li invia in sequenza e unisce i testi. OpenAI documenta un limite di 25 MB per file. Il TXT viene pubblicato soltanto a lavoro completo; non ci sono tentativi automatici dopo un errore. Annullare non ritira l'audio gia inviato e non annulla eventuali costi maturati. [Requisiti dell'API](https://developers.openai.com/api/docs/guides/speech-to-text).

Il contratto di caricamento dell'API accetta FLAC, MP3, MP4, MPEG, MPGA, M4A, OGG, WAV e WebM. Per gli audio compatibili con l'app, FLAC e OGG restano fra i formati inviati direttamente: fino a 24 MB mantengono i byte originali e non richiedono FFmpeg. [Formati del caricamento nell'API](https://developers.openai.com/api/reference/resources/audio/subresources/transcriptions/methods/create).

Per una registrazione suddivisa in segmenti, le richieste successive possono reinviare come `prompt` fino a 200 caratteri di parole intere gia trascritte della stessa registrazione, con spazi e caratteri di controllo normalizzati. La prima richiesta e gli invii singoli non includono questo contesto.

Il backend richiede una risposta JSON da GPT-Transcribe ed estrae il campo `text` per creare il normale file TXT UTF-8: il risultato per l'utente resta un documento di testo. La preferenza salvata `transcription.language` rimane compatibile; quando impostata viene inviata come una sola voce `languages[]`, mentre il riconoscimento automatico non invia un suggerimento di lingua. [Contratto di migrazione OpenAI](https://developers.openai.com/cookbook/examples/migrating_from_whisper_to_gpt_transcribe).

Il testo mantiene la lingua parlata, senza traduzione, timestamp o riconoscimento dei parlanti. La trascrizione puo contenere errori e va riletta. Dettagli sul trattamento dei dati: [privacy](docs/privacy.md).

### Limiti espliciti

- OCR richiede Tesseract e i modelli delle lingue; per i PDF serve anche Poppler. E riconoscimento del testo, senza ricostruzione della grafica o del layout.
- Le immagini GIF animate e i TIFF multipagina sono letti dal primo fotogramma/pagina. La conversione da video a GIF conserva l'animazione.
- WebP è codificato senza perdita. La qualità JPEG non influisce sui formati senza perdita.
- Ogni immagine genera il proprio PDF. Puoi poi unire i PDF nell'ordine scelto; unione e divisione non trasferiscono segnalibri, tag o metadati e rifiutano moduli interattivi. Compressione, divisione e unione rifiutano documenti cifrati o firmati. L'esportazione PDF in PNG/JPEG accetta la sola password proprietario; una password di apertura richiede di sbloccare prima il documento.
- I documenti Office, i fogli di calcolo, i CSV e HTML in PDF richiedono LibreOffice installato o configurato; l'impaginazione dipende anche dai font disponibili.
- PDF in DOCX/HTML e una ricostruzione del testo, senza immagini o impaginazione originale. Excel in CSV esporta soltanto il primo foglio e perde formule e formattazione.
- La coda interattiva usa un formato comune ai file selezionati. Se le categorie non hanno una destinazione comune, vanno convertite in gruppi separati.
- Le conversioni rapide lavorano in coda in background. La finestra di stato si apre su richiesta dal segnaposto, mentre la finestra principale mostra la coda completa. Il resoconto conserva l'ultima esecuzione rapida; chiudere la finestra di stato non interrompe il lavoro.

## Eseguibile e menu contestuale

La distribuzione locale pronta è in `dist\TheConverter\`. Mantieni questa cartella in una posizione stabile: contiene applicazione, CLI, DLL Explorer e motori.

Per installare il menu classico per il solo utente corrente, da PowerShell:

```powershell
.\dist\TheConverter\scripts\install-context-menu.ps1 -Mode Classic
```

Per rimuoverlo:

```powershell
.\dist\TheConverter\scripts\uninstall-context-menu.ps1 -Mode Classic
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

Lo staging pubblico usa una lista esplicita di file e una verifica che rifiuta motori esterni o DLL aggiuntive. L'output in `dist/releases/v<versione>/` contiene setup, archivio portable e `SHA256SUMS.txt`. Gli avvisi delle dipendenze Rust e dei font sono inclusi in entrambi gli artefatti.

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
.\the-converter-cli.exe --convert --settings .\settings.json --files "C:\Audio\registrazione.wav"
```

`--convert` lavora senza UI, continua sugli altri file dopo un errore e termina con codice non zero se almeno una conversione fallisce. Ctrl+C annulla il processo e pulisce i temporanei. I percorsi sono passati direttamente ai processi senza interpolazione in una shell.

`--pdf-action compress|split|merge` richiede destinazione PDF e soli PDF in ingresso; il default e `compress`. L'unione richiede almeno due file e rispetta l'ordine degli argomenti. `--ocr` attiva il riconoscimento delle scansioni PDF nelle destinazioni testuali; immagini in TXT usano sempre OCR. `--ocr-language ita+eng` sceglie i modelli, senza attivare da solo OCR dei PDF. `--csv-separator comma|semicolon|tab|pipe` sceglie il separatore sia in ingresso sia in uscita; il default e `comma`.

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

I test automatici della trascrizione usano un servizio simulato locale, senza inviare audio a OpenAI o sostenere costi API. Non costituiscono una prova di accesso, fatturazione o disponibilita del servizio reale. I report di verifica si riferiscono al pacchetto e al codice effettivamente testati: quelli precedenti all'integrazione OpenAI non coprono questa funzione.

Vedi anche [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) per provenienza e licenze dei motori locali.
