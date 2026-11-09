# Installs only the selected Explorer integration, for the current user.
[CmdletBinding(SupportsShouldProcess)]
param(
    [ValidateSet('Classic', 'Modern')][string]$Mode = 'Classic',
    [string]$InstallDirectory,
    [string]$PackagePath,
    [ValidateSet('system', 'it', 'en')][string]$UiLanguage = 'system'
)
$ErrorActionPreference = 'Stop'
# An explicit choice wins; otherwise use the app preference before Windows.
if (-not $PSBoundParameters.ContainsKey('UiLanguage') -and $env:APPDATA) {
    $preferencesFile = Join-Path $env:APPDATA 'SalvatoreCorsi\TheConverter\config\preferences.json'
    try {
        if ((Get-Item -LiteralPath $preferencesFile -ErrorAction Stop).Length -le 65536) {
            $savedLanguage = (Get-Content -LiteralPath $preferencesFile -Raw -Encoding UTF8 | ConvertFrom-Json).ui_language
            if ($savedLanguage -cin @('system', 'it', 'en')) { $UiLanguage = $savedLanguage }
        }
    } catch { } # Missing or corrupt preferences fall back to the system language.
}
$useEnglish = $UiLanguage -eq 'en' -or ($UiLanguage -eq 'system' -and [Globalization.CultureInfo]::CurrentUICulture.TwoLetterISOLanguageName -ne 'it')
function Get-LocalizedText([string]$Italian, [string]$English) {
    if ($useEnglish) { $English } else { $Italian }
}
$env:PSModulePath = (Join-Path $PSHOME 'Modules') + [IO.Path]::PathSeparator + $env:PSModulePath
if ($Mode -eq 'Modern' -and $PSVersionTable.PSEdition -eq 'Core') {
    # Appx is a Windows PowerShell module; avoid PowerShell 7 compatibility remoting.
    $systemDirectory = if ([Environment]::Is64BitProcess) { 'System32' } else { 'Sysnative' }
    $windowsPowerShell = Join-Path $env:WINDIR "$systemDirectory\WindowsPowerShell\v1.0\powershell.exe"
    $forward = @('-NoLogo', '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $PSCommandPath, '-Mode', $Mode, '-UiLanguage', $UiLanguage)
    if ($InstallDirectory) { $forward += @('-InstallDirectory', $InstallDirectory) }
    if ($PackagePath) { $forward += @('-PackagePath', $PackagePath) }
    if ($WhatIfPreference) { $forward += '-WhatIf' }
    # powershell.exe 5.1 -File cannot bind explicit Boolean switch arguments.
    if ($PSBoundParameters.ContainsKey('Confirm') -and [bool]$PSBoundParameters['Confirm']) { $forward += '-Confirm' }
    & $windowsPowerShell @forward
    if ($LASTEXITCODE -ne 0) { throw (Get-LocalizedText "Registrazione menu Windows 11 fallita (PowerShell: $LASTEXITCODE)." "Windows 11 menu registration failed (PowerShell: $LASTEXITCODE).") }
    return
}
if (-not $InstallDirectory) {
    $adjacentApp = Join-Path $PSScriptRoot '..\the-converter.exe'
    if (Test-Path -LiteralPath $adjacentApp -PathType Leaf) { $InstallDirectory = Join-Path $PSScriptRoot '..' }
    else { $InstallDirectory = Join-Path $PSScriptRoot '..\dist\ConvertHere' }
}
$installPath = (Resolve-Path -LiteralPath $InstallDirectory).Path
$executable = Join-Path $installPath 'the-converter.exe'
$library = Join-Path $installPath 'the_converter_shell.dll'
foreach ($file in @($executable, $library)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw (Get-LocalizedText "File mancante: $file. Esegui prima build-release.ps1." "Missing file: $file. Run build-release.ps1 first.") }
}
if (-not [Environment]::Is64BitOperatingSystem -or -not [Environment]::Is64BitProcess -or $env:PROCESSOR_ARCHITECTURE -eq 'ARM64' -or $env:PROCESSOR_ARCHITEW6432 -eq 'ARM64') {
    throw (Get-LocalizedText 'Questa distribuzione shell è x64; usa una build della stessa architettura di Esplora file.' 'This shell distribution is x64; use a build matching the architecture of File Explorer.')
}
if ($Mode -eq 'Modern') {
    if ([Environment]::OSVersion.Version.Build -lt 22000) { throw (Get-LocalizedText 'Il menu moderno richiede Windows 11.' 'The modern menu requires Windows 11.') }
    if (-not $PackagePath) { $PackagePath = Join-Path $installPath 'TheConverter.Identity.msix' }
    $resolvedPackage = (Resolve-Path -LiteralPath $PackagePath).Path
    Import-Module (Join-Path $PSScriptRoot 'modern-menu-package.psm1') -Force
    $info = Get-ModernMenuPackageInfo -PackagePath $resolvedPackage -InstallDirectory $installPath -RequireTrustedSignature
    $savedWhatIf = $WhatIfPreference
    try { $WhatIfPreference = $false; Import-Module Appx -ErrorAction Stop } finally { $WhatIfPreference = $savedWhatIf }
    $allRegistered = @(Get-AppxPackage -Name $info.Name | Where-Object { $_.Name -ceq $info.Name })
    foreach ($otherIdentity in @($allRegistered | Where-Object { $_.Publisher -ine $info.Publisher })) {
        $otherPath = Get-ModernMenuExternalPath -Package $otherIdentity
        if (-not $otherPath -or [string]::Equals([IO.Path]::GetFullPath($otherPath).TrimEnd('\'), $installPath.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) {
            throw (Get-LocalizedText 'Una precedente identità usa un Publisher diverso. Rimuovi prima il menu con uninstall-context-menu.ps1 -Mode Modern -InstallDirectory e il percorso di questa installazione.' 'An existing identity uses a different Publisher. First remove its menu with uninstall-context-menu.ps1 -Mode Modern -InstallDirectory and the path of that installation.')
        }
    }
    $registered = @($allRegistered | Where-Object { $_.Publisher -ieq $info.Publisher })
    if ($registered.Count -gt 1) { throw (Get-LocalizedText 'Sono presenti più identità Convert Here per questo utente; registrazione interrotta.' 'Multiple Convert Here identities exist for this user; registration stopped.') }
    $existing = $registered | Select-Object -First 1
    $existingPath = $null
    if ($existing) {
        if ([version]$existing.Version -gt $info.Version) { throw (Get-LocalizedText 'È già registrata una versione più recente del menu Windows 11.' 'A newer version of the Windows 11 menu is already registered.') }
        $existingPath = Get-ModernMenuExternalPath -Package $existing
        if ([version]$existing.Version -eq $info.Version -and $existingPath -and
            [string]::Equals([IO.Path]::GetFullPath($existingPath).TrimEnd('\'), $installPath.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) {
            Write-Host (Get-LocalizedText 'Il menu Windows 11 è già registrato per questa installazione.' 'The Windows 11 menu is already registered for this installation.')
            return
        }
    }
    if ($PSCmdlet.ShouldProcess($resolvedPackage, (Get-LocalizedText 'Registra identità MSIX e menu Windows 11 per questo utente' 'Register MSIX identity and Windows 11 menu for this user'))) {
        try {
            Add-AppxPackage -Path $resolvedPackage -ExternalLocation $installPath -ErrorAction Stop
        } catch {
            $registrationError = $_
            $errorText = [string]$_.Exception + ' ' + [string]$_.ErrorDetails + ' ' + [string]$_
            # Never remove a working registration on trust, schema or unrelated errors.
            # Only an exact-version conflict with a verified different location is repairable.
            $conflict = $errorText -match '(?i)\b0x80073CF[9B]\b'
            $stillRegistered = @(Get-AppxPackage -Name $info.Name | Where-Object {
                $_.Name -ceq $info.Name -and $_.Publisher -ieq $info.Publisher -and [version]$_.Version -eq $info.Version
            })
            $conflictPath = $null
            if ($stillRegistered.Count -eq 1) { $conflictPath = Get-ModernMenuExternalPath -Package $stillRegistered[0] }
            if (-not $conflict -or -not $existing -or [version]$existing.Version -ne $info.Version -or
                -not $existingPath -or -not $conflictPath -or
                -not [string]::Equals([IO.Path]::GetFullPath($existingPath), [IO.Path]::GetFullPath($conflictPath), [StringComparison]::OrdinalIgnoreCase) -or
                [string]::Equals([IO.Path]::GetFullPath($conflictPath).TrimEnd('\'), $installPath.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) {
                throw $registrationError
            }
            if ($PSCmdlet.ShouldProcess($stillRegistered[0].PackageFullName, (Get-LocalizedText "Aggiorna il percorso esterno da $conflictPath a $installPath" "Update external path from $conflictPath to $installPath"))) {
                Remove-AppxPackage -Package $stillRegistered[0].PackageFullName -ErrorAction Stop
                Add-AppxPackage -Path $resolvedPackage -ExternalLocation $installPath -ErrorAction Stop
            } else { return }
        }
        Write-Host (Get-LocalizedText 'Menu Windows 11 registrato. Se non appare, riavvia Esplora file o accedi nuovamente a Windows.' 'Windows 11 menu registered. If it does not appear, restart File Explorer or sign in to Windows again.')
    }
    return
}
$classId = '{67B51BA1-8A78-4B4B-A5F9-08A825BE91F8}'
$classKey = "HKCU:\Software\Classes\CLSID\$classId"
$menuKey = 'HKCU:\Software\Classes\*\shell\TheConverter'
# Canonical AQS uses only the fast file-extension property, independent of locale.
# Keep this input whitelist aligned with target_mask in the Explorer DLL.
$classicExtensions = @('mp3', 'mpga', 'wav', 'flac', 'aac', 'm4a', 'ogg', 'opus', 'wma', 'aiff', 'aif', 'alac', 'ape', 'm4b', 'mka', 'mp2', 'oga', 'weba', 'ac3', 'au', 'caf', 'amr', 'dts', 'tta', 'voc', 'wv', 'tak', 'w64', 'aifc', 'ra', 'mp4', 'mkv', 'mov', 'avi', 'webm', 'wmv', 'm4v', 'mpeg', 'mpg', 'ts', 'mts', 'm2ts', '3gp', 'flv', 'asf', 'vob', 'ogv', 'm2v', 'mxf', 'f4v', 'divx', '3g2', 'm2t', 'mpe', 'mpv', 'png', 'jpg', 'jpeg', 'webp', 'bmp', 'tif', 'tiff', 'gif', 'ico', 'qoi', 'tga', 'ff', 'hdr', 'exr', 'pnm', 'pam', 'pbm', 'pgm', 'ppm', 'avif', 'heic', 'heif', 'pdf', 'xls', 'xlsx', 'xlsm', 'ods', 'csv', 'html', 'htm', 'doc', 'docx', 'odt', 'rtf', 'txt', 'ppt', 'pptx', 'odp')
$appliesTo = ($classicExtensions | ForEach-Object { 'System.FileExtension:=".' + $_ + '"' }) -join ' OR '
if ($PSCmdlet.ShouldProcess('HKCU\Software\Classes', (Get-LocalizedText 'Installa il comando classico Convert Here' 'Install the classic Convert Here command'))) {
    New-Item -Path $classKey -Force | Out-Null
    Set-Item -LiteralPath $classKey -Value 'Convert Here Explorer Command'
    $server = New-Item -Path "$classKey\InprocServer32" -Force
    Set-Item -LiteralPath $server.PSPath -Value $library
    New-ItemProperty -LiteralPath $server.PSPath -Name 'ThreadingModel' -Value 'Apartment' -PropertyType String -Force | Out-Null
    New-Item -Path $menuKey -Force | Out-Null
    New-ItemProperty -LiteralPath $menuKey -Name 'MUIVerb' -Value (Get-LocalizedText 'Converti in' 'Convert to') -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath $menuKey -Name 'Icon' -Value ('"' + $executable + '",0') -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath $menuKey -Name 'ExplorerCommandHandler' -Value $classId -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath $menuKey -Name 'MultiSelectModel' -Value 'Player' -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath $menuKey -Name 'AppliesTo' -Value $appliesTo -PropertyType String -Force | Out-Null
    Write-Host (Get-LocalizedText 'Menu classico installato per questo utente. Su Windows 11 si trova in Mostra altre opzioni.' 'Classic menu installed for this user. On Windows 11 it appears under Show more options.')
}
