# Sviluppo e distribuzione

**Italiano** · [English](development.en.md)

Questa guida descrive come compilare Convert Here, verificare il codice e preparare le release pubbliche con i rispettivi sorgenti. Esegui i comandi PowerShell dalla radice della repository. I sorgenti aggiornati includono l'interfaccia italiano/inglese; i pacchetti v0.4.0 precedentemente pubblicati mantengono la GUI italiana.

## Compilazione

Windows x64, Rust stable MSVC e Visual Studio C++ Build Tools/Windows SDK:

```powershell
.\scripts\fetch-tools.ps1
cargo build --release --locked
cargo build --release --locked --manifest-path .\shell-extension\Cargo.toml
.\scripts\build-release.ps1 -SkipBuild
```

`fetch-tools.ps1` scarica versioni fissate di FFmpeg e Poppler e ne verifica gli SHA-256 prima di copiarle; non installa nulla nel sistema. È opzionale se usi strumenti già disponibili. `scripts\build-release.ps1` esegue entrambe le compilazioni. `-CertificateThumbprint` permette di firmare eseguibili, DLL e MSIX con un certificato già disponibile, ricavandone il Publisher per entrambi i manifest. MakeAppx è necessario per generare il pacchetto identità; l'applicazione può essere usata anche senza.

Per preparare gli artefatti pubblici, aggiorna prima gli avvisi:

```powershell
.\release-support\collect-rust-licenses.ps1
```

Revisiona e salva in Git sorgenti, lockfile e avvisi, poi esegui la build e la preparazione dallo stesso commit con cartella di lavoro pulita:

```powershell
.\scripts\build-release.ps1
.\scripts\prepare-public-release.ps1 -CompilerPath C:\Percorso\InnoSetup\ISCC.exe
```

Lo staging pubblico usa una lista esplicita di file e una verifica che rifiuta motori esterni o DLL aggiuntive. Per la 0.4.0 prepara `dist/releases/v0.4.0/` con setup, portable, `ConvertHere-0.4.0-source.zip` e `SHA256SUMS.txt` per tutti e tre gli archivi. Licenza del progetto e avvisi di dipendenze e font sono inclusi nei pacchetti.

La preparazione richiama `release-support/build-corresponding-source.ps1`: esporta il commit Git, verifica la provenienza dei binari e usa `cargo vendor --locked --versioned-dirs --sync shell-extension/Cargo.toml` per entrambi i lockfile. L'archivio conserva sorgenti e licenze upstream, configurazione Cargo relativa a `vendor/`, metadati di checksum, toolchain e `BUILDING-OFFLINE.txt`. Dopo aver installato la toolchain indicata, MSVC x64 e Windows SDK, puoi compilare app ed estensione con Cargo offline seguendo quella ricetta. Gli strumenti di sviluppo generali non sono inclusi; Inno Setup serve solo per generare il setup. Motori esterni, certificati e credenziali non fanno parte dell'archivio sorgente.

Per una distribuzione con il menu moderno, usa `prepare-public-release.ps1 -RequireModernMenu` dopo una build firmata con un certificato di produzione. Il comando include MSIX e loghi esterni, oppure si interrompe se manca una firma attendibile. Il relativo setup abilita per default la voce diretta su Windows 11 e la rimuove durante la disinstallazione. Non importa certificati di prova: la procedura locale e i requisiti di firma sono descritti in [integrazione con Windows](windows-integration.md).

## Verifica

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --manifest-path .\shell-extension\Cargo.toml
.\scripts\test-shell-extension.ps1
```

I test esterni usano i motori trovati localmente; se FFmpeg, Poppler, LibreOffice, Tesseract o libheif mancano, i relativi fixture vengono saltati esplicitamente. I fixture verificano codec e decodifica dei nuovi contenitori, roundtrip HEIC/AVIF e degli altri formati immagine, precisione HDR, i quattro separatori CSV con Unicode/virgolette/righe multiple e formule trattate come testo, primo foglio Excel, ricostruzione PDF in DOCX/HTML con e senza OCR, ordine delle pagine, compressione/unione/divisione PDF, collisioni dei nomi, annullamento e coda. Le verifiche del menu caricano la DLL e usano le API COM di Windows senza registrarla; una verifica di parità confronta tutte le estensioni e destinazioni della DLL con il modello applicativo. La comparsa effettiva nel menu moderno richiede un'installazione firmata e una verifica su Windows 11.

## Licenza e sorgenti

Copyright (C) 2026 Salvatore Corsi. Il codice originale di Convert Here è distribuito secondo la **GNU General Public License versione 3 soltanto** (`GPL-3.0-only`), senza alcuna garanzia nei termini della [licenza](../LICENSE).

L'archivio `ConvertHere-0.4.0-source.zip` include il commit della release, entrambi i lockfile, dipendenze Cargo vendorizzate, script e istruzioni di build offline. I checksum coprono setup, portable e sorgenti. L'app mostra licenza e collegamento ai sorgenti nelle informazioni sul programma.

Dipendenze, font e motori esterni conservano le rispettive licenze: vedi [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md) e i [testi delle licenze](release-licenses/README.md).

[Torna al README](../README.md)
