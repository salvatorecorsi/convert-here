# Catture Store · Convert Here 0.3.0

Catture native Windows del 4 novembre 2026, ottenute dalla UI del commit `d6ad0171dd002d01136e58b511f5e9e5e6dbfffb`. L'harness locale usa i sorgenti live della 0.3.0, con preferenze e coda isolate in `.tools/fluent-harness/data/store-0.3.0/`, file dimostrativi senza dati personali e viewport interno 1536 × 880, non massimizzato. Le immagini includono la cornice nativa della finestra.

I JPEG restituiti dalla cattura sono stati convertiti con la CLI release 0.3.0 (`--convert --target png`), senza ridimensionamenti, ritocchi o sovrapposizioni. Verificati firma PNG, dimensioni e uguaglianza di tutti i pixel RGBA decodificati rispetto ai JPEG acquisiti.

| File | Dimensioni | Byte | SHA-256 |
| --- | --- | ---: | --- |
| [01-conversione-immagini.png](01-conversione-immagini.png) | 1538 × 911 | 228513 | `aa1d28f3b7d653bf77d063200310fb7fc6e6df23bb041c48fc7132ecd3d3e4db` |
| [02-pdf-merge.png](02-pdf-merge.png) | 1538 × 911 | 203167 | `c33fc7538a654d229bdd36881c8a893addddf553ecb23122a30b67132e9d634b` |

| File | Didascalia IT | Caption EN |
| --- | --- | --- |
| 01-conversione-immagini.png | Conversione locale di due immagini in JPEG completata, con gli originali conservati. | Two images converted locally to JPEG, with the original files preserved. |
| 02-pdf-merge.png | Unione di immagini e PDF: trascina le singole pagine nell'ordine desiderato prima di creare un PDF unico. | Merge images and PDFs: drag individual pages into the desired order before creating one PDF. |

La UI mostrata è italiana; le didascalie inglesi descrivono le stesse schermate. Le catture documentano l'interfaccia eseguita nell'harness isolato: non attestano l'installazione, il collaudo o la certificazione del pacchetto MSIX.
