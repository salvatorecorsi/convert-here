# Catture native 0.8.1

Due schermate acquisite il **10 novembre 2026** dalla build **debug nativa 0.8.1**, con finestra **2562 × 1441 pixel** e zoom dell'app al **150%**. Dimensioni e formati sono stati letti dai file con Pillow. Non sono ingrandimenti successivi: puntatore e alone sono stati posizionati fuori dalla finestra prima delle catture, senza ritoccare le immagini.

| PNG | Contenuto | Byte | SHA-256 |
| --- | --- | --- | --- |
| [01-wizard-it.png](01-wizard-it.png) | Prima apertura della configurazione guidata: cinque motori, dipendenza FFmpeg di Whisper, dimensioni, sito LibreOffice, Salta e Continua. | 558733 | `91530f8c1f9bd592c31825346e224ed6d93fae748a54784471544fd453fba4fd` |
| [02-formati-it.png](02-formati-it.png) | Cinque categorie con Seleziona tutti in fondo a ogni lista, otto uscite predefinite e Impostazioni a destra. | 423770 | `42323f196a3e93840aec4b27c9e8bcf021d2f8b9dcbbb713f2d5a003fb2adc1b` |

Gli originali della cattura sono i JPEG omonimi nella stessa cartella. La CLI **Convert Here 0.8.1** li ha convertiti in PNG senza ridimensionamento o ritaglio. [captures.json](captures.json) registra dimensioni, byte e SHA-256 di tutti e quattro i file. Binario GUI di cattura: **`bfba78128e3ee6bf10bf38b71a7deb3a4d31b3af65a2b12bc200d580919bd969`**, dalla working tree 0.8.1 non ancora committata, basata su `1d75f5298db01bb134c63c232b6a43c42d2590bd`.

L'interfaccia mostrata è **italiana**, anche nella galleria proposta per la scheda inglese. Gli strumenti e le preferenze visibili appartengono a un ambiente di prova isolato. La schermata del wizard documenta la scelta prima del download: **non è stato premuto il pulsante Download**, quindi questa QA non attesta un download o un'installazione reale attraverso la nuova procedura. Le [verifiche 0.8.1](../../store-testing-0.8.1.md) distinguono prove automatiche e interazione nativa.

## Galleria Store salvata nella Submission 6

| Ordine | File | Provenienza |
| --- | --- | --- |
| 1 | [02-formati-it.png](02-formati-it.png) | Nuova cattura 0.8.1; checkbox collettive in fondo alle categorie. |
| 2 | [01-wizard-it.png](01-wizard-it.png) | Nuova cattura 0.8.1; selezione dei motori prima della conferma. |
| 3 | [03-timeline-it.png](../0.8.0/03-timeline-it.png) | Cattura storica 0.8.0 riusata: editor audio/video invariato, interfaccia italiana. |
| 4 | [04-trascrizione-it.png](../0.8.0/04-trascrizione-it.png) | Cattura storica 0.8.0 riusata: editor di trascrizione invariato, interfaccia italiana. |

I digest e la provenienza delle ultime due immagini restano nel [registro 0.8.0](../0.8.0/captures.json) e sono richiamati nel nuovo `captures.json`. Nessuna cattura attesta installazione del nuovo MSIX, prova su host Windows 10 reale o WACK. Le schede IT/EN della **Submission 6**, ID **1152921505702367000**, contengono queste quattro PNG e didascalie salvate e rilette dal server. La release GitHub 0.8.1 è pubblicata dalle **13:42:06 UTC** del 10 novembre 2026; il pacchetto Store **1.0.9.0** è stato inviato e mostra **Update in certification** alle **13:44:28 UTC**, con pubblicazione automatica dopo approvazione. La nuova 1.0.9.0 non è ancora dichiarata approvata o pubblica; la precedente 1.0.8.0 è pubblicata. Gli artefatti finali derivano dal commit pulito **`07fe5f749327b73e93a92d5101d80b9b4f5317cb`**; questo dato non cambia la provenienza debug non committata delle catture.
