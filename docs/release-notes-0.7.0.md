# Convert Here 0.7.0 — profili, opzioni dei motori e cronologia

**Versione in preparazione.** Applicazione prevista **0.7.0**, pacchetto Microsoft Store **1.0.6.0**. Questa pagina non annuncia ancora la pubblicazione dei pacchetti. Sviluppatore e publisher: **Salvatore Corsi**. [English](release-notes-0.7.0.en.md).

La finestra principale è organizzata in quattro schede: **File**, **Cronologia**, **Impostazioni** e **Profili**. Le opzioni del formato restano vicine alla conversione; motori, uscite abilitate e aspetto hanno una pagina dedicata.

Il nuovo manager crea, modifica, duplica, rinomina ed elimina ricette, sceglie il predefinito e ne gestisce la visibilità in Explorer. Restano i limiti di **32 profili salvati** e **8 nel menu**. Ogni profilo può ammettere tutti gli ingressi compatibili oppure solo estensioni scelte. Queste restrizioni sono verificate anche prima dell'esecuzione; i formati supportati dal programma restano **44 in uscita e 93 estensioni in ingresso**.

**Importa JSON**, **Esporta** ed **Esporta tutti** trasferiscono i profili. L'importazione crea nuove copie e non sovrascrive ricette esistenti. File non validi, dimensioni oltre **256 KiB** o un totale oltre 32 profili non applicano modifiche parziali. Le ricette oltre gli otto posti visibili vengono importate fuori dal menu, con un conteggio nel messaggio. Il JSON include opzioni, filtri ed eventuale cartella di uscita, senza file di prova o percorsi degli eseguibili.

I profili PDF possono comprimere, dividere o **unire**. Un profilo Merge richiede almeno due ingressi ammessi e apre l'editor per ordinare le pagine prima di creare il PDF unico. Modifiche immagine e impaginazione conservano i propri limiti di compatibilità.

I **limiti comuni** permettono di scegliere peso massimo per risultato e durata massima audio/video. Il peso è controllato sui byte effettivi prima della pubblicazione; se il limite non è rispettato, il risultato non viene salvato. L'editor JPEG/WebP può adattare la qualità e, solo se autorizzato, le dimensioni. Non viene promesso un adattamento automatico del bitrate audio/video. I vecchi limiti immagine restano rispettati; se coesistono, viene usato il minore. Il cap comune vale anche per i PDF da immagini e testo.

Per audio, video e GIF da video, **Avanzate** offre opzioni ricercabili di codec, filtri, metadati e prestazioni. Il programma rileva i controlli specifici dall'encoder FFmpeg locale e verifica valori e compatibilità prima di usarli. Non accetta comandi o script liberi. Un profilo con queste opzioni richiede sorgenti audio/video; immagini in GIF usano il percorso nativo senza opzioni FFmpeg.

**Cronologia** riunisce i lavori rapidi con quelli della finestra e i tentativi di salvataggio dell'editor e di PDF Merge. I tentativi GUI terminati restano tra gli avvii, fino a **300 voci entro 1 MiB**. L'aggiornamento e il salvataggio avvengono in background; dati non leggibili vengono segnalati e conservati. I file originali e i risultati non vengono cancellati dalla pulizia della cronologia.

Il requisito del pacchetto Store è **Windows 10 build 19041 o successivo, x64**. Il menu contestuale usa l'integrazione classica su Windows 10 e quella moderna su Windows 11 dove prevista dal pacchetto. La modifica del requisito è stata verificata con schema e fixture; non equivale a un test di installazione su un host Windows 10 reale.

Conversioni, editor e OCR restano locali e gratuiti. I motori opzionali mantengono i propri requisiti; il download facoltativo di FFmpeg/Poppler richiede conferma e Internet, mentre HTML con risorse collegate può accedere alla rete tramite LibreOffice.

La distribuzione prevista comprende installer, portable, sorgenti corrispondenti con dipendenze Cargo vendorizzate e checksum. Il codice originale resta **GPL-3.0-only**; dipendenze, font e motori conservano le proprie licenze. Nessun pagamento è richiesto per continuare a usare Convert Here.

Approfondimenti: [editor e manager profili](image-editor-profiles.md), [configurazione](configuration.md), [cronologia e coda](queued-conversions.md), [CLI](command-line.md), [privacy](privacy.md), [stato di verifica](store-validation.md).
