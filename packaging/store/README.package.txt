Convert Here
Developed and published by Salvatore Corsi

Converts images locally and integrates compatible conversions into the Windows
File Explorer context menu. Choose a format to start a queued conversion with
your saved settings. Custom settings opens the application's options window.

Audio, video, PDF extraction and Office document conversions require compatible
conversion tools. In Settings > Engines, explicitly confirm installation of
FFmpeg or Poppler from the pinned upstream packages, or select existing tools.
LibreOffice must be installed separately. Engines are stored in user data, outside
the read-only package. No engines or PowerShell installation scripts are bundled.

This Microsoft Store package provides the graphical application and File Explorer
commands. For terminal commands, use the separate portable/GitHub distribution;
the console CLI is not included in the Store package.

The Microsoft Store manages installation, updates and removal. The package
directory is read-only; settings and queue data use the user's application-data
folders. Converted documents remain at the destination selected by the user.

Application code is GPL-3.0-only; see LICENSE. Corresponding source and locked Rust
dependencies are provided in the GitHub release's source.zip archive. Dependencies
retain their own licenses; see THIRD-PARTY-NOTICES.md and Docs\licenses.
