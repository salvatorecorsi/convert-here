# Release 0.6.0 / candidato Store 1.0.5.0 — istruzioni e prove

La [release GitHub 0.6.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.6.0) è pubblicata dal **9 novembre 2026, 18:28:13 UTC**, commit `9b959405e64e62f1f05f49a821bb9444747b3620`; [CI 42672724894](https://github.com/salvatorecorsi/convert-here/actions/runs/42672724894) **SUCCESS**. Il pacchetto Store 1.0.5.0 è costruito e verificato, **non caricato e non inviato**. La Submission 3 **1152921505702351165** è in bozza: IT salvata e riletta Updated, EN salvata ma non riletta. Queste istruzioni non attestano installazione del MSIX, persistenza EN o approvazione Microsoft.

## Additional Testing Information — English

```text
Convert Here 0.6.0 / full MSIX 1.0.5.0, Windows 11 x64. Publisher: Salvatore Corsi. The application and Explorer menu support Italian and English. Settings > Language selects System, Italiano or English independently from OCR language. The instructions below use Italian control names. No in-app account or login is needed. The app is free, with no subscriptions, in-app purchases or usage charges; no payment is required to keep using it.

Native tests need no external engine: add PNG/JPEG images, choose JPG or PNG and convert to a new file. Open Modifica… in an image row. Try crop, resize and custom pixel canvas/background/anchor, then PNG, JPEG, WebP or PDF output. Adatta fits the preview; Dettaglio 1:1 shows image pixels. JPEG/WebP quality is adjustable. Peso massimo uses decimal kB: 300 kB is 300000 bytes. Extra downsizing is opt-in. If the cap cannot be met, save is disabled; quick conversion fails without publishing an oversized output. Salva con nome… writes the prepared bytes and refuses an existing output name.

For native text PDF, select Nuovo foglio… and type ordinary text, or add a UTF-8 TXT and choose Modifica…. Select A4/A5, orientation, margins, font size, line spacing and alignment. The PDF embeds Ubuntu Light with selectable text. Unsupported characters return an explicit error. This is plain-text/image layout, not a Word or PDF content editor.

Use Salva le mie impostazioni to name a profile and enable Mostra in Converti in. Up to 32 profiles are saved, with 8 visible in the menu. In Impostazioni > I miei profili, apply, update or remove them; Crea «Salva per web» creates a WebP 300 kB example. Profiles contain parameters, not sample images or typed text. In Explorer, select compatible files, reopen Converti in and choose the profile. It queues a job with saved recipe settings. Disabled target formats and incompatible inputs hide the profile. Existing queue jobs keep their settings.

For multiple PDF-compatible images/PDFs, PDF (Bulk) creates separate outputs; PDF (Merge) opens individual pages to drag between sources or move with arrows/Alt+up/down. Create one PDF and check its order. Existing files are preserved with numbered output names. Compress and split existing PDFs also run locally; encrypted/signed PDFs and interactive forms have the documented operation restrictions.

Optional engines are not bundled: FFmpeg for audio/video, Poppler for PDF image/text export, LibreOffice for Office/Excel/HTML, libheif for HEIC/AVIF, Tesseract plus language models for OCR. Native image editing and A4/A5 text layout do not require them. Impostazioni > Motori selects local engines. Explicit FFmpeg/Poppler download requires confirmation of source, size, licenses and folder, uses HTTPS and checks SHA-256. It does not upload conversion files. HTML-linked resources may access local files/network. Conversion and OCR run on the PC.

runFullTrust supports user-selected file access, converted output, packaged COM Explorer integration and launching optional local engines. The native dependency installer writes confirmed FFmpeg/Poppler tools only to the user's tools folder. No system-wide engine installation is required.

Original code: GPL-3.0-only; dependencies/fonts/engines retain their licenses. License and source links are under Impostazioni > Informazioni e licenze. Source release: https://github.com/salvatorecorsi/convert-here/releases/tag/v0.6.0
Privacy: https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/docs/privacy.md
License: https://github.com/salvatorecorsi/convert-here/blob/v0.6.0/LICENSE
Support: salvo@salvatorecorsi.com
```

## Prove locali del ramo editor, prima dell'integrazione bilingue

Questi esiti sono stati osservati prima del merge con la 0.5.0 bilingue pubblicata dall'altra sessione. La build 0.6.0 integrata richiede controlli propri: non si trasferisce automaticamente l'esito di una suite al nuovo commit.

- [x] Formattazione e Clippy superati; **265 test app** e **12 test DLL** superati. Un test degli archivi upstream in cache, già separato dalla suite ordinaria, resta ignorato.
- [x] CLI: un lavoro mantiene lo snapshot del profilo anche dopo aggiornamento ed eliminazione del profilo. Risultato di **220068 byte**, **640 × 498 pixel**, SHA-256 dell'originale invariato.
- [x] GUI nativa: WebP **900 × 700** da **508858 byte** a **297678 byte**, qualità **44**, limite **300000 byte**. Il file salvato è esattamente di **297678 byte**; con limite **100 byte** il salvataggio è disattivato.
- [x] Editor nativo: ritaglio trascinato **453 × 349**, scala **300 × 231**, tela bianca con dimensione impostata a **1024 pixel**. Creato e persistito il profilo **«Immagine quadrata»**, poi rinominato **«Quadrato per il sito»** e scelto come predefinito. Al riavvio è ricaricato con la stessa tela e lo stesso risultato di **47454 byte**.
- [x] TXT UTF-8 in PDF **A5**, font **18 pt**, **4 pagine**, **176031 byte**. `pdfinfo` riporta **419.528 × 595.276 pt**; `pdftotext` conserva correttamente accenti e simbolo **€**. Il rendering Poppler è stato ispezionato.
- [x] Sostituzione del testo nell'editor: il testo digitato rigenera correttamente **una pagina A5 di 173633 byte**. Il contenuto non viene conservato nei profili.
- [x] Correzione overflow dei controlli della tela verificata: nessuna area nera e **Trasparente** passa alla riga successiva.

Queste prove riguardano sorgenti, CLI e GUI locale: non attestano installazione del MSIX 1.0.5.0 o registrazione effettiva del menu Explorer.

## Checklist restante per il candidato finale

- [x] Audit del MSIX finale 1.0.5.0: **7123294 byte**, SHA-256 `185212fc3118f0d499763e415fcad33ce1eec1999bce5c678de892a8a27d9e5e`, 57 elementi payload, 59 voci ZIP e 243 avvisi; distinto dalla distribuzione GitHub.
- [x] Commit finale: CI **42672724894 SUCCESS**; sorgenti con 431 crate vendorizzati e veri `cargo check --offline --locked --all-targets` da cache vuota per app e DLL superati, senza attestare una build completa con linking. I quattro digest degli asset GitHub corrispondono ai locali.
- [ ] Completare caricamento Store, verificare il nuovo pacchetto nel portale e rileggere la scheda EN dopo Save; invio per certificazione non eseguito.
- [ ] Verificare app ed editor integrati nelle due lingue; completare le varianti PNG/JPEG/PDF e le combinazioni di tela/sfondo sul commit finale.
- [ ] Verificare esplicitamente riduzione delle dimensioni solo se autorizzata e assenza di risultati oltre il limite nelle ulteriori varianti.
- [ ] A4/A5 verticale e orizzontale da immagine, testo digitato e TXT UTF-8; verificare pagine, testo estraibile e errore per caratteri non disponibili.
- [ ] Provare profili con input compatibile e incompatibile, formato disabilitato e limite di peso nella coda rapida sul commit integrato; verificare il menu effettivamente registrato.
- [ ] Riordinare pagine di almeno un PDF multipagina e un'immagine in PDF Merge; controllare il risultato senza modificare gli originali.
- [ ] Verificare nome già esistente, originale/collegamento fisico, annullamento e chiusura durante la preparazione.
- [ ] Confermare che anteprime, immagini di prova e testo digitato non compaiano in preferenze o profili; controllare separatamente i record della coda.
- [ ] Verificare installazione/aggiornamento/disinstallazione, attivazione del menu moderno e strumenti opzionali nell'effettivo pacchetto Store su ambiente dedicato.
- [ ] Prima della submission: verificare URL pubblici 0.6.0, catture aggiornate, testi IT/EN e persistenza dei campi dopo il salvataggio nel portale.

Le caselle residue restano aperte: gli esiti vanno registrati soltanto dopo la prova corrispondente. Il WACK non viene dichiarato eseguito o superato.
