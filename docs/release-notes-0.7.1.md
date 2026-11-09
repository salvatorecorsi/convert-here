# Convert Here 0.7.1 — menu Store e schermate

**Versione candidata, in preparazione.** Applicazione **0.7.1**, pacchetto Microsoft Store previsto **1.0.7.0**. Queste note non attestano pubblicazione GitHub, caricamento o invio del nuovo pacchetto Store. Sviluppatore e publisher: **Salvatore Corsi**. [English](release-notes-0.7.1.en.md).

Il manifest del menu Explorer usa l'identificatore del comando **`0000ConvertHere`** come workaround per il mancato rilevamento del menu del pacchetto Store su Windows 10. Restano invariati nome visibile, identità del pacchetto, CLSID, impostazioni e azioni di conversione. Il requisito resta **Windows 10 versione 2004/build 19041 o successivo, x64**, incluso Windows 11.

La modifica dell'identificatore non equivale a una verifica del menu su Windows 10 reale: **non è disponibile un host Windows 10 di prova e la risoluzione su quel sistema non è ancora attestata**. Il controllo `scripts/test-packaged-context-menu.ps1` distingue attivazione COM packaged e costruzione del menu classico effettivo dopo l'installazione del MSIX; un esito su Windows 11 non dimostra il comportamento di Windows 10.

La [galleria corrente](store-assets/0.7.1/README.md) contiene due nuove catture di Impostazioni e Profili, **2562 × 1430 pixel**, con zoom nativo al **150%** e senza ingrandimento successivo. Provengono dalla build debug **0.7.0**, con interfaccia invariata nella 0.7.1; non attestano una cattura della nuova build. Provenienza, formato PNG e digest sono registrati nella galleria.

Le funzioni della [0.7.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.0) restano invariate: quattro schede, manager dei profili e import/export JSON, editor immagini e fogli A4/A5, limiti di peso e durata, controlli FFmpeg avanzati, cronologia, **44 destinazioni e 93 estensioni riconosciute**. Conversioni e OCR restano locali e gratuiti. I motori opzionali non sono inclusi; il download facoltativo di FFmpeg/Poppler richiede conferma e Internet, e le risorse HTML collegate possono accedere alla rete tramite LibreOffice.

I pacchetti previsti sono `ConvertHere-0.7.1-windows-x64-setup.exe`, `ConvertHere-0.7.1-windows-x64-portable.zip`, `ConvertHere-0.7.1-source.zip` con dipendenze Cargo vendorizzate e `SHA256SUMS.txt`. Il codice originale resta **GPL-3.0-only**; dipendenze, font e motori conservano le proprie licenze. Nessun pagamento è richiesto per continuare a usare Convert Here. Tag, artefatti e report della 0.7.0 non vengono riscritti.

Riferimenti: [prove della candidata](store-testing-0.7.1.md), [stato Store](store-submission.md), [integrazione Windows](windows-integration.md), [privacy](privacy.md).
