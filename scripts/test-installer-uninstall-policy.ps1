[CmdletBinding()]
param([string]$InstallerPath)
$ErrorActionPreference = 'Stop'
if (-not $InstallerPath) { $InstallerPath = Join-Path $PSScriptRoot '..\installer\TheConverter.iss' }

$source = Get-Content -LiteralPath $InstallerPath -Raw -Encoding UTF8
$procedure = [regex]::Match($source, '(?ms)^procedure CurUninstallStepChanged\([^\r\n]+\r?\n.*?^end;')
if (-not $procedure.Success) { throw 'CurUninstallStepChanged source was not found.' }
$phasePattern = @'
(?s)if\s+CurUninstallStep\s*=\s*usUninstall\s+then\s*begin(?<body>.*?)SetIniString\('Integration',\s*'ModernMenuInstalled',\s*'0',
'@
$phase = [regex]::Match($procedure.Value, $phasePattern.Trim())
if (-not $phase.Success) { throw 'The uninstall phase and its subsequent state clear were not found.' }
$body = $phase.Groups['body'].Value
$statePattern = @'
(?m)^\s*(?<variable>\w+)\s*:=\s*GetIniString\('Integration',\s*'ModernMenuInstalled',\s*'(?<default>[^']*)',\s*ExpandConstant\('\{app\}\\installer-state\.ini'\)\)\s*=\s*'(?<installed>[^']*)';
'@
$state = [regex]::Match($body, $statePattern.Trim())
if (-not $state.Success) { throw 'Removal failure is not gated by the recorded ModernMenuInstalled state.' }
if ($state.Groups['default'].Value -cne '0' -or $state.Groups['installed'].Value -cne '1') {
    throw 'The stored modern-menu flag must default to 0 and require exactly 1.'
}
if ($body -match '(?m)^\s*#if.*ModernMenuAvailable') { throw 'A build flag must not override persisted modern-menu ownership.' }
$attemptPattern = @'
(?s)if\s+IsWindows11\s+then\s*(?:begin\s*)?if\s+not\s+RunScript\('uninstall-context-menu\.ps1',\s*'-Mode Modern -InstallDirectory '\s*\+\s*Quoted\(ExpandConstant\('\{app\}'\)\)\)\s+then\s*begin\s*(?<failure>.*?)\bend;
'@
$attempt = [regex]::Match($body, $attemptPattern.Trim())
if (-not $attempt.Success) { throw 'Modern-menu cleanup must still be attempted on Windows 11.' }
if ($state.Index -ge $attempt.Index) { throw 'Persisted state must be read before attempting removal.' }
$guardPattern = '(?s)if\s+' + [regex]::Escape($state.Groups['variable'].Value) + @'
\s+then\s+RaiseException\(CustomMessage\('ModernRemovalFailed'\)\)\s+else\s+Log\('(?:[^']|'')*'\);
'@.Trim()
if (-not [regex]::IsMatch($attempt.Groups['failure'].Value, $guardPattern)) {
    throw 'Only an installed modern menu may make removal failure fatal; other failures must be logged.'
}
if ([regex]::Matches($body, '\bRaiseException\(').Count -ne 1) { throw 'Unexpected unguarded exception in uninstall phase.' }

$cases = @(
    @{ Name = 'Windows 11, installed, removal failed'; Win11 = $true; Stored = '1'; Success = $false; Fatal = $true; Logged = $false },
    @{ Name = 'Windows 11, not installed, removal failed'; Win11 = $true; Stored = '0'; Success = $false; Fatal = $false; Logged = $true },
    @{ Name = 'Windows 11, missing state, removal failed'; Win11 = $true; Stored = $null; Success = $false; Fatal = $false; Logged = $true },
    @{ Name = 'Windows 11, other state, removal failed'; Win11 = $true; Stored = 'yes'; Success = $false; Fatal = $false; Logged = $true },
    @{ Name = 'Windows 11, installed, removal succeeded'; Win11 = $true; Stored = '1'; Success = $true; Fatal = $false; Logged = $false },
    @{ Name = 'Windows 10, modern cleanup not attempted'; Win11 = $false; Stored = '1'; Success = $false; Fatal = $false; Logged = $false }
)
Write-Output 'Static source-flow validation only; no installer, uninstaller, registry or Appx code is executed.'
foreach ($case in $cases) {
    $stored = $case.Stored
    if ($null -eq $stored) { $stored = $state.Groups['default'].Value }
    $installed = $stored -ceq $state.Groups['installed'].Value
    $failure = $case.Win11 -and -not $case.Success
    $fatal = $failure -and $installed
    $logged = $failure -and -not $installed
    if ($fatal -ne $case.Fatal -or $logged -ne $case.Logged) { throw ('Unexpected source policy: ' + $case.Name) }
    Write-Output ('PASS ' + $case.Name)
}
Write-Output ("Verified stored-state guard, ordering and {0} static policy cases." -f $cases.Count)
