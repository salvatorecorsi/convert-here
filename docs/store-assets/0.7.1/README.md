# Galleria corrente 0.7.1 — catture del 9 novembre 2026

Le due schermate mostrano **Impostazioni** e **Profili** della finestra nativa in italiano. Sono state acquisite tramite CUA dalla build **debug 0.7.0**, la cui interfaccia è invariata nella 0.7.1: non sono catture di una build 0.7.1. L'app era mostrata con **zoom nativo al 150%**; non è stato applicato ingrandimento alle immagini dopo la cattura. Il materiale precedente con puntatore o alone di acquisizione è escluso dalla galleria corrente e resta consultabile nello storico.

Gli originali JPEG della finestra misurano **2562 × 1430 pixel**. La CLI release **0.7.0** li ha convertiti in PNG senza ridimensionamento. Firma PNG e dimensioni di entrambi i file finali sono verificate; entrambe le catture sono state esaminate come pulite. Il cambio di formato non aggiunge dettagli a quelli della cattura originale.

| File PNG | Contenuto | Dimensioni | Byte | SHA-256 |
| --- | --- | --- | ---: | --- |
| [01-impostazioni-it.png](01-impostazioni-it.png) | Formati di uscita in cinque colonne e griglia dei motori | 2562 × 1430 | 421525 | `81d2495f697c2128d6aa1491c8bcca353e997be67f1c35f680f976e59f114f5e` |
| [02-profili-it.png](02-profili-it.png) | Manager dei profili, ricette e relativi controlli | 2562 × 1430 | 445756 | `92737b2301696a7944016458ef8891b996d3b0568d6542e17724bb0664980d99` |

Gli originali di acquisizione sono `01-impostazioni-it.jpg` (**163499 byte**, SHA-256 `5d9ffdab80faab164b5fc3ed80281d6c7cdcc21e34b4888619bf62c7c4dc8674`) e `02-profili-it.jpg` (**179827 byte**, SHA-256 `2485e62a3d8f881f868c607f2ee5f596a606624c84ac8ff5c05798297fec4d65`). Il binario debug usato per le catture è la copia locale `.tools/ui-manager-qa/bin/the-converter.exe`, SHA-256 `98952f18dc2966aaa2f1ebeb9ad231392f9afa11f31faf0afec5819c494ec8c7`; non è il candidato MSIX.

Queste immagini documentano esclusivamente l'interfaccia. Non attestano installazione del nuovo pacchetto Store, WACK, certificazione o funzionamento del menu su Windows 10 reale. Non aggiungono nuove prove di conversione, download o uso dei profili.

Vedi l'[indice Store](../README.md), l'[indice degli screenshot](../../screenshots/README.md) e il [registro storico 0.7.0](https://github.com/salvatorecorsi/convert-here/blob/v0.7.0/docs/store-assets/0.7.0/README.md).
