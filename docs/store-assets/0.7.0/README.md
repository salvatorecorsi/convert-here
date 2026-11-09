# Catture native 0.7.0 — 9 novembre 2026

Le immagini di questo registro sono storiche. Dalla 0.7.1 sono state ritirate dalle gallerie e dai file correnti perché contengono il puntatore della cattura con alone blu. I collegamenti rimandano allo snapshot immutato [v0.7.0](https://github.com/salvatorecorsi/convert-here/tree/v0.7.0/docs/store-assets/0.7.0); provenienza e digest qui sotto restano riferiti agli originali.

Schermate della build **debug 0.7.0** di Convert Here eseguita su Windows durante la QA locale del **9 novembre 2026**. I sorgenti dell'interfaccia sono quelli destinati alla successiva build release. Le immagini non attestano una build release già distribuita, l'installazione del nuovo MSIX, la certificazione Store o un collaudo su Windows 10.

Entrambi i file sono effettivamente **JPEG**, **1722 × 959 pixel**; formato e dimensioni sono stati verificati leggendo i file. Le catture documentano la finestra nativa in italiano e restano distinte dalle schermate delle release precedenti.

| File | Contenuto | Byte | SHA-256 |
| --- | --- | ---: | --- |
| [01-impostazioni-it.jpg](https://github.com/salvatorecorsi/convert-here/blob/v0.7.0/docs/store-assets/0.7.0/01-impostazioni-it.jpg) | Scheda Impostazioni: formati disposti in colonne e griglia dei motori | 90601 | `a1d904f3a8f4f66a70f02eb0bf8bbe32c1362c03f5df8d125b575f6cb182f335` |
| [02-profili-it.jpg](https://github.com/salvatorecorsi/convert-here/blob/v0.7.0/docs/store-assets/0.7.0/02-profili-it.jpg) | Scheda Profili e nuovo manager delle ricette personali | 102278 | `b9f8d44dd894f3591989fc361cc1b19d2f034ee82cb8c6e205529dcbc37f58c1` |

La QA ha verificato le quattro schede, il rifiuto atomico dell'importazione non valida e l'importazione di quattro profili validi. Una conversione rapida FLAC → MP3 con limite di 30 secondi ha prodotto 721649 byte e durata FFprobe 30.000000 secondi; la cronologia è stata osservata aggiornata anche da un'altra finestra. Le schermate non sostituiscono queste prove funzionali.

Per il caricamento Store, che accetta PNG, la CLI release 0.7.0 ha convertito queste catture conservando dimensioni e pixel decodificati. Le due varianti PNG sono rispettivamente **272102 byte**, SHA-256 `0cdf4d297505baf80977ddf42872056b3ac4e6931d2f8919d6f6327a7644f25e`, e **261864 byte**, SHA-256 `20df8de1fee5919a38d48489ccb34111e3d2c7b1565da2a96e1ac3b759f24604`. Sono caricate nelle schede IT/EN insieme alle catture riutilizzate di PDF Merge e della conferma download, la cui interfaccia resta invariata.

Vedi il [resoconto di validazione](../../store-validation.md), l'[indice Store](../README.md) e l'[indice degli screenshot](../../screenshots/README.md). La [release GitHub 0.7.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.0) è pubblicata; Store 1.0.6.0 è in certificazione, con pubblicazione automatica dopo l'approvazione.
