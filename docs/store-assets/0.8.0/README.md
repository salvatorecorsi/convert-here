# Catture native 0.8.0

Quattro schermate acquisite il **10 novembre 2026** dalla build **debug nativa 0.8.0**, con finestra **2562 × 1441 pixel** e zoom dell'app al **150%**. Non sono ingrandimenti successivi. Il puntatore e il relativo alone sono stati posizionati fuori dalla finestra prima di ogni cattura; le immagini non sono state ritoccate.

| PNG per lo Store | Contenuto |
| --- | --- |
| [01-formati-it.png](01-formati-it.png) | Schede senza freccia iniziale; File, Cronologia e Profili a sinistra, Impostazioni a destra. Formati in cinque colonne; TXT abilitato esplicitamente per la prova. |
| [02-motori-it.png](02-motori-it.png) | Griglia dei sei motori, con installazione portabile di Tesseract, libheif e Whisper/Tiny. |
| [03-timeline-it.png](03-timeline-it.png) | Clip dimostrativa con forma d'onda, fotogrammi e intervallo scelto trascinando le maniglie; controllo del gain. |
| [04-trascrizione-it.png](04-trascrizione-it.png) | Destinazione TXT, lingua di trascrizione e stesso intervallo della timeline; gain e muto disabilitati per il testo. |

L'interfaccia nelle immagini è **italiana**, anche quando queste catture vengono usate nella scheda inglese. L'app offre anche l'inglese. La clip contiene un pattern video generato con FFmpeg e parlato sintetico di prova; non contiene file personali. Gli otto formati predefiniti del prodotto restano invariati: il TXT visibile è una preferenza del solo ambiente di prova.

Gli originali restituiti dalla cattura nativa sono i quattro JPEG nella stessa cartella. La CLI **Convert Here 0.8.0** li ha convertiti in PNG, senza ridimensionamento o ritaglio. [captures.json](captures.json) registra dimensioni, byte, SHA-256 e hash del binario GUI usato (`001acf141b7b8e3f7c976bf2d4bbbf6d1cd0484e1cbf6a2b65d32f6639a5ec97`). La cattura precede il commit della release; non è una cattura del MSIX installato e non attesta collaudo Windows 10 o WACK.

Le copie dei motori mostrate come disponibili appartengono all'ambiente locale di prova: i pacchetti dell'app non includono motori o modelli. La cattura della griglia non costituisce, da sola, una prova di installazione; download e motori sono stati verificati separatamente come descritto nelle [istruzioni di test 0.8.0](../../store-testing-0.8.0.md).

Il **10 novembre 2026** le quattro PNG sono state caricate, salvate e rilette nelle schede italiana e inglese della **Submission 5**, ID **1152921505702365703**. Le didascalie inglesi dichiarano l'interfaccia italiana. Questo record attesta la persistenza delle immagini nel portale, non approvazione o pubblicazione Store. Le catture e i relativi hash restano quelli di `captures.json`; gli artefatti finali della release derivano separatamente dal commit **`f1f3f95fceed52507ef715fa4ac6893acc63544a`**.

La release GitHub 0.8.0 è pubblicata dalle **12:09:39 UTC**; la Submission 5 è stata inviata intorno alle **12:10 UTC** e mostra **Update in certification**. Questi stati non trasformano le catture debug in una prova di installazione o approvazione del MSIX.
