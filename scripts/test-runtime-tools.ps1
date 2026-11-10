# Inert CLI mocks only: this test never downloads, extracts or executes engines.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$wrapper = Join-Path $PSScriptRoot 'install-runtime-tools.ps1'
$temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
$fixture = Join-Path $temporaryRoot ('convert-here-runtime-wrapper-' + [guid]::NewGuid().ToString('N'))
$installRoot = Join-Path $fixture ('app con spazi & citt' + [char]0xE0)
$cli = Join-Path $installRoot 'the-converter-cli.exe'
$checks = 0
function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message }; $script:checks++ }
try {
    New-Item -ItemType Directory -Path $installRoot -Force | Out-Null
    [IO.File]::WriteAllText($cli, 'Inert test fixture. Never execute.')
    $global:convertHereRuntimeWrapperCalls = [Collections.Generic.List[string]]::new()
    $global:convertHereRuntimeWrapperFail = $false
    # PowerShell resolves this exact-path function before the inert EXE file.
    Set-Item -Path ('Function:global:' + $cli) -Value {
        param($Flag, $Engine)
        $global:convertHereRuntimeWrapperCalls.Add((@($Flag, $Engine) -join '|'))
        $global:LASTEXITCODE = $(if ($global:convertHereRuntimeWrapperFail) { 7 } else { 0 })
    }
    $source = Get-Content -LiteralPath $wrapper -Raw
    Assert ($source -notmatch 'Invoke-WebRequest|Expand-Archive|Start-Process|Remove-Item|Move-Item|Copy-Item') 'Wrapper must delegate installation, not implement another downloader or manipulate engine directories.'
    & $wrapper -InstallDirectory $installRoot | Out-Null
    Assert ($global:convertHereRuntimeWrapperCalls.Count -eq 2) 'Default invocation must request exactly two engines.'
    Assert ($global:convertHereRuntimeWrapperCalls[0] -ceq '--install-runtime|ffmpeg') 'FFmpeg argument boundaries changed.'
    Assert ($global:convertHereRuntimeWrapperCalls[1] -ceq '--install-runtime|poppler') 'Poppler argument boundaries changed.'
    Assert (-not (Test-Path -LiteralPath (Join-Path $installRoot 'tools'))) 'Wrapper must not install inside the application folder.'
    $global:convertHereRuntimeWrapperCalls.Clear()
    & $wrapper -InstallDirectory $installRoot -Engine @('poppler', 'poppler') | Out-Null
    Assert ($global:convertHereRuntimeWrapperCalls.Count -eq 1 -and $global:convertHereRuntimeWrapperCalls[0] -ceq '--install-runtime|poppler') 'Explicit engine selection must be deduplicated.'
    $global:convertHereRuntimeWrapperCalls.Clear()
    & $wrapper -InstallDirectory $installRoot -Engine @('tesseract', 'libheif', 'whisper', 'whisper') | Out-Null
    Assert (($global:convertHereRuntimeWrapperCalls -join ',') -ceq '--install-runtime|tesseract,--install-runtime|libheif,--install-runtime|whisper') 'Portable OCR, HEIF and local speech must delegate with exact arguments and no duplicate downloads.'
    $global:convertHereRuntimeWrapperCalls.Clear()
    $global:convertHereRuntimeWrapperFail = $true
    $rejected = $false
    try { & $wrapper -InstallDirectory $installRoot | Out-Null } catch { $rejected = $_.Exception.Message.Contains('codice 7') }
    Assert $rejected 'CLI errors must propagate from the wrapper.'
    Assert ($global:convertHereRuntimeWrapperCalls.Count -eq 1) 'A failed installation must stop before the next engine.'
    $global:convertHereRuntimeWrapperCalls.Clear()
    $rejected = $false
    try { & $wrapper -InstallDirectory $installRoot -Engine 'not-an-engine' | Out-Null } catch { $rejected = $true }
    Assert ($rejected -and $global:convertHereRuntimeWrapperCalls.Count -eq 0) 'Unknown engine must fail before invoking the CLI.'
    $rejected = $false
    try { & $wrapper -InstallDirectory $fixture | Out-Null } catch { $rejected = $_.Exception.Message.Contains('the-converter-cli.exe') }
    Assert $rejected 'Missing CLI must fail without downloading anything.'
    Assert ([IO.File]::ReadAllText($cli) -ceq 'Inert test fixture. Never execute.') 'The CLI fixture must remain unchanged.'
    Write-Output "Runtime wrapper: $checks checks passed."
} finally {
    Remove-Item -LiteralPath ('Function:global:' + $cli) -ErrorAction SilentlyContinue
    Remove-Variable -Name convertHereRuntimeWrapperCalls, convertHereRuntimeWrapperFail -Scope Global -ErrorAction SilentlyContinue
    $resolved = [IO.Path]::GetFullPath($fixture)
    if (-not $resolved.StartsWith($temporaryRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($resolved) -notlike 'convert-here-runtime-wrapper-*') { throw 'Unsafe test cleanup target.' }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}
