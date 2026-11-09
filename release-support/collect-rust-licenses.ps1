# Collect the exact Windows dependency notices from the locked Cargo graph.
[CmdletBinding()]
param([string]$CargoPath = 'cargo')
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$output = Join-Path $root 'docs\release-licenses'
New-Item -ItemType Directory -Force -Path $output | Out-Null
if ($CargoPath -eq 'cargo' -and -not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $CargoPath = Join-Path $root '.tools\cargo\bin\cargo.exe'
    $env:CARGO_HOME = Join-Path $root '.tools\cargo'
    $env:RUSTUP_HOME = Join-Path $root '.tools\rustup'
}
$packages = @{}
foreach ($manifest in @((Join-Path $root 'Cargo.toml'), (Join-Path $root 'shell-extension\Cargo.toml'))) {
    $raw = & $CargoPath metadata --format-version 1 --locked --filter-platform x86_64-pc-windows-msvc --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed.' }
    $metadata = ($raw -join "`n") | ConvertFrom-Json
    foreach ($package in $metadata.packages) {
        if ($package.source -like 'registry+*') { $packages[($package.name + '@' + $package.version)] = $package }
    }
}
$upstream = @{
    'clipboard-win' = @{ Repository = 'DoumanAsh/clipboard-win'; Files = @('LICENSE') }
    'ecolor' = @{ Repository = 'emilk/egui'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'eframe' = @{ Repository = 'emilk/egui'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'egui' = @{ Repository = 'emilk/egui'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'egui_glow' = @{ Repository = 'emilk/egui'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'egui-winit' = @{ Repository = 'emilk/egui'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'emath' = @{ Repository = 'emilk/egui'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'epaint' = @{ Repository = 'emilk/egui'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'epaint_default_fonts' = @{ Repository = 'emilk/egui'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'gl_generator' = @{ Repository = 'brendanzab/gl-rs'; Files = @('LICENSE') }
    'khronos_api' = @{ Repository = 'brendanzab/gl-rs'; Files = @('LICENSE') }
    'profiling' = @{ Repository = 'aclysma/profiling'; Files = @('LICENSE-MIT', 'LICENSE-APACHE') }
    'pulp-wasm-simd-flag' = @{ Repository = 'sarah-quinones/pulp'; Files = @('LICENSE') }
    'zune-inflate' = @{ Repository = 'etemesi254/zune-image'; Files = @('LICENSE.md', 'LICENSE-ZLIB') }
}

function Get-DeclaredWrapperMitNotice($Package, [string]$Source, [string]$ProjectRoot) {
    if ($Package.name -ne 'libwebp-sys' -or $Package.version -ne '0.14.4') { return $null }
    if ($Package.license -cne 'MIT') { throw 'The libwebp-sys declared-license fallback requires an explicit MIT package declaration.' }
    $standardPath = Join-Path $ProjectRoot 'release-support\licenses\SPDX-MIT.txt'
    $standardText = [IO.File]::ReadAllText($standardPath).Replace("`r`n", "`n")
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $standardHash = ([BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($standardText)))).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
    if ($standardHash -cne 'b05785f9f18e6716bab63424b11454513b9943a222595b70411009202fc592b5') { throw 'The local standard SPDX MIT text does not match its recorded source.' }
    $vcs = Get-Content -LiteralPath (Join-Path $Source '.cargo_vcs_info.json') -Raw | ConvertFrom-Json
    $metadataNote = @(
        'This is the standard text of the MIT license declared in the published package Cargo metadata.',
        'The published libwebp-sys 0.14.4 crate does not include a standalone MIT copyright/license notice for its Rust wrapper.',
        'The package author metadata is preserved in this component header; it is not asserted to be a copyright notice.',
        ('VCS commit recorded by the published package: ' + $vcs.git.sha1),
        'The literal year/copyright-holder fields below belong to the unfilled SPDX standard text. Convert Here has not supplied a year or copyright holder.',
        'The vendored C library COPYING and PATENTS are separate original upstream notices, reproduced independently.',
        ''
    ) -join "`n"
    return @{
        Name = 'MIT - standard text of the license declared by the Rust wrapper package'
        Text = $metadataNote + "`n" + $standardText
        Origin = 'https://raw.githubusercontent.com/spdx/license-list-data/v3.27.0/text/MIT.txt'
    }
}

$text = [Text.StringBuilder]::new()
[void]$text.AppendLine('Convert Here - third-party Rust dependency notices')
[void]$text.AppendLine('Generated from Cargo.lock and shell-extension/Cargo.lock for x86_64-pc-windows-msvc.')
[void]$text.AppendLine('Includes build-time dependencies as a conservative superset. The application has its own rights; these texts concern the identified third-party components only.')
$index = @()
foreach ($package in @($packages.Values | Sort-Object name,version)) {
    $source = Split-Path -Parent $package.manifest_path
    $files = @(Get-ChildItem -LiteralPath $source -Recurse -File | Where-Object {
        $_.Name -match '^(LICENSE|LICENCE|COPYING|NOTICE|COPYRIGHT|PATENTS)([._-]|$)' -or
        $_.Name -match '^(OFL|UFL)\.txt$' -or $_.Name -eq 'Hack-Regular.txt' -or $_.Name -eq 'emoji-icon-font-mit-license.txt'
    } | Sort-Object FullName)
    $notices = @()
    foreach ($file in $files) {
        $relative = $file.FullName.Substring($source.Length + 1).Replace('\','/')
        $notices += @{ Name = $relative; Text = [IO.File]::ReadAllText($file.FullName); Origin = ('crate:' + $relative) }
    }
    $declaredNotice = Get-DeclaredWrapperMitNotice -Package $package -Source $source -ProjectRoot $root
    if ($null -ne $declaredNotice) { $notices += $declaredNotice }
    if ($upstream.ContainsKey($package.name)) {
        $definition = $upstream[$package.name]
        $vcs = Get-Content -LiteralPath (Join-Path $source '.cargo_vcs_info.json') -Raw | ConvertFrom-Json
        foreach ($name in $definition.Files) {
            $url = 'https://raw.githubusercontent.com/' + $definition.Repository + '/' + $vcs.git.sha1 + '/' + $name
            $response = Invoke-WebRequest -Uri $url
            $notices += @{ Name = $name; Text = $response.Content; Origin = $url }
        }
    }
    if ($notices.Count -eq 0) { throw "No license text collected for $($package.name) $($package.version)" }
    [void]$text.AppendLine("`n================================================================================")
    [void]$text.AppendLine($package.name + ' ' + $package.version)
    [void]$text.AppendLine('Declared license: ' + $package.license)
    [void]$text.AppendLine(('Authors: ' + ($package.authors -join '; ')).TrimEnd())
    [void]$text.AppendLine('Repository: ' + $package.repository)
    [void]$text.AppendLine('Exact package source: https://crates.io/api/v1/crates/' + $package.name + '/' + $package.version + '/download')
    foreach ($notice in $notices) {
        [void]$text.AppendLine("`n--- " + $notice.Name + ' ---')
        [void]$text.AppendLine('Origin: ' + $notice.Origin)
        [void]$text.AppendLine($notice.Text)
    }
    $index += [ordered]@{ name=$package.name; version=$package.version; license=$package.license; repository=$package.repository; source=('https://crates.io/api/v1/crates/'+$package.name+'/'+$package.version+'/download'); notice_count=$notices.Count }
}
[IO.File]::WriteAllText((Join-Path $output 'RUST-THIRD-PARTY-LICENSES.txt'), $text.ToString(), [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText((Join-Path $output 'rust-components.json'), ($index | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))

# This unmodified MPL-2.0 dependency is distributed in source form alongside its notice.
$mpl = @($packages.Values | Where-Object name -eq 'option-ext')
foreach ($package in $mpl) {
    $destination = Join-Path $output ('option-ext-' + $package.version)
    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    $source = Split-Path -Parent $package.manifest_path
    foreach ($file in @(Get-ChildItem -LiteralPath $source -Recurse -File | Where-Object Name -ne '.cargo-ok')) {
        $relative = $file.FullName.Substring($source.Length+1)
        $target = Join-Path $destination $relative
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
        Copy-Item -LiteralPath $file.FullName -Destination $target -Force
    }
}
# Rust's statically linked standard library also ships its own copyright inventory.
$cargoCommand = Get-Command $CargoPath -ErrorAction Stop
$rustc = Join-Path (Split-Path -Parent $cargoCommand.Source) 'rustc.exe'
$sysroot = (& $rustc --print sysroot | Select-Object -First 1)
if ($LASTEXITCODE -ne 0) { throw 'Unable to locate Rust runtime notices.' }
$runtimeSource = Join-Path $sysroot 'share\doc\rust'
$runtimeDestination = Join-Path $output 'rust-runtime'
New-Item -ItemType Directory -Force -Path $runtimeDestination | Out-Null
Copy-Item -LiteralPath (Join-Path $runtimeSource 'COPYRIGHT-library.html') -Destination $runtimeDestination -Force
$runtimeLicenses = Join-Path $runtimeDestination 'licenses'
New-Item -ItemType Directory -Force -Path $runtimeLicenses | Out-Null
Copy-Item -Path (Join-Path $runtimeSource 'licenses\*') -Destination $runtimeLicenses -Force
Set-Content -LiteralPath (Join-Path $runtimeDestination 'compiler-version.txt') -Value (& $rustc --version) -Encoding utf8
Write-Host "Collected notices for $($packages.Count) locked Windows crates in $output"
