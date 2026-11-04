# Validate source checks with isolated copies; no DLL activation or registration.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
$stage = Join-Path $temporaryRoot ('converter-shell-parity-' + [guid]::NewGuid().ToString('N'))
try {
    foreach ($relative in @('scripts/test-shell-extension.ps1', 'scripts/install-context-menu.ps1', 'installer/TheConverter.iss', 'shell-extension/src/lib.rs', 'src/model.rs')) {
        $target = Join-Path $stage $relative
        New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $repo $relative) -Destination $target
    }
    $validation = Join-Path $stage 'scripts/test-shell-extension.ps1'
    & $validation -ValidateRegistrationOnly
    $modelPath = Join-Path $stage 'src/model.rs'
    $model = Get-Content -LiteralPath $modelPath -Raw
    foreach ($case in @(
        @{ Name = 'new extension missing from Explorer'; From = '"png" |'; To = '"syntheticformat" | "png" |' },
        @{ Name = 'removed extension still offered by Explorer'; From = '"mp3" | "mpga" |'; To = '"mp3" |' }
    )) {
        $changed = $model.Replace($case.From, $case.To)
        if ($changed -ceq $model) { throw ('Fixture was not applied: ' + $case.Name) }
        [IO.File]::WriteAllText($modelPath, $changed, [Text.UTF8Encoding]::new($false))
        $rejected = $false
        try { & $validation -ValidateRegistrationOnly | Out-Null }
        catch {
            if ($_.Exception.Message -ne 'Application model and shell input extensions differ.') { throw }
            $rejected = $true
        }
        if (-not $rejected) { throw ('Inconsistent model was accepted: ' + $case.Name) }
        Write-Output ('Rejected fixture: ' + $case.Name)
    }
    Write-Output 'Source parity verified: matching sources accepted and both inconsistent fixtures rejected.'
} finally {
    $resolved = [IO.Path]::GetFullPath($stage)
    if (-not $resolved.StartsWith($temporaryRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or
        [IO.Path]::GetFileName($resolved) -notlike 'converter-shell-parity-*') {
        throw 'Unexpected parity fixture path; cleanup refused.'
    }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}
