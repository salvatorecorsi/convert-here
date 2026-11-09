# Exercises the real Windows COM ABI without registering a class or a menu.
[CmdletBinding()]
param(
    [string]$LibraryPath,
    # Only point this at the output of an isolated argument-capture test helper.
    [string]$InvokeCaptureFile,
    [switch]$ValidateRegistrationOnly
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$registration = Join-Path $PSScriptRoot 'install-context-menu.ps1'
$installer = Join-Path $repo 'installer\TheConverter.iss'
$shellSource = Join-Path $repo 'shell-extension\src\lib.rs'
$modelSource = Join-Path $repo 'src\model.rs'
if ((Test-Path -LiteralPath $installer) -and (Test-Path -LiteralPath $shellSource)) {
    $installSource = Get-Content -LiteralPath $registration -Raw
    $list = [regex]::Match($installSource, '(?s)\$classicExtensions\s*=\s*@\((.*?)\)')
    if (-not $list.Success) { throw 'Classic registration extension whitelist missing.' }
    $extensions = @([regex]::Matches($list.Groups[1].Value, "'([a-z0-9]+)'") | ForEach-Object { $_.Groups[1].Value })
    $expectedQuery = ($extensions | ForEach-Object { 'System.FileExtension:=".' + $_ + '"' }) -join ' OR '
    $line = @(Get-Content -LiteralPath $installer | Where-Object { $_ -match 'ValueName: "AppliesTo";' })
    if ($line.Count -ne 1) { throw 'Installer must declare one classic AppliesTo filter.' }
    $actualQuery = [regex]::Match($line[0], 'ValueData: "(.*)"; Tasks: contextmenu').Groups[1].Value.Replace('""', '"')
    if ($actualQuery -cne $expectedQuery -or $installSource -notmatch "-Name 'AppliesTo' -Value \`$appliesTo") { throw 'Classic script and installer AppliesTo filters differ.' }
    $rust = Get-Content -LiteralPath $shellSource -Raw
    $maskSource = $rust.Substring($rust.IndexOf('fn target_mask('), $rust.IndexOf('fn common_target_mask') - $rust.IndexOf('fn target_mask('))
    $shellExtensions = @([regex]::Matches($maskSource, '"([a-z0-9]+)"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
    if ((Compare-Object @($extensions | Sort-Object -Unique) $shellExtensions)) { throw 'Classic whitelist and shell input extensions differ.' }
    if (-not (Test-Path -LiteralPath $modelSource -PathType Leaf)) { throw 'Application model source missing.' }
    $model = Get-Content -LiteralPath $modelSource -Raw
    $classification = [regex]::Match($model, '(?ms)^impl\s+MediaKind\s*\{(?<classification>.*?)^\}')
    if (-not $classification.Success) { throw 'MediaKind::from_path extension classification missing.' }
    $modelExtensions = @([regex]::Matches($classification.Groups['classification'].Value, '"([a-z0-9]+)"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
    if ((Compare-Object $modelExtensions $shellExtensions)) { throw 'Application model and shell input extensions differ.' }
    if ($extensions -notcontains 'txt' -or $extensions -notcontains 'mpga' -or $extensions -contains 'exe' -or $extensions -contains 'unknown') { throw 'Unexpected classic whitelist capabilities.' }
    Write-Output "Classic AppliesTo verified: $($extensions.Count) input extensions match app, DLL and installer; TXT documents included, unknown inputs excluded."
} elseif ($ValidateRegistrationOnly) { throw 'Run registration-only validation from the source checkout.' }
if ($ValidateRegistrationOnly) { return }
if (-not $LibraryPath) {
    $adjacentLibrary = Join-Path $PSScriptRoot '..\the_converter_shell.dll'
    if (Test-Path -LiteralPath $adjacentLibrary -PathType Leaf) { $LibraryPath = $adjacentLibrary }
    else { $LibraryPath = Join-Path $PSScriptRoot '..\shell-extension\target\release\the_converter_shell.dll' }
}
$library = (Resolve-Path -LiteralPath $LibraryPath).Path
if (-not ('ConverterShellSmokeV2' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public static class ConverterShellSmokeV2 {
    [DllImport("kernel32", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern IntPtr LoadLibrary(string path);
    [DllImport("kernel32", CharSet=CharSet.Ansi)]
    static extern IntPtr GetProcAddress(IntPtr library, string name);
    [DllImport("kernel32")]
    static extern bool FreeLibrary(IntPtr library);
    [DllImport("ole32")]
    static extern int CoInitializeEx(IntPtr reserved, uint flags);
    [DllImport("ole32")]
    static extern void CoUninitialize();
    [DllImport("shell32", CharSet=CharSet.Unicode, PreserveSig=true)]
    static extern int SHCreateItemFromParsingName(string path, IntPtr bind, ref Guid iid, out IntPtr item);
    [DllImport("shell32", PreserveSig=true)]
    static extern int SHCreateShellItemArrayFromShellItem(IntPtr item, ref Guid iid, out IntPtr array);
    [DllImport("shell32", CharSet=CharSet.Unicode, PreserveSig=true)]
    static extern int SHParseDisplayName(string path, IntPtr bind, out IntPtr pidl, uint attributes, out uint actual);
    [DllImport("shell32", PreserveSig=true)]
    static extern int SHCreateShellItemArrayFromIDLists(uint count, [In] IntPtr[] pidls, out IntPtr array);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int GetClassObject(ref Guid clsid, ref Guid iid, out IntPtr instance);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int CanUnload();
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int CreateInstance(IntPtr self, IntPtr outer, ref Guid iid, out IntPtr instance);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int GetState(IntPtr self, IntPtr items, int slow, out uint state);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int GetTitle(IntPtr self, IntPtr items, out IntPtr title);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int GetCanonical(IntPtr self, out Guid canonical);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int EnumCommands(IntPtr self, out IntPtr enumerator);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int NextCommand(IntPtr self, uint requested, out IntPtr command, out uint fetched);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int GetFlags(IntPtr self, out uint flags);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    delegate int InvokeCommand(IntPtr self, IntPtr items, IntPtr bind);

    static Delegate Slot(IntPtr instance, int slot, Type delegateType) {
        return Marshal.GetDelegateForFunctionPointer(Marshal.ReadIntPtr(Marshal.ReadIntPtr(instance), slot * IntPtr.Size), delegateType);
    }
    static void Check(int status) { Marshal.ThrowExceptionForHR(status); }
    static uint State(IntPtr command, string file) {
        Guid shellItem = new Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe");
        Guid shellArray = new Guid("b63ea76d-1f85-456f-a19c-48159efa858b");
        IntPtr item = IntPtr.Zero, array = IntPtr.Zero;
        try {
            Check(SHCreateItemFromParsingName(file, IntPtr.Zero, ref shellItem, out item));
            Check(SHCreateShellItemArrayFromShellItem(item, ref shellArray, out array));
            uint state;
            Check(((GetState)Slot(command, 7, typeof(GetState)))(command, array, 0, out state));
            return state;
        } finally {
            if (array != IntPtr.Zero) Marshal.Release(array);
            if (item != IntPtr.Zero) Marshal.Release(item);
        }
    }
    static string Title(IntPtr command) {
        IntPtr text;
        Check(((GetTitle)Slot(command, 3, typeof(GetTitle)))(command, IntPtr.Zero, out text));
        try { return Marshal.PtrToStringUni(text); } finally { Marshal.FreeCoTaskMem(text); }
    }
    static string[] Visible(IntPtr parent, params string[] files) {
        IntPtr[] pidls = new IntPtr[files.Length];
        IntPtr array = IntPtr.Zero, enumerator = IntPtr.Zero;
        var labels = new List<string>();
        try {
            for (int index = 0; index < files.Length; index++) {
                uint attributes;
                Check(SHParseDisplayName(files[index], IntPtr.Zero, out pidls[index], 0, out attributes));
            }
            Check(SHCreateShellItemArrayFromIDLists((uint)pidls.Length, pidls, out array));
            uint parentState;
            Check(((GetState)Slot(parent, 7, typeof(GetState)))(parent, array, 1, out parentState));
            if (parentState != 0) return labels.ToArray();
            Check(((EnumCommands)Slot(parent, 10, typeof(EnumCommands)))(parent, out enumerator));
            var next = (NextCommand)Slot(enumerator, 3, typeof(NextCommand));
            for (;;) {
                IntPtr child; uint fetched;
                int status = next(enumerator, 1, out child, out fetched);
                Check(status);
                if (fetched == 0) break;
                try {
                    uint state;
                    Check(((GetState)Slot(child, 7, typeof(GetState)))(child, array, 1, out state));
                    if (state == 0) labels.Add(Title(child));
                } finally { Marshal.Release(child); }
            }
            return labels.ToArray();
        } finally {
            if (enumerator != IntPtr.Zero) Marshal.Release(enumerator);
            if (array != IntPtr.Zero) Marshal.Release(array);
            foreach (IntPtr pidl in pidls) if (pidl != IntPtr.Zero) Marshal.FreeCoTaskMem(pidl);
        }
    }
    static void Expect(string label, string[] actual, string[] expected) {
        if (String.Join("|", actual) != String.Join("|", expected)) throw new Exception(label + ": " + String.Join(", ", actual));
    }
    static string ProfilePreferences(ulong revision) {
        return "{\"enabled_output_formats\":[\"png\",\"webp\",\"pdf\"],\"conversion_profiles\":[{\"id\":\"0123456789abcdef0123456789abcdef\",\"revision\":" + revision + ",\"name\":\"Salva per web & città\",\"show_in_menu\":true,\"output_extension\":\"webp\",\"input_policy\":\"images\",\"settings\":{\"format\":\"Webp\",\"image\":{\"edit\":{}}}},{\"id\":\"1123456789abcdef0123456789abcdef\",\"revision\":71,\"name\":\"Editor PNG\",\"show_in_menu\":true,\"output_extension\":\"png\",\"input_policy\":\"images\",\"settings\":{\"format\":\"Png\",\"image\":{\"edit\":{}}}},{\"id\":\"2123456789abcdef0123456789abcdef\",\"revision\":71,\"name\":\"Pagina A4\",\"show_in_menu\":true,\"output_extension\":\"pdf\",\"input_policy\":\"images_or_text\",\"settings\":{\"format\":\"Pdf\",\"image\":{\"page\":{}},\"pdf\":{\"action\":\"Compress\"}}}]}";
    }
    static void ValidateProfiles(IntPtr parent, string image, string pdf, string audio, string text, string document, string preferences) {
        string previous = System.IO.File.ReadAllText(preferences);
        const string custom = "Impostazioni personalizzate…";
        try {
            System.IO.File.WriteAllText(preferences, ProfilePreferences(71));
            Expect("Profili immagini", Visible(parent, image), new string[]{"PNG", "WebP", "PDF", "Salva per web & città · WEBP", "Editor PNG · PNG", "Pagina A4 · PDF", custom});
            Expect("Editor nascosto per PDF", Visible(parent, pdf), new string[]{"PNG", "PDF · comprimi", custom});
            Expect("Pagina TXT nativa", Visible(parent, text), new string[]{"PDF", "Pagina A4 · PDF", custom});
            Expect("Pagina esclusa Word", Visible(parent, document), new string[]{"PDF", custom});
            Expect("Profili esclusi batch immagini e PDF", Visible(parent, image, pdf), new string[]{"PNG", "PDF (Bulk)", "PDF (Merge)", custom});
            WithSelection(new string[]{image}, delegate(IntPtr selection) {
                uint state; Check(((GetState)Slot(parent, 7, typeof(GetState)))(parent, selection, 1, out state));
                IntPtr enumerator = IntPtr.Zero, profile = IntPtr.Zero;
                try {
                    Check(((EnumCommands)Slot(parent, 10, typeof(EnumCommands)))(parent, out enumerator));
                    var next = (NextCommand)Slot(enumerator, 3, typeof(NextCommand));
                    for (;;) {
                        IntPtr child; uint fetched; Check(next(enumerator, 1, out child, out fetched));
                        if (fetched == 0) break;
                        if (Title(child) == "Salva per web & città · WEBP") { profile = child; break; }
                        Marshal.Release(child);
                    }
                    if (profile == IntPtr.Zero) throw new Exception("Profilo dinamico mancante");
                    Guid canonical; Check(((GetCanonical)Slot(profile, 6, typeof(GetCanonical)))(profile, out canonical));
                    if (canonical != new Guid("d61a4567-89ab-cdef-0123-456789abcdef")) throw new Exception("GUID profilo non stabile");
                    foreach (string replacement in new string[]{ProfilePreferences(72), "{\"enabled_output_formats\":[\"webp\"]}", ProfilePreferences(71).Replace("[\"png\",\"webp\",\"pdf\"]", "[]")}) {
                        System.IO.File.WriteAllText(preferences, replacement);
                        Check(((GetState)Slot(profile, 7, typeof(GetState)))(profile, selection, 1, out state));
                        if (state != 2 || ((InvokeCommand)Slot(profile, 8, typeof(InvokeCommand)))(profile, selection, IntPtr.Zero) >= 0) throw new Exception("Profilo obsoleto/eliminato/disabilitato eseguibile");
                    }
                    System.IO.File.WriteAllText(preferences, ProfilePreferences(71));
                    WithSelection(new string[]{audio}, delegate(IntPtr incompatible) {
                        if (((InvokeCommand)Slot(profile, 8, typeof(InvokeCommand)))(profile, incompatible, IntPtr.Zero) >= 0) throw new Exception("Profilo ignora selezione modificata");
                    });
                } finally { if (profile != IntPtr.Zero) Marshal.Release(profile); if (enumerator != IntPtr.Zero) Marshal.Release(enumerator); }
            });
        } finally { System.IO.File.WriteAllText(preferences, previous); }
    }
    static void WithSelection(string[] files, Action<IntPtr> action) {
        IntPtr[] pidls = new IntPtr[files.Length];
        IntPtr array = IntPtr.Zero;
        try {
            for (int index = 0; index < files.Length; index++) {
                uint attributes;
                Check(SHParseDisplayName(files[index], IntPtr.Zero, out pidls[index], 0, out attributes));
            }
            Check(SHCreateShellItemArrayFromIDLists((uint)pidls.Length, pidls, out array));
            action(array);
        } finally {
            if (array != IntPtr.Zero) Marshal.Release(array);
            foreach (IntPtr pidl in pidls) if (pidl != IntPtr.Zero) Marshal.FreeCoTaskMem(pidl);
        }
    }
    static Guid ExpectedCanonical(byte xor) {
        byte[] bytes = new Guid("67b51ba1-8a78-4b4b-a5f9-08a825be91f8").ToByteArray();
        bytes[15] ^= xor;
        return new Guid(bytes);
    }
    static void ValidatePdfActions(IntPtr parent, string image, string pdf, string audio, string preferences) {
        WithSelection(new string[]{image, pdf}, delegate(IntPtr valid) {
            uint state;
            Check(((GetState)Slot(parent, 7, typeof(GetState)))(parent, valid, 1, out state));
            IntPtr enumerator = IntPtr.Zero, bulk = IntPtr.Zero, merge = IntPtr.Zero;
            try {
                Check(((EnumCommands)Slot(parent, 10, typeof(EnumCommands)))(parent, out enumerator));
                var next = (NextCommand)Slot(enumerator, 3, typeof(NextCommand));
                for (;;) {
                    IntPtr child; uint fetched;
                    Check(next(enumerator, 1, out child, out fetched));
                    if (fetched == 0) break;
                    if (Title(child) == "PDF (Bulk)") bulk = child;
                    else if (Title(child) == "PDF (Merge)") merge = child;
                    else Marshal.Release(child);
                }
                if (bulk == IntPtr.Zero || merge == IntPtr.Zero) throw new Exception("Azioni PDF multiple mancanti");
                Guid bulkId, mergeId;
                Check(((GetCanonical)Slot(bulk, 6, typeof(GetCanonical)))(bulk, out bulkId));
                Check(((GetCanonical)Slot(merge, 6, typeof(GetCanonical)))(merge, out mergeId));
                if (bulkId != ExpectedCanonical(20) || mergeId != ExpectedCanonical(46)) throw new Exception("GUID PDF non preservati");
                WithSelection(new string[]{image}, delegate(IntPtr single) {
                    Check(((GetState)Slot(merge, 7, typeof(GetState)))(merge, single, 1, out state));
                    if (state != 2) throw new Exception("Merge visibile per un solo file");
                    if (((InvokeCommand)Slot(merge, 8, typeof(InvokeCommand)))(merge, single, IntPtr.Zero) >= 0) throw new Exception("Merge accetta un solo file");
                });
                WithSelection(new string[]{image, audio}, delegate(IntPtr incompatible) {
                    Check(((GetState)Slot(merge, 7, typeof(GetState)))(merge, incompatible, 1, out state));
                    if (state != 2) throw new Exception("Merge accetta audio incompatibile con PDF");
                    Check(((GetState)Slot(bulk, 7, typeof(GetState)))(bulk, incompatible, 1, out state));
                    if (state != 2) throw new Exception("Bulk PDF accetta audio incompatibile");
                    // Invoke must inspect the new array, even after a valid GetState.
                    foreach (IntPtr child in new IntPtr[]{bulk, merge}) {
                        Check(((GetState)Slot(child, 7, typeof(GetState)))(child, valid, 1, out state));
                        if (state != 0) throw new Exception("Selezione PDF valida disabilitata");
                        if (((InvokeCommand)Slot(child, 8, typeof(InvokeCommand)))(child, incompatible, IntPtr.Zero) >= 0) throw new Exception("Invoke usa una selezione PDF obsoleta");
                    }
                });
                string previous = System.IO.File.ReadAllText(preferences);
                try {
                    System.IO.File.WriteAllText(preferences, "{\"enabled_output_formats\":[]}");
                    foreach (IntPtr child in new IntPtr[]{bulk, merge}) {
                        Check(((GetState)Slot(child, 7, typeof(GetState)))(child, valid, 1, out state));
                        if (state != 2) throw new Exception("Azione PDF disabilitata ancora visibile");
                        if (((InvokeCommand)Slot(child, 8, typeof(InvokeCommand)))(child, valid, IntPtr.Zero) >= 0) throw new Exception("Invoke PDF ignora le preferenze aggiornate");
                    }
                } finally { System.IO.File.WriteAllText(preferences, previous); }
            } finally {
                if (bulk != IntPtr.Zero) Marshal.Release(bulk);
                if (merge != IntPtr.Zero) Marshal.Release(merge);
                if (enumerator != IntPtr.Zero) Marshal.Release(enumerator);
            }
        });
    }
    public static void CaptureInvoke(string library, string capture, string[] files, string requested) {
        int initialized = CoInitializeEx(IntPtr.Zero, 2);
        IntPtr module = LoadLibrary(library), factory = IntPtr.Zero, parent = IntPtr.Zero, enumerator = IntPtr.Zero, child = IntPtr.Zero, array = IntPtr.Zero;
        IntPtr[] pidls = new IntPtr[files.Length];
        if (module == IntPtr.Zero) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
        try {
            var activate = (GetClassObject)Marshal.GetDelegateForFunctionPointer(GetProcAddress(module, "DllGetClassObject"), typeof(GetClassObject));
            Guid clsid = new Guid("67b51ba1-8a78-4b4b-a5f9-08a825be91f8"), factoryId = new Guid("00000001-0000-0000-C000-000000000046");
            Guid commandId = new Guid("a08ce4d0-fa25-44ab-b57c-c7b1c323e0b9");
            Check(activate(ref clsid, ref factoryId, out factory));
            Check(((CreateInstance)Slot(factory, 3, typeof(CreateInstance)))(factory, IntPtr.Zero, ref commandId, out parent));
            for (int index = 0; index < files.Length; index++) {
                uint attributes;
                Check(SHParseDisplayName(files[index], IntPtr.Zero, out pidls[index], 0, out attributes));
            }
            Check(SHCreateShellItemArrayFromIDLists((uint)pidls.Length, pidls, out array));
            uint parentState;
            Check(((GetState)Slot(parent, 7, typeof(GetState)))(parent, array, 1, out parentState));
            if (parentState != 0) throw new Exception("Selezione Invoke non valida");
            Check(((EnumCommands)Slot(parent, 10, typeof(EnumCommands)))(parent, out enumerator));
            var next = (NextCommand)Slot(enumerator, 3, typeof(NextCommand));
            for (int index = 0; ; index++) {
                uint fetched;
                if (child != IntPtr.Zero) { Marshal.Release(child); child = IntPtr.Zero; }
                Check(next(enumerator, 1, out child, out fetched));
                if (fetched != 1) throw new Exception("Sottocomando mancante");
                string title = Title(child);
                if ((requested == "mp3" && title == "MP3") ||
                    (requested == "custom" && title == "Impostazioni personalizzate…") ||
                    (requested == "profile" && title == "Salva per web & città · WEBP") ||
                    (requested == "pdf-bulk" && title == "PDF (Bulk)") ||
                    (requested == "pdf-merge" && title == "PDF (Merge)")) break;
                if (index >= 127) throw new Exception("Troppi sottocomandi");
            }
            Check(((InvokeCommand)Slot(child, 8, typeof(InvokeCommand)))(child, array, IntPtr.Zero));
            var elapsed = System.Diagnostics.Stopwatch.StartNew();
            while (!System.IO.File.Exists(capture) && elapsed.ElapsedMilliseconds < 5000) System.Threading.Thread.Sleep(50);
            if (!System.IO.File.Exists(capture)) throw new Exception("Helper Invoke non avviato");
        } finally {
            if (array != IntPtr.Zero) Marshal.Release(array);
            if (child != IntPtr.Zero) Marshal.Release(child);
            if (enumerator != IntPtr.Zero) Marshal.Release(enumerator);
            if (parent != IntPtr.Zero) Marshal.Release(parent);
            if (factory != IntPtr.Zero) Marshal.Release(factory);
            foreach (IntPtr pidl in pidls) if (pidl != IntPtr.Zero) Marshal.FreeCoTaskMem(pidl);
            FreeLibrary(module);
            if (initialized >= 0) CoUninitialize();
        }
    }
    public static string Run(string library, string supported, string unsupported, string audio, string video, string pdf, string document, string text, string mpga, string directory, string spreadsheet, string csv, string html, string qoi, string avif, string heif, string preferences) {
        int initialized = CoInitializeEx(IntPtr.Zero, 2);
        IntPtr module = LoadLibrary(library), factory = IntPtr.Zero, command = IntPtr.Zero;
        if (module == IntPtr.Zero) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
        try {
            var activate = (GetClassObject)Marshal.GetDelegateForFunctionPointer(GetProcAddress(module, "DllGetClassObject"), typeof(GetClassObject));
            var unload = (CanUnload)Marshal.GetDelegateForFunctionPointer(GetProcAddress(module, "DllCanUnloadNow"), typeof(CanUnload));
            if (unload() != 0) throw new Exception("DLL inizialmente occupata");
            Guid clsid = new Guid("67b51ba1-8a78-4b4b-a5f9-08a825be91f8"), factoryId = new Guid("00000001-0000-0000-C000-000000000046");
            Guid commandId = new Guid("a08ce4d0-fa25-44ab-b57c-c7b1c323e0b9");
            Check(activate(ref clsid, ref factoryId, out factory));
            Check(((CreateInstance)Slot(factory, 3, typeof(CreateInstance)))(factory, IntPtr.Zero, ref commandId, out command));
            string label = Title(command);
            if (label != "Converti in") throw new Exception("Titolo del menu padre errato: " + label);
            uint flags;
            Check(((GetFlags)Slot(command, 9, typeof(GetFlags)))(command, out flags));
            if (flags != 1) throw new Exception("ECF_HASSUBCOMMANDS mancante");
            if (State(command, supported) != 0) throw new Exception("PNG non abilitato");
            if (State(command, unsupported) != 2) throw new Exception("Estensione sconosciuta non nascosta");
            const string custom = "Impostazioni personalizzate…";
            Expect("Default FLAC", Visible(command, audio), new string[]{"MP3", "WAV", custom});
            Expect("Default PNG + PDF", Visible(command, supported, pdf), new string[]{"PNG", "JPEG", "PDF (Bulk)", "PDF (Merge)", custom});
            System.IO.File.WriteAllText(preferences, "{\"enabled_output_formats\":[\"flac\",\"txt\",\"avif\",\"unknown\"]}");
            Expect("FLAC attivato senza riavvio", Visible(command, audio), new string[]{"FLAC", custom});
            Expect("OCR attivato senza PDF", Visible(command, supported, pdf), new string[]{"TXT · OCR / testo PDF", custom});
            Expect("AVIF attivato", Visible(command, heif), new string[]{"TXT · OCR", "AVIF", custom});
            System.IO.File.WriteAllText(preferences, "{\"enabled_output_formats\":[]}");
            if (State(command, supported) != 0) throw new Exception("Input valido nascosto con tutte le destinazioni disabilitate");
            Expect("Destinazioni vuote FLAC", Visible(command, audio), new string[]{custom});
            Expect("Destinazioni vuote PDF batch", Visible(command, supported, pdf), new string[]{custom});
            Expect("Destinazioni vuote sconosciuto", Visible(command, unsupported), new string[]{});
            foreach (string invalid in new string[]{"{broken", "{\"enabled_output_formats\":{}}", "{\"enabled_output_formats\":[\"unknown\"]}", "{\"settings\":{\"format\":\"Flac\"}}"}) {
                System.IO.File.WriteAllText(preferences, invalid);
                Expect("Preferenze non valide", Visible(command, audio), new string[]{"MP3", "WAV", custom});
            }
            System.IO.File.WriteAllText(preferences, "{\"enabled_output_formats\":[\"mp3\",\"wav\",\"flac\",\"aac\",\"ogg\",\"opus\",\"aiff\",\"mp4\",\"mkv\",\"mov\",\"webm\",\"avi\",\"gif\",\"png\",\"jpg\",\"webp\",\"bmp\",\"tiff\",\"ico\",\"pdf\",\"txt\",\"m4a\",\"wma\",\"ac3\",\"au\",\"caf\",\"mpeg\",\"wmv\",\"flv\",\"ts\",\"m4v\",\"3gp\",\"qoi\",\"tga\",\"ff\",\"exr\",\"hdr\",\"pnm\",\"csv\",\"xlsx\",\"docx\",\"html\",\"avif\",\"heic\"]}");
            Expect("Audio + video", Visible(command, audio, video), new string[]{"MP3", "WAV", "FLAC", "AAC", "OGG", "Opus", "AIFF", "M4A", "WMA", "AC3", "AU", "CAF", custom});
            Expect("PNG + PDF", Visible(command, supported, pdf), new string[]{"PNG", "JPEG", "PDF (Bulk)", "PDF (Merge)", "TXT · OCR / testo PDF", custom});
            Expect("DOCX + PNG", Visible(command, document, supported), new string[]{"PDF (Bulk)", "PDF (Merge)", custom});
            Expect("DOCX + PNG + PDF", Visible(command, document, supported, pdf), new string[]{"PDF (Bulk)", "PDF (Merge)", custom});
            Expect("Due PDF", Visible(command, pdf, pdf), new string[]{"PNG", "JPEG", "PDF (Bulk)", "PDF (Merge)", "TXT", "Word (DOCX) · testo", "HTML · testo", custom});
            Expect("PNG + audio", Visible(command, supported, audio), new string[]{});
            Expect("Testo", Visible(command, text), new string[]{"PDF", custom});
            Expect("Testo + audio", Visible(command, text, audio), new string[]{});
            Expect("Cartella WAV", Visible(command, directory), new string[]{});
            var audioFormats = new List<string>(new string[]{"MP3", "WAV", "FLAC", "AAC", "OGG", "Opus", "AIFF"});
            audioFormats.AddRange(new string[]{"M4A", "WMA", "AC3", "AU", "CAF"});
            audioFormats.Add(custom);
            Expect("Audio e mpga", Visible(command, audio, mpga), audioFormats.ToArray());
            Expect("PDF", Visible(command, pdf), new string[]{"PNG", "JPEG", "PDF · comprimi", "TXT", "Word (DOCX) · testo", "HTML · testo", custom});
            Expect("Audio + PDF", Visible(command, audio, pdf), new string[]{});
            Expect("Excel e CSV", Visible(command, spreadsheet, csv), new string[]{"PDF (Bulk)", "PDF (Merge)", "CSV", "XLSX", custom});
            Expect("HTML", Visible(command, html), new string[]{"PDF", custom});
            Expect("HTML e Excel", Visible(command, html, spreadsheet), new string[]{"PDF (Bulk)", "PDF (Merge)", custom});
            Expect("QOI", Visible(command, qoi), Visible(command, supported));
            Expect("AVIF", Visible(command, avif), Visible(command, supported));
            Expect("HEIF", Visible(command, heif), Visible(command, supported));
            Expect("Sconosciuto", Visible(command, unsupported), new string[]{});
            ValidatePdfActions(command, supported, pdf, audio, preferences);
            ValidateProfiles(command, supported, pdf, audio, text, document, preferences);
            Marshal.Release(command); command = IntPtr.Zero;
            Marshal.Release(factory); factory = IntPtr.Zero;
            if (unload() != 0) throw new Exception("Oggetti COM non rilasciati");
            return "COM smoke test OK: " + label + "; preferenze default/personalizzate/vuote/non valide e aggiornamento live, filtri batch, PDF disabilitato blocca Invoke, selezioni reali IShellItemArray e unload OK.";
        } finally {
            if (command != IntPtr.Zero) Marshal.Release(command);
            if (factory != IntPtr.Zero) Marshal.Release(factory);
            FreeLibrary(module);
            if (initialized >= 0) CoUninitialize();
        }
    }
}
'@
}
$token = [Guid]::NewGuid().ToString('N')
$supported = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token città & mare.png"
$unsupported = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.unknown"
$audio = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.flac"
$video = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.mp4"
$pdf = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.pdf"
$document = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.docx"
$text = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.txt"
$mpga = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.mpga"
$directory = Join-Path ([IO.Path]::GetTempPath()) "the-converter-folder-$token.wav"
$spreadsheet = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.xlsx"
$csv = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.csv"
$html = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.html"
$qoi = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.qoi"
$avif = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.avif"
$heif = Join-Path ([IO.Path]::GetTempPath()) "the-converter-smoke-$token.heif"
$testFiles = @($supported, $unsupported, $audio, $video, $pdf, $document, $text, $mpga, $spreadsheet, $csv, $html, $qoi, $avif, $heif)
$preferencesRoot = Join-Path ([IO.Path]::GetTempPath()) "convert-here-preferences-$token"
$previousAppData = [Environment]::GetEnvironmentVariable('APPDATA', 'Process')
$previousLocalAppData = [Environment]::GetEnvironmentVariable('LOCALAPPDATA', 'Process')
try {
    $env:APPDATA = Join-Path $preferencesRoot 'Roaming'
    $env:LOCALAPPDATA = Join-Path $preferencesRoot 'Local'
    $preferences = Join-Path $env:APPDATA 'SalvatoreCorsi\TheConverter\config\preferences.json'
    New-Item -ItemType Directory -Path (Split-Path -Parent $preferences) -Force | Out-Null
    $packageName = [regex]::Match((Split-Path -Leaf (Split-Path -Parent $library)), '^([A-Za-z0-9.-]+)_\d+\.\d+\.\d+\.\d+_(?:x64|x86|arm64|arm|neutral)_[^_]*_([a-z0-9]{13})$')
    if ($packageName.Success) {
        # Data-only package-path fixture: the DLL must prefer its own LocalCache
        # over conflicting historical Roaming settings, without installing MSIX.
        [IO.File]::WriteAllText($preferences, '{"enabled_output_formats":["heic"]}')
        $family = $packageName.Groups[1].Value + '_' + $packageName.Groups[2].Value
        $preferences = Join-Path $env:LOCALAPPDATA "Packages\$family\LocalCache\Roaming\SalvatoreCorsi\TheConverter\config\preferences.json"
        New-Item -ItemType Directory -Path (Split-Path -Parent $preferences) -Force | Out-Null
        [IO.File]::WriteAllText($preferences, '{}')
        Write-Output "Package preferences fixture: $family; conflicting historical preferences remain isolated."
    }
    foreach ($file in $testFiles) { [IO.File]::WriteAllBytes($file, [byte[]]@()) }
    New-Item -ItemType Directory -Path $directory | Out-Null
    [ConverterShellSmokeV2]::Run($library, $supported, $unsupported, $audio, $video, $pdf, $document, $text, $mpga, $directory, $spreadsheet, $csv, $html, $qoi, $avif, $heif, $preferences)
    if ($InvokeCaptureFile) {
        foreach ($case in @(
            @{ Command = 'mp3'; Files = @($audio, $video); Prefix = @('--quick', '--target', 'mp3', '--selection-file') },
            @{ Command = 'custom'; Files = @($audio, $video); Prefix = @('--custom-settings', '--selection-file') },
            @{ Command = 'pdf-bulk'; Files = @($document, $supported, $pdf); Prefix = @('--quick', '--target', 'pdf', '--pdf-batch', 'bulk', '--selection-file') },
            @{ Command = 'pdf-merge'; Files = @($pdf, $supported, $document); Prefix = @('--target', 'pdf', '--pdf-batch', 'merge', '--selection-file') },
            @{ Command = 'profile'; Files = @($supported); Prefix = @('--quick', '--profile', '0123456789abcdef0123456789abcdef', '--profile-revision', '71', '--selection-file') }
        )) {
            if (Test-Path -LiteralPath $InvokeCaptureFile) { throw 'Il file di cattura deve essere assente prima del test.' }
            if ($case.Command -ceq 'profile') {
                [IO.File]::WriteAllText($preferences, '{"enabled_output_formats":["webp"],"conversion_profiles":[{"id":"0123456789abcdef0123456789abcdef","revision":71,"name":"Salva per web & città","show_in_menu":true,"output_extension":"webp","input_policy":"images","settings":{"format":"Webp","image":{"edit":{}}}}]}')
            }
            [ConverterShellSmokeV2]::CaptureInvoke($library, $InvokeCaptureFile, [string[]]$case.Files, $case.Command)
            $captured = [IO.File]::ReadAllLines($InvokeCaptureFile)
            if ($captured.Count -ne $case.Prefix.Count + 1) { throw ('Numero argomenti Invoke non valido: ' + $case.Command) }
            for ($index = 0; $index -lt $case.Prefix.Count; $index++) {
                if ($captured[$index] -cne $case.Prefix[$index]) { throw ('Argomento Invoke non valido: ' + $case.Command) }
            }
            $request = $captured[-1]
            try {
                $selection = @([IO.File]::ReadAllText($request) | ConvertFrom-Json)
                if ($selection.Count -ne $case.Files.Count) { throw 'Numero file batch non conservato.' }
                for ($index = 0; $index -lt $case.Files.Count; $index++) {
                    if ($selection[$index] -cne $case.Files[$index]) { throw 'Ordine o percorso batch non conservato.' }
                }
            } finally {
                if ($request -like (Join-Path ([IO.Path]::GetTempPath()) 'the-converter-selection-*.json')) { Remove-Item -LiteralPath $request -Force }
                Remove-Item -LiteralPath $InvokeCaptureFile -Force
            }
        }
        Write-Output 'Invoke smoke test OK: conversioni rapide/profilo ID-revisione; PDF Merge GUI; personalizzate con --custom-settings; ordine/percorso JSON conservati.'
    }
} finally {
    [Environment]::SetEnvironmentVariable('APPDATA', $previousAppData, 'Process')
    [Environment]::SetEnvironmentVariable('LOCALAPPDATA', $previousLocalAppData, 'Process')
    foreach ($file in $testFiles) {
        if (Test-Path -LiteralPath $file) { Remove-Item -LiteralPath $file -Force }
    }
    if (Test-Path -LiteralPath $directory -PathType Container) { Remove-Item -LiteralPath $directory -Force }
    $temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
    $checked = [IO.Path]::GetFullPath($preferencesRoot)
    if (-not $checked.StartsWith($temporaryRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($checked) -notlike 'convert-here-preferences-*') { throw 'Unexpected preferences fixture directory.' }
    if (Test-Path -LiteralPath $checked) { Remove-Item -LiteralPath $checked -Recurse -Force }
}
