<#
.SYNOPSIS
Checks installed packaged COM activation and the real classic File Explorer menu.
.EXAMPLE
.\scripts\test-packaged-context-menu.ps1 -ExpectedVersion 1.0.7.0 -SamplePath C:\Samples\image.png,C:\Samples\document.pdf
.NOTES
Run in 64-bit Windows PowerShell after installing the release MSIX on each OS
being tested. Provide existing files supported by Convert Here; their contents
are not modified. Set ExpectedVersion to the new MSIX version for a release gate,
so a previous installation cannot produce a misleading pass.
This check neither registers extensions nor invokes conversion
commands, changes associations, restarts Explorer, or modifies certificate stores.
The classic menu is queried through IShellFolder/IContextMenu, including on
Windows 11 where it corresponds to "Show more options". A pass on Windows 11
does not establish Windows 10 compatibility; run on a real Windows 10 host too.
It checks the root command for single files, not multiselection or lazy submenus.
If Explorer has not refreshed after an MSIX update, sign out and back in before
repeating. The script deliberately does not restart the user's shell.
Use an account without a competing standalone or preview installation. Known
classic registrations are rejected to avoid falsely attributing their menu to
the Store package. COM title and menu construction failures are reported
separately from a missing menu item.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string[]]$SamplePath,
    [string]$PackageName = 'SalvatoreCorsi.ConvertHere',
    [version]$ExpectedVersion
)
$ErrorActionPreference = 'Stop'
if (-not [Environment]::Is64BitProcess) { throw 'Run this check from 64-bit PowerShell.' }
$packages = @(Get-AppxPackage -Name $PackageName)
if ($packages.Count -ne 1) { throw "Expected one installed package named $PackageName." }
$package = $packages[0]
if ($ExpectedVersion -and [version]$package.Version -ne $ExpectedVersion) {
    throw "Expected MSIX version $ExpectedVersion; installed version is $($package.Version)."
}
$manifest = Get-AppxPackageManifest -Package $package.PackageFullName
$classId = '67B51BA1-8A78-4B4B-A5F9-08A825BE91F8'
$competingKeys = @(
    'Software\Classes\*\shell\TheConverter',
    ('Software\Classes\CLSID\{' + $classId + '}\InprocServer32')
)
foreach ($hive in @('HKCU:', 'HKLM:')) {
    foreach ($key in $competingKeys) {
        $registration = $hive + '\' + $key
        if (Test-Path -LiteralPath $registration) {
            throw "A competing classic registration prevents an isolated MSIX test: $registration. Use a Store-only test account; no registration was changed."
        }
    }
}
foreach ($name in @('SalvatoreCorsi.ConvertHere', 'Corsivo.TheConverter.StorePreview', 'SalvatoreCorsi.TheConverter')) {
    if ($name -cne $package.Name -and @(Get-AppxPackage -Name $name).Count -gt 0) {
        throw "A competing Convert Here package prevents isolated COM activation: $name. Use a Store-only test account."
    }
}
$classes = @($manifest.SelectNodes('//*[local-name()="Class" and @Id="' + $classId + '"]'))
if ($classes.Count -ne 1 -or $classes[0].GetAttribute('Path') -cne 'the_converter_shell.dll') {
    throw 'The installed package does not declare the expected Explorer COM class.'
}
$samples = @($SamplePath | ForEach-Object {
    $resolved = (Resolve-Path -LiteralPath $_).Path
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) { throw "Not a sample file: $resolved" }
    $resolved
})
if (-not ('ConvertHere.PackagedMenuProbe' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
namespace ConvertHere {
    public static class PackagedMenuProbe {
        [DllImport("ole32")] static extern int CoInitializeEx(IntPtr reserved, uint flags);
        [DllImport("ole32")] static extern void CoUninitialize();
        [DllImport("ole32")] static extern int CoCreateInstance(ref Guid clsid, IntPtr outer, uint context, ref Guid iid, out IntPtr instance);
        [DllImport("ole32")] static extern void CoTaskMemFree(IntPtr value);
        [DllImport("shell32", CharSet=CharSet.Unicode)]
        static extern int SHParseDisplayName(string path, IntPtr bind, out IntPtr pidl, uint flags, out uint attributes);
        [DllImport("shell32")]
        static extern int SHBindToParent(IntPtr pidl, ref Guid iid, out IntPtr parent, out IntPtr child);
        [DllImport("user32")] static extern IntPtr CreatePopupMenu();
        [DllImport("user32")] static extern bool DestroyMenu(IntPtr menu);
        [DllImport("user32")] static extern int GetMenuItemCount(IntPtr menu);
        [DllImport("user32")] static extern IntPtr GetSubMenu(IntPtr menu, int position);
        [DllImport("user32", CharSet=CharSet.Unicode)]
        static extern int GetMenuString(IntPtr menu, uint item, StringBuilder text, int capacity, uint flags);
        [UnmanagedFunctionPointer(CallingConvention.Winapi)]
        delegate int GetUIObjectOf(IntPtr self, IntPtr hwnd, uint count, IntPtr[] children, ref Guid iid, IntPtr reserved, out IntPtr instance);
        [UnmanagedFunctionPointer(CallingConvention.Winapi)]
        delegate int QueryContextMenu(IntPtr self, IntPtr menu, uint position, uint first, uint last, uint flags);
        [UnmanagedFunctionPointer(CallingConvention.Winapi)]
        delegate int GetTitle(IntPtr self, IntPtr items, out IntPtr title);
        static Delegate Slot(IntPtr instance, int slot, Type type) {
            return Marshal.GetDelegateForFunctionPointer(Marshal.ReadIntPtr(Marshal.ReadIntPtr(instance), slot * IntPtr.Size), type);
        }
        static void Check(int status) { Marshal.ThrowExceptionForHR(status); }
        static void ReadMenu(IntPtr menu, List<string> names, int depth) {
            if (depth > 8) return;
            for (int i = 0; i < GetMenuItemCount(menu); i++) {
                var name = new StringBuilder(1024);
                GetMenuString(menu, (uint)i, name, name.Capacity, 0x400); // MF_BYPOSITION
                if (name.Length != 0) names.Add(name.ToString());
                IntPtr child = GetSubMenu(menu, i);
                if (child != IntPtr.Zero) ReadMenu(child, names, depth + 1);
            }
        }
        public static string Activate() {
            bool initialized = CoInitializeEx(IntPtr.Zero, 2) >= 0; // STA
            IntPtr command = IntPtr.Zero, title = IntPtr.Zero;
            try {
                var clsid = new Guid("67B51BA1-8A78-4B4B-A5F9-08A825BE91F8");
                var iid = new Guid("a08ce4d0-fa25-44ab-b57c-c7b1c323e0b9");
                // LOCAL_SERVER exercises the packaged surrogate, rather than loading the DLL directly.
                Check(CoCreateInstance(ref clsid, IntPtr.Zero, 4, ref iid, out command));
                Check(((GetTitle)Slot(command, 3, typeof(GetTitle)))(command, IntPtr.Zero, out title));
                return Marshal.PtrToStringUni(title);
            } finally {
                if (title != IntPtr.Zero) CoTaskMemFree(title);
                if (command != IntPtr.Zero) Marshal.Release(command);
                if (initialized) CoUninitialize();
            }
        }
        public static string[] Inspect(string path) {
            bool initialized = CoInitializeEx(IntPtr.Zero, 2) >= 0;
            IntPtr pidl = IntPtr.Zero, parent = IntPtr.Zero, context = IntPtr.Zero, menu = IntPtr.Zero;
            try {
                var shellFolder = new Guid("000214e6-0000-0000-c000-000000000046");
                var contextMenu = new Guid("000214e4-0000-0000-c000-000000000046");
                uint attributes;
                IntPtr child;
                Check(SHParseDisplayName(path, IntPtr.Zero, out pidl, 0, out attributes));
                Check(SHBindToParent(pidl, ref shellFolder, out parent, out child));
                Check(((GetUIObjectOf)Slot(parent, 10, typeof(GetUIObjectOf)))(parent, IntPtr.Zero, 1, new IntPtr[] { child }, ref contextMenu, IntPtr.Zero, out context));
                menu = CreatePopupMenu();
                if (menu == IntPtr.Zero) throw new InvalidOperationException("CreatePopupMenu failed.");
                Check(((QueryContextMenu)Slot(context, 3, typeof(QueryContextMenu)))(context, menu, 0, 1, 32767, 0));
                var names = new List<string>();
                ReadMenu(menu, names, 0);
                return names.ToArray();
            } finally {
                if (menu != IntPtr.Zero) DestroyMenu(menu);
                if (context != IntPtr.Zero) Marshal.Release(context);
                if (parent != IntPtr.Zero) Marshal.Release(parent);
                if (pidl != IntPtr.Zero) CoTaskMemFree(pidl);
                if (initialized) CoUninitialize();
            }
        }
    }
}
'@
}
$title = [ConvertHere.PackagedMenuProbe]::Activate()
if ([string]::IsNullOrWhiteSpace($title)) { throw 'Packaged COM command returned no title.' }
$os = Get-CimInstance Win32_OperatingSystem
Write-Output "OS: $($os.Caption), build $($os.BuildNumber); MSIX: $($package.PackageFullName)"
Write-Output "Packaged COM activation: $title"
foreach ($sample in $samples) {
    $names = @([ConvertHere.PackagedMenuProbe]::Inspect($sample))
    $matching = @($names | Where-Object { $_.Replace('&', '').Trim() -ceq $title })
    if ($matching.Count -ne 1) {
        throw "Expected one '$title' entry in the classic Shell menu for $sample; found $($matching.Count). Menu: $($names -join ' | ')"
    }
    Write-Output "Classic Shell menu: $title [$sample]"
}
