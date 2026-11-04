# Convert Here 0.4.0 — formati visibili e motori su richiesta

Editore e sviluppatore: **Salvatore Corsi**.

Nelle **Impostazioni** scegli quali formati di uscita mostrare nel selettore della GUI e nel menu di Esplora file. Il default e **MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4**; puoi abilitare le altre destinazioni. Le preferenze restano salvate e il menu viene aggiornato senza riavviare Explorer. Le **93 estensioni in ingresso** e le **44 destinazioni del motore** restano supportate: il filtro di visualizzazione non blocca la CLI, richieste rapide esplicite o lavori gia accodati.

In **Impostazioni → Motori** puoi richiedere FFmpeg, Poppler o entrambi, con conferma di fonte, versione, dimensioni, licenze e destinazione. Il download HTTPS usa le distribuzioni Gyan e Poppler per Windows ospitate su GitHub e relativi CDN, verifica lo SHA-256 prima dell'estrazione e conserva licenze e documentazione. I motori sono estratti nella cartella utente `%LOCALAPPDATA%\ConvertHere\tools`; resta riconosciuta in lettura la precedente cartella dei motori.

Per scaricare serve Internet: i distributori ricevono IP e normali dati HTTP, incluso User-Agent e versione dell'app. Non vengono caricati i file da convertire. Conversioni e OCR restano sul PC. LibreOffice, libheif, Tesseract e modelli OCR si configurano da installazioni locali. Nessun motore e incluso nel pacchetto dell'app.

Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. Nessun pagamento richiesto per continuare a usarlo.

Il codice originale e ora distribuito con **GPL-3.0-only**, Copyright (C) 2026 Salvatore Corsi. Il [testo integrale della licenza](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/LICENSE) e il [repository dei sorgenti](https://github.com/salvatorecorsi/convert-here/tree/v0.4.0) sono accessibili anche dalle informazioni dell'app. Dipendenze, font e motori mantengono le proprie licenze. La conversione non cambia i diritti sui contenuti dell'utente.

Distribuzione per Windows x64:

- `ConvertHere-0.4.0-windows-x64-setup.exe`
- `ConvertHere-0.4.0-windows-x64-portable.zip`
- `ConvertHere-0.4.0-source.zip`, con sorgenti, dipendenze Cargo vendorizzate e istruzioni di build offline
- `SHA256SUMS.txt`

La distribuzione GitHub richiede Windows 10 build 19041 o successivo, x64; il pacchetto Store separato richiede Windows 11 x64. I tag e gli artefatti precedenti rimangono immutati.

Dettagli: [README](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/README.md), [privacy](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/docs/privacy.md), [verifiche](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/docs/store-validation.md). La [release 0.3.2](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.2) conserva i propri artefatti e metadati.
