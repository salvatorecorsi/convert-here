# Convert Here — full MSIX for Microsoft Store

This packaging path targets **Windows 11 x64, build 22000 or later**, and contains one graphical application, the Explorer COM DLL, execution-alias metadata, app logos and third-party notices. It uses `packagedClassicApp`, a visible app-list entry and `runFullTrust`. It does not use sparse identity or external application content. The console CLI, FFmpeg, Poppler, LibreOffice and the PowerShell runtime downloader are excluded; terminal commands remain available in the separate portable/GitHub distribution.

Application **0.4.0** is [published on GitHub](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.4.0) from clean commit `b4d5d0866a59b821125e35053566c454dd598471` (4 November 2026, 18:56:51 UTC). [Windows checks 41890745922](https://github.com/salvatorecorsi/convert-here/actions/runs/41890745922) passed, including the source and runtime-wrapper fixtures. Store package **1.0.3.0** is built, audited, uploaded, saved and **Validated**: **6348284 bytes**, SHA-256 `2b042f23f96e5d12d2cd1ca104bf6c5252214b83df4718a8eed11e1a2e4fbbd9`. **Submission 2**, ID **1152921505702316905**, was submitted on **4 November 2026** and shows **Update in certification / Product update: In certification**. Submission is Complete, Pre-processing is In progress, Certification and Publishing are Not started. The 0.4.0 update is not yet approved or live in the Store; publication will start automatically after certification passes.

Defaults show MP3, WAV, JPG, PNG, WebP, GIF, PDF and MP4 in the GUI and Explorer menu. All 93 input extensions and 44 engine outputs remain supported; display preferences do not block the CLI, explicit quick requests or existing queue jobs.

Engine downloads require confirmation of source, version, size, licenses and user-folder destination. HTTPS requests use the named GitHub distributions and CDNs; archives are verified using SHA-256 before extraction under `%LOCALAPPDATA%\ConvertHere\tools`, retaining upstream licenses and documentation. Existing local engines can still be selected. The MSIX contains no bundled engines or PowerShell downloader; the optional native download workflow belongs to the app. Conversion files are not uploaded, and conversion/OCR remain local.

The project's original code is **GPL-3.0-only**, Copyright (C) 2026 Salvatore Corsi. The package preserves complete `LICENSE` and third-party notices. The public `ConvertHere-0.4.0-source.zip` contains both lockfiles, 429 vendored crates and an offline build recipe; an actual offline all-targets check of the app and DLL passed using an initially empty Cargo home and the exact Rust 1.99.1 compiler. Versioned [LICENSE](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/LICENSE) and [privacy](https://github.com/salvatorecorsi/convert-here/blob/v0.4.0/docs/privacy.md) URLs are publicly verified. Italian and English listings are saved with 17 features and four screenshots; 3446-character testing notes and the 433-character full-trust reason persist after saving. The new MSIX has 57 payload and 59 archive entries, with 236 Windows dependency notices. A separate smoke test loaded its extracted COM DLL without registering it; the new MSIX was not installed and WACK was not run.

## Previous verified 0.3.2 / Store 1.0.2.0

Application **0.3.2** is [published on GitHub](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.2) from commit `8f7532bf9d16f80ef65b5229dba93929e11c1d82`. The public publisher name **Salvatore Corsi** is approved: Verification Summary is **Authorized**, the account is **Active**, and Public Info Publisher Name is **Salvatore Corsi**. Store package **1.0.2.0** has been built, audited, uploaded, saved and **Validated**, replacing 1.0.1.0. Its size is **6165973 bytes**, SHA-256 `c6d655b58156e4a14e13653a0f5115f3bffc2ba5ff086dbe435bc34440a053b7`.

All six sections were Complete before Submission 1, ID **1152921505702314630** for product **9NPKPPCBNBM8**. Application 0.3.2 / Store 1.0.2.0 is **In Microsoft Store**, and installation of the full 1.0.2.0 package with **SignatureKind: Store** was observed. The earlier certification timeline is historical. The 0.3.2 privacy URL and 1977-character testing notes were saved and reloaded; that version reused the three native 0.3.1 screenshots.

The previously submitted 0.3.1 identity was verified in Partner Center on 4 November 2026: `Name` **SalvatoreCorsi.ConvertHere**, `Publisher` **CN=B44BD46A-0514-4ADA-8B88-C6CD28C16E17**, `PublisherDisplayName` **Corsivo**, reserved name **Convert Here**, Store ID **9NPKPPCBNBM8**. The developer is **Salvatore Corsi**. Align the Git-ignored local `identity.partner-center.json` with the new display name only after it is confirmed in Partner Center; do not change the assigned `Name` or `Publisher` subject arbitrarily.

The previous application **0.3.1**, Store package **1.0.1.0**, is a free local file converter with no subscriptions, in-app purchases or usage-based charges. No payment is required to keep using it. The [GitHub release](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.3.1) is published from commit `884cb2c610a9197d8b4853f0fa741dbe9035b5ed`, with [Windows checks](https://github.com/salvatorecorsi/convert-here/actions/runs/41852511198) successful.

The previously submitted `ConvertHere-1.0.1.0-x64-store.msix` is **6165956 bytes**, SHA-256 `e5f63b0c59983ebde7209a1a2813a63c4df8941c1dc7d77f813bea91f5dfdfdd`, built from that clean commit. Partner Center reports **Packages: Complete / Validated** and replaced the earlier 1.0.0.0 automatically when the higher version was saved. Italian and English listings each contain three 0.3.1 screenshots with captions, the 300 x 300 logo, 15 features and seven keywords. Properties and Submission options are Complete, including the [versioned privacy URL](https://github.com/salvatorecorsi/convert-here/blob/v0.3.1/docs/privacy.md), 1977-character testing instructions and a 424-character `runFullTrust` reason. Pricing remains free in all 240 markets, with publication as soon as approved.

**Age ratings is Complete**, with IARC/Microsoft/PEGI 3+ ratings. The previous 1.0.1.0 submission was withdrawn before the 1.0.2.0 upload replaced its package. The 1.0.1.0 candidate was not installed or run through WACK; those build reports remain immutable. The later 1.0.2.0 publication and observed Store installation are recorded above. See [submission status](../../docs/store-submission.md).

## Preview build

```powershell
.\scripts\test-store-package.ps1
.\scripts\build-store-package.ps1
```

The preview is deliberately named `Corsivo.TheConverter.StorePreview`, displays **Convert Here Store Preview**, uses publisher display name **Salvatore Corsi** and development publisher `CN=Salvatore Corsi`. Its default package version is `1.0.0.0`. These are local preview settings and do not attest approval of the production publisher name. The preview must not be uploaded as the production app.

The script does not sign, install, register or trust the preview. Local installation is a separate action requiring an appropriately signed package and explicit authorization. Because preview and standalone Explorer extensions currently use the same COM CLSID, coexistence can cause duplicate menu entries or ambiguous activation during testing; verify the executable path of the invoked app or use a clean test profile.

## Partner Center candidate

Copy `identity.partner-center.example.json` to a local config file, then replace **Name** and **Publisher** with the exact Package/Identity values from Partner Center. **DisplayName** must match the reserved app name; **PublisherDisplayName** must match the account's publisher display name. Keep the account's actual spelling and punctuation.

```powershell
.\scripts\build-store-package.ps1 -ForSubmission -IdentityConfig C:\Build\corsivo-partner-center.json
```

`-ForSubmission` rejects missing fields, placeholder values and the preview identity before creating build output. The Store package version comes from the JSON config, independently of `Cargo.toml`: it must have a positive first component and the final component must be zero, for example `1.0.0.0`. Increase it for each subsequent submission. The Cargo application version is recorded separately in the report.

Microsoft signs accepted Store submissions, so this build intentionally produces an **unsigned** `.msix`; it needs no production signing certificate. `MakeAppx pack` runs normal semantic validation, without `/nv`. This verifies the package structure, not certification or app behavior.

## Build contract

- Windows x64, Rust MSVC, Visual Studio C++ Build Tools and Windows SDK MakeAppx are required. `-CargoPath` and `-MakeAppxPath` accept explicit executable paths.
- The Cargo cache defaults to `target/store-msix`; it does not overwrite standalone/sparse binaries. `-BuildDirectory` changes this cache.
- The PE application manifest is supplied through `THE_CONVERTER_APP_MANIFEST`. It requests `asInvoker` and omits the sparse `msix` element: the full MSIX grants runtime package identity. The GUI executable resource is checked after compilation.
- `+crt-static` avoids an undeclared Visual C++ runtime requirement. Data-only PE inspection checks x64 and rejects dynamic MSVCP/VCRUNTIME/MSVCR imports.
- Logos retain unqualified base files and provide StoreLogo scales 100/125/150/200/400, Square44/Square150 scales 100/200/400 and light/dark unplated target sizes 16/24/32/48/256. Their actual pixel dimensions are validated.
- Existing dependency notices must cover the locked Windows Cargo graph. If missing, first run `release-support/collect-rust-licenses.ps1`.
- Each run creates a fresh `dist/store/<preview|submission>/<version>-<UTC>-<id>/` directory. `-OutputDirectory` changes the parent directory, not the fresh staging rule. No previous output is recursively removed.

Each run returns the package path and produces `submission-report.json`, `SUBMISSION-REPORT.md`, `SHA256SUMS.txt`, tool logs and the exact package layout. The report records identity, both versions, source commit/dirty state, imported libraries, payload hashes and checks that were not performed. No account upload occurs.

The GUI execution alias is `corsivo-the-converter-preview.exe` for preview and `corsivo-the-converter.exe` for submission. The application declares multiple-instance support so quick conversions, queue workers and status windows can run alongside an existing GUI without activation being redirected to it. `store-runtime.json` next to the executable contains only the matching `execution_alias`; the application uses this stable alias for queue placeholders across Store updates.

Package names, execution aliases and COM CLSIDs remain stable technical identifiers across the public rename to Convert Here. The canonical source repository is [salvatorecorsi/convert-here](https://github.com/salvatorecorsi/convert-here).

A preliminary two-application package passed MakeAppx validation but WACK 1.0.1 reported a failed optional Store application-count test: the Store does not support more than one declared application. That warning was not treated as a clean certification result. This layout therefore declares exactly one GUI application and has no CLI binary or console alias. MakeAppx also rejects multiple app-execution-alias extensions inside one application, and the alias schema has no per-alias executable override, so no unsupported workaround is used. Final WACK testing remains separate from package generation.

The package declares only the implemented Italian UI language (`it-it`). An English Store listing is independent of application localization.

The previous 1.0.2.0 package is published in Microsoft Store and was observed installed with Store signature. The 1.0.3.0 update is now submitted and In certification. Free availability in all 240 markets and automatic publication after certification remain saved. Versioned privacy and GPL URLs, both listings, 3446-character testing notes and the 433-character full-trust explanation were reloaded and verified; phone/address fields remain blank. WACK is deprecated and optional; it has not been run on these candidates. A clean-machine app/menu/queue test of 1.0.3.0 is not attested. [Local validation](https://learn.microsoft.com/en-us/windows/msix/package/packaging-uwp-apps#validate-your-app-package-locally).

## Microsoft references

- [Generating full MSIX components for desktop apps](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-manual-conversion)
- [Runtime identity and read-only packaged application files](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes)
- [Store signs submitted packages](https://learn.microsoft.com/en-us/windows/msix/package/sign-msix-package-guide#production-microsoft-store-distribution)
- [Uploading MSIX packages](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/upload-app-packages)
- [App execution alias schema](https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-uap5-appexecutionalias)
- [Icon sizes and scale variants](https://learn.microsoft.com/en-us/windows/apps/design/style/iconography/app-icon-construction)
