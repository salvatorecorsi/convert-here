# Convert Here 0.7.1 — menu Store e schermate

Applicazione **0.7.1 [pubblicata su GitHub](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.1)** il **9 novembre 2026 alle 21:08:01 UTC**, commit `2c6d6968159a46baef37d8f20aef049e7be4c8c4`; [Windows checks 42698306310](https://github.com/salvatorecorsi/convert-here/actions/runs/42698306310) **SUCCESS**. Sviluppatore e publisher: **Salvatore Corsi**. [English](release-notes-0.7.1.en.md).

Il pacchetto Microsoft Store **1.0.7.0** è stato caricato, validato e inviato nella **Submission 4**, ID `1152921505702358188`. Alle **21:09 UTC** del 9 novembre 2026 il portale mostrava **Update in certification / Product update: In certification**: Submission Complete, Pre-processing In progress, Certification Not started, Publishing Not started. È selezionata la pubblicazione automatica dopo l'approvazione; la 0.7.1 non è ancora approvata o pubblicata nello Store. Il precedente aggiornamento 0.7.0 / Store 1.0.6.0 era già approvato.

Il manifest del menu Explorer usa l'identificatore del comando **`0000ConvertHere`** come workaround per il mancato rilevamento del menu del pacchetto Store su Windows 10. Restano invariati nome visibile, identità del pacchetto, CLSID, impostazioni e azioni di conversione. Il requisito resta **Windows 10 versione 2004/build 19041 o successivo, x64**, incluso Windows 11.

La modifica dell'identificatore non equivale a una verifica del menu su Windows 10 reale: **non è disponibile un host Windows 10 di prova e la risoluzione su quel sistema non è ancora attestata**. Il controllo `scripts/test-packaged-context-menu.ps1` distingue attivazione COM packaged e costruzione del menu classico effettivo dopo l'installazione del MSIX; un esito su Windows 11 non dimostra il comportamento di Windows 10.

La [galleria corrente](store-assets/0.7.1/README.md) contiene due nuove catture di Impostazioni e Profili, **2562 × 1430 pixel**, con zoom nativo al **150%** e senza ingrandimento successivo. Provengono dalla build debug **0.7.0**, con interfaccia invariata nella 0.7.1; non attestano una cattura della nuova build. Provenienza, formato PNG e digest sono registrati nella galleria.

Le funzioni della [0.7.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.0) restano invariate: quattro schede, manager dei profili e import/export JSON, editor immagini e fogli A4/A5, limiti di peso e durata, controlli FFmpeg avanzati, cronologia, **44 destinazioni e 93 estensioni riconosciute**. Conversioni e OCR restano locali e gratuiti. I motori opzionali non sono inclusi; il download facoltativo di FFmpeg/Poppler richiede conferma e Internet, e le risorse HTML collegate possono accedere alla rete tramite LibreOffice.

La release comprende `ConvertHere-0.7.1-windows-x64-setup.exe`, `ConvertHere-0.7.1-windows-x64-portable.zip`, `ConvertHere-0.7.1-source.zip` con dipendenze Cargo vendorizzate e `SHA256SUMS.txt`. Il codice originale resta **GPL-3.0-only**; dipendenze, font e motori conservano le proprie licenze. Nessun pagamento è richiesto per continuare a usare Convert Here. Tag, artefatti e report della 0.7.0 non vengono riscritti.

Riferimenti: [istruzioni e prove](store-testing-0.7.1.md), [stato Store](store-submission.md), [integrazione Windows](windows-integration.md), [privacy](privacy.md).
