# Verifiche e pubblicazione 0.7.1 / Store 1.0.7.0

Applicazione **0.7.1**, pacchetto Store **1.0.7.0**, publisher **Salvatore Corsi**. La [release GitHub v0.7.1](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.1) è stata pubblicata il **9 novembre 2026 alle 21:08:01 UTC**, dal commit pulito **`2c6d6968159a46baef37d8f20aef049e7be4c8c4`**. La **Submission 4**, ID **`1152921505702358188`**, è stata inviata a Microsoft alle **21:09 UTC**: il portale mostra **Update in certification**, con pre-processing in corso e pubblicazione automatica al superamento della certificazione. L'aggiornamento Store non è ancora dichiarato disponibile al pubblico.

La modifica mirata è l'identificatore manifest **`0000ConvertHere`**, un workaround per il menu packaged non visibile su Windows 10. Restano MinVersion **10.0.19041.0**, architettura x64, identità e CLSID. **Non è disponibile un host Windows 10 reale: il menu su quel sistema non è ancora verificato.** Il record [0.7.0 / Store 1.0.6.0](store-testing-0.7.0.md) resta storico e non trasferisce i suoi test alla nuova build.

## Additional Testing Information — testo salvato nel portale

Le istruzioni seguenti provengono dal testo locale `.tools/store-testing-071.txt`. Il campo è stato salvato e riletto in Partner Center; il valore letto dal portale contiene **2380 caratteri**, con normalizzazione delle terminazioni di riga. Non è attestato un confronto esatto byte per byte con il file locale.

```text
Convert Here 0.7.1 / MSIX 1.0.7.0. Publisher: Salvatore Corsi. Windows 10 version 2004/build 19041 or later, x64, including Windows 11. No account or payment is required. Conversion, editing and OCR are local. Optional conversion engines are not bundled; requested FFmpeg/Poppler downloads require confirmation, HTTPS and checksum validation. Linked HTML resources may access local files or the network through LibreOffice.

This update changes the packaged Explorer verb identifier to 0000ConvertHere as a workaround for a menu missing on Windows 10. The visible command, CLSID and conversion behavior remain unchanged. It has not been verified on a real Windows 10 host; do not infer that result from Windows 11 tests.

After installing the candidate, select a supported PNG/JPEG/PDF in File Explorer. On Windows 10 check the classic context menu; on Windows 11 check both the modern menu and Show more options. Look for Convert to, choose a compatible native output and verify the queued result without changing the source. If an update has not refreshed Explorer, sign out and back in before repeating. The diagnostic script scripts/test-packaged-context-menu.ps1 in the source repository checks installed packaged COM activation and the actual classic menu separately; it does not register extensions or invoke conversion commands.

The four tabs, profile manager, JSON import/export, allowed-input filters, editor, A4/A5 text pages, size/duration limits, advanced FFmpeg controls and local history are unchanged from 0.7.0. Native image and PDF operations work without external engines. Existing names are preserved. A queued profile keeps its settings snapshot. An impossible size limit does not publish an oversized result. Audio/video needs FFmpeg; PDF image/text export needs Poppler; Office/Excel/HTML needs LibreOffice; OCR needs Tesseract models and Poppler for scanned PDFs; HEIC/AVIF needs libheif.

The MSIX contains the GUI app and Explorer COM DLL, native runtime installer and notices; the separate CLI is in the GitHub distribution. It runs asInvoker with no service, driver or elevation.

Original code is GPL-3.0-only. Corresponding source with vendored dependencies and offline build instructions is published with release v0.7.1 at https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.1. WACK and a real Windows 10 host test have not been run.
```

## Verifiche dei sorgenti e degli artefatti

I controlli della 0.7.1 hanno completato: **37** verifiche branding, **51** fixture Store con validazione semantica MakeAppx senza `/nv`, **26** fixture del pacchetto sparse, **7** roundtrip dei literal installer, **6** casi di disinstallazione e **21** controlli dei sorgenti GPL. Il controllo di parità confronta **93 estensioni** e rifiuta due fixture incoerenti. Anche la [CI del commit di release](https://github.com/salvatorecorsi/convert-here/actions/runs/42698306310) risulta riuscita. La logica Rust e le dipendenze non sono cambiate in questa patch.

Setup, portable, sorgenti e MSIX sono stati ricostruiti e auditati dal commit di release pulito. Gli hash degli asset GitHub caricati corrispondono a quelli locali. Il portable contiene **84 file**, corrispondenti allo staging; i tre binari interni sono x64, con CRT statico e senza import di motori esterni. Setup e binari standalone sono privi di firma Authenticode; l'installer non è stato eseguito in questo audit.

| Artefatto | Byte | SHA-256 |
| --- | ---: | --- |
| `ConvertHere-0.7.1-windows-x64-setup.exe` | 11749805 | `57baae391bbc6131ea3d7dbbd82b0bc4f28016cc979c42336627be16bf26f17b` |
| `ConvertHere-0.7.1-windows-x64-portable.zip` | 14990007 | `c0b1ed27b7fa7f2c8a02905ebe38783c28be9ef25db1914de8c7c94db717803b` |
| `ConvertHere-0.7.1-source.zip` | 82724658 | `a8db6bd83383322366081bba9eeadaa16d1af69bf440644da71a87eef7dbb728` |
| `ConvertHere-1.0.7.0-x64-store.msix` | 7441267 | `60ec178179ade97e1f5f23c40a26c259a25da2ed48cea78c1a1456927ec55d87` |

Il MSIX comprende **57 file payload**, verificati con hash individuali, su **59 elementi** dell'archivio. Sono stati controllati identità, versione **1.0.7.0**, GUI **0.7.1**, DLL del crate **0.7.1**, `asInvoker`, MinVersion **19041**, verbo **`0000ConvertHere`** e unica capability **`runFullTrust`**. La DLL non contiene una risorsa PE di versione. Non sono inclusi CLI, motori di conversione o downloader PowerShell; i due URL e i pin SHA-256 dell'installer nativo FFmpeg/Poppler sono presenti nella GUI. Non risultano import CRT dinamici. Il testo GPLv3 è completo, i componenti con notice sono **243** e **26 file di licenza** del MSIX corrispondono al commit. Il pacchetto inviato è unsigned; non è stato installato o registrato localmente.

L'archivio dei sorgenti include **186 file tracciati**, **431 crate vendorizzati** e **22801 file vendor** verificati. È stato estratto in una cartella isolata e sottoposto a veri `cargo check --locked --offline --all-targets` per applicazione e DLL, entrambi conclusi con codice **0**, usando un `CARGO_HOME` inizialmente vuoto e il compilatore **1.99.1** corrispondente alla provenienza. Sono controlli di compilazione senza linking finale; le build release degli artefatti sono verifiche distinte. Il digest dell'archivio sorgente è rimasto invariato dopo i controlli.

I report locali di questa esecuzione sono `.tools/standalone-071-audit.json`, `.tools/standalone-071-payload-audit.json`, `.tools/store-071-audit.json` e `.tools/source-offline-071-fa491735cf384b60ac134a55afc23b9d/offline-check.json`. Documentano le verifiche statiche della nuova build, senza esecuzione del payload.

## Schermate e submission

Le gallerie italiana e inglese salvate e rilette nel portale contengono due PNG **2562 × 1430**, **Impostazioni** e **Profili**, senza puntatore. Firma PNG, dimensioni e hash sono verificati. Le catture native provengono dalla build **debug 0.7.0**, con interfaccia invariata nella 0.7.1 e zoom nativo al 150%; gli originali JPEG sono stati convertiti in PNG senza ridimensionamento. Non sono prove di esecuzione del nuovo MSIX. Provenienza e checksum completi sono nel [registro delle catture](store-assets/0.7.1/README.md).

Le schede IT/EN, i riferimenti alla licenza, le nuove gallerie e le istruzioni di test sono stati salvati e riletti prima dell'invio. La schermata finale di Partner Center mostra **Submission 4 / In certification** e la pubblicazione automatica appena superata la certificazione. Il materiale precedente con puntatore è escluso dalla galleria corrente; gli artefatti e i report storici restano invariati.

## Checklist della 0.7.1

- [x] Controlli dei sorgenti 0.7.1 e fixture del manifest/menu, distinti dalla suite 0.7.0.
- [x] Audit di setup, portable, sorgenti e MSIX finali: commit pulito, versioni 0.7.1/1.0.7.0, identità, MinVersion 19041, hash, import PE e licenze.
- [x] Veri `cargo check --locked --offline --all-targets` per app e DLL dai sorgenti estratti con cache inizialmente vuota; distinti dalle build release con linking.
- [x] Catture native più alte, provenienza e formato reale verificati prima dell'inserimento nelle schede.
- [ ] Attivazione COM packaged e menu classico effettivo su **MSIX 1.0.7.0 installato**: non eseguiti. Il probe sul PC Windows 11 build 26200 riguardava l'installazione precedente **1.0.6.0** e non attesta la nuova registrazione.
- [ ] Menu su host **Windows 10 reale**: non verificato.
- [ ] Conversione nativa da Explorer, coda, collisione del nome e originale invariato con la nuova release: non eseguita.
- [x] Upload, salvataggio/rilettura campi IT/EN e istruzioni, invio e verifica dello stato **In certification** con pubblicazione automatica.
- [ ] WACK: non eseguito.

La certificazione è in corso. Non sono dichiarati approvazione Store, installazione del MSIX 1.0.7.0, test runtime su Windows 10 o WACK.