# Catture Store · Convert Here 0.3.1

Catture native Windows del 4 novembre 2026, ottenute dai sorgenti UI della 0.3.1 con un harness locale rigenerato. Preferenze e coda sono isolate dalle cartelle personali; i file sono dimostrativi, senza dati personali. Il viewport interno e 1536 × 880, non massimizzato; le immagini includono la cornice nativa della finestra. Riferimento della release: [v0.3.1](https://github.com/salvatorecorsi/convert-here/tree/v0.3.1).

I JPEG restituiti dalla cattura sono stati convertiti in PNG con la CLI dell'app 0.3.1, senza ridimensionamenti, ritocchi o sovrapposizioni. Verificati firma PNG, dimensioni 1538 × 911 e uguaglianza di tutti i pixel RGBA decodificati rispetto ai JPEG acquisiti.

| File | Dimensioni | Byte | SHA-256 |
| --- | --- | ---: | --- |
| [01-conversione-immagini.png](01-conversione-immagini.png) | 1538 × 911 | 228134 | `f30c2fa16ee0c2e571a2cda42aebbf1aaddc2a8a71b7d7d856abeb5b5e33f583` |
| [02-pdf-merge.png](02-pdf-merge.png) | 1538 × 911 | 212074 | `55fe63e32826b359119eed96320c018ac07003079d5258cbeb78faea481f18b6` |
| [03-strumenti.png](03-strumenti.png) | 1538 × 911 | 290302 | `29b69e071e48c5d25c7ef898ef0ac18adeeca128300e0b4a4a9efdb0df9cfb98` |

Verifiche native della sessione:

- Conversione di due PNG in JPEG completata, con zero errori.
- Pagina Strumenti con controlli Aspetto, motori locali e la dicitura: «Convert Here è gratuito, senza abbonamenti, acquisti in-app o costi a consumo».
- PDF Merge preparato da tre sorgenti per cinque pagine. L'immagine Illustrazione e stata trascinata fra la prima e la seconda pagina del PDF; il risultato unico di 118653 byte contiene cinque pagine, con numero e ordine verificati tramite `pdfinfo` e `pdftotext`.

| File | Didascalia IT | Caption EN |
| --- | --- | --- |
| 01-conversione-immagini.png | Conversione locale di due immagini in JPEG completata, con gli originali conservati. | Two images converted locally to JPEG, with the original files preserved. |
| 02-pdf-merge.png | Unione di immagini e PDF: trascina le singole pagine nell'ordine desiderato prima di creare un PDF unico. | Merge images and PDFs: drag individual pages into the desired order before creating one PDF. |
| 03-strumenti.png | Configura aspetto e motori locali opzionali: Convert Here e gratuito, senza abbonamenti, acquisti in-app o costi a consumo. | Configure appearance and optional local engines: Convert Here is free, with no subscriptions, in-app purchases or usage-based charges. |

La UI mostrata e italiana; le didascalie inglesi descrivono le stesse schermate. I motori visibili appartengono al PC di prova e non sono inclusi nel pacchetto Store. Le catture documentano l'app eseguita nell'harness isolato: non attestano installazione, collaudo o certificazione del MSIX.
