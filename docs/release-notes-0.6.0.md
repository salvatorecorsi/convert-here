# Convert Here 0.6.0 — editor locale e profili personali

Editore e sviluppatore: **Salvatore Corsi**. Materiale della candidata **0.6.0**, pacchetto Store previsto **1.0.5.0**: questo documento non attesta la pubblicazione della release o l'invio allo Store. [English release notes](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/release-notes-0.6.0-en.md).

## Modifica le immagini prima di convertirle

**Modifica…** nella riga di un'immagine apre l'editor locale. Ritaglia nell'anteprima o con percentuali, ridimensiona mantenendo le proporzioni oppure crea una tela con risoluzione, colore, ancoraggio e spostamenti scelti. Esporta **PNG, JPEG, WebP o PDF**. Per JPEG e WebP puoi regolare la qualità; il codec WebP è integrato e lavora sul PC.

L'anteprima delle immagini con perdita viene decodificata dai byte già compressi. Il peso mostrato è quello effettivo del risultato e **Salva con nome…** scrive gli stessi byte, senza sovrascrivere file esistenti o l'originale. Puoi impostare un peso massimo: **300 kB = 300000 byte**. Per JPEG e WebP il motore cerca una qualità che rispetti il limite; la riduzione ulteriore delle dimensioni richiede un consenso esplicito nelle opzioni. Se il limite non è raggiungibile, il salvataggio resta disattivato e una conversione rapida con quel profilo termina con un errore, senza produrre un risultato oltre il limite. Per i PDF regola qualità, dimensioni e DPI.

## Immagini e testo su A4 o A5

Scegli **A4 o A5**, orientamento, margini e adattamento dell'immagine. **Nuovo foglio…** permette di scrivere testo semplice; **Modifica…** su un TXT apre un file **UTF-8** nell'impaginatore nativo. Dimensione del testo, interlinea e allineamento sono regolabili, con ritorni a capo e pagine automatiche. Il PDF incorpora Ubuntu Light e conserva testo selezionabile; i caratteri non supportati sono segnalati senza eliminarli in silenzio.

L'impaginazione semplice non richiede LibreOffice. I documenti Word mantengono la conversione Office esistente; l'editor non modifica liberamente il contenuto dei PDF o dei DOCX. Le immagini animate e i contenitori multipli usano il primo fotogramma o la prima immagine.

## Riusa le opzioni da Esplora file

**Salva le mie impostazioni** conserva una ricetta con nome: fino a **32 profili**, di cui **8 visibili in Converti in**. Puoi applicarli, aggiornarli, rinominarli o usarli come predefiniti dalle Impostazioni. Il profilo conserva parametri e un'eventuale cartella di destinazione, senza l'immagine di prova, il testo scritto o le anteprime.

I profili compaiono nel menu per selezioni compatibili, quando il formato di uscita è abilitato. Una conversione rapida entra nella coda con la ricetta salvata al momento dell'invio. **Crea «Salva per web»** prepara un profilo WebP da 300 kB, qualità massima 90 e riduzione delle dimensioni consentita. Restano **44 destinazioni**, **93 estensioni in ingresso** e le otto uscite predefinite.

## Locale e gratuito

Editor, impaginazione, conversioni e OCR sono elaborati sul PC. I motori facoltativi mantengono i requisiti esistenti: FFmpeg per audio/video, Poppler per esportare PDF, LibreOffice per Office/Excel/HTML, libheif per HEIC/AVIF e Tesseract con modelli per OCR. Il download esplicito di FFmpeg/Poppler usa HTTPS, verifica gli archivi e richiede Internet; non carica i file da convertire. Le risorse collegate nei file HTML possono accedere alla rete.

Convert Here è gratuito, senza abbonamenti, acquisti in-app o costi a consumo. **Nessun pagamento richiesto per continuare a usarlo.** Il codice originale è **GPL-3.0-only**; dipendenze, font e motori conservano le rispettive licenze e i diritti sui file convertiti restano invariati.

Artefatti previsti per Windows x64: `ConvertHere-0.6.0-windows-x64-setup.exe`, `ConvertHere-0.6.0-windows-x64-portable.zip`, `ConvertHere-0.6.0-source.zip` e `SHA256SUMS.txt`. L'archivio sorgente è previsto con dipendenze Cargo vendorizzate e istruzioni offline. La distribuzione GitHub richiede Windows 10 build 19041 o successivo; il pacchetto Store separato richiede Windows 11 x64. Gli artefatti delle release precedenti restano immutati.

Dettagli: [guida editor e profili](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/image-editor-profiles.md), [privacy](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/privacy.md), [licenza](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/LICENSE), [stato delle verifiche](https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/store-validation.md).
