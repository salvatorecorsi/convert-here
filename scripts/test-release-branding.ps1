# Uses inert files and a compiler-capture helper; never runs an installer or fixture binary.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$appVersionMatch = [regex]::Match((Get-Content -LiteralPath (Join-Path $root 'Cargo.toml') -Raw), '(?m)^version\s*=\s*"(\d+\.\d+\.\d+)"')
if (-not $appVersionMatch.Success) { throw 'Numeric application version missing from Cargo.toml.' }
$appVersion = $appVersionMatch.Groups[1].Value
$temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
$fixture = Join-Path $temporaryRoot ('convert-here-release-tests-' + [guid]::NewGuid().ToString('N'))
$checks = 0
function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message }; $script:checks++ }
function Write-Fixture([string]$Relative, [string]$Content = 'Inert fixture data') {
    $path = Join-Path $fixture $Relative
    New-Item -ItemType Directory -Path (Split-Path -Parent $path) -Force | Out-Null
    [IO.File]::WriteAllText($path, $Content, [Text.UTF8Encoding]::new($false))
}
try {
    New-Item -ItemType Directory -Path $fixture | Out-Null
    $installer = Get-Content -LiteralPath (Join-Path $root 'installer/TheConverter.iss') -Raw
    Assert ($installer -match '(?m)^AppPublisher=Salvatore Corsi\r?$') 'Installer public publisher is incorrect.'
    $resources = Get-Content -LiteralPath (Join-Path $root 'build.rs') -Raw
    Assert ($resources.Contains('resource.set("CompanyName", "Salvatore Corsi")')) 'PE public company metadata is incorrect.'
    Assert ($installer.Contains('Source: "{#SourceDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion') -and $installer.Contains('Source: "{#SourceDir}\README.en.md"; DestDir: "{app}"; Flags: ignoreversion')) 'Installer omits the application GPL license or English README.'
    Assert ($installer -notmatch 'WizardIsTaskSelected\(''downloadtools''\)|Name: "downloadtools"|\{cm:InstallTools\}|RunScript\(''install-runtime-tools\.ps1''') 'Installer still downloads runtimes or creates a separate runtime-acquisition shortcut.'
    Assert ($installer -notmatch '(?m)^LicenseFile=') 'Executing the application must not require a GPL acceptance checkbox.'
    Assert ($installer -match '(?m)^AppId=\{\{441C8D1C-A78E-4D93-9221-2FA8B207E408\}') 'Installer upgrade identity changed.'
    Assert ($installer -match '(?m)^DefaultDirName=\{localappdata\}\\Programs\\The Converter\r?$') 'Legacy installation directory changed.'
    Assert ($installer -match '(?m)^UsePreviousAppDir=yes\r?$' -and $installer -match '(?m)^UsePreviousGroup=yes\r?$') 'Upgrade must retain remembered directory and Start group.'
    Assert ($installer -match '(?m)^DefaultGroupName=Convert Here\r?$') 'New installation Start group has old branding.'
    foreach ($entry in @('Name: "{group}\Convert Here"; Filename: "{app}\the-converter.exe"', 'Name: "{autodesktop}\Convert Here"; Filename: "{app}\the-converter.exe"')) {
        Assert ($installer.Contains($entry)) 'Public shortcut name or stable target changed.'
    }
    Assert ($installer.Contains('#define RepositoryUrl "https://github.com/salvatorecorsi/convert-here"')) 'Installer points at the old repository.'
    Assert ($installer.Contains('#define AppVersion "' + $appVersion + '"') -and $installer.Contains('#define FileVersion "' + $appVersion + '.0"')) 'Installer fallback version differs from the release.'
    [xml]$sparseApplication = Get-Content -LiteralPath (Join-Path $root 'shell-extension/app.manifest') -Raw -Encoding UTF8
    [xml]$sparsePackage = Get-Content -LiteralPath (Join-Path $root 'shell-extension/package/AppxManifest.xml') -Raw -Encoding UTF8
    Assert ($sparseApplication.assembly.assemblyIdentity.version -ceq "$appVersion.0" -and $sparsePackage.Package.Identity.Version -ceq "$appVersion.0") 'Sparse application and package template versions differ from the release.'
    $storeWorkflow = Get-Content -LiteralPath (Join-Path $root '.github/workflows/microsoft-store.yml') -Raw -Encoding UTF8
    Assert ($storeWorkflow -match '(?ms)publisher_display_name:\s*\n\s*description: [^\r\n]+\r?\n\s*type: string\r?\n\s*default: Salvatore Corsi\r?\n' -and $storeWorkflow.Contains('name: convert-here-store-${{ inputs.for_submission')) 'Store workflow public publisher or artifact branding is incorrect.'
    Assert ($installer.Contains('#define ConverterClass "{67B51BA1-8A78-4B4B-A5F9-08A825BE91F8}"')) 'Explorer CLSID changed.'
    foreach ($script in @('build-release.ps1', 'build-installer.ps1', 'prepare-public-release.ps1', 'install-context-menu.ps1')) {
        $source = Get-Content -LiteralPath (Join-Path $PSScriptRoot $script) -Raw
        Assert ($source.Contains('dist\ConvertHere') -and -not $source.Contains('dist\TheConverter')) "Distribution defaults differ in $script."
    }
    foreach ($relative in @('scripts/build-installer.ps1', 'scripts/prepare-public-release.ps1', 'scripts/modern-menu-package.psm1', 'release-support/verify-thin-release.ps1', 'installer/TheConverter.iss')) {
        $target = Join-Path $fixture $relative
        New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $root $relative) -Destination $target
    }
    foreach ($script in @('install-context-menu.ps1', 'uninstall-context-menu.ps1', 'install-runtime-tools.ps1')) { Write-Fixture "scripts/$script" '# Inert packaged script fixture' }
    Write-Fixture 'Cargo.toml' "[package]`nname = `"fixture`"`nversion = `"$appVersion`"`n"
    foreach ($name in @('the-converter.exe', 'the-converter-cli.exe', 'the_converter_shell.dll')) { Write-Fixture "dist/ConvertHere/$name" }
    Write-Fixture 'dist/ConvertHere/tools/ffmpeg.exe' 'Must not be packaged'
    foreach ($name in @('README.md', 'README.en.md', 'LICENSE', 'THIRD-PARTY-NOTICES.md')) { Write-Fixture $name }
    foreach ($name in @('README.md', 'RUST-THIRD-PARTY-LICENSES.txt', 'option-ext-0.2.0/LICENSE.txt', 'option-ext-0.2.0/src/lib.rs', 'option-ext-0.2.0/src/impl.rs', 'rust-runtime/COPYRIGHT-library.html')) { Write-Fixture "docs/release-licenses/$name" }
    Write-Fixture 'docs/release-licenses/rust-components.json' '[{"name":"fixture","version":"1","notice_count":1}]'
    Write-Fixture 'docs/configuration.en.md' '# English guide fixture'
    Write-Fixture 'docs/screenshots/fixture.png' 'Inert screenshot fixture'
    # build-installer also validates these inputs before invoking the compiler.
    foreach ($name in @('README.md', 'README.en.md', 'LICENSE', 'THIRD-PARTY-NOTICES.md')) { Write-Fixture "dist/ConvertHere/$name" }
    # The actual Git/vendor exporter is tested by test-corresponding-source.ps1.
    # This inert fixture tests the public packaging/checksum handoff only.
    Write-Fixture 'release-support/build-corresponding-source.ps1' @'
param($Version, $OutputDirectory, $CargoPath, $RustcPath, $DistributionDirectory, $StoreIdentityConfig)
$archive = Join-Path $OutputDirectory "ConvertHere-$Version-source.zip"
[IO.File]::WriteAllText($archive, 'Inert source-archive fixture; never execute')
[pscustomobject]@{ SourceArchive = $archive }
'@
    Write-Fixture 'compiler-capture.ps1' @'
$ErrorActionPreference = 'Stop'
$arguments = @($args)
$arguments | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'compiler-arguments.json')
$version = ($arguments | Where-Object { $_ -like '/DAppVersion=*' }) -replace '^/DAppVersion=', ''
$output = ($arguments | Where-Object { $_ -like '/DOutputPath=*' }) -replace '^/DOutputPath=', ''
$source = Get-Content -LiteralPath $arguments[-1] -Raw
$name = [regex]::Match($source, '(?m)^OutputBaseFilename=([^\r\n]+)').Groups[1].Value.Replace('{#AppVersion}', $version)
if (-not $name) { throw 'Missing output filename in actual installer source.' }
[IO.File]::WriteAllText((Join-Path $output ($name + '.exe')), 'Inert setup fixture; never execute')
$global:LASTEXITCODE = 0
'@
    & (Join-Path $fixture 'scripts/prepare-public-release.ps1') -CompilerPath (Join-Path $fixture 'compiler-capture.ps1') | Out-Null
    $output = Join-Path $fixture "dist/releases/v$appVersion"
    $setup = Join-Path $output "ConvertHere-$appVersion-windows-x64-setup.exe"
    $portable = Join-Path $output "ConvertHere-$appVersion-windows-x64-portable.zip"
    $sourceArchive = Join-Path $output "ConvertHere-$appVersion-source.zip"
    Assert (Test-Path -LiteralPath $setup -PathType Leaf) 'Public setup filename does not match the compiler output.'
    Assert (Test-Path -LiteralPath $portable -PathType Leaf) 'Public portable filename has old branding.'
    Assert (Test-Path -LiteralPath $sourceArchive -PathType Leaf) 'Corresponding source is missing from the public assets.'
    $arguments = Get-Content -LiteralPath (Join-Path $fixture 'compiler-arguments.json') -Raw | ConvertFrom-Json
    Assert ($arguments -contains "/DAppVersion=$appVersion" -and $arguments -contains "/DFileVersion=${appVersion}.0") 'Release version did not reach the compiler.'
    Assert ($arguments -contains '/DModernMenuAvailable=0') 'Unsigned classic fixture unexpectedly enables modern registration.'
    Assert ($arguments -contains ('/DSourceDir=' + (Join-Path $fixture "dist/public-stage/$appVersion/classic/ConvertHere"))) 'Installer did not consume the branded public stage.'
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($portable)
    try {
        $entries = @($archive.Entries | ForEach-Object { $_.FullName.Replace('\', '/') })
        Assert (@($entries | Where-Object { -not $_.StartsWith('ConvertHere/') }).Count -eq 0) 'Portable root folder has old branding.'
        foreach ($name in @('the-converter.exe', 'the-converter-cli.exe', 'the_converter_shell.dll')) { Assert ($entries -contains "ConvertHere/$name") 'Portable internal binary contract changed.' }
        Assert ($entries -contains 'ConvertHere/LICENSE') 'Portable archive omits the application license.'
        Assert ($entries -contains 'ConvertHere/README.en.md' -and $entries -contains 'ConvertHere/docs/configuration.en.md' -and $entries -contains 'ConvertHere/docs/screenshots/fixture.png') 'Portable archive omits the English README or linked documentation assets.'
        Assert (@($entries | Where-Object { $_ -match '(^|/)tools/|TheConverter\.Identity\.msix$' }).Count -eq 0) 'Classic public fixture contains runtime binaries or an unsigned identity package.'
    } finally { $archive.Dispose() }
    $checksums = @(Get-Content -LiteralPath (Join-Path $output 'SHA256SUMS.txt'))
    Assert ($checksums.Count -eq 3) 'Checksum inventory must include setup, portable and corresponding source.'
    foreach ($asset in @($setup, $portable, $sourceArchive)) {
        $expected = (Get-FileHash -LiteralPath $asset -Algorithm SHA256).Hash.ToLowerInvariant() + '  ' + [IO.Path]::GetFileName($asset)
        Assert ($checksums -ccontains $expected) 'Checksum does not match its renamed artifact.'
    }
    Write-Output "PASS: $checks branding, upgrade-contract and public packaging checks; inert compiler only, no installer, registry, Appx or certificate changes."
} finally {
    $checked = [IO.Path]::GetFullPath($fixture)
    if (-not $checked.StartsWith($temporaryRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($checked) -notlike 'convert-here-release-tests-*') { throw 'Unexpected fixture cleanup directory.' }
    if (Test-Path -LiteralPath $checked) { Remove-Item -LiteralPath $checked -Recurse -Force }
}
