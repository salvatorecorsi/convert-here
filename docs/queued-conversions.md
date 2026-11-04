# Conversioni rapide visibili in Esplora file

Scegliendo **Converti in → MP3**, **WAV** o un altro formato, Convert Here accoda il lavoro e mostra subito un segnaposto nella cartella di destinazione. Il nome corrisponde al risultato previsto, per esempio `registrazione.mp3`, con un’icona di lavorazione. La destinazione predefinita è la stessa cartella dell’originale; viene rispettata una cartella diversa salvata nelle impostazioni.

Il segnaposto è un normale collegamento Windows temporaneo, con estensione interna `.lnk`. Esplora file normalmente nasconde questa estensione. È distinto da un file audio, video o PDF incompleto: il doppio clic apre una piccola finestra di stato con destinazione, avanzamento e comando di annullamento. Non avvia un lettore multimediale su dati ancora incompleti.

L’icona nel file manager è statica: indica il lavoro in attesa o in corso, senza promettere un’animazione di Esplora file. La finestra di stato mostra invece l’avanzamento aggiornato. Al termine il segnaposto lascia il posto al file vero, con la sua normale icona e il nome già assegnato. Se esistono già file con quel nome, viene scelto un nome numerato senza sovrascriverli.

Le conversioni rapide condividono una coda persistente. Le richieste aggiunte mentre un’altra conversione è in corso attendono il proprio turno. La finestra delle opzioni non viene aperta automaticamente e chiudere una finestra di stato non annulla il lavoro. Per annullarlo occorre usare **Annulla**.

La cronologia viene ripulita quando il processo di lavoro riprende o termina la coda: i record completati o annullati scadono dopo 7 giorni e quelli falliti dopo 30 giorni; la pulizia tenta inoltre di conservare al massimo i 300 record terminali piu recenti. Non elimina originali, risultati o lavori attivi. I record illeggibili vengono isolati e conservati per la verifica; eventuali file estranei alle registrazioni della coda vengono preservati.

La cartella nascosta `.the-converter-reservations` contiene soltanto le prenotazioni dei nomi dei risultati: impedisce che richieste simultanee scelgano lo stesso nome e mantiene quelli mostrati nei segnaposto. Viene eliminata automaticamente quando l'ultimo lavoro libera la propria prenotazione, anche dopo un errore o un annullamento. Finche ci sono altri lavori in coda per quella destinazione, conserva le loro prenotazioni.

La scheda **Coda rapida** contiene i lavori attivi, con stato, nome del risultato, cartella di destinazione e dettagli degli eventuali errori. **Cronologia** raccoglie i lavori terminati e il resoconto dell’ultima conversione rapida. Gli snapshot si aggiornano in background circa due volte al secondo.

In caso di errore il segnaposto mostra una X rossa; un doppio clic apre la spiegazione. Per un lavoro fallito, **Rimuovi segnaposto** elimina soltanto i collegamenti ancora appartenenti al lavoro e mantiene cronologia e risultati; un collegamento sostituito da un altro programma viene preservato e segnalato. Un lavoro annullato rimuove il proprio segnaposto e i temporanei. I file già esistenti o creati da altri programmi non vengono sovrascritti. Un'interruzione forzata del processo viene riconosciuta al successivo avvio: il lavoro interrotto viene segnalato e gli altri lavori in attesa proseguono.

## Audio in TXT con OpenAI

La trascrizione online compare per gli audio dopo l'abilitazione facoltativa con una chiave API nella pagina **Strumenti**. Scegliere **TXT con OpenAI** e l'azione che autorizza l'invio dell'audio al servizio: il lavoro usa la stessa coda e lo stesso segnaposto, ma richiede internet e puo generare costi sul proprio account API. Le conversioni di formato e il TXT estratto dai PDF con Poppler restano locali.

La coda conserva impostazioni, percorsi e stato del lavoro; non conserva la chiave API, che viene letta dal Gestore credenziali Windows al momento dell'esecuzione. Rimuovere la chiave impedisce nuove trascrizioni; per fermare un lavoro gia in corso usare **Annulla**. Chiudere la finestra di stato non ferma l'invio o l'elaborazione.

Se occorre, FFmpeg prepara segmenti audio locali che vengono trascritti in sequenza. Dopo il primo segmento, le richieste possono reinviare come contesto fino a 200 caratteri di parole intere gia trascritte della stessa registrazione. Il file TXT finale compare solo quando tutti i segmenti sono stati elaborati. Errori e annullamenti non pubblicano un TXT parziale; l'app tenta di pulire i temporanei. Non viene ripetuta automaticamente una richiesta fallita. I segmenti gia inviati possono essere stati elaborati e addebitati dal fornitore anche se annulli o il lavoro termina con errore. Vedi [requisiti e limiti](../README.md#trascrizione-audio-txt-con-openai-facoltativa) e [privacy](privacy.md).
