# Riga di comando

**Italiano** · [English](command-line.en.md)

La CLI di Convert Here permette di convertire file, automatizzare elaborazioni e installare i motori opzionali. Esegui gli esempi PowerShell dalla cartella dell'app. Per i requisiti delle singole conversioni, consulta la [guida ai formati](conversions.md).

Usa l'eseguibile console dedicato, che restituisce output, diagnostica e codici di uscita affidabili:

```powershell
.\the-converter-cli.exe --language it --diagnose
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

`--language system|it|en` sceglie la lingua dell'interfaccia e dei messaggi applicativi per l'esecuzione corrente senza cambiare le preferenze salvate. Senza override viene usata la scelta salvata. `system` segue Windows: italiano per la lingua di visualizzazione italiana, inglese altrimenti. Questa opzione è disponibile dalla versione 0.5.0. Nomi delle opzioni, chiavi JSON e suffissi dei risultati restano stabili fra le lingue. I messaggi di Windows e dei motori esterni possono avere una lingua propria.

`--convert` lavora senza UI, continua sugli altri file dopo un errore e termina con codice non zero se almeno una conversione fallisce. Ctrl+C annulla il processo e pulisce i temporanei. I percorsi sono passati direttamente ai processi senza interpolazione in una shell.

`--install-runtime ffmpeg|poppler` è una modalità separata: richiede un download esplicito del motore fissato e verificato, senza opzioni di conversione o file in ingresso. Scrive nella cartella personale dei motori, conserva licenze e documentazione e non esegue il programma scaricato. Ctrl+C annulla l'operazione. Lo script manuale `install-runtime-tools.ps1 -InstallDirectory <cartella-app> -Engine ffmpeg` richiama la stessa CLI; omettendo `-Engine` richiede entrambi i motori. La cartella dell'app serve a trovare la CLI, non è la destinazione dei motori.

`--pdf-action compress|split|merge` richiede destinazione PDF; `compress` e `split` accettano soltanto PDF, mentre `merge` accetta i file convertibili in PDF descritti nella [guida alle conversioni](conversions.md#comprimi-dividi-e-unisci-pdf). Il default è `compress`. L'unione CLI rispetta l'ordine degli argomenti e delle pagine dentro ogni sorgente; per intercalare singole pagine usa l'editor grafico. `--ocr` attiva il riconoscimento delle scansioni PDF nelle destinazioni testuali; immagini in TXT usano sempre OCR. `--ocr-language ita+eng` sceglie i modelli, senza attivare da solo OCR dei PDF; è indipendente da `--language`. `--csv-separator comma|semicolon|tab|pipe` sceglie il separatore sia in ingresso sia in uscita; il default è `comma`.

Esempio di impostazioni JSON (i campi omessi usano i default):

```json
{
  "format": "Mp3",
  "audio": { "bitrate_kbps": 256, "sample_rate": 48000, "channels": 2, "normalize": true },
  "video": { "trim_start": 5.0, "trim_end": 35.0 }
}
```

[Torna al README](../README.md)
