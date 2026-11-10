# Configurazione, motori e dati locali

**Italiano** · [English](configuration.en.md)

Questa guida descrive le impostazioni di Convert Here, inclusa la scelta fra italiano e inglese. Per iniziare, vedi il [README](../README.md); per le conversioni disponibili, la [guida ai formati](conversions.md).

**Stato al 10 novembre 2026:** [0.8.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.0) pubblicata su GitHub alle **12:09:39 UTC**, commit `f1f3f95fceed52507ef715fa4ac6893acc63544a`. Store **1.0.8.0**, Submission 5 (`1152921505702365703`): **Update in certification**, non ancora approvata o pubblicata sullo Store.

## Lingua dell'interfaccia

In **Impostazioni → Lingua** scegli **Sistema**, **Italiano** o **English**. Sistema segue la lingua di visualizzazione di Windows: italiano per Windows in italiano, inglese per le altre lingue. La scelta è salvata nel campo `ui_language` di `preferences.json`, con valore `system`, `it` o `en`. Le impostazioni precedenti senza questo campo usano Sistema. La scelta non modifica lingua OCR o del parlato, formati, nomi dei risultati o altre preferenze.

L'interfaccia e il setup sono disponibili in italiano e inglese.

Per una sola esecuzione della CLI usa `--language system|it|en`: prevale sulla scelta salvata senza riscriverla. I messaggi di Windows e dei motori esterni possono seguire le rispettive impostazioni. [Guida alla CLI](command-line.md).

## Motori di conversione

In **Impostazioni → Motori** puoi scegliere i percorsi degli eseguibili o rilevare gli strumenti installati. L'app cerca nelle impostazioni salvate, nella cartella `tools` accanto all'eseguibile, in `%LOCALAPPDATA%\ConvertHere\tools`, nel `PATH` e nella cartella indicata da `THE_CONVERTER_TOOLS`. Sono ancora riconosciute in lettura le precedenti cartelle dei motori per compatibilità con le vecchie versioni. LibreOffice e Tesseract vengono cercati anche nelle posizioni standard di installazione.

Per Poppler:

- `pdftoppm.exe` esporta le pagine in immagini e prepara i PDF per OCR.
- `pdftotext.exe` estrae il testo incorporato.
- `pdfinfo.exe` conta le pagine. Se manca, il conteggio usa il lettore PDF interno.

Selezionando uno degli strumenti Poppler, l'app cerca gli altri nella stessa installazione, rispettando eventuali percorsi espliciti separati.

Per l'OCR configura `tesseract.exe` e conserva accanto la cartella `tessdata` con i modelli richiesti, per esempio `ita.traineddata` e `eng.traineddata`. Altri codici funzionano se i rispettivi modelli sono installati. La lingua OCR non modifica la lingua dell'interfaccia.

Per libheif mantieni `heif-dec.exe` e `heif-enc.exe` insieme alle DLL e ai plugin della stessa distribuzione. La build deve contenere i decoder e gli encoder HEVC/AV1 necessari.

Per la trascrizione configura `whisper-cli.exe` e il modello `ggml-tiny.bin`. Servono anche FFmpeg e ffprobe: il solo runtime Whisper non legge direttamente tutti i contenitori audio/video. La lingua del parlato e i thread CPU si configurano nei controlli di trascrizione, separati dalla lingua dell'interfaccia e dall'OCR. Il default rileva automaticamente la lingua; i profili salvano `speech.language` e `speech.threads`.

## Download facoltativi dei motori

I pacchetti dell'app non contengono i motori esterni o il modello vocale. In **Impostazioni → Motori** richiedi soltanto quelli necessari. La versione 0.8.0 aggiunge Tesseract, libheif e Whisper + Tiny ai download di FFmpeg e Poppler. La conferma mostra origine, versione, dimensioni, spazio estratto, licenze e destinazione. L'installazione avviene nella sola cartella personale `%LOCALAPPDATA%\ConvertHere\tools`, senza UAC o installer di sistema.

| Motore | Distribuzione fissata | Download |
| --- | --- | --- |
| FFmpeg + Poppler | [FFmpeg 9.0.2 Gyan](https://github.com/GyanD/codexffmpeg/releases/tag/9.0.2) e [Poppler 26.09.0](https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0) | 158.478.032 byte complessivi; circa 615 MB liberi durante download/estrazione |
| Tesseract + ita/eng | [Tesseract 5.5.4 da MSYS2](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-tesseract-ocr) e modelli OCR | 38.276.135 byte; circa 60 MB estratti |
| libheif | [libheif 1.23.7 da MSYS2](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-libheif) | 28.215.281 byte; circa 71,4 MB estratti |
| Whisper + Tiny | [Whisper 1.9.5.r258 da MSYS2](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-whisper.cpp) e [Tiny multilingue](https://huggingface.co/ggerganov/whisper.cpp) | 87.939.016 byte, inclusi 77.691.713 del modello; circa 101,5 MB estratti |

Il [catalogo dei pacchetti portabili](../src/runtime_install/portable-packages.json) riporta versioni, origini e SHA-256 fissati per Tesseract, libheif e Whisper/Tiny. Lo SHA-256 viene verificato prima dell'estrazione; licenze e documentazione sono conservate. Il download non esegue gli strumenti scaricati. Le distribuzioni portabili includono le dipendenze necessarie; Whisper offre il backend CPU generico senza richiedere un'installazione manuale del runtime Visual C++.

Se scegli Whisper dall'app e manca FFmpeg oppure ffprobe, viene aggiunto anche FFmpeg alla richiesta di conferma. Il suo peso resta separato dagli 87.939.016 byte di Whisper + Tiny. Nessun modello o motore viene scaricato automaticamente all'avvio dell'app.

Le richieste HTTPS raggiungono i distributori e i loro CDN: GitHub/Gyan per i motori esistenti, MSYS2 e relativi mirror, GitHub per licenze/modelli OCR, Hugging Face e CDN per Tiny. Ricevono IP e normali dati HTTP, incluso User-Agent con versione dell'app. I file da convertire non vengono caricati. Annullamento ed errore tentano di rimuovere lo staging temporaneo; un arresto improvviso può lasciare residui.

Il setup non avvia download e non aggiunge un collegamento Start per installare i motori. LibreOffice resta da installare e selezionare separatamente; puoi continuare a usare copie locali degli altri motori. Le copie di sviluppo sotto `.tools/` non fanno parte dei pacchetti pubblici.

### Alternativa dalla riga di comando

```powershell
.\the-converter-cli.exe --install-runtime ffmpeg
.\the-converter-cli.exe --install-runtime poppler
.\the-converter-cli.exe --install-runtime tesseract
.\the-converter-cli.exe --install-runtime libheif
.\the-converter-cli.exe --install-runtime whisper
.\scripts\install-runtime-tools.ps1 -InstallDirectory "C:\Programmi\ConvertHere" -Engine ffmpeg
```

I tre comandi aggiuntivi sono disponibili dalla versione 0.8.0. La CLI installa il solo motore richiesto: per trascrivere installa separatamente anche FFmpeg. Lo script richiama la stessa CLI; omettendo `-Engine` richiede **FFmpeg e Poppler**, non tutti i motori. `-InstallDirectory` individua la cartella dell'app, non la destinazione dei motori. I distributori vengono contattati soltanto quando avvii il comando. [Guida alla CLI](command-line.md).

## Formati visibili e aspetto

In **Impostazioni → Formati di uscita** scegli le destinazioni da mostrare nell'app e nel menu Explorer, raggruppate per audio, video, immagini, documenti e **Raw**. Le otto uscite iniziali restano MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4. Abilita **TXT** in Raw per rendere visibili OCR e trascrizione.

Le scelte vengono salvate e il menu le rilegge senza riavviare Esplora file. Anche disabilitando tutte le uscite resta disponibile **Impostazioni personalizzate…**. Il filtro non disabilita ingressi, CLI, richieste `--quick` esplicite o lavori già accodati. La versione 0.8.0 mantiene 44 formati di uscita e riconosce 94 estensioni di ingresso, inclusa SVG. **File**, **Cronologia** e **Profili** sono a sinistra nella navigazione; **Impostazioni** è a destra.

Per l'aspetto puoi seguire il tema di Windows oppure scegliere **Chiaro** o **Scuro**.

## Profili personali

Nella scheda **Profili**, crea, modifica, duplica, rinomina, elimina e scegli la ricetta predefinita. Si possono salvare **32 profili** e mostrarne **8 in Converti in**, se il formato è abilitato e ogni ingresso selezionato è compatibile. Puoi ammettere tutti gli ingressi compatibili o limitare una ricetta a estensioni specifiche. Il filtro non cambia le 94 estensioni riconosciute nella versione 0.8.0. Riapri il menu Explorer dopo aver modificato un profilo; i lavori già accodati mantengono la propria ricetta. I profili PDF Merge richiedono almeno due file e aprono l'ordinamento pagine.

**Importa JSON…**, **Esporta…** ed **Esporta tutti…** trasferiscono le ricette. Il JSON comprende parametri, filtri, visibilità ed eventuale cartella di uscita, senza percorsi dei motori o contenuti di prova. L'importazione è atomica, crea nuove copie e non sovrascrive profili; oltre gli 8 visibili importa le ricette aggiuntive fuori dal menu. Un file oltre 256 KiB o un totale superiore a 32 profili viene rifiutato. Il nome scelto dall'utente non viene tradotto. [Guida completa ed esempio JSON](image-editor-profiles.md).

### Peso, durata e Avanzate

Il **peso massimo per file** comune viene verificato prima di pubblicare ogni risultato. I vecchi limiti immagine continuano a funzionare; se coesistono, vale il minore. L'editor JPEG/WebP può cercare una qualità adatta e ridimensionare solo se autorizzato. Gli altri percorsi falliscono se il risultato resta troppo grande; il limite audio/video non è una promessa di bitrate automatico.

La **durata massima audio/video** si applica alle sorgenti audio/video, anche per estrazione audio, GIF e trascrizione, dopo l'inizio scelto e insieme alla fine del taglio. Non tronca immagini o file TXT usati come ingresso.

**Avanzate** raccoglie controlli tipizzati e ricercabili per codec, filtri, metadati e prestazioni. Le opzioni specifiche dell'encoder sono rilevate dal FFmpeg locale configurato, non da un servizio online. Le disponibilità dipendono da build, codec e contenitore; opzioni incompatibili o non verificabili producono un errore. Le ricette con opzioni FFmpeg richiedono ingressi audio/video, anche quando l'uscita è GIF. Non sono accettati comandi liberi, script o percorsi eseguibili nel catalogo.

Nell'editor audio/video il gain e il muto diventano `media_options.audio_volume`; il taglio usa `video.trim_start` e `video.trim_end` anche per l'audio. Per TXT si applicano soltanto taglio, limiti e impostazioni del parlato: gain, muto e opzioni di codifica vengono esclusi. **Usa queste impostazioni** aggiorna le impostazioni attive; **Salva profilo…** conserva i parametri, senza contenuto del file o trascrizione completa.

## Preferenze e resoconti

Le preferenze e l'ultimo resoconto rapido si trovano in `%APPDATA%\SalvatoreCorsi\TheConverter\config`, nei file `preferences.json` e `report-last.json`. `history.json` conserva fino a 300 conversioni terminate dalla finestra, dall'editor e da PDF Merge, entro 1 MiB; la scheda **Cronologia** le riunisce ai lavori della coda rapida. Questi file includono percorsi locali ed errori e non vengono trasmessi. Le scritture della cronologia avvengono in background; un file non leggibile viene segnalato e conservato.

In un'installazione MSIX Windows può reindirizzare i percorsi nei dati privati del pacchetto. L'app non cifra i file di preferenze e stato: si applicano le protezioni dell'account Windows e del dispositivo. [Informativa privacy](privacy.md).

## Coda rapida e recupero

La coda rapida viene salvata nei dati locali dell'app sotto `%LOCALAPPDATA%`. Un solo processo elabora i lavori, anche quando arrivano da più comandi del menu contestuale. Dopo un'interruzione forzata, i lavori già in corso vengono segnalati come interrotti e quelli ancora in attesa ripartono al successivo avvio dell'app o invio.

I nomi vengono prenotati prima di iniziare e restano quelli mostrati nei segnaposto. Per i PDF esportati in immagini viene prenotato un nome per pagina. Le cartelle temporanee nella destinazione sono nascoste. La clessidra del segnaposto in Esplora file è un'icona statica.

La pulizia della cronologia, quando il processo di lavoro riprende o termina la coda, elimina i record completati o annullati dopo 7 giorni e quelli falliti dopo 30 giorni; tenta inoltre di conservare al massimo i 300 record terminali più recenti. Originali, risultati e lavori attivi vengono preservati. I record illeggibili vengono isolati e conservati, insieme agli eventuali file estranei.

Per un lavoro fallito, **Rimuovi segnaposto** elimina soltanto i collegamenti ancora appartenenti a quel lavoro, mantenendo record e risultati. Prima di cancellare manualmente i dati locali, termina o annulla i lavori e chiudi l'app. [Dettagli della coda](queued-conversions.md).

## Menu della versione portable

Estrai l'archivio in una cartella stabile. Per installare il menu classico per l'utente corrente, esegui dalla cartella estratta:

```powershell
.\scripts\install-context-menu.ps1 -Mode Classic
```

Per rimuoverlo:

```powershell
.\scripts\uninstall-context-menu.ps1 -Mode Classic
```

Su Windows 11 compare in **Mostra altre opzioni**. La [guida all'integrazione con Windows](windows-integration.md) descrive anche l'estensione per il menu moderno e i suoi requisiti.
