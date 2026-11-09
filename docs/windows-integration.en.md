# File Explorer integration

[Italiano](windows-integration.md) · **English**

Convert Here uses a Rust COM DLL implementing `IExplorerCommand`, the Windows contract for File Explorer commands. The **Convert to** submenu offers formats compatible with every selected file and enabled in Settings. Choosing MP3, WAV, PDF or another format immediately starts conversion with saved preferences. **Custom settings…** opens the window to change quality and destination. Conversion runs in a separate process from Explorer.

The updated sources support Italian and English. The submenu is **Converti in** in Italian and **Convert to** in English. Choose **System**, **Italiano** or **English** in the app's **Settings → Language**; System uses Italian for Italian Windows and English otherwise. Existing v0.4.0 packages keep their Italian UI. Interface language does not change format identifiers, CLI option names or output filenames.

MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4 are visible by default. Settings lets you choose them by category; the menu rereads output preferences on each opening without restarting Explorer. **Custom settings…** remains accessible even with every output hidden. The display filter does not change the 93 recognized extensions, 44 engine destinations or explicit quick requests and already queued jobs.

For one PDF, **PDF · compress** always applies lossless compression even when another PDF operation is saved. With multiple sources, **PDF (Bulk)** keeps separate results and **PDF (Merge)** explicitly opens the individual-page editor for one output. Select splitting in the main window or CLI. CLI merging follows argument order; the graphical editor lets you interleave pages before saving.

```powershell
.\the-converter-cli.exe --convert --target pdf --pdf-action split --files "C:\Documents\document.pdf"
.\the-converter-cli.exe --convert --target pdf --pdf-action merge --files "C:\Documents\first.pdf" "C:\Documents\second.pdf"
```

## Building for x64

Use Rust for `x86_64-pc-windows-msvc` with C++ Build Tools. The identity package requires Windows SDK MakeAppx. From the project folder:

```powershell
cargo build --release
cargo build --release --manifest-path .\shell-extension\Cargo.toml
.\scripts\build-release.ps1 -SkipBuild
```

The result is `dist\ConvertHere\`. Keep `the-converter.exe` and `the_converter_shell.dll` together in a stable location before registering the menu. Executable names and COM/MSIX identifiers stay stable across public product-name and language changes. The x64 DLL must match Explorer's architecture; the current package is not for ARM64 Explorer.

## Classic menu without administrator privileges

```powershell
.\scripts\install-context-menu.ps1 -Mode Classic -InstallDirectory .\dist\ConvertHere
.\scripts\uninstall-context-menu.ps1 -Mode Classic
```

On Windows 10, the command appears in the context menu. On Windows 11, this method appears under **Show more options**. It registers the DLL and verb under `HKCU\Software\Classes` without changing default file associations. Use `-WhatIf` to inspect the action without installing it. Uninstalling removes only the app's two keys.

## Modern Windows 11 menu

The modern menu needs **package identity** as well as `IExplorerCommand`; registry entries alone are insufficient. The project includes an external-content MSIX manifest, COM surrogate and `windows.fileExplorerContextMenus` declaration.

`build-release.ps1` prepares `TheConverter.Identity.msix` without installing it. A distributed package must be signed with a certificate trusted by the destination computer. The script derives the Publisher from the selected certificate's Subject in `Cert:\CurrentUser\My`, applies it to both the MSIX manifest and the executable's embedded manifest and rebuilds the app. Package version comes from `Cargo.toml`. Sign during preparation with:

```powershell
.\scripts\build-release.ps1 -CertificateThumbprint 'CERTIFICATE_THUMBPRINT'
.\scripts\install-context-menu.ps1 -Mode Modern -InstallDirectory .\dist\ConvertHere
```

Do not use `-SkipBuild` to change Publisher: executables must embed the same identity as the package. When reusing binaries, the script checks their actual embedded manifests before preparing the distribution.

Signing uses SHA-256 and an RFC 3161 timestamp to retain validity after certificate expiry. The default server is `http://timestamp.digicert.com`; `-TimestampServer` selects your signing service's server. A signing or timestamp error stops preparation.

The installation script checks signature, Publisher, version, COM server and external logos, then calls `Add-AppxPackage -ExternalLocation` for the current user. An identical existing registration is preserved. **The script does not create certificates, import trust or restart Explorer.** After a new registration, signing out and back into Windows may be necessary.

The setup includes MSIX and `Assets` only when the package signature is trusted on the build machine. In that case, its modern-menu option is selected by default on Windows 11. Successful registration removes the app's previous classic entry. Uninstalling first removes the identity associated with its folder; if Windows refuses removal, uninstalling stops and preserves the files.

For a public release with the modern menu, use a production signature recognized by destination computers and explicitly require the signed package:

```powershell
.\scripts\prepare-public-release.ps1 -RequireModernMenu
```

This command stops before producing artifacts if a trusted signature is missing; it does not substitute a classic-only setup. It also includes the identity package and referenced logos in the portable archive. Self-signed certificates are rejected for public releases.

### Development-certificate tests

A development certificate can be used for a local test. Microsoft's external-content package guide refers to `Cert:\CurrentUser\TrustedPeople`. In the recorded test on **27 October 2026, Windows 11 build 26200**, that store was insufficient: `Get-AuthenticodeSignature` returned `UnknownError` and `Add-AppxPackage` rejected the package with `0x800B0109`.

Microsoft's [general MSIX testing procedure](https://learn.microsoft.com/en-us/windows/msix/package/sign-msix-package-guide#testing-distribute-to-testers-with-a-self-signed-certificate) uses **Local Computer → Trusted People** (`Cert:\LocalMachine\TrustedPeople`). For the recorded configuration, manually importing only the public `.cer` into that store required administrator privileges, UAC and explicit consent because it extends trust to **every user on the PC**. Application registration still belongs only to the current user.

This step is separate from the installer: setup **never imports certificates automatically** or changes Developer Mode. Check the certificate's origin and thumbprint before import; keep the private `.pfx` confidential. A test certificate is not a production signature; remove it from each trust store when no longer needed. Uninstalling the app does not remove certificate trust. See Microsoft's [development-certificate trust guide](https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/security#what-winapp-cert-install-grants).

```powershell
.\scripts\uninstall-context-menu.ps1 -Mode Modern -InstallDirectory .\dist\ConvertHere
```

This removes the current user's identity package only if it belongs to the specified folder, leaving the app and preferences. It does not remove certificates. You can also register the classic menu separately to keep both menu entries.

## Multiple selections and process safety

The command accepts up to 10,000 items and passes absolute paths through a temporary UTF-8 JSON file. Quick formats directly launch `the-converter.exe --quick --target <format> --selection-file <path>`; the custom-settings command omits `--quick`. Spaces, Unicode and long selections therefore do not become a huge command line. Selected paths are never interpolated into PowerShell or `cmd`. The app validates paths and removes its disposable request file after reading it.

Selecting at least two PDF-compatible items offers **PDF (Bulk)** and **PDF (Merge)**. Bulk adds `--pdf-batch bulk` and keeps `--quick`: it queues individual results, losslessly compressing PDF inputs. Merge adds `--pdf-batch merge`, omits `--quick` and opens the interactive page editor. Images, PDFs, Word/Office documents, Excel/CSV and HTML are compatible; audio/video do not expose these actions. Merge is an action, not a new destination: the model stays at 44 formats and 93 extensions.

The editor prepares hidden temporary PDFs and lets you drag every individual page between sources. Arrows and Alt+↑ / Alt+↓ offer the same ordering. Initial order follows paths received from Explorer, with no guarantee of filename or date sorting. Only **Create merged PDF** saves an output; the session does not create persistent quick-queue jobs. The main app uses the same editor. Existing PDFs and native images do not need extra engines or Poppler for merging; Office/HTML/Excel need LibreOffice and HEIC/AVIF need libheif. Encrypted, signed PDFs and PDFs with interactive forms remain rejected.

The parent exposes `ECF_HASSUBCOMMANDS` and an `IEnumExplorerCommand` enumerator with `Next`, `Skip`, `Reset` and `Clone`. Each format's `GetState` checks the whole selection and hides incompatible conversions. Recognized extensions include AVIF, HEIC and HEIF. Audio/video selections share twelve audio outputs; mixed images/PDFs can offer PNG, JPEG, PDF (Bulk), PDF (Merge) and TXT subject to display preferences. Excel/CSV offer PDF, CSV and XLSX; PDFs also offer Word and HTML as text exports.

TXT uses **TXT · OCR** for images and **TXT** for embedded PDF text. Mixed images/PDFs show **TXT · OCR / PDF text**. Audio/video have no TXT destination. Image OCR stays local and needs Tesseract language data. Missing tools are reported by the app before processing; a menu entry does not prove its engine is available.

Internal masks are 64-bit with upper bits reserved for selection categories. Existing format and custom-settings GUIDs remain stable as formats are added. The classic-menu and installer allowlists match the DLL; parity tests compare every destination and the 93 extensions against the app model. For more than 16 items, the fast call returns `E_PENDING`, asking Windows to repeat the check in the background with `fOkToBeSlow`. The DLL does not probe codecs or external engines or open documents. Virtual items, empty selections and unknown extensions hide the command.

## Verification

Build and COM tests can run without registering the DLL:

```powershell
cargo test --manifest-path .\shell-extension\Cargo.toml
.\scripts\test-shell-extension.ps1
.\scripts\test-modern-menu-package.ps1
```

The smoke test loads the real DLL without registering it, activates COM objects and passes real Windows `IShellItemArray` selections. It checks compatible menus for audio/video, images/PDFs and documents/images; contextual OCR and PDF-compression titles; XLSX/CSV/HTML and QOI/AVIF/HEIF; string allocation and object release. Optional `-InvokeCaptureFile` is reserved for an isolated test helper that records launched-process arguments and checks quick conversion separately from opening settings.

Modern-package tests use fixtures and mocked Appx commands in a separate Windows PowerShell process. They cover invalid manifests/assets, mismatched signatures, `WhatIf`, repeated registration, folder/Publisher changes, removal of only the selected installation and error propagation. They do not modify actual certificates, registry entries or packages.

Historical records describe only the versions and configurations actually tested. The [Italian verification record](windows-integration.md#stato-di-verifica) documents the 27 October 2026 local test, including a working modern-menu WAV-to-MP3 conversion after explicitly authorized development-certificate import. It does not attest a newly signed public release or current installation/upgrade/uninstall behavior. Actual modern-menu visibility must be tested after signing and registering on Windows 11; an unsigned MSIX is not an installed menu. No registration happens automatically when starting or building the app.

## Microsoft references

- [File Explorer commands with IExplorerCommand and MSIX](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer)
- [External-content package identity: build, sign and register](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps)
- [Development signing for external-content packages](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps#build-and-sign-the-identity-package)
- [Test MSIX distribution and LocalMachine TrustedPeople](https://learn.microsoft.com/en-us/windows/msix/package/sign-msix-package-guide#testing-distribute-to-testers-with-a-self-signed-certificate)
- [Scope of development-certificate trust](https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/security#what-winapp-cert-install-grants)
- [Windows 11 context-menu requirements and behavior](https://blogs.windows.com/blog/2021/07/19/extending-the-context-menu-and-share-dialog-in-windows-11/)

[Back to README](../README.en.md)
