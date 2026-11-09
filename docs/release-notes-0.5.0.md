# Convert Here 0.5.0 — italiano e inglese

**Italiano** · [English](release-notes-0.5.0.en.md)

Sviluppato da **Salvatore Corsi**. La release GitHub 0.5.0 porta l'interfaccia bilingue nei pacchetti installer e portable pronti all'uso.

## Scegli la lingua

Apri **Impostazioni → Lingua** e scegli **Sistema**, **Italiano** o **English**. Sistema usa l'italiano quando la lingua di visualizzazione di Windows è italiana e l'inglese negli altri casi. Il cambio si applica subito alla finestra dell'app e resta salvato per gli avvii successivi.

Sono tradotti anche il menu **Converti in / Convert to**, le finestre di stato e unione PDF, i dialoghi dei motori, gli avvisi, l'avanzamento e i messaggi applicativi. Il menu di Esplora file rilegge la scelta salvata. I messaggi prodotti da Windows o dai motori esterni possono seguire le rispettive impostazioni.

La lingua dell'interfaccia è separata da quella dell'OCR. Tesseract continua a usare i modelli scelti, con `ita+eng` come impostazione predefinita.

## CLI e compatibilità

Per scegliere la lingua della singola esecuzione, senza cambiare la preferenza salvata:

```powershell
.\the-converter-cli.exe --language en --help
.\the-converter-cli.exe --language it --convert --target jpg --files "C:\Immagini\foto.png"
```

`--language system|it|en` si applica anche alla diagnostica e alle conversioni rapide. Ogni lavoro rapido conserva l'eventuale scelta esplicita della CLI; gli altri lavori seguono le preferenze salvate. Preferenze e lavori creati dalle versioni precedenti restano leggibili. Formati, chiavi JSON, opzioni CLI e nomi dei risultati conservano i propri identificativi.

La release include **44 formati di destinazione** e **93 estensioni di ingresso riconosciute**, conversioni multiple, OCR locale, operazioni PDF e integrazione con Esplora file. Le guide e il [README inglese](../README.en.md) documentano le stesse funzioni della versione italiana e includono quattro [catture reali in inglese](screenshots/README.md).

## Scarica e installa

Dalla [release GitHub 0.5.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.5.0):

- `ConvertHere-0.5.0-windows-x64-setup.exe`: installazione per l'utente corrente, senza privilegi amministrativi; menu contestuale facoltativo.
- `ConvertHere-0.5.0-windows-x64-portable.zip`: estrai in una cartella stabile e avvia `the-converter.exe`; include CLI e script del menu.
- `ConvertHere-0.5.0-source.zip`: sorgenti corrispondenti, entrambi i lockfile, dipendenze Cargo vendorizzate e istruzioni di build offline.
- `SHA256SUMS.txt`: checksum SHA-256 di installer, portable e sorgenti.

Requisiti: **Windows 10 build 19041 o successivo, x64**. Su Windows 11 il menu classico della distribuzione GitHub si trova in **Mostra altre opzioni**. I pacchetti GitHub non sono firmati Authenticode.

I pacchetti non includono motori esterni. Immagini native, immagini in PDF e operazioni sui PDF esistenti funzionano subito; per audio/video, esportazione PDF, documenti Office, OCR e HEIC/AVIF configura i [motori richiesti](configuration.md). FFmpeg e Poppler si possono scaricare dall'app dopo conferma di fonte, versione, dimensioni, licenze e destinazione. Conversioni e OCR avvengono sul PC; il download dei motori non carica i file da convertire.

## Licenza

Convert Here è gratuito, senza account, abbonamenti o acquisti in-app. Copyright (C) 2026 Salvatore Corsi. Il codice originale è distribuito con **GPL-3.0-only**, senza garanzia nei termini della [licenza completa](../LICENSE). Dipendenze, font e motori conservano le rispettive licenze. Vedi anche l'[informativa privacy](privacy.md).

[README italiano](../README.md) · [README inglese](../README.en.md)
