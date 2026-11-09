# Configurazione, motori e dati locali

**Italiano** · [English](configuration.en.md)

Questa guida descrive le impostazioni di Convert Here, incluse le lingue aggiunte nei sorgenti successivi alla 0.4.0. Per iniziare, vedi il [README](../README.md); per le conversioni disponibili, la [guida ai formati](conversions.md).

## Lingua dell'interfaccia

In **Impostazioni → Lingua** scegli **Sistema**, **Italiano** o **English**. Sistema segue la lingua di visualizzazione di Windows: italiano per Windows in italiano, inglese per le altre lingue. La scelta è salvata nel campo `ui_language` di `preferences.json`, con valore `system`, `it` o `en`. Le impostazioni precedenti senza questo campo usano Sistema. La scelta non modifica lingua OCR, formati, nomi dei risultati o altre preferenze.

L'interfaccia bilingue è disponibile compilando i sorgenti aggiornati; i pacchetti v0.4.0 già pubblicati mantengono la GUI italiana. L'installer supporta già italiano e inglese.

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

## Download facoltativo di FFmpeg e Poppler

I pacchetti pubblici non contengono i motori. In **Impostazioni → Motori** puoi richiedere FFmpeg, Poppler o entrambi. La conferma mostra origine, versione, dimensioni, licenze e destinazione; l'installazione avviene soltanto nella cartella dell'utente `%LOCALAPPDATA%\ConvertHere\tools`.

La 0.4.0 scarica [FFmpeg 9.0.2 dalla distribuzione Gyan](https://github.com/GyanD/codexffmpeg/releases/tag/9.0.2) e [Poppler 26.09.0 per Windows](https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0): 158478032 byte complessivi, circa 159 MB. Durante download ed estrazione servono circa 615 MB liberi. Lo SHA-256 viene verificato prima dell'estrazione; il layout conserva licenze e documentazione upstream.

Le richieste HTTPS raggiungono i distributori e i loro CDN, che ricevono IP e normali dati HTTP, incluso User-Agent con versione dell'app. I file da convertire non vengono caricati. Annullamento ed errore tentano di rimuovere lo staging temporaneo; un arresto improvviso può lasciare residui.

Il setup non avvia download e non aggiunge un collegamento Start per installare i motori. LibreOffice, libheif, Tesseract e i modelli OCR si configurano da installazioni locali. Le copie di sviluppo sotto `.tools/` non fanno parte dei pacchetti pubblici.

### Alternativa dalla riga di comando

```powershell
.\the-converter-cli.exe --install-runtime ffmpeg
.\the-converter-cli.exe --install-runtime poppler
.\scripts\install-runtime-tools.ps1 -InstallDirectory "C:\Programmi\ConvertHere" -Engine ffmpeg
```

Lo script richiama la stessa CLI; omettendo `-Engine` richiede entrambi i motori. `-InstallDirectory` individua la cartella dell'app, non la destinazione dei motori. I distributori vengono contattati soltanto quando avvii il comando. [Guida alla CLI](command-line.md).

## Formati visibili e aspetto

In **Impostazioni → Formati di uscita** scegli le destinazioni da mostrare nell'app e nel menu Explorer, raggruppate per audio, video, immagini e documenti. Le otto uscite iniziali sono MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4.

Le scelte vengono salvate e il menu le rilegge senza riavviare Esplora file. Anche disabilitando tutte le uscite resta disponibile **Impostazioni personalizzate…**. Il filtro non blocca la CLI, le richieste `--quick` esplicite o i lavori già accodati. Restano supportati tutti i 44 formati del motore e le 93 estensioni di ingresso.

Per l'aspetto puoi seguire il tema di Windows oppure scegliere **Chiaro** o **Scuro**.

## Preferenze e resoconti

Le preferenze e l'ultimo resoconto rapido si trovano in `%APPDATA%\SalvatoreCorsi\TheConverter\config`, nei file `preferences.json` e `report-last.json`. Il resoconto include percorsi locali ed errori e non viene trasmesso. La conversione interattiva mantiene i propri risultati nella coda corrente.

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
