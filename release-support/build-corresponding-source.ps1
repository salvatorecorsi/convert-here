# Export the exact committed application source and both locked Rust dependency
# graphs. This never downloads, packages or redistributes FFmpeg/Poppler binaries.
[CmdletBinding()]
param(
    [string]$Version,
    [string]$OutputDirectory,
    [string]$CargoPath = 'cargo',
    [string]$RustcPath = 'rustc',
    [string]$ExpectedCommit,
    [string]$DistributionDirectory,
    [string]$StoreIdentityConfig
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$commit = [string](& git -C $root rev-parse HEAD)
if ($LASTEXITCODE -ne 0 -or $commit -notmatch '^[0-9a-f]{40}$') { throw 'A committed Git source tree is required.' }
if (@(& git -C $root status --porcelain --untracked-files=normal).Count -ne 0) { throw 'Corresponding source requires a clean committed tree.' }
if ($ExpectedCommit -and $commit -cne $ExpectedCommit) { throw 'Source HEAD differs from the expected build commit.' }
$cargoSource = Get-Content -LiteralPath (Join-Path $root 'Cargo.toml') -Raw
$sourceVersion = [regex]::Match($cargoSource, '(?m)^version\s*=\s*"(\d+\.\d+\.\d+)"').Groups[1].Value
if (-not $Version) { $Version = $sourceVersion }
if ($Version -notmatch '^\d+\.\d+\.\d+$' -or $Version -cne $sourceVersion) { throw 'Source version differs from the requested release.' }
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $root "dist\releases\v$Version" }
$output = [IO.Path]::GetFullPath($OutputDirectory)
if ($CargoPath -eq 'cargo' -and -not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $CargoPath = Join-Path $root '.tools\cargo\bin\cargo.exe'
    $env:CARGO_HOME = Join-Path $root '.tools\cargo'
    $env:RUSTUP_HOME = Join-Path $root '.tools\rustup'
}
if ($RustcPath -eq 'rustc' -and (Test-Path -LiteralPath $CargoPath -PathType Leaf)) {
    $candidate = Join-Path (Split-Path -Parent $CargoPath) 'rustc.exe'
    if (Test-Path -LiteralPath $candidate -PathType Leaf) { $RustcPath = $candidate }
}
$cargoVersion = [string](& $CargoPath --version)
if ($LASTEXITCODE -ne 0) { throw 'Cargo version probe failed.' }
$rustcVersion = @(& $RustcPath --version --verbose)
if ($LASTEXITCODE -ne 0) { throw 'Rust compiler version probe failed.' }
$toolchain = [regex]::Match(($rustcVersion -join "`n"), '(?m)^rustc (\d+\.\d+\.\d+)\b').Groups[1].Value
if (-not $toolchain) { throw 'A stable numeric Rust toolchain is required for the source build recipe.' }
$provenance = $null
if ($DistributionDirectory) {
    $distribution = (Resolve-Path -LiteralPath $DistributionDirectory).Path
    $provenance = Get-Content -LiteralPath (Join-Path $distribution 'build-info.json') -Raw | ConvertFrom-Json
    if ($provenance.SourceCommit -cne $commit -or $provenance.SourceDirty -ne $false -or $provenance.ApplicationVersion -cne $Version) { throw 'Binary build provenance does not match this clean source commit/version.' }
    if (($provenance.RustcVersion -join "`n") -cne ($rustcVersion -join "`n")) { throw 'Source vendoring toolchain differs from the binary build.' }
    foreach ($name in @('the-converter.exe', 'the-converter-cli.exe', 'the_converter_shell.dll')) {
        if ($provenance.BinarySHA256.$name -cne (Get-FileHash -LiteralPath (Join-Path $distribution $name) -Algorithm SHA256).Hash.ToLowerInvariant()) { throw "Binary differs from recorded build: $name" }
    }
}
$storeIdentity = $null
if ($StoreIdentityConfig) {
    $configuration = Get-Content -LiteralPath $StoreIdentityConfig -Raw | ConvertFrom-Json
    $storeIdentity = [ordered]@{}
    foreach ($field in @('Name', 'Publisher', 'DisplayName', 'PublisherDisplayName', 'Version')) {
        if (-not $configuration.PSObject.Properties[$field] -or $configuration.$field -isnot [string] -or [string]::IsNullOrWhiteSpace($configuration.$field)) { throw "Missing public Store identity field: $field" }
        $storeIdentity[$field] = $configuration.$field
    }
}
$stageParent = Join-Path $root 'dist\source-stage'
$stage = Join-Path $stageParent ([guid]::NewGuid().ToString('N'))
$sourceName = "ConvertHere-$Version-source"
$export = Join-Path $stage $sourceName
New-Item -ItemType Directory -Path $export, $output -Force | Out-Null
Add-Type -AssemblyName System.IO.Compression.FileSystem
try {
    $tracked = @(& git -C $root ls-files --stage)
    if (@($tracked | Where-Object { $_ -match '^(160000|120000) ' }).Count) { throw 'Source exporter requires materialized submodule/symlink content; unsupported tracked link found.' }
    $snapshot = Join-Path $stage 'committed.zip'
    & git -C $root archive --format=zip "--output=$snapshot" $commit
    if ($LASTEXITCODE -ne 0) { throw 'Git source export failed.' }
    [IO.Compression.ZipFile]::ExtractToDirectory($snapshot, $export)
    foreach ($required in @('LICENSE', 'Cargo.toml', 'Cargo.lock', 'shell-extension\Cargo.toml', 'shell-extension\Cargo.lock', 'scripts\build-release.ps1', 'scripts\build-store-package.ps1')) {
        if (-not (Test-Path -LiteralPath (Join-Path $export $required) -PathType Leaf)) { throw "Committed source is incomplete: $required" }
    }
    if (Test-Path -LiteralPath (Join-Path $export 'vendor')) { throw 'Committed vendor directory must be reconciled before generating source.' }
    foreach ($config in @('.cargo\config', '.cargo\config.toml')) {
        if (Test-Path -LiteralPath (Join-Path $export $config)) { throw 'Existing Cargo configuration needs an explicit offline-source merge.' }
    }
    $locks = @{}
    foreach ($lock in @('Cargo.lock', 'shell-extension\Cargo.lock')) { $locks[$lock] = (Get-FileHash -LiteralPath (Join-Path $export $lock) -Algorithm SHA256).Hash }
    Push-Location -LiteralPath $export
    try {
        $savedPreference = $ErrorActionPreference
        try {
            $ErrorActionPreference = 'Continue'
            $vendorConfiguration = @(& $CargoPath vendor --locked --versioned-dirs --sync shell-extension/Cargo.toml vendor 2> (Join-Path $output 'source-vendor.log'))
            $vendorStatus = $LASTEXITCODE
        } finally { $ErrorActionPreference = $savedPreference }
        if ($vendorStatus -ne 0) { throw "Locked Cargo vendoring failed ($vendorStatus). See source-vendor.log." }
    } finally { Pop-Location }
    foreach ($lock in $locks.Keys) {
        if ((Get-FileHash -LiteralPath (Join-Path $export $lock) -Algorithm SHA256).Hash -cne $locks[$lock]) { throw "Vendoring modified committed lockfile: $lock" }
    }
    $configurationText = $vendorConfiguration -join "`n"
    if ($configurationText -notmatch '(?m)^\[source\.crates-io\]\s*$' -or $configurationText -notmatch '(?m)^replace-with\s*=\s*"vendored-sources"\s*$' -or $configurationText -notmatch '(?m)^directory\s*=\s*"vendor"\s*$') { throw 'Cargo did not return a relocatable vendor configuration.' }
    $vendors = @(Get-ChildItem -LiteralPath (Join-Path $export 'vendor') -Directory)
    if (-not $vendors.Count) { throw 'Vendored dependency sources are empty.' }
    foreach ($crate in $vendors) {
        foreach ($name in @('Cargo.toml', '.cargo-checksum.json')) {
            if (-not (Test-Path -LiteralPath (Join-Path $crate.FullName $name) -PathType Leaf)) { throw "Incomplete vendored dependency: $($crate.Name)/$name" }
        }
    }
    $cargoDirectory = Join-Path $export '.cargo'
    New-Item -ItemType Directory -Path $cargoDirectory | Out-Null
    [IO.File]::WriteAllText((Join-Path $cargoDirectory 'config.toml'), "[net]`noffline = true`n`n$configurationText`n", [Text.UTF8Encoding]::new($false))
    if (-not (Test-Path -LiteralPath (Join-Path $export 'rust-toolchain.toml'))) {
        [IO.File]::WriteAllText((Join-Path $export 'rust-toolchain.toml'), "[toolchain]`nchannel = `"$toolchain`"`nprofile = `"minimal`"`n", [Text.UTF8Encoding]::new($false))
    }
    [IO.File]::WriteAllText((Join-Path $export 'SOURCE-COMMIT.txt'), "$commit`n", [Text.UTF8Encoding]::new($false))
    if ($provenance) { Copy-Item -LiteralPath (Join-Path $distribution 'build-info.json') -Destination (Join-Path $export 'BUILD-PROVENANCE.json') }
    if ($storeIdentity) { [IO.File]::WriteAllText((Join-Path $export 'STORE-IDENTITY.json'), ($storeIdentity | ConvertTo-Json), [Text.UTF8Encoding]::new($false)) }
    $instructions = @"
Convert Here $Version - corresponding source
Commit: $commit
Cargo: $cargoVersion
Compiler:
$($rustcVersion -join "`r`n")

Prerequisites (install before building offline): Windows x64, the Rust toolchain
$toolchain with x86_64-pc-windows-msvc, Microsoft C++ Build Tools with MSVC x64
and the Windows SDK, PowerShell 5.1 or newer. Inno Setup is optional for setup.exe.
These general-purpose tools are not included in this archive.

The committed source, both Cargo.lock files, build/install scripts and vendored
Rust dependency sources are included. .cargo/config.toml uses relative vendor/
and offline=true. Cargo vendor --locked --sync included both workspaces and all
locked platforms, preserving upstream files, licenses and checksum manifests.

From this extracted folder in a developer PowerShell:
  `$env:RUSTFLAGS = '-C target-feature=+crt-static'
  cargo build --release --locked --offline
  cargo build --release --locked --offline --manifest-path shell-extension/Cargo.toml
For standalone release manifests/assets use scripts/build-release.ps1 -CargoPath cargo.
That script also supports this extracted archive without Git; such local rebuilds
are marked SourceDirty=true because no clean Git checkout can be verified.
For full MSIX use scripts/build-store-package.ps1 with -CargoPath cargo. If
STORE-IDENTITY.json is present, pass -ForSubmission -IdentityConfig STORE-IDENTITY.json.
Otherwise provide your public Partner Center identity using the committed example.
Build provenance, when present, records the original flags, publisher, toolchain,
commit and binary hashes. Follow those values for the corresponding binary build.
Signing certificates and credentials are not needed to compile unsigned binaries
and are not included. Signing/installing packages is outside the offline build.

Application-owned code: GPL-3.0-only, copyright 2026 Salvatore Corsi; see LICENSE.
Dependencies and bundled notices retain their own copyright and license terms.
See vendor/* license files, THIRD-PARTY-NOTICES.md and docs/release-licenses/.
FFmpeg/Poppler binaries or source archives are not bundled in this application
source archive. Runtime acquisition is optional; its pinned upstream source and
binary URLs remain in the committed downloader/install scripts.
"@
    [IO.File]::WriteAllText((Join-Path $export 'BUILDING-OFFLINE.txt'), $instructions, [Text.UTF8Encoding]::new($false))
    $archive = Join-Path $output "ConvertHere-$Version-source.zip"
    $temporaryArchive = Join-Path $output ('.source-' + [guid]::NewGuid().ToString('N') + '.zip')
    try {
        $zip = [IO.Compression.ZipFile]::Open($temporaryArchive, [IO.Compression.ZipArchiveMode]::Create)
        try {
            foreach ($file in Get-ChildItem -LiteralPath $export -File -Recurse -Force) {
                $relative = $sourceName + '/' + $file.FullName.Substring($export.Length + 1).Replace('\', '/')
                [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $file.FullName, $relative, [IO.Compression.CompressionLevel]::Optimal) | Out-Null
            }
        } finally { $zip.Dispose() }
        Move-Item -LiteralPath $temporaryArchive -Destination $archive -Force
    } finally { if (Test-Path -LiteralPath $temporaryArchive) { Remove-Item -LiteralPath $temporaryArchive } }
    [pscustomobject]@{ SourceArchive = $archive; Commit = $commit; Version = $Version; VendoredCrates = $vendors.Count; SHA256 = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() }
} finally {
    $checked = [IO.Path]::GetFullPath($stage)
    $parent = [IO.Path]::GetFullPath($stageParent).TrimEnd('\')
    if (-not $checked.StartsWith($parent + '\', [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($checked) -notmatch '^[a-f0-9]{32}$') { throw 'Unsafe corresponding-source cleanup path.' }
    if (Test-Path -LiteralPath $checked) { Remove-Item -LiteralPath $checked -Recurse -Force }
}
