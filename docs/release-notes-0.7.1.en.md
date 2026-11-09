# Convert Here 0.7.1 — Store menu and screenshots

Application **0.7.1 [was published on GitHub](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.1)** on **9 November 2026 at 21:08:01 UTC**, from commit `2c6d6968159a46baef37d8f20aef049e7be4c8c4`; [Windows checks 42698306310](https://github.com/salvatorecorsi/convert-here/actions/runs/42698306310) **SUCCESS**. Developer and publisher: **Salvatore Corsi**. [Italiano](release-notes-0.7.1.md).

Microsoft Store package **1.0.7.0** was uploaded, validated and submitted in **Submission 4**, ID `1152921505702358188`. At **21:09 UTC** on 9 November 2026 the portal showed **Update in certification / Product update: In certification**: Submission Complete, Pre-processing In progress, Certification Not started, Publishing Not started. Automatic publication after approval is selected; 0.7.1 is not yet approved or published in the Store. The previous 0.7.0 / Store 1.0.6.0 update was already approved.

The Explorer menu manifest uses the command identifier **`0000ConvertHere`** as a workaround for the Store package's context menu not appearing on Windows 10. The visible name, package identity, CLSID, settings and conversion actions remain unchanged. Requirements remain **Windows 10 version 2004/build 19041 or later, x64**, including Windows 11.

Changing the identifier is not a real Windows 10 menu test: **no Windows 10 test host is available, so resolution on that OS has not been established**. `scripts/test-packaged-context-menu.ps1` checks packaged COM activation separately from construction of the actual classic menu after MSIX installation. A Windows 11 pass does not establish Windows 10 behavior.

The [current gallery](store-assets/0.7.1/README.md) contains two new Settings and Profiles captures, **2562 × 1430 pixels**, at native **150%** UI zoom without later upscaling. They come from debug build **0.7.0**, whose interface is unchanged in 0.7.1; they do not claim to capture the new build. Provenance, PNG format and digests are recorded in the gallery.

Features from [0.7.0](https://github.com/salvatorecorsi/convert-here/releases/tag/v0.7.0) remain unchanged: four tabs, a profile manager and JSON import/export, an image editor and A4/A5 pages, size and duration limits, advanced FFmpeg controls, history, **44 output formats and 93 recognized input extensions**. Conversion and OCR remain local and free. Optional engines are not bundled. Requested FFmpeg/Poppler downloads require confirmation and Internet; linked HTML resources may access the network through LibreOffice.

The release includes `ConvertHere-0.7.1-windows-x64-setup.exe`, `ConvertHere-0.7.1-windows-x64-portable.zip`, `ConvertHere-0.7.1-source.zip` with vendored Cargo dependencies and `SHA256SUMS.txt`. Original code remains **GPL-3.0-only**; dependencies, fonts and engines retain their licenses. No payment is required to keep using Convert Here. Published 0.7.0 tags, artifacts and reports are not rewritten.

Details: [testing instructions and evidence](store-testing-0.7.1.md), [Store status](store-submission.md), [Windows integration](windows-integration.en.md), [privacy](privacy.md).
