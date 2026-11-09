# Quick conversions visible in File Explorer

[Italiano](queued-conversions.md) · **English**

Choosing **Convert to → MP3**, **WAV** or another format queues the job and immediately shows a placeholder in the destination folder. Its name matches the planned result, for example `recording.mp3`, with a processing icon. The default destination is the original's folder; a different saved output folder is respected.

For multiple PDF-compatible sources, **PDF (Bulk)** uses the same queue: one job and separate output per source. Existing PDFs are compressed losslessly; images and documents become PDFs. **PDF (Merge)** opens a separate interactive session to create one PDF rather than persistent queue jobs.

The placeholder is a normal temporary Windows shortcut with the internal `.lnk` extension, which Explorer usually hides. It is separate from an incomplete audio, video or PDF file: double-clicking opens a small status window with the destination, progress and cancellation. It does not open a media player on incomplete data.

The Explorer icon is static and indicates a queued or running job. The status window displays updated progress. When finished, the real file replaces the placeholder with its normal icon and assigned name. An occupied name receives a numeric suffix without overwriting existing files.

Quick conversions share a persistent queue. Requests submitted while another conversion is running wait their turn. The options window does not open automatically and closing a status window does not cancel processing; use **Cancel** to stop the job. The visible-output filter applies to the GUI and menu, while explicit quick requests and already queued jobs retain their requested format.

History cleanup runs when the worker resumes or finishes the queue. Completed or cancelled records expire after 7 days and failed records after 30 days; cleanup also attempts to retain at most the 300 newest terminal records. It does not remove originals, results or active jobs. Unreadable records are quarantined and retained for inspection; unrelated files are preserved.

The hidden `.the-converter-reservations` folder contains only output-name reservations. It prevents simultaneous requests from choosing the same name and preserves the names displayed by placeholders. The folder is removed when the last job releases its reservation, including after an error or cancellation. Reservations for other queued jobs targeting that folder remain intact.

The **Quick queue** tab contains active jobs, their state, output name, destination folder and any error details. **History** contains finished jobs and the last quick-conversion report. Snapshots refresh in the background about twice per second.

After an error, the placeholder displays a red X; double-click to read the explanation. For a failed job, **Remove placeholder** deletes only shortcuts still owned by that job and keeps history and results. A shortcut replaced by another program is preserved and reported. A cancelled job removes its placeholder and temporary files. Existing files and files created by other programs are not overwritten. Forced process interruption is detected on the next start: interrupted work is reported and other waiting jobs proceed.

## One PDF with arranged pages

**PDF (Merge)** accepts compatible images, PDFs, Office documents, Excel/CSV and HTML, including mixed selections. Audio and video are excluded. After preparation, drag individual pages between sources, or use arrows and **Alt+↑ / Alt+↓** on the selected page. Initial ordering follows the files received from Explorer and original page order within each source; check it before choosing **Create merged PDF**.

Sources are prepared once in a hidden temporary user folder. Intermediate PDFs do not appear in the output folder; originals are unchanged. Saving creates one file named after the first source with the `-unito.pdf` suffix, adding a number on conflicts. This session and its page order are not saved in quick-queue `job.json` records. Cancellation and closing attempt to remove intermediates; an unexpected shutdown may leave residual files. The main app's Merge command opens the same editor. See [PDF requirements and limitations](conversions.en.md#compress-split-and-merge-pdfs).

## Text from images and PDFs

TXT is available for images through local OCR and for PDFs through text extraction or local OCR. Audio and video use FFmpeg for format conversion. See the [OCR guide](conversions.en.md#local-ocr-for-images-and-scanned-pdfs) and [privacy statement](privacy.md) for engine requirements and data handling.

## Language

Version 0.5.0 supports Italian and English for the app and Explorer commands. Choose the interface language in **Settings → Language**; the saved choice is independent of OCR models. Existing records may contain progress or diagnostic text saved under the language used when the job ran. See [language settings](configuration.en.md#interface-language).

[Back to README](../README.en.md)
