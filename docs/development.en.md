# Development and distribution

[Italiano](development.md) · **English**

This guide covers building Convert Here, checking the code and preparing public releases with their corresponding source. Run PowerShell commands from the repository root. Release 0.5.0 includes the Italian/English interface in its installer and portable packages.

## Building

Use Windows x64, Rust stable MSVC and Visual Studio C++ Build Tools/Windows SDK:

```powershell
.\scripts\fetch-tools.ps1
cargo build --release --locked
cargo build --release --locked --manifest-path .\shell-extension\Cargo.toml
.\scripts\build-release.ps1 -SkipBuild
```

`fetch-tools.ps1` downloads pinned FFmpeg and Poppler versions and checks SHA-256 before copying them; it does not install anything system-wide. It is optional when using existing tools. `scripts\build-release.ps1` performs both builds. `-CertificateThumbprint` signs executables, DLLs and MSIX using an existing certificate and derives the Publisher for both manifests. MakeAppx is required to generate the identity package; the app also works without it.

Update notices before preparing public artifacts:

```powershell
.\release-support\collect-rust-licenses.ps1
```

Review and commit sources, lockfiles and notices, then build and prepare from the same commit with a clean working tree:

```powershell
.\scripts\build-release.ps1
.\scripts\prepare-public-release.ps1 -CompilerPath C:\Path\InnoSetup\ISCC.exe
```

Public staging uses an explicit file allowlist and rejects external engines or extra DLLs. For 0.5.0, it prepares `dist/releases/v0.5.0/` with the setup, portable archive, `ConvertHere-0.5.0-source.zip` and `SHA256SUMS.txt` for all three archives. Packages include the project license and dependency/font notices.

Preparation calls `release-support/build-corresponding-source.ps1`: it exports the Git commit, checks binary provenance and uses `cargo vendor --locked --versioned-dirs --sync shell-extension/Cargo.toml` for both lockfiles. The archive preserves upstream sources and licenses, Cargo configuration relative to `vendor/`, checksum metadata, the toolchain and `BUILDING-OFFLINE.txt`. Install the specified toolchain, x64 MSVC and Windows SDK, then build the app and extension offline with Cargo using that recipe. General development tools are not included; Inno Setup is only needed to generate the installer. External engines, certificates and credentials are excluded from the source archive.

For a distribution with the modern menu, use `prepare-public-release.ps1 -RequireModernMenu` after a build signed with a production certificate. The command includes MSIX and external logos or fails if a trusted signature is missing. Its installer enables the direct Windows 11 menu entry by default and removes it during uninstall. It does not import test certificates; see [Windows integration](windows-integration.en.md) for local procedures and signing requirements.

## Verification

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --manifest-path .\shell-extension\Cargo.toml
.\scripts\test-shell-extension.ps1
```

External tests use locally detected engines; the corresponding fixtures are explicitly skipped when FFmpeg, Poppler, LibreOffice, Tesseract or libheif is missing. Fixtures check codecs and container decoding, HEIC/AVIF and other image roundtrips, HDR precision, the four CSV separators with Unicode/quotes/multiple lines and formulas treated as text, the first Excel sheet, PDF-to-DOCX/HTML reconstruction with and without OCR, page order, PDF compression/merging/splitting, name collisions, cancellation and queue behavior.

Menu checks load the DLL and use Windows COM APIs without registering it; a parity check compares every extension and destination against the app model. Actual modern-menu visibility requires a signed installation and verification on Windows 11. Language checks should cover saved-language compatibility, system-language fallback, explicit language selection and both menu languages. Interface language and OCR recognition language are independent.

## License and corresponding source

Copyright (C) 2026 Salvatore Corsi. The original Convert Here code is distributed under **GNU General Public License version 3 only** (`GPL-3.0-only`), without warranty under the [license terms](../LICENSE).

`ConvertHere-0.5.0-source.zip` includes the release commit, both lockfiles, vendored Cargo dependencies, scripts and offline build instructions. Checksums cover setup, portable and source archives. The app's information section links to its license and sources.

Dependencies, fonts and external engines keep their own licenses: see [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md) and the [license texts](release-licenses/README.md).

[Back to README](../README.en.md)
