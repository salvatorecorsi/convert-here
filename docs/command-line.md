# Riga di comando

**Italiano** · [English](command-line.en.md)

La CLI di Convert Here permette di convertire file, automatizzare elaborazioni e installare i motori opzionali. Esegui gli esempi PowerShell dalla cartella dell'app. Per i requisiti delle singole conversioni, consulta la [guida ai formati](conversions.md).

**Stato al 10 novembre 2026:** [0.8.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.0) pubblicata su GitHub alle **12:09:39 UTC**, commit `f1f3f95fceed52507ef715fa4ac6893acc63544a`. Store **1.0.8.0**, Submission 5 (`1152921505702365703`): **Update in certification**, non ancora approvata o pubblicata sullo Store.

Usa l'eseguibile console dedicato, che restituisce output, diagnostica e codici di uscita affidabili:

```powershell
.\the-converter-cli.exe --language it --diagnose
.\the-converter-cli.exe --install-runtime ffmpeg
.\the-converter-cli.exe --install-runtime poppler
.\the-converter-cli.exe --install-runtime tesseract
.\the-converter-cli.exe --install-runtime libheif
.\the-converter-cli.exe --install-runtime whisper
.\the-converter-cli.exe --convert --target mp3 --preset quality --files "C:\Video\clip.mp4"
.\the-converter-cli.exe --convert --target png --output-dir "C:\Risultati" --files "C:\Documenti\file.pdf"
.\the-converter-cli.exe --convert --target csv --files "C:\Documenti\tabella.xlsx"
.\the-converter-cli.exe --convert --target xlsx --csv-separator semicolon --files "C:\Documenti\tabella.csv"
.\the-converter-cli.exe --convert --target docx --files "C:\Documenti\file.pdf"
.\the-converter-cli.exe --convert --target docx --ocr --ocr-language ita+eng --files "C:\Documenti\scansione.pdf"
.\the-converter-cli.exe --convert --target txt --ocr-language ita+eng --files "C:\Documenti\scansione.png"
.\the-converter-cli.exe --convert --target avif --files "C:\Immagini\foto.heic"
.\the-converter-cli.exe --convert --target png --files "C:\Immagini\illustrazione.svg"
.\the-converter-cli.exe --convert --target pdf --files "C:\Immagini\illustrazione.svg"
.\the-converter-cli.exe --convert --target txt --files "C:\Audio\registrazione.wav"
.\the-converter-cli.exe --custom-settings --target txt --files "C:\Audio\registrazione.wav"
.\the-converter-cli.exe --convert --target pdf --files "C:\Documenti\pagina.html"
.\the-converter-cli.exe --convert --target pdf --pdf-action compress --files "C:\Documenti\file.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action split --files "C:\Documenti\file.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Documenti\prima.pdf" "C:\Documenti\seconda.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Immagini\foto.png" "C:\Documenti\documento.docx" "C:\Documenti\pagine.pdf"
.\the-converter-cli.exe --convert --settings .\settings.json --files "C:\Audio\registrazione.wav"
```

`--language system|it|en` sceglie la lingua dell'interfaccia e dei messaggi applicativi per l'esecuzione corrente senza cambiare le preferenze salvate. Senza override viene usata la scelta salvata. `system` segue Windows: italiano per la lingua di visualizzazione italiana, inglese altrimenti. Questa opzione è disponibile dalla versione 0.5.0. Nomi delle opzioni, chiavi JSON e suffissi dei risultati restano stabili fra le lingue. I messaggi di Windows e dei motori esterni possono avere una lingua propria.

`--convert` lavora senza UI, continua sugli altri file dopo un errore e termina con codice non zero se almeno una conversione fallisce. Ctrl+C annulla il processo e pulisce i temporanei. I percorsi sono passati direttamente ai processi senza interpolazione in una shell.

`--install-runtime ffmpeg|poppler|tesseract|libheif|whisper` è una modalità separata: richiede un download esplicito del motore fissato e verificato, senza opzioni di conversione o file in ingresso. Gli ultimi tre valori sono disponibili dalla versione 0.8.0. Scrive in `%LOCALAPPDATA%\ConvertHere\tools`, conserva licenze e documentazione e non esegue il programma scaricato. Tesseract include i modelli `ita`/`eng`; Whisper include Tiny multilingue. Ctrl+C annulla l'operazione. Lo script manuale `install-runtime-tools.ps1 -InstallDirectory <cartella-app> -Engine ffmpeg` richiama la stessa CLI; omettendo `-Engine` richiede **FFmpeg e Poppler**. La cartella dell'app serve a trovare la CLI, non è la destinazione dei motori.

Per audio/video → TXT installa **sia FFmpeg sia Whisper**: il comando CLI Whisper non aggiunge automaticamente FFmpeg. La trascrizione avviene sul PC, senza account o chiavi API. Senza impostazioni specifiche rileva automaticamente la lingua. `--language` sceglie la lingua dell'app, `--ocr-language` quella OCR; nessuno dei due sceglie la lingua del parlato. Per quest'ultima usa `speech.language` nel JSON.

`--pdf-action compress|split|merge` richiede destinazione PDF; `compress` e `split` accettano soltanto PDF, mentre `merge` accetta i file convertibili in PDF descritti nella [guida alle conversioni](conversions.md#comprimi-dividi-e-unisci-pdf). Il default è `compress`. L'unione CLI rispetta l'ordine degli argomenti e delle pagine dentro ogni sorgente; per intercalare singole pagine usa l'editor grafico. `--ocr` attiva il riconoscimento delle scansioni PDF nelle destinazioni testuali; immagini in TXT usano sempre OCR. `--ocr-language ita+eng` sceglie i modelli, senza attivare da solo OCR dei PDF; è indipendente da `--language`. `--csv-separator comma|semicolon|tab|pipe` sceglie il separatore sia in ingresso sia in uscita; il default è `comma`.

Esempio di impostazioni JSON (i campi omessi usano i default):

```json
{
  "format": "Mp3",
  "audio": { "bitrate_kbps": 256, "sample_rate": 48000, "channels": 2, "normalize": true },
  "video": { "trim_start": 5.0, "trim_end": 35.0 },
  "limits": { "max_bytes": 3000000, "max_duration": 30.0 },
  "media_options": { "threads": "1", "audio_volume": "0.5" }
}
```

`max_bytes` è il limite per ogni risultato: un output oltre il limite non viene pubblicato. `max_duration` limita gli ingressi audio/video dopo `trim_start`, insieme all'eventuale `trim_end`, anche per trascrizione; non tronca immagini o TXT usati come ingresso. Le opzioni FFmpeg sono valori tipizzati del catalogo, non argomenti arbitrari. Qui `audio_volume: "0.5"` dimezza l'ampiezza, circa −6 dB; il gain viene applicato dopo l'eventuale normalizzazione. Un ingresso non audio/video con `media_options` attive viene rifiutato, anche per immagine → GIF. Il catalogo disponibile si trova in **Profili → Avanzate**.

Esempio `speech-settings.json` per trascrivere il segmento 5–35 secondi in italiano con quattro thread CPU:

```json
{
  "format": "Txt",
  "video": { "trim_start": 5.0, "trim_end": 35.0 },
  "speech": { "language": "it", "threads": 4 },
  "limits": { "max_duration": 30.0, "max_bytes": 100000 }
}
```

```powershell
.\the-converter-cli.exe --convert --settings .\speech-settings.json --files "C:\Audio\registrazione.wav"
```

`video.trim_start` e `video.trim_end` valgono anche per l'audio. Ometti `speech.language` o usa `auto` per il rilevamento automatico; `it` e `en` selezionano italiano e inglese. Le opzioni gain/muto e `media_options` di codifica non sono ammesse per TXT. Il lavoro usa WAV mono 16 kHz e TXT temporanei in blocchi di massimo 10 minuti, rimossi a fine lavoro o annullamento. Il risultato finale non sovrascrive un file esistente.

Per usare una ricetta già salvata, `--quick --profile <id> --profile-revision <revisione> --files ...` usa il profilo corrente solo se la revisione coincide e gli ingressi sono ammessi. Il menu Explorer passa questi valori automaticamente. I profili normali accodano una copia delle impostazioni; un profilo PDF Merge richiede almeno due file e apre l'ordinamento pagine. `--custom-settings` apre l'editor compatibile o le opzioni della finestra senza avviare una conversione. Importazione ed esportazione dei profili avvengono dalla scheda **Profili**; il loro JSON portabile è distinto dal singolo `settings.json` di questo esempio.

[Torna al README](../README.md)
