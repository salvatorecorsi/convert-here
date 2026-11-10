# Convert Here 0.8.1 — configurazione guidata dei motori

**Bozza della candidata 0.8.1**, pacchetto Store previsto **1.0.9.0**. Build, artefatti, verifica finale e pubblicazione non ancora attestati. La precedente **0.8.0 / Store 1.0.8.0 è pubblicata**. Sviluppatore e publisher: **Salvatore Corsi**. [English](release-notes-0.8.1.en.md).

In **Impostazioni → Formati di uscita**, una checkbox **Seleziona tutti** in fondo a ogni categoria sostituisce i due pulsanti con spunta e croce. Attivandola selezioni tutti i formati della categoria; disattivandola li deselezioni. Le singole estensioni restano modificabili: togliendone una, **Seleziona tutti** si disattiva. Il filtro riguarda le uscite; i formati in ingresso restano supportati. Le otto uscite predefinite restano MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4.

Al primo avvio della finestra principale senza file o formato preimpostato, una **configurazione guidata** propone i motori mancanti. Puoi scegliere **FFmpeg**, **Poppler**, **Tesseract con italiano/inglese**, **libheif** e **Whisper con Tiny multilingue**; quelli mancanti sono preselezionati. **LibreOffice** offre il collegamento al sito ufficiale e resta un'installazione separata. Dopo la conferma di fonti, versioni, dimensioni, licenze e destinazione, i motori scelti vengono scaricati **uno alla volta**, riusando verifica SHA-256 e installazione nella cartella personale.

Saltando o completando la procedura viene creato il marker locale **`engine-setup-v1.done`**, accanto alle preferenze. Memorizzare questa scelta non riscrive `preferences.json` e non può essere annullato da una vecchia finestra che salva impostazioni obsolete. Le installazioni riuscite conservano normalmente i percorsi dei motori nelle preferenze. La procedura può essere riaperta da **Impostazioni → Motori → Configurazione guidata**; CLI, conversioni rapide e impostazioni personalizzate non la aprono automaticamente.

Nessun download parte prima della conferma. Motori, modelli, licenze e conversioni restano quelli della 0.8.0. [Guida alla configurazione](configuration.md), [verifiche da completare](store-testing-0.8.1.md).
