# Verifiche candidata 0.8.1 / Store 1.0.9.0

**Bozza in preparazione**, applicazione **0.8.1**, pacchetto Store previsto **1.0.9.0**, publisher **Salvatore Corsi**. La precedente **0.8.0 / Store 1.0.8.0 risulta pubblicata**. Nessuna build finale, release GitHub, upload o submission della 0.8.1 è attestata da questo record. [English](store-testing-0.8.1.en.md).

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

## Verifiche e artefatti ancora da registrare

- [x] Test applicazione, DLL, regressioni mirate del wizard e Clippy all-targets.
- [x] QA della nuova UI in italiano e inglese, con preferenze e cartelle di test isolate.
- [x] Due catture nuove con dimensioni, hash e provenienza; galleria proposta con due immagini storiche 0.8.0 dell'editor invariato.
- [ ] Commit pulito, versioni PE e manifest, checksum e inventari di setup, portable, sorgenti e MSIX.
- [ ] Controlli offline dei sorgenti vendorizzati finali e prove dei binari finali.
- [ ] Pubblicazione GitHub e stato effettivo di caricamento, invio o pubblicazione Store della 1.0.9.0.

Il requisito previsto resta **Windows 10 build 19041 o successivo, x64**, incluso Windows 11; identità tecnica, CLSID e capability `runFullTrust` restano invariati. Non sono attestati installazione del nuovo MSIX, test su host Windows 10 reale o WACK. I risultati della precedente 0.8.0 non sostituiscono le verifiche di questa candidata.
