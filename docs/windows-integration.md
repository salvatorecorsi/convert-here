# Integrazione con Esplora file

Convert Here usa una DLL COM Rust con `IExplorerCommand`, lo stesso contratto di Windows per i comandi di Esplora file. Il sottomenu **Converti in** offre i formati compatibili con tutti i file selezionati e abilitati nelle Impostazioni: scegliere MP3, WAV, PDF o un altro formato avvia direttamente la conversione con le impostazioni salvate. L'ultima voce **Impostazioni personalizzate…** apre la finestra per modificare qualita e destinazione. La conversione viene eseguita in un processo separato da Esplora file.

Nella 0.4.0 le uscite visibili per default sono MP3, WAV, JPG, PNG, WebP, GIF, PDF e MP4. Le Impostazioni permettono di sceglierle per categoria; il menu rilegge le preferenze a ogni apertura senza riavviare Explorer. Anche con tutte le uscite nascoste resta accessibile la voce personalizzata. Il filtro non cambia le 93 estensioni riconosciute, le 44 destinazioni del motore o il formato di richieste rapide esplicite e lavori gia accodati.

Per un PDF singolo la voce rapida **PDF · comprimi** applica sempre la compressione senza perdita, anche se nelle preferenze e rimasta selezionata un'altra operazione PDF. Nelle selezioni multiple **PDF (Bulk)** mantiene i risultati individuali e **PDF (Merge)** apre esplicitamente l'ordine delle singole pagine per un risultato unico. **Dividi** si sceglie nella finestra principale o nella CLI. L'unione CLI segue i percorsi degli argomenti; l'editor grafico permette di intercalare le pagine prima del salvataggio.

```powershell
.\the-converter-cli.exe --convert --target pdf --pdf-action split --files "C:\Documenti\documento.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Documenti\primo.pdf" "C:\Documenti\secondo.pdf"
```

## Compilazione x64

Serve Rust per `x86_64-pc-windows-msvc` con i Build Tools C++. Il pacchetto identita si genera con MakeAppx del Windows SDK. Dalla cartella del progetto:

```powershell
cargo build --release
cargo build --release --manifest-path .\shell-extension\Cargo.toml
.\scripts\build-release.ps1 -SkipBuild
```

Il risultato e `dist\ConvertHere\`: `the-converter.exe` e `the_converter_shell.dll` devono stare nella stessa cartella. Questi nomi tecnici e gli identificativi COM/MSIX restano stabili dopo la rinomina pubblica. Mantieni quella cartella in una posizione stabile prima di registrare il menu. La DLL x64 deve corrispondere all'architettura di Esplora file; il pacchetto corrente non e per Explorer ARM64.

## Menu classico, senza privilegi amministrativi

```powershell
.\scripts\install-context-menu.ps1 -Mode Classic -InstallDirectory .\dist\ConvertHere
.\scripts\uninstall-context-menu.ps1 -Mode Classic
```

Su Windows 10 il comando compare nel menu contestuale. Su Windows 11 questo metodo compare in **Mostra altre opzioni**. Registra la DLL e il verbo sotto `HKCU\Software\Classes`; non modifica le associazioni predefinite dei file. `-WhatIf` permette di esaminare l'azione senza installarla. La disinstallazione rimuove esclusivamente le due chiavi dell'app.

## Menu moderno di Windows 11

Il menu moderno richiede **package identity**, oltre a `IExplorerCommand`. La sola scrittura nel registro non e sufficiente. Il progetto include il manifest di un pacchetto MSIX con contenuto esterno, COM surrogate e `windows.fileExplorerContextMenus`.

`build-release.ps1` prepara `TheConverter.Identity.msix` senza installarlo. Per la distribuzione il pacchetto deve essere firmato con un certificato attendibile sul computer destinatario. Lo script ricava il Publisher dal Subject del certificato selezionato in `Cert:\CurrentUser\My`, lo applica sia al manifest MSIX sia al manifest incorporato negli eseguibili e ricompila l'app. La versione del pacchetto deriva da `Cargo.toml`. E possibile firmare durante la preparazione:

```powershell
.\scripts\build-release.ps1 -CertificateThumbprint 'IMPRONTA_DEL_CERTIFICATO'
.\scripts\install-context-menu.ps1 -Mode Modern -InstallDirectory .\dist\ConvertHere
```

Non usare `-SkipBuild` per cambiare Publisher: gli eseguibili devono incorporare la stessa identita del pacchetto. Anche quando si riusano binari compilati, lo script controlla il manifest effettivamente incorporato prima di produrre la distribuzione.

La firma usa SHA-256 e un timestamp RFC 3161 per conservare la validita dopo la scadenza del certificato. Il server predefinito e `http://timestamp.digicert.com`; `-TimestampServer` consente di scegliere quello del proprio servizio di firma. Un errore di firma o timestamp interrompe la preparazione.

Lo script di installazione verifica firma, Publisher, versione, COM server e presenza dei loghi nella cartella esterna, poi chiama `Add-AppxPackage -ExternalLocation` per l'utente corrente. Una registrazione identica e gia presente viene conservata. **Non crea certificati, non importa certificati attendibili e non riavvia Esplora file.** Dopo una nuova registrazione puo essere necessario uscire e rientrare in Windows.

Il setup include MSIX e cartella `Assets` solo quando la firma del pacchetto e attendibile sulla macchina di compilazione. In questo caso, su Windows 11 l'opzione per il menu moderno e selezionata per default. Dopo una registrazione riuscita rimuove la precedente voce classica gestita dall'app. Il disinstaller rimuove prima l'identita associata alla propria cartella; se Windows rifiuta la rimozione, interrompe la disinstallazione conservando i file.

Per produrre una release pubblica con il menu moderno, usa una firma di produzione riconosciuta dai computer destinatari e richiedi esplicitamente il pacchetto firmato:

```powershell
.\scripts\prepare-public-release.ps1 -RequireModernMenu
```

Questo comando si interrompe prima di creare gli artefatti se manca una firma attendibile; non produce in sua vece un setup con il solo menu classico. Include il pacchetto identita e tutti i loghi referenziati anche nell'archivio portable. Un certificato autofirmato e rifiutato per la release pubblica.

Per una prova sul proprio PC e possibile usare un certificato di sviluppo. La guida Microsoft per i pacchetti con contenuto esterno indica `Cert:\CurrentUser\TrustedPeople`, ma nella prova del 27 ottobre 2026 su **Windows 11 build 26200** questa importazione non e stata sufficiente: `Get-AuthenticodeSignature` ha continuato a restituire `UnknownError` e `Add-AppxPackage` ha rifiutato il pacchetto con `0x800B0109`.

La [procedura generale Microsoft per MSIX di test](https://learn.microsoft.com/en-us/windows/msix/package/sign-msix-package-guide#testing-distribute-to-testers-with-a-self-signed-certificate) richiede invece **Computer locale → Persone attendibili** (`Cert:\LocalMachine\TrustedPeople`). Per la configurazione provata occorre questo archivio: importare manualmente il solo certificato pubblico `.cer` richiede privilegi amministrativi, conferma UAC e consenso esplicito, perche modifica la fiducia di **tutti gli utenti del PC**. La registrazione dell'applicazione resta per il solo utente corrente.

Questo passaggio e separato dall'installer: il setup **non importa certificati automaticamente** e non modifica Developer Mode. Verifica provenienza e impronta del certificato prima dell'importazione; la chiave privata `.pfx` resta riservata. Un certificato di test non e una firma di produzione e va rimosso dagli archivi in cui e stato importato quando non serve piu. La disinstallazione dell'app non rimuove la fiducia nel certificato. Vedi anche la [guida Microsoft alla fiducia dei certificati di sviluppo](https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/security#what-winapp-cert-install-grants).

```powershell
.\scripts\uninstall-context-menu.ps1 -Mode Modern -InstallDirectory .\dist\ConvertHere
```

Questo rimuove il pacchetto identita dell'utente corrente soltanto se appartiene alla cartella indicata, lasciando applicazione e preferenze. Non rimuove certificati. Puoi registrare separatamente anche il menu classico, se desideri averlo in entrambi i menu.

## Selezioni multiple e sicurezza del processo

Il comando accetta fino a 10.000 elementi e passa i percorsi assoluti con un file JSON UTF-8 temporaneo. I formati rapidi lanciano direttamente `the-converter.exe --quick --target <formato> --selection-file <percorso>`; la voce personalizzate omette `--quick` e apre la finestra. In questo modo spazi, Unicode e selezioni lunghe non diventano una riga di comando enorme. Nessun percorso selezionato viene interpolato in PowerShell o `cmd`. Il processo applicazione convalida i percorsi e rimuove il file richiesta dopo averlo letto.

Con almeno due elementi tutti convertibili in PDF compaiono **PDF (Bulk)** e **PDF (Merge)**. Bulk passa anche `--pdf-batch bulk` e mantiene `--quick`: accoda risultati individuali, comprimendo senza perdita gli eventuali PDF in ingresso. Merge passa `--pdf-batch merge`, omette `--quick` e apre l'editor interattivo delle pagine. Sono compatibili immagini, PDF, documenti Word/Office, Excel/CSV e HTML; audio e video non espongono queste azioni. Il nuovo comando Merge e un'azione, non una nuova destinazione: il modello resta a 44 formati e 93 estensioni.

L'editor prepara le sorgenti in PDF temporanei nascosti e permette di trascinare ogni singola pagina, anche intercalandola con pagine di altre sorgenti. Le frecce e Alt+↑ / Alt+↓ offrono lo stesso riordino. L'ordine iniziale e quello dei percorsi ricevuti da Esplora file, non una garanzia di ordinamento per nome o data. Solo **Crea PDF unico** pubblica un risultato; la sessione non crea lavori persistenti della coda rapida. Il comando Merge della finestra principale usa lo stesso editor. PDF e immagini native non richiedono nuovi motori o Poppler per essere uniti; Office/HTML/Excel richiedono LibreOffice e HEIC/AVIF libheif. Restano rifiutati PDF cifrati, firmati e con moduli interattivi.

Il padre espone `ECF_HASSUBCOMMANDS` e un vero enumeratore `IEnumExplorerCommand` con `Next`, `Skip`, `Reset` e `Clone`. Il `GetState` di ogni formato controlla la selezione completa e nasconde le conversioni incompatibili. Le 93 estensioni riconosciute includono AVIF, HEIC e HEIF; le destinazioni sono 44. Selezionando audio e video compaiono i dodici formati audio comuni; selezionando un'immagine e un PDF compaiono PNG, JPEG, PDF (Bulk), PDF (Merge) e TXT. Excel/CSV propongono PDF, CSV e XLSX nelle categorie compatibili; i PDF propongono anche Word e HTML, esplicitamente indicati come esportazioni del testo.

Il titolo TXT distingue **TXT · OCR** per le immagini e **TXT** per il testo incorporato nei PDF. Le selezioni miste di immagini e PDF usano **TXT · OCR / testo PDF**. Audio e video non hanno una destinazione TXT nella 0.3.1. OCR delle immagini resta locale e richiede Tesseract con i dati delle lingue richieste. Gli strumenti mancanti vengono segnalati dall'app prima di eseguire il lavoro; la presenza della voce nel menu non attesta la disponibilita del motore.

Le maschere interne sono a 64 bit, con i bit superiori riservati alle categorie della selezione. I GUID dei formati precedenti e della voce personalizzate restano stabili quando vengono aggiunti nuovi formati. La whitelist del menu classico e dell'installer corrisponde a quella della DLL; i test confrontano tutte le destinazioni e le 93 estensioni con il modello dell'app. Per piu di 16 elementi la chiamata rapida torna `E_PENDING`, richiedendo a Windows di rieseguire il controllo in background con `fOkToBeSlow`. La DLL non sonda codec o strumenti esterni e non apre i documenti. Elementi virtuali, selezioni vuote ed estensioni sconosciute nascondono il comando.

## Stato di verifica

Compilazione e test COM sono eseguibili senza registrare la DLL:

```powershell
cargo test --manifest-path .\shell-extension\Cargo.toml
.\scripts\test-shell-extension.ps1
.\scripts\test-modern-menu-package.ps1
```

Lo smoke test carica la DLL reale senza registrarla, attiva gli oggetti COM e passa veri `IShellItemArray` creati da Windows: verifica il sottomenu e i formati comuni a batch audio/video, immagini/PDF e documenti/immagini, i titoli contestuali OCR e compressione PDF, XLSX/CSV/HTML e immagini QOI/AVIF/HEIF, allocazione stringhe e rilascio degli oggetti. La modalita opzionale `-InvokeCaptureFile` e riservata a un helper di test isolato che registra gli argomenti del processo lanciato; verifica separatamente la conversione rapida e l'apertura delle impostazioni. I report di test della 0.3.0 restano storici e non attestano la nuova DLL 0.3.1.

I test del pacchetto moderno usano fixture e comandi Appx simulati in un processo Windows PowerShell separato: coprono manifest e asset non validi, firma incoerente, `WhatIf`, registrazione ripetuta, cambio cartella e Publisher, rimozione della sola installazione indicata e propagazione degli errori. Non modificano certificati, registro o pacchetti reali.

La prova locale del **27 ottobre 2026 su Windows 11 build 26200** ha completato la preparazione di DLL, pacchetto identita firmato e setup di prova; i 25 controlli del pacchetto sono passati. Il trust limitato a `CurrentUser\TrustedPeople` e stato rifiutato come descritto sopra. Dopo l'importazione esplicitamente autorizzata del certificato pubblico in `LocalMachine\TrustedPeople`, con UAC, la firma e risultata `Valid` e il setup ha registrato il pacchetto per l'utente corrente.

Le verifiche in Windows PowerShell 5.1 hanno confermato una sola identita `SalvatoreCorsi.TheConverter`, versione `0.1.0.0`, con `EffectiveExternalPath` uguale alla cartella installata `%LOCALAPPDATA%\Programs\The Converter`, tutti i loghi presenti e `ModernMenuInstalled=1`. E presente la voce di disinstallazione di Windows; le precedenti chiavi del menu classico sono assenti. Eseguibile, DLL e MSIX installati corrispondono alla build di prova. La disinstallazione reale del pacchetto non e stata esercitata in questa prova.

In Esplora file **Converti in** e comparso direttamente nel menu moderno, sopra **Mostra altre opzioni**. La scelta **Converti in → MP3** su `prova-audio.wav` ha creato `prova-audio.mp3` nella stessa cartella senza aprire la finestra delle opzioni; `ffprobe` ha confermato codec MP3 e durata di 2 secondi. Questo risultato riguarda la configurazione locale provata: il certificato di sviluppo non costituisce una firma di produzione e non e stata pubblicata una nuova release con quel certificato.

Su altre configurazioni la presenza nel menu moderno va verificata dopo aver firmato e registrato il pacchetto su Windows 11; un MSIX non firmato non e una installazione completata. Nessuna registrazione viene eseguita automaticamente all'avvio dell'app o durante la compilazione.

## Fonti Microsoft

- [Comandi di Esplora file con IExplorerCommand e manifest MSIX](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer)
- [Package identity con contenuto esterno: compilazione, firma e registrazione](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps)
- [Firma di sviluppo e trust per pacchetti con contenuto esterno](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps#build-and-sign-the-identity-package)
- [Distribuzione di MSIX di test e archivio LocalMachine TrustedPeople](https://learn.microsoft.com/en-us/windows/msix/package/sign-msix-package-guide#testing-distribute-to-testers-with-a-self-signed-certificate)
- [Ambito della fiducia nei certificati di sviluppo](https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/security#what-winapp-cert-install-grants)
- [Requisiti e comportamento del menu di Windows 11](https://blogs.windows.com/blog/2021/07/19/extending-the-context-menu-and-share-dialog-in-windows-11/)
