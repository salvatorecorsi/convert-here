# Verifiche 0.8.1 / Store 1.0.9.0

Applicazione **0.8.1**, pacchetto Store **1.0.9.0**, publisher **Salvatore Corsi**, dal commit pulito **`07fe5f749327b73e93a92d5101d80b9b4f5317cb`**. [Release GitHub](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.1) pubblicata il **10 novembre 2026 alle 13:42:06 UTC**; artefatti finali verificati. **Submission 6**, ID **1152921505702367000**, inviata: stato osservato **Update in certification** alle **13:44:28 UTC**, con pubblicazione automatica appena approvato. La nuova 1.0.9.0 non è ancora dichiarata approvata o disponibile; la precedente **0.8.0 / Store 1.0.8.0 è pubblicata**. [English](store-testing-0.8.1.en.md).

## Comportamento previsto

- Checkbox **Seleziona tutti** in fondo a ciascuna categoria: selezione e deselezione collettive, modifica singola che aggiorna il controllo collettivo, otto default invariati e filtro limitato alle uscite.
- Wizard automatico soltanto nella finestra principale avviata senza file, formato preimpostato o modalità personalizzata. Conversioni rapide, stato coda, CLI e PDF Merge seguono i percorsi esistenti.
- Motori mancanti preselezionati: FFmpeg, Poppler, Tesseract ita/eng, libheif, Whisper/Tiny. LibreOffice offre soltanto il sito ufficiale. La dipendenza FFmpeg/ffprobe della trascrizione non produce richieste duplicate.
- Conferma di fonti, versioni, dimensioni, licenze e destinazione prima di qualsiasi download. Un solo motore alla volta, con progressi, annullamento, errori e ripresa dei motori rimasti da verificare in ambiente isolato.
- Marker locale `engine-setup-v1.done` pubblicato atomicamente senza sovrascrivere file esistenti. Saltare o finire non riscrive le preferenze; vecchie finestre non possono cancellare questo stato salvando snapshot obsoleti. Collisioni o errori di scrittura vengono segnalati senza eliminare dati.
- Riapertura esplicita da **Impostazioni → Motori → Configurazione guidata**, anche dopo aver completato o saltato il primo avvio.

## Verifiche locali completate

- **424 test applicazione e 18 test DLL passati: 442 complessivi**, con un test facoltativo ignorato, escluso dai passati. Log locali: `.tools/tests-081-final.log` e `.tools/tests-dll-081.log`.
- Dopo l'ultima compattazione puramente visiva delle card, ricontrollati **24 test mirati del wizard fra GUI e CLI**, già compresi nella suite complessiva, e **Clippy all-targets con `-D warnings`**: PASS. Modello e persistenza non modificati dopo la suite completa. Log: `.tools/tests-081-wizard-final.log`, `.tools/clippy-081-final.log`.
- Le regressioni coprono pianificazione di tutte le scelte senza duplicazioni, dipendenze FFmpeg/Whisper, stato dei tool incompleti, sequenza e dimensioni, errore/annullamento con conservazione dei successi, ripresa dei soli motori mancanti e mancata conferma di un successo da eventi incompleti. Il marker è verificato con cartelle isolate, collisioni, preferenze obsolete e otto scritture concorrenti.
- **QA nativa italiana e inglese PASS**, con configurazione e cartelle isolate: prima apertura automatica; Continua verso conferma delle fonti; Annulla che riporta alla scelta; Salta che crea il marker lasciando invariati i byte delle preferenze; riapertura manuale; riavvio in italiano senza wizard. Record `.tools/ui-081-qa.json`; schermata inglese locale `.tools/ui-081-english.jpg`.
- Nella QA dei formati, **Seleziona tutti Audio** porta il totale delle uscite attive a **18**; togliere MP3 lo porta a **17** e deseleziona il controllo collettivo; svuotare Audio lascia **6** uscite delle altre categorie; **Più comuni** ripristina gli **8** default. Questi sono conteggi globali delle uscite attive, non il numero di formati della famiglia Audio.
- Acquisite [due nuove schermate native 0.8.1](store-assets/0.8.1/README.md), JPEG e PNG **2562 × 1441**, zoom 150%, senza puntatore, alone o ritocchi. Binario GUI debug SHA-256 `bfba78128e3ee6bf10bf38b71a7deb3a4d31b3af65a2b12bc200d580919bd969`; conversione JPEG → PNG con la CLI dell'app senza ridimensionamento o ritaglio. I digest sono nel [registro delle catture](store-assets/0.8.1/captures.json).

**Il pulsante Download non è stato premuto nella QA del wizard.** Non viene quindi attestato un download o un'installazione reale tramite la nuova GUI. Le prove automatiche e la navigazione fino alla conferma non sostituiscono tale verifica. Le catture sono da build debug della working tree; non sono immagini di un nuovo MSIX installato.

## Riepilogo delle verifiche

- [x] Test applicazione, DLL, regressioni mirate del wizard e Clippy all-targets.
- [x] QA della nuova UI in italiano e inglese, con preferenze e cartelle di test isolate.
- [x] Due catture nuove con dimensioni, hash e provenienza; galleria proposta con due immagini storiche 0.8.0 dell'editor invariato.
- [x] Commit pulito, versioni PE e manifest, checksum e inventari di setup, portable, sorgenti e MSIX.
- [x] Controlli offline dei sorgenti vendorizzati finali e prove dei binari finali.
- [x] Pubblicazione GitHub; MSIX Validated salvato e Submission 6 inviata, in certificazione.

Il requisito resta **Windows 10 build 19041 o successivo, x64**, incluso Windows 11; identità tecnica, CLSID e capability `runFullTrust` restano invariati. Non sono attestati installazione del nuovo MSIX, test su host Windows 10 reale o WACK. Le verifiche di questa release sono distinte dai risultati storici della 0.8.0.

## Artefatti finali e provenienza

| Artefatto | Byte | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.8.1-windows-x64-setup.exe` | 16400262 | `864aa68a5cab9d653dcedb002df4141b6ae5502f5675bf77cfe89bc605077d9e` |
| `ConvertHere-0.8.1-windows-x64-portable.zip` | 21682023 | `3a0e7a8e31b21f93b59f3e85869fa484fe9603ade09a8e3b456b968e70de0dbd` |
| `ConvertHere-0.8.1-source.zip` | 90238585 | `5049b91425261350e00977a8aa690d6f8ac9997c20431413d1730c9803124326` |
| `SHA256SUMS.txt` | 313 | `a8169e159ff7766e3509419afd9a0a4062cceea31568ea4c1f270ff07af53503` |
| `ConvertHere-1.0.9.0-x64-store.msix` | 9359517 | `0b7af68333a5587e0509c65909ebb01cba541e72fcbe4307cc2d2a2e24def772` |


Tutti gli artefatti derivano dal commit **`07fe5f749327b73e93a92d5101d80b9b4f5317cb`**, con `SourceDirty: false`. I quattro asset GitHub sono stati scaricati nuovamente e confrontati: dimensioni, SHA-256 e contenuto di `SHA256SUMS.txt` corrispondono ai file locali verificati. Record locale `.tools/remote-release-081-f0e38951b2c4436396432966aa4fb721/verification.json`.

Audit standalone **PASS**: **108 file** portable byte per byte uguali allo staging, tre binari corrispondenti a `build-info.json`, app/CLI PE **0.8.1**, manifest **0.8.1.0**, publisher Salvatore Corsi e CRT statica. Le **277 componenti Windows** conservano gli avvisi. Record `.tools/standalone-081-audit.json` e `.tools/standalone-081-payload-audit.json`. L'audit non esegue il setup e non installa o registra il programma.

Audit Store **PASS**: **57 elementi payload / 59 voci ZIP**, GUI e DLL, installer nativo dei motori, **277 componenti** e **26 file di licenza** corrispondenti al commit. CLI separata, motori, modelli e script downloader non sono nel MSIX. Identità `SalvatoreCorsi.ConvertHere`, publisher tecnico `CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17`, x64, MinVersion **10.0.19041.0**, CLSID `67B51BA1-8A78-4B4B-A5F9-08A825BE91F8`, verbo `0000ConvertHere` e sola capability `runFullTrust` sono invariati. Gli import PE non contengono dipendenze CRT dinamiche. La versione del crate DLL è 0.8.1; l'audit non attribuisce alla DLL una versione PE non presente. Record `.tools/store-081-audit.json`.

## Sorgenti offline, smoke e CI

Veri **`cargo check --locked --offline --all-targets`** su app (**81,23 s**) e DLL (**4,17 s**) dai sorgenti estratti, con cache Cargo e target inizialmente vuoti e `+crt-static`. Compilatore confrontato con la provenienza: PASS. Verificati **465 crate**, **24.338 file vendor** e **240 file tracciati** rispetto al commit; per **200** file è ammessa soltanto la normalizzazione esatta CRLF → LF. L'hash dell'archivio resta invariato; nella cache finale compare solo `CACHEDIR.TAG`, senza pacchetti o indice scaricati. Non è una build offline completa con linking.

Record `.tools/source-offline-081-e77647330ce84fa9b27e64a944e4fe83/offline-check.json`, SHA-256 **`f2ee8756d0152f8d3df4823f3906b2ba77f4c4982ed94eb35e04f9d0eb5d13cc`**. I contatori di licenze dei diversi helper si riferiscono a insiemi diversi e non vengono sommati.

Smoke dei binari finali **13/13 PASS**: SVG PNG/JPG/PDF, trascrizione audio/video con modello locale, taglio/gain, HEIC/AVIF e OCR. Originali e file esistenti preservati, senza usare preferenze personali o scaricare motori durante le prove. Rapporto RMS misurato **0,5001102921267032**. Record `.tools/release-smoke-081-324df6a7ee364e5b8cf2784167d14715/report.json`, SHA-256 **`3a0c0c4fbcb663e42cba35e11343e65adfa84ccbcb9f96e59de6bf62256092c5`**.

[Windows checks 42825704990](https://github.com/salvatorecorsi/convert-here/actions/runs/42825704990) è **SUCCESS** sul commit di build; job verify **6 min 31 s**. Le prove mirate già comprese nei 442 passati non vengono sommate di nuovo.

## Portale Store e limiti residui

Il MSIX **1.0.9.0** è stato caricato, salvato come **Validated** e inviato nella **Submission 6**, ID **1152921505702367000**. Il **10 novembre 2026 alle 13:44:28 UTC** il portale mostra **Update in certification**; non vengono dedotte sottofasi non mostrate. È configurata la pubblicazione automatica dopo approvazione. La precedente **1.0.8.0 è pubblicata**; non è attestata approvazione o disponibilità Store della nuova 1.0.9.0.

Schede IT/EN salvate e rilette dal server con **20 funzionalità, quattro PNG e quattro didascalie per lingua**. Galleria: due catture nuove 0.8.1 e due storiche 0.8.0 dell'editor invariato; ogni didascalia dichiara l'interfaccia italiana. Privacy e licenza al tag pubblico **v0.8.1** sono salvate e verificate. Le Additional Testing Information di **5.835 caratteri** sono salvate e rilette con confronto normalizzato degli spazi; motivazione `runFullTrust` aggiornata ai cinque motori, **470 caratteri**, senza aggiungere capability.

Evidenze locali: `.tools/store-081-submission-proof.jpg`, `.tools/store-081-submission-proof.txt`, `.tools/store-notes-081-final.txt`. Il pulsante Download della nuova GUI non è stato premuto nella QA nativa; **installazione del nuovo MSIX, host Windows 10 reale e WACK non eseguiti**. Questi limiti restano distinti dai download reali dei motori provati nella 0.8.0 e dai nuovi smoke CLI.

## Additional Testing Information — salvate e rilette nel portale

```text
Convert Here 0.8.1 / MSIX 1.0.9.0. Publisher: Salvatore Corsi. Windows 10 build 19041 or later, x64, including Windows 11. No in-app account or payment is required. Conversion, OCR and speech transcription run locally. Optional engines and language models are not bundled. Requested downloads require confirmation, HTTPS and SHA-256 verification; no selected images, documents, recordings or transcripts are uploaded. Linked HTML resources may access files or the network through LibreOffice.

After installing the candidate, use non-sensitive test samples. Open each main tab: Files, History, Profiles and Settings. Defaults remain MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4. Enable TXT under Settings > Output formats > Raw for OCR or transcription. Recognized inputs total 94, including static SVG; outputs remain 44. In Settings > Output formats, check Select all below Audio, clear one format and confirm that Select all clears too; individual formats stay clickable. Clearing Select all disables that category. Most common restores the eight defaults. Filtering outputs does not remove supported inputs.

For first-run setup, use a fresh test configuration and open the normal main window without files or a preselected target. The wizard preselects missing engines: FFmpeg, Poppler, Tesseract, libheif and Whisper/Tiny. LibreOffice offers its official website only. Click Continue to review sources, versions, sizes, licenses and destination, then confirm before any download. The selected engines install sequentially; FFmpeg is included once before Whisper when FFmpeg or ffprobe is missing. Cancellation or an error preserves completed engines, and retry plans only what remains missing.

Cancel the download confirmation and verify that the wizard returns to engine selection. Skip or finish the wizard and restart: automatic setup should not return. A separate engine-setup-v1.done marker beside preferences remembers acknowledgement without rewriting preferences.json; successful engine installs save tool paths normally. Reopen setup from Settings > Conversion engines > Setup wizard. Quick conversions, CLI, queue-status windows, PDF Merge and custom-settings launches do not automatically open the wizard.

Convert a simple local SVG to PNG and PDF without external engines. SVG is rasterized, not exported as vectors. External references, scripts, DTDs, embedded raster images and oversized documents are rejected.

In Settings > Conversion engines, request Tesseract OCR installation and confirm its sources, versions, size and licenses. Italian and English data are included. Recognize text in a test image. Scanned-PDF OCR additionally needs Poppler and remains opt-in. Recognition can contain errors.

Request libheif installation, then encode and decode HEIC/AVIF samples. For audio/video, request FFmpeg and open Edit. Use timeline handles or numeric times to trim, then apply gain/mute to a compatible media output. Waveforms and frames show the source before effects. Positive gain can clip; this is not a playback or multitrack editor.

Request Whisper with Tiny multilingual. The model is 77,691,713 bytes; runtime plus model downloads total 87,939,016 bytes, excluding FFmpeg. The GUI includes FFmpeg if FFmpeg or ffprobe is missing. Choose TXT for a short recording, select language or Auto and trim the interval. TXT uses trimming without gain/mute. Verify the local transcript and review recognition errors.

In File Explorer, check Convert to in the Windows 10 classic menu or Windows 11 modern menu/Show more options. With TXT enabled, audio/video offers TXT · Transcription. Test a native conversion, queue progress, cancellation and a conflicting destination name: preserve originals and existing results. Verify a profile, JSON import/export, PDF Bulk/Merge and history after restart.

The final package was built from clean source commit 07fe5f749327b73e93a92d5101d80b9b4f5317cb, https://github.com/salvatorecorsi/convert-here/tree/07fe5f749327b73e93a92d5101d80b9b4f5317cb. Original code is GPL-3.0-only; engine and model licenses are retained. Optional AI use is limited to local speech recognition of recordings. The audited MSIX contains GUI, Explorer COM DLL, native runtime downloader, notices and license, with runFullTrust as its only capability and no separate CLI, conversion engine or model. Package size: 9,359,517 bytes. MSIX SHA-256: 0b7af68333a5587e0509c65909ebb01cba541e72fcbe4307cc2d2a2e24def772. Final source archive SHA-256: 5049b91425261350e00977a8aa690d6f8ac9997c20431413d1730c9803124326. Offline cargo check --locked --offline --all-targets passed for the app (81.23 seconds) and DLL (4.17 seconds), using initially empty Cargo cache and target folders with static CRT flags. This was a check, not a complete linked build. Source audit verified all 240 tracked files against the commit and 465 vendored crates; the archive hash was unchanged afterwards. Final-binary CLI smoke testing passed 13 conversions.

Preparation evidence: 424 application tests plus 18 DLL tests passed (442 total, with one optional ignored test excluded), along with all-targets Clippy and native Italian/English UI checks. The final 24 focused wizard tests are already included in the total. Native GUI QA checked first opening, source confirmation, cancellation, skipping, reopening, restart and format selection; its Download button was not pressed. GitHub Windows checks 42825704990 passed: https://github.com/salvatorecorsi/convert-here/actions/runs/42825704990. GitHub release v0.8.1 was published on 10 November 2026 at 13:42:06 UTC, with corresponding source, vendored dependencies and offline build instructions: https://github.com/salvatorecorsi/convert-here/releases/tag/v0.8.1. No new-MSIX installation, physical Windows 10 host test, WACK result or actual GUI download/install is claimed.
```
