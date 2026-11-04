# Isolated data-only Git repository and Cargo/Rustc fixtures. No network, Rust
# build, installer execution, certificate store or application data is touched.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
$fixture = Join-Path $temporaryRoot ('convert-here-source-tests-' + [guid]::NewGuid().ToString('N'))
$repository = Join-Path $fixture 'repository'
$output = Join-Path $fixture 'output'
$checks = 0
function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message }; $script:checks++ }
function Assert-Fails([scriptblock]$Action, [string]$Message) { $failed = $false; try { & $Action | Out-Null } catch { $failed = $true }; Assert $failed $Message }
function Write-Fixture([string]$Relative, [string]$Content) {
    $path = Join-Path $repository $Relative
    New-Item -ItemType Directory -Path (Split-Path -Parent $path) -Force | Out-Null
    [IO.File]::WriteAllText($path, $Content, [Text.UTF8Encoding]::new($false))
}
function Read-Entry($Archive, [string]$Name) {
    $entry = $Archive.GetEntry($Name)
    if (-not $entry) { throw "Missing source archive entry: $Name" }
    $reader = New-Object IO.StreamReader($entry.Open())
    try { $reader.ReadToEnd() } finally { $reader.Dispose() }
}
try {
    New-Item -ItemType Directory -Path $repository -Force | Out-Null
    $script = Join-Path $repository 'release-support\build-corresponding-source.ps1'
    New-Item -ItemType Directory -Path (Split-Path -Parent $script) -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $root 'release-support\build-corresponding-source.ps1') -Destination $script
    Write-Fixture 'Cargo.toml' "[package]`nname = `"source-fixture`"`nversion = `"0.4.0`"`nlicense = `"GPL-3.0-only`"`n"
    Write-Fixture 'Cargo.lock' '# exact application lockfile fixture'
    Write-Fixture 'shell-extension/Cargo.toml' "[package]`nname = `"shell-fixture`"`nversion = `"0.4.0`"`n"
    Write-Fixture 'shell-extension/Cargo.lock' '# exact independently locked shell fixture'
    Write-Fixture 'LICENSE' 'Application GPL license fixture'
    Write-Fixture 'src/lib.rs' '// committed application source'
    Write-Fixture 'scripts/build-release.ps1' '# committed Windows build instructions'
    Write-Fixture 'scripts/build-store-package.ps1' '# committed full MSIX build instructions'
    Write-Fixture '.gitignore' "private-ignored.txt`ndist/`n"
    Write-Fixture 'private-ignored.txt' 'Must never be exported'
    & git -C $repository init --quiet
    if ($LASTEXITCODE) { throw 'Fixture Git initialization failed.' }
    & git -C $repository add -- .
    & git -C $repository -c user.name=Fixture -c user.email=fixture@example.invalid commit --quiet -m 'Isolated source fixture'
    if ($LASTEXITCODE) { throw 'Isolated fixture commit failed.' }
    $commit = [string](& git -C $repository rev-parse HEAD)
    $cargo = Join-Path $fixture 'cargo-fixture.ps1'
    [IO.File]::WriteAllText($cargo, @'
if ($args[0] -eq '--version') { 'cargo 1.99.0 (fixture)'; $global:LASTEXITCODE = 0; return }
if (($args -join '|') -cne 'vendor|--locked|--versioned-dirs|--sync|shell-extension/Cargo.toml|vendor') { throw 'Incorrect locked dual-workspace vendoring arguments.' }
[IO.File]::WriteAllText((Join-Path $PSScriptRoot 'cargo-arguments.txt'), ($args -join '|'))
foreach ($crate in @('fixture-1.0.0', 'windows-only-2.0.0')) {
    $path = Join-Path (Get-Location) "vendor/$crate"
    New-Item -ItemType Directory -Path $path -Force | Out-Null
    [IO.File]::WriteAllText((Join-Path $path 'Cargo.toml'), '[package]')
    [IO.File]::WriteAllText((Join-Path $path '.cargo-checksum.json'), '{"files":{},"package":"fixture"}')
    [IO.File]::WriteAllText((Join-Path $path 'LICENSE-MIT'), "Upstream copyright and MIT license`nKeep unchanged")
}
if (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'change-lock')) { [IO.File]::WriteAllText((Join-Path (Get-Location) 'Cargo.lock'), 'Changed lockfile') }
'[source.crates-io]'
'replace-with = "vendored-sources"'
'[source.vendored-sources]'
'directory = "vendor"'
$global:LASTEXITCODE = 0
'@, [Text.UTF8Encoding]::new($false))
    $rustc = Join-Path $fixture 'rustc-fixture.ps1'
    [IO.File]::WriteAllText($rustc, "'rustc 1.99.0 (fixture 2026-01-01)'`n'host: x86_64-pc-windows-msvc'`n`$global:LASTEXITCODE = 0`n", [Text.UTF8Encoding]::new($false))
    $distribution = Join-Path $fixture 'distribution'
    New-Item -ItemType Directory -Path $distribution | Out-Null
    $hashes = [ordered]@{}
    foreach ($name in @('the-converter.exe', 'the-converter-cli.exe', 'the_converter_shell.dll')) {
        $path = Join-Path $distribution $name
        [IO.File]::WriteAllText($path, 'Inert binary fixture')
        $hashes[$name] = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $buildInfo = [ordered]@{ SourceCommit = $commit; SourceDirty = $false; ApplicationVersion = '0.4.0'; RustcVersion = @('rustc 1.99.0 (fixture 2026-01-01)', 'host: x86_64-pc-windows-msvc'); BinarySHA256 = $hashes }
    [IO.File]::WriteAllText((Join-Path $distribution 'build-info.json'), ($buildInfo | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))
    $identity = Join-Path $fixture 'identity.json'
    [IO.File]::WriteAllText($identity, '{"Name":"Fixture.Application","Publisher":"CN=fixture","DisplayName":"Fixture","PublisherDisplayName":"Fixture Publisher","Version":"1.0.3.0","PrivateCredential":"do-not-export"}', [Text.UTF8Encoding]::new($false))
    $parameters = @{ Version = '0.4.0'; OutputDirectory = $output; CargoPath = $cargo; RustcPath = $rustc; ExpectedCommit = $commit; DistributionDirectory = $distribution; StoreIdentityConfig = $identity }
    $result = & $script @parameters
    Assert ($result.Commit -ceq $commit -and $result.VendoredCrates -eq 2) 'Source report lost commit or vendor inventory.'
    Assert ([IO.Path]::GetFileName($result.SourceArchive) -ceq 'ConvertHere-0.4.0-source.zip') 'Corresponding source public filename is incorrect.'
    Assert ((Get-Content -LiteralPath (Join-Path $fixture 'cargo-arguments.txt') -Raw) -ceq 'vendor|--locked|--versioned-dirs|--sync|shell-extension/Cargo.toml|vendor') 'Both locked workspaces were not vendored.'
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($result.SourceArchive)
    try {
        $prefix = 'ConvertHere-0.4.0-source/'
        Assert ((Read-Entry $archive ($prefix + 'SOURCE-COMMIT.txt')).Trim() -ceq $commit) 'Archive is not tied to the exact commit.'
        Assert ((Read-Entry $archive ($prefix + 'Cargo.lock')) -ceq '# exact application lockfile fixture') 'Application lockfile changed.'
        Assert ((Read-Entry $archive ($prefix + 'shell-extension/Cargo.lock')) -ceq '# exact independently locked shell fixture') 'Shell lockfile changed.'
        Assert ((Read-Entry $archive ($prefix + 'LICENSE')) -ceq 'Application GPL license fixture') 'Application license missing or modified.'
        $config = Read-Entry $archive ($prefix + '.cargo/config.toml')
        Assert ($config.Contains('offline = true') -and $config.Contains('directory = "vendor"') -and -not $config.Contains($fixture)) 'Offline vendor config is missing or not relocatable.'
        Assert ((Read-Entry $archive ($prefix + 'rust-toolchain.toml')).Contains('channel = "1.99.0"')) 'Original Rust toolchain was not pinned.'
        Assert ((Read-Entry $archive ($prefix + 'vendor/fixture-1.0.0/LICENSE-MIT')) -ceq "Upstream copyright and MIT license`nKeep unchanged") 'Third-party license was overwritten/relicensed.'
        Assert ($null -ne $archive.GetEntry($prefix + 'vendor/windows-only-2.0.0/.cargo-checksum.json')) 'Vendor checksum metadata was omitted.'
        Assert ($null -ne $archive.GetEntry($prefix + 'BUILD-PROVENANCE.json')) 'Binary provenance missing.'
        $store = Read-Entry $archive ($prefix + 'STORE-IDENTITY.json') | ConvertFrom-Json
        Assert (@($store.PSObject.Properties).Count -eq 5 -and $store.PublisherDisplayName -ceq 'Fixture Publisher') 'Store identity whitelist did not preserve only public fields.'
        Assert (@($archive.Entries | Where-Object FullName -Match 'private-ignored|(^|/)\.git/|(^|/)target/').Count -eq 0) 'Ignored private files or build products leaked into source.'
        Assert ((Read-Entry $archive ($prefix + 'BUILDING-OFFLINE.txt')).Contains('MSVC') -and (Read-Entry $archive ($prefix + 'BUILDING-OFFLINE.txt')).Contains('--offline')) 'Windows offline build instructions are incomplete.'
    } finally { $archive.Dispose() }
    $originalHash = (Get-FileHash -LiteralPath $result.SourceArchive -Algorithm SHA256).Hash
    $bad = $parameters.Clone(); $bad.ExpectedCommit = ('0' * 40)
    Assert-Fails { & $script @bad } 'Different source commit accepted.'
    $bad = $parameters.Clone(); $bad.Version = '0.4.1'
    Assert-Fails { & $script @bad } 'Different application version accepted.'
    [IO.File]::AppendAllText((Join-Path $repository 'src/lib.rs'), '// dirty')
    Assert-Fails { & $script @parameters } 'Dirty source tree accepted.'
    & git -C $repository checkout -- src/lib.rs
    [IO.File]::WriteAllText((Join-Path $distribution 'the-converter.exe'), 'Replaced binary fixture')
    Assert-Fails { & $script @parameters } 'Binary/provenance mismatch accepted.'
    [IO.File]::WriteAllText((Join-Path $distribution 'the-converter.exe'), 'Inert binary fixture')
    [IO.File]::WriteAllText((Join-Path $fixture 'change-lock'), 'Inert toggle')
    Assert-Fails { & $script @parameters } 'Vendoring was allowed to modify a committed lockfile.'
    Assert ((Get-FileHash -LiteralPath $result.SourceArchive -Algorithm SHA256).Hash -ceq $originalHash) 'Rejected build replaced the valid source archive.'
    Write-Output "PASS: $checks corresponding-source checks; exact Git fixture, offline dual-workspace vendor config, provenance and third-party licenses; no network or compilation."
} finally {
    $checked = [IO.Path]::GetFullPath($fixture)
    if (-not $checked.StartsWith($temporaryRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($checked) -notlike 'convert-here-source-tests-*') { throw 'Unsafe source fixture cleanup path.' }
    if (Test-Path -LiteralPath $checked) { Remove-Item -LiteralPath $checked -Recurse -Force }
}
