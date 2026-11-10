# Explicit manual alternative to Impostazioni > Motori. The application owns
# HTTPS, pinned hashes, safe extraction, cancellation, locking and rollback.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$InstallDirectory,
    [ValidateSet('ffmpeg', 'poppler', 'tesseract', 'libheif', 'whisper')][string[]]$Engine = @('ffmpeg', 'poppler')
)
$ErrorActionPreference = 'Stop'
$installRoot = (Resolve-Path -LiteralPath $InstallDirectory -ErrorAction Stop).ProviderPath
$cli = Join-Path $installRoot 'the-converter-cli.exe'
if (-not (Test-Path -LiteralPath $cli -PathType Leaf)) {
    throw 'La cartella selezionata non contiene the-converter-cli.exe di Convert Here.'
}
Write-Host 'Download esplicito di motori portabili verificati, senza eseguirli.'
Write-Host 'Destinazione personale: %LOCALAPPDATA%\ConvertHere\tools (non la cartella del programma).'
foreach ($name in ($Engine | Select-Object -Unique)) {
    Write-Host "Installazione di $name; Ctrl+C per annullare."
    $global:LASTEXITCODE = 0
    & $cli '--install-runtime' $name
    if ($LASTEXITCODE -ne 0) {
        throw "Installazione di $name non completata (codice $LASTEXITCODE)."
    }
}
Write-Host 'Motori pronti. Le licenze e la documentazione sono conservate nelle rispettive cartelle.'
