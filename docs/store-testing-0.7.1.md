# Candidata 0.7.1 / Store 1.0.7.0 — istruzioni e verifiche

Applicazione **0.7.1**, pacchetto Store previsto **1.0.7.0**, publisher **Salvatore Corsi**. La candidata non è ancora dichiarata pubblicata, caricata o inviata. Il record [0.7.0 / Store 1.0.6.0](store-testing-0.7.0.md) resta storico: i suoi test e checksum non vengono trasferiti automaticamente alla nuova build.

La modifica mirata è l'identificatore manifest **`0000ConvertHere`**, un workaround per il menu packaged non visibile su Windows 10. Restano MinVersion **10.0.19041.0**, architettura x64, identità e CLSID. **Non è disponibile un host Windows 10 reale: il menu su quel sistema non è ancora verificato.** Le catture più alte e gli audit del candidato finale saranno registrati dopo l'esecuzione.

## Additional Testing Information — bozza inglese

```text
Convert Here 0.7.1 / candidate MSIX 1.0.7.0. Publisher: Salvatore Corsi. Windows 10 version 2004/build 19041 or later, x64, including Windows 11. No account or payment is required. Conversion, editing and OCR are local. Optional conversion engines are not bundled; requested FFmpeg/Poppler downloads require confirmation, HTTPS and checksum validation. Linked HTML resources may access local files or the network through LibreOffice.

This update changes the packaged Explorer verb identifier to 0000ConvertHere as a workaround for a menu missing on Windows 10. The visible command, CLSID and conversion behavior remain unchanged. It has not been verified on a real Windows 10 host; do not infer that result from Windows 11 tests.

After installing the candidate, select a supported PNG/JPEG/PDF in File Explorer. On Windows 10 check the classic context menu; on Windows 11 check both the modern menu and Show more options. Look for Convert to, choose a compatible native output and verify the queued result without changing the source. If an update has not refreshed Explorer, sign out and back in before repeating. The diagnostic script scripts/test-packaged-context-menu.ps1 in the source repository checks installed packaged COM activation and the actual classic menu separately; it does not register extensions or invoke conversion commands.

The four tabs, profile manager, JSON import/export, allowed-input filters, editor, A4/A5 text pages, size/duration limits, advanced FFmpeg controls and local history are unchanged from 0.7.0. Native image and PDF operations work without external engines. Existing names are preserved. A queued profile keeps its settings snapshot. An impossible size limit does not publish an oversized result. Audio/video needs FFmpeg; PDF image/text export needs Poppler; Office/Excel/HTML needs LibreOffice; OCR needs Tesseract models and Poppler for scanned PDFs; HEIC/AVIF needs libheif.

The MSIX contains the GUI app and Explorer COM DLL, native runtime installer and notices; the separate CLI is in the GitHub distribution. It runs asInvoker with no service, driver or elevation.

Original code is GPL-3.0-only. Corresponding source with vendored dependencies will accompany the 0.7.1 release. These are candidate instructions, not a claim of upload, completed certification, installation testing or WACK execution.
```

## Checklist della nuova candidata

- [ ] Controlli dei sorgenti 0.7.1 e fixture del manifest/menu, distinti dalla suite 0.7.0.
- [ ] Audit di setup, portable, sorgenti e MSIX finali: commit pulito, versioni 0.7.1/1.0.7.0, identità, MinVersion 19041, hash, import PE e licenze.
- [ ] Veri `cargo check --locked --offline --all-targets` per app e DLL dai sorgenti estratti con cache inizialmente vuota; distinguere il check da una build completa con linking.
- [ ] Catture native più alte, provenienza e formato reale verificati prima dell'inserimento nelle schede.
- [ ] Attivazione COM packaged e menu classico effettivo su pacchetto installato; verificare separatamente Windows 10 e Windows 11.
- [ ] Conversione nativa da Explorer, coda, collisione del nome e originale invariato.
- [ ] Upload, salvataggio/rilettura campi IT/EN e istruzioni, invio e stato effettivo della certificazione.

Il WACK e un'installazione su host Windows 10 reale non sono dichiarati eseguiti. Nessun artefatto o report storico viene modificato.
