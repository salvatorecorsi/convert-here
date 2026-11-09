# Editor immagini, fogli e profili

Disponibile da **Convert Here 0.6.0**.

## Aprire l'editor

Aggiungi un'immagine e scegli **Modifica…** nella sua riga; nelle opzioni trovi anche **Modifica immagine…**. Per un TXT, **Modifica…** apre l'impaginazione del testo; nelle opzioni trovi **Impagina testo…**. **Nuovo foglio…** apre un testo da scrivere direttamente nell'app. L'editor esporta PNG, JPEG, WebP e PDF; il selettore delle uscite immagine rispetta i formati abilitati nelle Impostazioni.

Il file originale resta invariato. **Salva con nome…** scrive i byte del risultato preparato e non sovrascrive un file esistente, né l'originale o un suo collegamento fisico. Scegli un nome libero se la destinazione esiste già. L'anteprima precedente può restare visibile mentre cambiano le opzioni: attendi il nuovo risultato prima di salvare.

## Ritaglio, scala e tela

- **Ritaglio:** abilita **Seleziona area nell'anteprima** e trascina sull'immagine, oppure imposta le percentuali. **Immagine intera** ripristina tutta l'area. Nel profilo il ritaglio è relativo: la stessa regione percentuale si applica a ogni nuova immagine, dopo il suo orientamento.
- **Scala:** attiva **Ridimensiona**, indica larghezza e/o altezza e scegli se mantenere le proporzioni.
- **Tela e sfondo:** scegli la risoluzione della tela in pixel, posizione di ancoraggio, spostamenti e colore di sfondo. Puoi mantenere l'immagine alle sue dimensioni, adattarla alla tela o riempire l'area con ritaglio.

Le immagini animate e i contenitori con più immagini usano il primo fotogramma o la prima immagine. HEIC/HEIF e AVIF richiedono ancora il decoder libheif locale configurato. JPEG e le immagini inserite nelle pagine PDF hanno sfondo bianco quando l'immagine è trasparente; PNG e WebP possono conservare l'alfa.

## Qualità, peso e anteprima

Per JPEG e WebP regola **Qualità**. WebP con qualità regolabile è codificato localmente con il codec integrato; PNG conserva i pixel senza perdita e non usa una qualità JPEG. **Adatta** mostra l'intero risultato, **Zoom anteprima** ne cambia la visualizzazione e **Dettaglio 1:1**, quando disponibile per l'immagine, mostra un pixel immagine per pixel schermo.

Il peso è calcolato sui byte effettivi già codificati, non stimato dai megapixel. Il salvataggio riusa quegli stessi byte. Per immagini con perdita, l'anteprima viene decodificata dal risultato compresso; la pagina PDF usa la stessa geometria, il font incorporato e l'immagine JPEG preparata.

**Peso massimo** usa kB decimali: **300 kB = 300000 byte**. Per JPEG e WebP, con il limite attivo, il controllo indica la qualità massima e il motore cerca una qualità che rispetti il peso. **Riduci anche le dimensioni se necessario** autorizza anche una risoluzione minore: è una scelta esplicita, non un comportamento delle conversioni normali. Per PNG senza perdita, un limite più basso può richiedere questa riduzione. Nei PDF regola qualità delle immagini, dimensioni e DPI per cambiare il peso.

Un limite può essere impossibile da rispettare. In quel caso **Salva resta disattivato**; una conversione rapida con il medesimo profilo termina con un errore e non pubblica un risultato sovradimensionato. Aumenta il limite oppure cambia dimensioni, qualità o contenuto.

## Immagini e testo su A4 o A5

Con uscita PDF scegli **Foglio**, A4 o A5, orientamento verticale o orizzontale e margini in millimetri. Per un'immagine, **Immagine intera** mantiene tutto il contenuto e **Riempi foglio** riempie l'area interna ai margini con un ritaglio centrale. I DPI limitano la risoluzione dell'immagine inserita; le misure fisiche del foglio restano quelle scelte.

Il testo scritto nell'app e i file TXT **UTF-8** vengono impaginati localmente: dimensione del carattere, interlinea e allineamento sono regolabili, con ritorni a capo e pagine automatiche. Il PDF incorpora Ubuntu Light e conserva testo selezionabile. I caratteri non disponibili nel font producono un errore leggibile: non vengono eliminati in silenzio. Il limite è 8 MiB di testo, un milione di caratteri disegnati e 1000 pagine.

Questo è un impaginatore di testo semplice. I file Word e gli altri documenti Office mantengono la conversione LibreOffice esistente; il contenuto completo di un PDF o di un DOCX non viene aperto come documento modificabile nell'editor. Senza le opzioni pagina, anche il percorso TXT tradizionale resta quello LibreOffice.

## Salvare e usare i profili

Nell'editor, **Salva le mie impostazioni** permette di dare un nome alle opzioni e scegliere **Mostra in Converti in**. Puoi salvare fino a **32 profili**, di cui **8 visibili nel menu**. Nelle Impostazioni, **I miei profili** consente di applicarli, modificarli, rinominarli, aggiornarli con le opzioni correnti, eliminarli o usarli come predefiniti.

Il profilo conserva le opzioni, il nome, la visibilità e l'eventuale cartella di destinazione. L'immagine usata per la prova, il testo digitato e le anteprime non fanno parte del profilo. Un profilo immagine si applica alle immagini; un profilo pagina creato per TXT o testo semplice si applica al dominio compatibile indicato dalle sue opzioni.

Per una scorciatoia pronta, **Crea «Salva per web»** configura WebP con massimo 300 kB, qualità massima 90 e riduzione delle dimensioni consentita. Puoi poi cambiarne le opzioni. Per esempio, un altro profilo può usare JPEG con larghezza 1200 pixel, oppure PDF A5 con margini e DPI scelti.

In Esplora file apri **Converti in** e scegli il nome del profilo: il lavoro entra nella coda rapida con quelle opzioni. Il profilo appare solo quando è abilitato nel menu, il suo formato di uscita è attivo e tutti i file selezionati sono compatibili. Se modifichi un profilo mentre il menu è già aperto, riapri il menu prima di usarlo. I lavori già accodati mantengono la ricetta con cui sono stati creati.

I profili aggiungono ricette ai **44 formati di destinazione**: non diventano nuovi formati e non cambiano le **93 estensioni di ingresso** riconosciute. Le otto uscite predefinite restano MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4; puoi abilitare le altre nelle Impostazioni.

Immagini, testo di lavoro e anteprime restano nella sessione dell'editor. Solo il salvataggio esplicito produce il risultato richiesto. Preferenze e profili sono locali; dettagli su cartelle, coda e temporanei dei codec sono nell'[informativa privacy](privacy.md).
