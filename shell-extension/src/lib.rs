//! Minimal COM server. Explorer only collects the selected paths; conversion runs
//! in the application process. All returned strings use the COM task allocator.
#![cfg(windows)]
#![allow(non_snake_case)]

use std::{
    ffi::{c_void, OsString},
    fs::OpenOptions,
    io::{Read, Write},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::Command,
    ptr,
    sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[path = "../../src/output_preferences_shared.rs"]
mod output_preferences_shared;
use output_preferences_shared as output_preferences;
#[path = "../../src/language_shared.rs"]
#[allow(dead_code)] // Shared language-code serialization is used by the application.
mod language_shared;
use language_shared::Language;

fn localized(language: Language, italian: &'static str, english: &'static str) -> &'static str {
    if language.is_english() {
        english
    } else {
        italian
    }
}

type HResult = i32;
const S_OK: HResult = 0;
const S_FALSE: HResult = 1;
const E_NOINTERFACE: HResult = 0x80004002u32 as i32;
const E_POINTER: HResult = 0x80004003u32 as i32;
const E_FAIL: HResult = 0x80004005u32 as i32;
const E_NOTIMPL: HResult = 0x80004001u32 as i32;
const E_PENDING: HResult = 0x8000000Au32 as i32;
const CLASS_E_NOAGGREGATION: HResult = 0x80040110u32 as i32;
const CLASS_E_CLASSNOTAVAILABLE: HResult = 0x80040111u32 as i32;
const SIGDN_FILESYSPATH: u32 = 0x80058000;
const ECS_ENABLED: u32 = 0;
const ECS_DISABLED: u32 = 1;
const ECS_HIDDEN: u32 = 2;
const CREATE_NO_WINDOW: u32 = 0x08000000;
const MAX_SELECTION: u32 = 10_000;
const UNKNOWN_SELECTION_MASK: u64 = u64::MAX;
const SELECTION_CONTAINS_IMAGE: u64 = 1 << 62;
const SELECTION_CONTAINS_PDF: u64 = 1 << 61;
const SELECTION_MULTIPLE: u64 = 1 << 60;
const SELECTION_SUPPORTED: u64 = 1 << 59;
const PDF_TARGET: u64 = 1 << 19;
const PDF_MERGE_GUID_SLOT: usize = 45;
const AUDIO_TARGETS: u64 = ((1 << 7) - 1) | (((1 << 5) - 1) << 21);
const VIDEO_TARGETS: u64 = AUDIO_TARGETS | (((1 << 6) - 1) << 7) | (((1 << 6) - 1) << 26);
const IMAGE_TARGETS: u64 =
    (((1 << 8) - 1) << 12) | (((1 << 6) - 1) << 32) | (3 << 42) | TEXT_TARGET;
const PDF_TARGETS: u64 = (1 << 13) | (1 << 14) | (1 << 19) | TEXT_TARGET | (1 << 40) | (1 << 41);
const SPREADSHEET_TARGETS: u64 = (1 << 19) | (1 << 38) | (1 << 39);
const TEXT_TARGET: u64 = 1 << 20;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Guid {
    a: u32,
    b: u16,
    c: u16,
    d: [u8; 8],
}

const CLSID_CONVERTER: Guid = Guid {
    a: 0x67b51ba1,
    b: 0x8a78,
    c: 0x4b4b,
    d: [0xa5, 0xf9, 0x08, 0xa8, 0x25, 0xbe, 0x91, 0xf8],
};
const IID_IUNKNOWN: Guid = Guid {
    a: 0,
    b: 0,
    c: 0,
    d: [0xc0, 0, 0, 0, 0, 0, 0, 0x46],
};
const IID_ICLASSFACTORY: Guid = Guid {
    a: 1,
    b: 0,
    c: 0,
    d: [0xc0, 0, 0, 0, 0, 0, 0, 0x46],
};
const IID_IEXPLORERCOMMAND: Guid = Guid {
    a: 0xa08ce4d0,
    b: 0xfa25,
    c: 0x44ab,
    d: [0xb5, 0x7c, 0xc7, 0xb1, 0xc3, 0x23, 0xe0, 0xb9],
};
const IID_IENUMEXPLORERCOMMAND: Guid = Guid {
    a: 0xa88826f8,
    b: 0x186f,
    c: 0x4987,
    d: [0xaa, 0xde, 0xea, 0x0c, 0xef, 0x8f, 0xbf, 0xe8],
};

// Keep these identifiers aligned with OutputFormat in the application core.
const TARGETS: [&str; 44] = output_preferences::ALL_OUTPUT_EXTENSIONS;
const ALL_TARGETS: u64 = (1 << TARGETS.len()) - 1;

fn text_target_title(selection: u64, language: Language) -> &'static str {
    if selection == UNKNOWN_SELECTION_MASK {
        return "TXT";
    }
    if selection & SELECTION_CONTAINS_IMAGE != 0 && selection & SELECTION_CONTAINS_PDF != 0 {
        localized(language, "TXT · OCR / testo PDF", "TXT · OCR / PDF text")
    } else if selection & SELECTION_CONTAINS_IMAGE != 0 {
        "TXT · OCR"
    } else {
        "TXT"
    }
}

fn pdf_target_title(selection: u64, language: Language) -> &'static str {
    if selection != UNKNOWN_SELECTION_MASK && selection & SELECTION_MULTIPLE != 0 {
        "PDF (Bulk)"
    } else if selection != UNKNOWN_SELECTION_MASK && selection & SELECTION_CONTAINS_PDF != 0 {
        localized(language, "PDF · comprimi", "PDF · compress")
    } else {
        "PDF"
    }
}

#[link(name = "shlwapi")]
extern "system" {
    fn SHStrDupW(source: *const u16, out: *mut *mut u16) -> HResult;
}
#[link(name = "ole32")]
extern "system" {
    fn CoTaskMemFree(value: *const c_void);
}
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleExW(flags: u32, name: *const u16, module: *mut *mut c_void) -> i32;
    fn GetModuleFileNameW(module: *mut c_void, path: *mut u16, size: u32) -> u32;
    fn PackageFamilyNameFromFullName(
        full_name: *const u16,
        length: *mut u32,
        family: *mut u16,
    ) -> i32;
}

static OBJECTS: AtomicUsize = AtomicUsize::new(0);
static LOCKS: AtomicUsize = AtomicUsize::new(0);
static REQUEST_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[repr(C)]
struct UnknownVtable {
    query: unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> HResult,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}

#[repr(C)]
struct FactoryVtable {
    unknown: UnknownVtable,
    create: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        *const Guid,
        *mut *mut c_void,
    ) -> HResult,
    lock: unsafe extern "system" fn(*mut c_void, i32) -> HResult,
}

#[repr(C)]
struct ExplorerVtable {
    unknown: UnknownVtable,
    title: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut u16) -> HResult,
    icon: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut u16) -> HResult,
    tooltip: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut u16) -> HResult,
    canonical: unsafe extern "system" fn(*mut c_void, *mut Guid) -> HResult,
    state: unsafe extern "system" fn(*mut c_void, *mut c_void, i32, *mut u32) -> HResult,
    invoke: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> HResult,
    flags: unsafe extern "system" fn(*mut c_void, *mut u32) -> HResult,
    enumerate: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HResult,
}

#[repr(C)]
struct ComObject<T> {
    vtable: *const T,
    refs: AtomicU32,
}

#[repr(C)]
struct ExplorerObject {
    header: ComObject<ExplorerVtable>,
    target: Option<String>,
    parent: bool,
    pdf_merge: bool,
    selection_mask: AtomicU64,
}

#[repr(C)]
struct EnumeratorVtable {
    unknown: UnknownVtable,
    next: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void, *mut u32) -> HResult,
    skip: unsafe extern "system" fn(*mut c_void, u32) -> HResult,
    reset: unsafe extern "system" fn(*mut c_void) -> HResult,
    clone: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HResult,
}

#[repr(C)]
struct EnumeratorObject {
    header: ComObject<EnumeratorVtable>,
    cursor: AtomicUsize,
    mask: u64,
}

static FACTORY_VTABLE: FactoryVtable = FactoryVtable {
    unknown: UnknownVtable {
        query: factory_query,
        add_ref: factory_add,
        release: factory_release,
    },
    create: factory_create,
    lock: factory_lock,
};
static EXPLORER_VTABLE: ExplorerVtable = ExplorerVtable {
    unknown: UnknownVtable {
        query: explorer_query,
        add_ref: explorer_add,
        release: explorer_release,
    },
    title: explorer_title,
    icon: explorer_icon,
    tooltip: explorer_tooltip,
    canonical: explorer_canonical,
    state: explorer_state,
    invoke: explorer_invoke,
    flags: explorer_flags,
    enumerate: explorer_enumerate,
};
static ENUMERATOR_VTABLE: EnumeratorVtable = EnumeratorVtable {
    unknown: UnknownVtable {
        query: enumerator_query,
        add_ref: enumerator_add,
        release: enumerator_release,
    },
    next: enumerator_next,
    skip: enumerator_skip,
    reset: enumerator_reset,
    clone: enumerator_clone,
};

fn allocate<T>(vtable: &'static T) -> *mut c_void {
    OBJECTS.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(ComObject {
        vtable,
        refs: AtomicU32::new(1),
    })) as *mut c_void
}

fn allocate_command(target: Option<&str>, parent: bool, pdf_merge: bool) -> *mut c_void {
    OBJECTS.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(ExplorerObject {
        header: ComObject {
            vtable: &EXPLORER_VTABLE,
            refs: AtomicU32::new(1),
        },
        target: target.map(str::to_owned),
        parent,
        pdf_merge,
        selection_mask: AtomicU64::new(UNKNOWN_SELECTION_MASK),
    })) as *mut c_void
}

fn allocate_enumerator(cursor: usize, mask: u64) -> *mut c_void {
    OBJECTS.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(EnumeratorObject {
        header: ComObject {
            vtable: &ENUMERATOR_VTABLE,
            refs: AtomicU32::new(1),
        },
        cursor: AtomicUsize::new(cursor),
        mask,
    })) as *mut c_void
}

unsafe fn query<T>(
    this: *mut c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
    supported: Guid,
) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    *out = ptr::null_mut();
    if iid.is_null() {
        return E_POINTER;
    }
    if *iid != IID_IUNKNOWN && *iid != supported {
        return E_NOINTERFACE;
    }
    add::<T>(this);
    *out = this;
    S_OK
}
unsafe fn add<T>(this: *mut c_void) -> u32 {
    (*(this as *mut ComObject<T>))
        .refs
        .fetch_add(1, Ordering::Relaxed)
        + 1
}
unsafe fn release<T>(this: *mut c_void) -> u32 {
    let object = this as *mut ComObject<T>;
    let remaining = (*object).refs.fetch_sub(1, Ordering::AcqRel) - 1;
    if remaining == 0 {
        drop(Box::from_raw(object));
        OBJECTS.fetch_sub(1, Ordering::Release);
    }
    remaining
}
unsafe extern "system" fn factory_query(
    this: *mut c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    query::<FactoryVtable>(this, iid, out, IID_ICLASSFACTORY)
}
unsafe extern "system" fn factory_add(this: *mut c_void) -> u32 {
    add::<FactoryVtable>(this)
}
unsafe extern "system" fn factory_release(this: *mut c_void) -> u32 {
    release::<FactoryVtable>(this)
}
unsafe extern "system" fn explorer_query(
    this: *mut c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    query::<ExplorerVtable>(this, iid, out, IID_IEXPLORERCOMMAND)
}
unsafe extern "system" fn explorer_add(this: *mut c_void) -> u32 {
    add::<ExplorerVtable>(this)
}
unsafe extern "system" fn explorer_release(this: *mut c_void) -> u32 {
    let object = this as *mut ExplorerObject;
    let remaining = (*object).header.refs.fetch_sub(1, Ordering::AcqRel) - 1;
    if remaining == 0 {
        drop(Box::from_raw(object));
        OBJECTS.fetch_sub(1, Ordering::Release);
    }
    remaining
}
unsafe extern "system" fn enumerator_query(
    this: *mut c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    query::<EnumeratorVtable>(this, iid, out, IID_IENUMEXPLORERCOMMAND)
}
unsafe extern "system" fn enumerator_add(this: *mut c_void) -> u32 {
    add::<EnumeratorVtable>(this)
}
unsafe extern "system" fn enumerator_release(this: *mut c_void) -> u32 {
    let object = this as *mut EnumeratorObject;
    let remaining = (*object).header.refs.fetch_sub(1, Ordering::AcqRel) - 1;
    if remaining == 0 {
        drop(Box::from_raw(object));
        OBJECTS.fetch_sub(1, Ordering::Release);
    }
    remaining
}

// Merge is an action inserted beside PDF, not another OutputFormat or mask bit.
// The final entry remains custom settings; canonical GUID slots are independent
// of this presentation order.
fn enumeration_entry(index: usize, mask: u64) -> Option<(Option<&'static str>, bool)> {
    const MERGE_POSITION: usize = 20;
    if index == MERGE_POSITION {
        return (mask & PDF_TARGET != 0 && mask & SELECTION_MULTIPLE != 0)
            .then_some((Some("pdf"), true));
    }
    let target_index = if index > MERGE_POSITION {
        index - 1
    } else {
        index
    };
    if target_index < TARGETS.len() {
        (mask & (1 << target_index) != 0).then_some((Some(TARGETS[target_index]), false))
    } else if target_index == TARGETS.len() && mask & (ALL_TARGETS | SELECTION_SUPPORTED) != 0 {
        Some((None, false))
    } else {
        None
    }
}

unsafe extern "system" fn enumerator_next(
    this: *mut c_void,
    requested: u32,
    out: *mut *mut c_void,
    fetched: *mut u32,
) -> HResult {
    if !fetched.is_null() {
        *fetched = 0;
    }
    if out.is_null() {
        return E_POINTER;
    }
    let cursor = &(*(this as *mut EnumeratorObject)).cursor;
    let mask = (*(this as *mut EnumeratorObject)).mask;
    let mut returned = 0;
    while returned < requested {
        let index = cursor.fetch_add(1, Ordering::AcqRel);
        if index > TARGETS.len() + 1 {
            break;
        }
        let Some((target, pdf_merge)) = enumeration_entry(index, mask) else {
            continue;
        };
        let child = allocate_command(target, false, pdf_merge);
        (*(child as *mut ExplorerObject))
            .selection_mask
            .store(mask, Ordering::Release);
        *out.add(returned as usize) = child;
        returned += 1;
    }
    if !fetched.is_null() {
        *fetched = returned;
    }
    if returned == requested {
        S_OK
    } else {
        S_FALSE
    }
}
unsafe extern "system" fn enumerator_skip(this: *mut c_void, count: u32) -> HResult {
    let object = &*(this as *mut EnumeratorObject);
    for _ in 0..count {
        loop {
            let index = object.cursor.fetch_add(1, Ordering::AcqRel);
            if index > TARGETS.len() + 1 {
                return S_FALSE;
            }
            if enumeration_entry(index, object.mask).is_some() {
                break;
            }
        }
    }
    S_OK
}
unsafe extern "system" fn enumerator_reset(this: *mut c_void) -> HResult {
    (*(this as *mut EnumeratorObject))
        .cursor
        .store(0, Ordering::Release);
    S_OK
}
unsafe extern "system" fn enumerator_clone(this: *mut c_void, out: *mut *mut c_void) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    *out = allocate_enumerator(
        (*(this as *mut EnumeratorObject))
            .cursor
            .load(Ordering::Acquire),
        (*(this as *mut EnumeratorObject)).mask,
    );
    S_OK
}

unsafe extern "system" fn factory_create(
    _this: *mut c_void,
    outer: *mut c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    *out = ptr::null_mut();
    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    let command = allocate_command(None, true, false);
    let result = explorer_query(command, iid, out);
    explorer_release(command);
    result
}
unsafe extern "system" fn factory_lock(_this: *mut c_void, lock: i32) -> HResult {
    if lock != 0 {
        LOCKS.fetch_add(1, Ordering::Relaxed);
    } else {
        let mut value = LOCKS.load(Ordering::Acquire);
        while value > 0 {
            match LOCKS.compare_exchange_weak(value, value - 1, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(actual) => value = actual,
            }
        }
    }
    S_OK
}

unsafe fn string_result(value: &str, out: *mut *mut u16) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    *out = ptr::null_mut();
    let wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    SHStrDupW(wide.as_ptr(), out)
}
unsafe extern "system" fn explorer_title(
    this: *mut c_void,
    _items: *mut c_void,
    out: *mut *mut u16,
) -> HResult {
    let command = &*(this as *mut ExplorerObject);
    let language = ui_language();
    if command.parent {
        string_result(localized(language, "Converti in", "Convert to"), out)
    } else if command.pdf_merge {
        string_result("PDF (Merge)", out)
    } else if let Some(target) = command.target.as_deref() {
        let label = match target {
            "txt" => text_target_title(command.selection_mask.load(Ordering::Acquire), language)
                .to_owned(),
            "pdf" => pdf_target_title(command.selection_mask.load(Ordering::Acquire), language)
                .to_owned(),
            "docx" => localized(language, "Word (DOCX) · testo", "Word (DOCX) · text").to_owned(),
            "html" => localized(language, "HTML · testo", "HTML · text").to_owned(),
            "jpg" => "JPEG".to_owned(),
            "webp" => "WebP".to_owned(),
            "ff" => "Farbfeld".to_owned(),
            "exr" => "OpenEXR".to_owned(),
            "webm" => "WebM".to_owned(),
            "opus" => "Opus".to_owned(),
            other => other.to_uppercase(),
        };
        string_result(&label, out)
    } else {
        string_result(
            localized(language, "Impostazioni personalizzate…", "Custom settings…"),
            out,
        )
    }
}
unsafe extern "system" fn explorer_tooltip(
    this: *mut c_void,
    _items: *mut c_void,
    out: *mut *mut u16,
) -> HResult {
    let command = &*(this as *mut ExplorerObject);
    let language = ui_language();
    if command.pdf_merge {
        string_result(
            localized(
                language,
                "Ordina le pagine e crea un unico PDF",
                "Reorder pages and create a single PDF",
            ),
            out,
        )
    } else if command.target.is_some() {
        string_result(
            localized(
                language,
                "Converti subito con le impostazioni salvate",
                "Convert immediately using saved settings",
            ),
            out,
        )
    } else {
        string_result(
            localized(
                language,
                "Scegli formato, qualità e destinazione",
                "Choose format, quality and destination",
            ),
            out,
        )
    }
}
unsafe extern "system" fn explorer_icon(
    _this: *mut c_void,
    _items: *mut c_void,
    out: *mut *mut u16,
) -> HResult {
    match executable_path() {
        Some(path) => string_result(&format!("{},0", path.display()), out),
        None => {
            if !out.is_null() {
                *out = ptr::null_mut();
            }
            E_NOTIMPL
        }
    }
}
fn target_guid_slot(target: Option<&str>) -> usize {
    target
        .and_then(|target| TARGETS.iter().position(|value| *value == target))
        // The original settings command occupied slot 21. Preserve its GUID
        // and the original format GUIDs when adding further targets.
        .map(|index| if index >= 21 { index + 1 } else { index })
        .unwrap_or(21)
}

unsafe extern "system" fn explorer_canonical(this: *mut c_void, out: *mut Guid) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    *out = CLSID_CONVERTER;
    let command = &*(this as *mut ExplorerObject);
    if !command.parent {
        let slot = if command.pdf_merge {
            PDF_MERGE_GUID_SLOT
        } else {
            target_guid_slot(command.target.as_deref())
        };
        (*out).d[7] ^= slot as u8 + 1;
    }
    S_OK
}
unsafe extern "system" fn explorer_flags(this: *mut c_void, out: *mut u32) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    let command = &*(this as *mut ExplorerObject);
    *out = if command.parent {
        1
    } else if command.target.is_none() {
        0x20
    } else {
        0
    }; // HASSUBCOMMANDS / SEPARATORBEFORE
    S_OK
}
unsafe extern "system" fn explorer_enumerate(this: *mut c_void, out: *mut *mut c_void) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    *out = ptr::null_mut();
    if !(*(this as *mut ExplorerObject)).parent {
        return E_NOTIMPL;
    }
    let mask = (*(this as *mut ExplorerObject))
        .selection_mask
        .load(Ordering::Acquire);
    // Some callers enumerate before GetState. Only filter an actual per-object
    // selection snapshot; child GetState and Invoke always validate their inputs.
    *out = allocate_enumerator(0, visible_target_mask(mask, enabled_target_mask()));
    S_OK
}

// The unused slots preserve the native IShellItemArray / IShellItem ABI.
#[repr(C)]
struct ItemArrayVtable {
    unknown: UnknownVtable,
    bind: *const c_void,
    store: *const c_void,
    descriptions: *const c_void,
    attributes: *const c_void,
    count: unsafe extern "system" fn(*mut c_void, *mut u32) -> HResult,
    item: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> HResult,
    enumerate: *const c_void,
}
#[repr(C)]
struct ItemVtable {
    unknown: UnknownVtable,
    bind: *const c_void,
    parent: *const c_void,
    display_name: unsafe extern "system" fn(*mut c_void, u32, *mut *mut u16) -> HResult,
    attributes: unsafe extern "system" fn(*mut c_void, u32, *mut u32) -> HResult,
    compare: *const c_void,
}
unsafe fn selection_count(items: *mut c_void) -> Result<u32, HResult> {
    if items.is_null() {
        return Err(E_POINTER);
    }
    let vtable = *(items as *mut *const ItemArrayVtable);
    let mut count = 0;
    let status = ((*vtable).count)(items, &mut count);
    if status < 0 {
        Err(status)
    } else {
        Ok(count)
    }
}
unsafe fn item_path(items: *mut c_void, index: u32) -> Result<PathBuf, HResult> {
    let vtable = *(items as *mut *const ItemArrayVtable);
    let mut item = ptr::null_mut();
    let status = ((*vtable).item)(items, index, &mut item);
    if status < 0 {
        return Err(status);
    }
    if item.is_null() {
        return Err(E_FAIL);
    }
    let item_vtable = *(item as *mut *const ItemVtable);
    let mut attributes = 0;
    let attributes_status = ((*item_vtable).attributes)(item, 0x60000000, &mut attributes);
    if attributes_status < 0 || attributes & 0x20000000 != 0 || attributes & 0x40000000 == 0 {
        ((*item_vtable).unknown.release)(item);
        return Err(E_FAIL);
    }
    let mut text = ptr::null_mut();
    let status = ((*item_vtable).display_name)(item, SIGDN_FILESYSPATH, &mut text);
    ((*item_vtable).unknown.release)(item);
    if status < 0 {
        return Err(status);
    }
    if text.is_null() {
        return Err(E_FAIL);
    }
    let mut length = 0;
    while *text.add(length) != 0 {
        length += 1;
    }
    let path = PathBuf::from(OsString::from_wide(std::slice::from_raw_parts(
        text, length,
    )));
    CoTaskMemFree(text as *const c_void);
    Ok(path)
}
fn target_mask(path: &Path) -> u64 {
    let Some(extension) = path.extension().and_then(|s| s.to_str()) else {
        return 0;
    };
    match extension.to_ascii_lowercase().as_str() {
        "mp3" | "mpga" | "wav" | "flac" | "aac" | "m4a" | "ogg" | "opus" | "wma" | "aiff"
        | "aif" | "alac" | "ape" | "m4b" | "mka" | "mp2" | "oga" | "weba" | "ac3" | "au"
        | "caf" | "amr" | "dts" | "tta" | "voc" | "wv" | "tak" | "w64" | "aifc" | "ra" => {
            AUDIO_TARGETS
        }
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "wmv" | "m4v" | "mpeg" | "mpg" | "ts" | "mts"
        | "m2ts" | "3gp" | "flv" | "asf" | "vob" | "ogv" | "m2v" | "mxf" | "f4v" | "divx"
        | "3g2" | "m2t" | "mpe" | "mpv" => VIDEO_TARGETS,
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tif" | "tiff" | "gif" | "ico" | "qoi"
        | "tga" | "ff" | "hdr" | "exr" | "pnm" | "pam" | "pbm" | "pgm" | "ppm" | "avif"
        | "heic" | "heif" => IMAGE_TARGETS,
        "pdf" => PDF_TARGETS,
        "xls" | "xlsx" | "xlsm" | "ods" | "csv" => SPREADSHEET_TARGETS,
        "html" | "htm" | "doc" | "docx" | "odt" | "rtf" | "txt" | "ppt" | "pptx" | "odp" => 1 << 19,
        _ => 0,
    }
}

fn common_target_mask<'a>(paths: impl IntoIterator<Item = &'a Path>) -> u64 {
    let mut paths = paths.into_iter();
    let Some(first) = paths.next() else {
        return 0;
    };
    paths.fold(target_mask(first), |mask, path| mask & target_mask(path))
}

fn extension_mask(extensions: impl IntoIterator<Item = &'static str>) -> u64 {
    extensions.into_iter().fold(0, |mask, extension| {
        mask | TARGETS
            .iter()
            .position(|target| *target == extension)
            .map_or(0, |index| 1 << index)
    })
}

fn visible_target_mask(selection: u64, enabled: u64) -> u64 {
    if selection == UNKNOWN_SELECTION_MASK {
        enabled | SELECTION_SUPPORTED
    } else if selection & ALL_TARGETS != 0 {
        (selection & !ALL_TARGETS) | SELECTION_SUPPORTED | (selection & enabled & ALL_TARGETS)
    } else {
        0
    }
}

fn package_family_name(full_name: &std::ffi::OsStr) -> Option<String> {
    let full_name: Vec<_> = full_name.encode_wide().chain(Some(0)).collect();
    let mut length = 0;
    unsafe {
        if PackageFamilyNameFromFullName(full_name.as_ptr(), &mut length, ptr::null_mut()) != 122
            || length == 0
            || length > 256
        {
            return None;
        }
        let mut family = vec![0; length as usize];
        if PackageFamilyNameFromFullName(full_name.as_ptr(), &mut length, family.as_mut_ptr()) != 0
        {
            return None;
        }
        String::from_utf16(family.get(..length.checked_sub(1)? as usize)?).ok()
    }
}

fn preferences_paths(
    module_directory: Option<&Path>,
    roaming: Option<&Path>,
    local: Option<&Path>,
) -> Vec<PathBuf> {
    const RELATIVE: &str = r"SalvatoreCorsi\TheConverter\config\preferences.json";
    let mut paths = Vec::new();
    if let Some(family) = module_directory
        .and_then(Path::file_name)
        .and_then(package_family_name)
    {
        if let Some(local) = local.filter(|path| path.is_absolute()) {
            paths.push(
                local
                    .join("Packages")
                    .join(family)
                    .join(r"LocalCache\Roaming")
                    .join(RELATIVE),
            );
        }
    }
    if let Some(roaming) = roaming.filter(|path| path.is_absolute()) {
        paths.push(roaming.join(RELATIVE));
    }
    paths
}

fn saved_preferences_at_paths(paths: &[PathBuf]) -> Option<serde_json::Value> {
    const MAX_PREFERENCES_BYTES: u64 = 64 * 1024;
    for path in paths {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => break,
        };
        let mut bytes = Vec::new();
        if file
            .take(MAX_PREFERENCES_BYTES + 1)
            .read_to_end(&mut bytes)
            .is_err()
            || bytes.len() as u64 > MAX_PREFERENCES_BYTES
        {
            break;
        }
        // A present but invalid package-local file must not expose historical
        // settings belonging to a different installation.
        return serde_json::from_slice(&bytes).ok();
    }
    None
}

fn enabled_target_mask_at_paths(paths: &[PathBuf]) -> u64 {
    let saved = saved_preferences_at_paths(paths);
    extension_mask(output_preferences::decode_extensions(
        saved
            .as_ref()
            .and_then(|saved| saved.get("enabled_output_formats")),
    ))
}

fn ui_language_at_paths(paths: &[PathBuf]) -> Language {
    saved_preferences_at_paths(paths)
        .as_ref()
        .and_then(|saved| saved.get("ui_language"))
        .and_then(serde_json::Value::as_str)
        .map(Language::from_code)
        .unwrap_or(Language::System)
}

fn current_preferences_paths() -> Vec<PathBuf> {
    let executable = executable_path();
    let roaming = std::env::var_os("APPDATA").map(PathBuf::from);
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    preferences_paths(
        executable.as_deref().and_then(Path::parent),
        roaming.as_deref(),
        local.as_deref(),
    )
}

fn enabled_target_mask() -> u64 {
    enabled_target_mask_at_paths(&current_preferences_paths())
}

fn ui_language() -> Language {
    ui_language_at_paths(&current_preferences_paths()).resolve()
}

unsafe extern "system" fn explorer_state(
    this: *mut c_void,
    items: *mut c_void,
    slow: i32,
    out: *mut u32,
) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    *out = ECS_HIDDEN;
    let command = &*(this as *mut ExplorerObject);
    command
        .selection_mask
        .store(UNKNOWN_SELECTION_MASK, Ordering::Release);
    let Ok(count) = selection_count(items) else {
        return S_OK;
    };
    if count == 0 {
        command.selection_mask.store(0, Ordering::Release);
        return S_OK;
    }
    if count > MAX_SELECTION {
        command.selection_mask.store(0, Ordering::Release);
        *out = ECS_DISABLED;
        return S_OK;
    }
    if command.pdf_merge && count < 2 {
        command.selection_mask.store(0, Ordering::Release);
        return S_OK;
    }
    // Windows retries E_PENDING on a background thread with fOkToBeSlow.
    if count > 16 && slow == 0 {
        *out = ECS_DISABLED;
        return E_PENDING;
    }
    let required = command
        .target
        .as_deref()
        .and_then(|target| TARGETS.iter().position(|value| *value == target))
        .map(|index| 1 << index);
    let mut paths = Vec::with_capacity(count as usize);
    for index in 0..count {
        match item_path(items, index) {
            Ok(path) => paths.push(path),
            _ => {
                command.selection_mask.store(0, Ordering::Release);
                return S_OK;
            }
        }
    }
    let mask = common_target_mask(paths.iter().map(PathBuf::as_path));
    let mut metadata = 0;
    for path in &paths {
        match target_mask(path) {
            IMAGE_TARGETS => metadata |= SELECTION_CONTAINS_IMAGE,
            PDF_TARGETS => metadata |= SELECTION_CONTAINS_PDF,
            _ => {}
        }
    }
    command.selection_mask.store(
        mask | if mask != 0 {
            metadata | if count >= 2 { SELECTION_MULTIPLE } else { 0 }
        } else {
            0
        },
        Ordering::Release,
    );
    if mask != 0
        && required.is_none_or(|required| mask & required != 0)
        && required.is_none_or(|required| enabled_target_mask() & required != 0)
    {
        *out = ECS_ENABLED;
    }
    S_OK
}

fn executable_path() -> Option<PathBuf> {
    let mut module = ptr::null_mut();
    let mut buffer = vec![0u16; 32768];
    unsafe {
        // FROM_ADDRESS | UNCHANGED_REFCOUNT: identify this DLL without DllMain.
        if GetModuleHandleExW(0x6, DllCanUnloadNow as *const () as *const u16, &mut module) == 0 {
            return None;
        }
        let count = GetModuleFileNameW(module, buffer.as_mut_ptr(), buffer.len() as u32) as usize;
        if count == 0 || count >= buffer.len() {
            return None;
        }
        let dll = PathBuf::from(OsString::from_wide(&buffer[..count]));
        Some(dll.parent()?.join("the-converter.exe"))
    }
}

unsafe extern "system" fn explorer_invoke(
    this: *mut c_void,
    items: *mut c_void,
    _bind: *mut c_void,
) -> HResult {
    let command = &*(this as *mut ExplorerObject);
    if command.parent {
        return E_NOTIMPL;
    }
    let count = match selection_count(items) {
        Ok(count) if count > 0 && count <= MAX_SELECTION => count,
        _ => return E_FAIL,
    };
    if command.pdf_merge && count < 2 {
        return E_FAIL;
    }
    let mut selected_paths = Vec::with_capacity(count as usize);
    for index in 0..count {
        match item_path(items, index) {
            Ok(path) => selected_paths.push(path),
            Err(error) => return error,
        }
    }
    let mask = common_target_mask(selected_paths.iter().map(PathBuf::as_path));
    let enabled = enabled_target_mask();
    if mask == 0
        || command.target.as_deref().is_some_and(|target| {
            TARGETS
                .iter()
                .position(|value| *value == target)
                .is_none_or(|index| mask & enabled & (1 << index) == 0)
        })
    {
        return E_FAIL;
    }
    let mut paths = Vec::with_capacity(selected_paths.len());
    for path in &selected_paths {
        match path.to_str() {
            Some(value) => paths.push(value.to_owned()),
            None => return E_FAIL,
        }
    }
    let Some(executable) = executable_path() else {
        return E_FAIL;
    };
    let Ok(payload) = serde_json::to_vec(&paths) else {
        return E_FAIL;
    };
    if payload.len() > 16 * 1024 * 1024 {
        return E_FAIL;
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let request = std::env::temp_dir().join(format!(
        "the-converter-selection-{}-{timestamp}-{sequence}.json",
        std::process::id()
    ));
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&request)
    {
        Ok(file) => file,
        Err(_) => return E_FAIL,
    };
    if file.write_all(&payload).is_err() {
        drop(file);
        let _ = std::fs::remove_file(&request);
        return E_FAIL;
    }
    drop(file);
    // Command performs Win32 argument quoting. Never interpolate into a shell.
    let mut process = Command::new(executable);
    if command.pdf_merge {
        process.args(["--target", "pdf", "--pdf-batch", "merge"]);
    } else if let Some(target) = command.target.as_deref() {
        process.arg("--quick").arg("--target").arg(target);
        if target == "pdf" && count >= 2 {
            process.args(["--pdf-batch", "bulk"]);
        }
    }
    match process
        .arg("--selection-file")
        .arg(&request)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
    {
        Ok(_) => S_OK,
        Err(_) => {
            let _ = std::fs::remove_file(&request);
            E_FAIL
        }
    }
}

/// Standard COM activation entry point, called by Explorer's COM surrogate.
/// # Safety
/// The caller must supply valid COM ABI pointers for the requested interface.
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    clsid: *const Guid,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    if out.is_null() {
        return E_POINTER;
    }
    *out = ptr::null_mut();
    if clsid.is_null() || iid.is_null() {
        return E_POINTER;
    }
    if *clsid != CLSID_CONVERTER {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory = allocate(&FACTORY_VTABLE);
    let result = factory_query(factory, iid, out);
    factory_release(factory);
    result
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HResult {
    if OBJECTS.load(Ordering::Acquire) == 0 && LOCKS.load(Ordering::Acquire) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

#[cfg(test)]
#[allow(dead_code)]
#[path = "../../src/model.rs"]
mod model;

// Core-model inclusion is for format and extension parity, independent of the
// application's process-wide UI selection. Explorer uses its own preference.
#[cfg(test)]
mod i18n {
    pub fn tr(italian: &'static str, _english: &'static str) -> &'static str {
        italian
    }
}

#[cfg(test)]
use model as core_model;

#[cfg(test)]
#[path = "../../src/output_preferences.rs"]
#[rustfmt::skip]
mod core_output_preferences;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_filter_destinations_without_hiding_supported_inputs_or_custom() {
        let defaults = extension_mask(output_preferences::COMMON_OUTPUT_EXTENSIONS);
        let audio = common_target_mask([Path::new("input.flac"), Path::new("clip.mp4")]);
        let filtered = visible_target_mask(audio, defaults);
        assert_eq!(filtered & ALL_TARGETS, (1 << 0) | (1 << 1));
        assert_eq!(
            enumeration_entry(TARGETS.len() + 1, filtered),
            Some((None, false))
        );
        let no_targets = visible_target_mask(audio, 0);
        assert_eq!(no_targets, SELECTION_SUPPORTED);
        assert_eq!(
            enumeration_entry(TARGETS.len() + 1, no_targets),
            Some((None, false))
        );
        assert_eq!(visible_target_mask(0, ALL_TARGETS), 0);
        assert_eq!(enumeration_entry(TARGETS.len() + 1, 0), None);
        let pdf_batch =
            PDF_TARGET | SELECTION_CONTAINS_IMAGE | SELECTION_CONTAINS_PDF | SELECTION_MULTIPLE;
        let enabled = visible_target_mask(pdf_batch, defaults);
        assert_eq!(enumeration_entry(19, enabled), Some((Some("pdf"), false)));
        assert_eq!(enumeration_entry(20, enabled), Some((Some("pdf"), true)));
        let disabled = visible_target_mask(pdf_batch, defaults & !PDF_TARGET);
        assert_eq!(enumeration_entry(19, disabled), None);
        assert_eq!(enumeration_entry(20, disabled), None);
        assert_eq!(
            enumeration_entry(TARGETS.len() + 1, disabled),
            Some((None, false))
        );
        let flac_only = visible_target_mask(audio, extension_mask(["flac"]));
        assert_eq!(flac_only & ALL_TARGETS, 1 << 2);
    }

    #[test]
    fn packaged_preferences_paths_use_the_module_identity_and_stable_fallback() {
        let roaming = Path::new(r"C:\Fixture\Roaming");
        let local = Path::new(r"C:\Fixture\Local");
        let package = Path::new(
            r"C:\Program Files\WindowsApps\Contoso.OutputPreferences_1.2.3.0_x64__m3ejbhsgg5t2y",
        );
        let paths = preferences_paths(Some(package), Some(roaming), Some(local));
        assert_eq!(paths, [
            local.join(r"Packages\Contoso.OutputPreferences_m3ejbhsgg5t2y\LocalCache\Roaming\SalvatoreCorsi\TheConverter\config\preferences.json"),
            roaming.join(r"SalvatoreCorsi\TheConverter\config\preferences.json"),
        ]);
        assert_eq!(
            preferences_paths(
                Some(Path::new(r"C:\Programs\The Converter")),
                Some(roaming),
                Some(local)
            ),
            [paths[1].clone()]
        );
        assert!(preferences_paths(Some(package), Some(Path::new("relative")), None).is_empty());
    }

    #[test]
    fn preferences_are_reread_and_corrupt_or_oversized_files_use_common_defaults() {
        let directory = std::env::temp_dir().join(format!(
            "convert-here-output-preferences-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let package = directory.join("packaged.json");
        let legacy = directory.join("standalone.json");
        let paths = [package.clone(), legacy.clone()];
        let defaults = extension_mask(output_preferences::COMMON_OUTPUT_EXTENSIONS);
        assert_eq!(enabled_target_mask_at_paths(&paths), defaults);
        std::fs::write(
            &legacy,
            br#"{"enabled_output_formats":["flac","txt","invalid",7]}"#,
        )
        .unwrap();
        assert_eq!(enabled_target_mask_at_paths(&paths), (1 << 2) | TEXT_TARGET);
        std::fs::write(&package, br#"{"enabled_output_formats":[]}"#).unwrap();
        assert_eq!(enabled_target_mask_at_paths(&paths), 0);
        std::fs::write(
            &package,
            br#"{"enabled_output_formats":["avif","heic","pdf"]}"#,
        )
        .unwrap();
        assert_eq!(enabled_target_mask_at_paths(&paths), (3 << 42) | PDF_TARGET);
        for bytes in [
            br#"{broken"#.as_slice(),
            br#"{"enabled_output_formats":{"flac":true}}"#,
            br#"{"enabled_output_formats":["unknown"]}"#,
            br#"{"settings":{"format":"Flac"}}"#,
        ] {
            std::fs::write(&package, bytes).unwrap();
            assert_eq!(enabled_target_mask_at_paths(&paths), defaults);
        }
        std::fs::write(&package, vec![b' '; 64 * 1024 + 1]).unwrap();
        assert_eq!(enabled_target_mask_at_paths(&paths), defaults);
        std::fs::remove_file(&package).unwrap();
        assert_eq!(enabled_target_mask_at_paths(&paths), (1 << 2) | TEXT_TARGET);
    }

    #[test]
    fn language_preferences_follow_package_precedence_and_refresh_without_cache() {
        let directory = std::env::temp_dir().join(format!(
            "convert-here-language-preferences-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let package = directory.join("packaged.json");
        let legacy = directory.join("standalone.json");
        let paths = [package.clone(), legacy.clone()];
        assert_eq!(ui_language_at_paths(&paths), Language::System);
        std::fs::write(&legacy, br#"{"ui_language":"it"}"#).unwrap();
        assert_eq!(ui_language_at_paths(&paths), Language::Italian);
        std::fs::write(&package, br#"{"ui_language":"en"}"#).unwrap();
        assert_eq!(ui_language_at_paths(&paths), Language::English);
        std::fs::write(&package, br#"{"ui_language":"it"}"#).unwrap();
        assert_eq!(ui_language_at_paths(&paths), Language::Italian);
        for bytes in [
            br#"{}"#.as_slice(),
            br#"{"ui_language":"unknown"}"#,
            br#"{"ui_language":17}"#,
            br#"{broken"#,
        ] {
            std::fs::write(&package, bytes).unwrap();
            assert_eq!(ui_language_at_paths(&paths), Language::System);
        }
        std::fs::write(&package, vec![b' '; 64 * 1024 + 1]).unwrap();
        assert_eq!(ui_language_at_paths(&paths), Language::System);
        std::fs::remove_file(&package).unwrap();
        assert_eq!(ui_language_at_paths(&paths), Language::Italian);
    }

    #[test]
    fn english_titles_keep_format_identifiers_and_pdf_semantics() {
        assert_eq!(
            localized(Language::English, "Converti in", "Convert to"),
            "Convert to"
        );
        assert_eq!(
            text_target_title(
                TEXT_TARGET | SELECTION_CONTAINS_IMAGE | SELECTION_CONTAINS_PDF,
                Language::English
            ),
            "TXT · OCR / PDF text"
        );
        assert_eq!(
            text_target_title(TEXT_TARGET | SELECTION_CONTAINS_IMAGE, Language::English),
            "TXT · OCR"
        );
        assert_eq!(
            pdf_target_title(SELECTION_CONTAINS_PDF, Language::English),
            "PDF · compress"
        );
        assert_eq!(
            pdf_target_title(
                SELECTION_MULTIPLE | SELECTION_CONTAINS_PDF,
                Language::English
            ),
            "PDF (Bulk)"
        );
    }

    #[test]
    fn com_lifetime_and_interface_contract() {
        unsafe {
            assert_eq!(DllCanUnloadNow(), S_OK);
            let mut factory = ptr::null_mut();
            assert_eq!(
                DllGetClassObject(&IID_IUNKNOWN, &IID_ICLASSFACTORY, &mut factory),
                CLASS_E_CLASSNOTAVAILABLE
            );
            assert!(factory.is_null());
            assert_eq!(
                DllGetClassObject(&CLSID_CONVERTER, &IID_ICLASSFACTORY, &mut factory),
                S_OK
            );
            assert_eq!(DllCanUnloadNow(), S_FALSE);
            let mut command = ptr::null_mut();
            assert_eq!(
                factory_create(factory, factory, &IID_IEXPLORERCOMMAND, &mut command),
                CLASS_E_NOAGGREGATION
            );
            assert!(command.is_null());
            assert_eq!(
                factory_create(
                    factory,
                    ptr::null_mut(),
                    &IID_IEXPLORERCOMMAND,
                    &mut command
                ),
                S_OK
            );
            let mut state = 99;
            assert_eq!(
                explorer_state(command, ptr::null_mut(), 0, &mut state),
                S_OK
            );
            assert_eq!(state, ECS_HIDDEN);
            let mut title = ptr::null_mut();
            assert_eq!(explorer_title(command, ptr::null_mut(), &mut title), S_OK);
            assert!(!title.is_null());
            CoTaskMemFree(title as *const c_void);
            let mut flags = 0;
            assert_eq!(explorer_flags(command, &mut flags), S_OK);
            assert_eq!(flags, 1);
            let mut enumerator = ptr::null_mut();
            assert_eq!(explorer_enumerate(command, &mut enumerator), S_OK);
            assert_eq!(enumerator_release(enumerator), 0);
            // Exercise the full ABI inventory independently of the user's file.
            enumerator = allocate_enumerator(0, UNKNOWN_SELECTION_MASK);
            let mut children = [ptr::null_mut(); 2];
            let mut fetched = 0;
            assert_eq!(
                enumerator_next(enumerator, 2, children.as_mut_ptr(), &mut fetched),
                S_OK
            );
            assert_eq!(fetched, 2);
            assert_eq!(
                (*(children[0] as *mut ExplorerObject)).target.as_deref(),
                Some("mp3")
            );
            assert_eq!(
                (*(children[1] as *mut ExplorerObject)).target.as_deref(),
                Some("wav")
            );
            for child in children {
                assert_eq!(explorer_release(child), 0);
            }
            let mut clone = ptr::null_mut();
            assert_eq!(enumerator_clone(enumerator, &mut clone), S_OK);
            assert_eq!(enumerator_skip(clone, (TARGETS.len() - 1) as u32), S_OK);
            let mut custom = ptr::null_mut();
            assert_eq!(enumerator_next(clone, 1, &mut custom, &mut fetched), S_OK);
            assert!(!(*(custom as *mut ExplorerObject)).parent);
            assert!((*(custom as *mut ExplorerObject)).target.is_none());
            assert_eq!(explorer_flags(custom, &mut flags), S_OK);
            assert_eq!(flags, 0x20);
            assert_eq!(explorer_release(custom), 0);
            assert_eq!(
                enumerator_next(clone, 1, &mut custom, &mut fetched),
                S_FALSE
            );
            assert_eq!(fetched, 0);
            assert_eq!(enumerator_release(clone), 0);
            assert_eq!(enumerator_reset(enumerator), S_OK);
            assert_eq!(
                enumerator_skip(enumerator, (TARGETS.len() + 2) as u32),
                S_OK
            );
            assert_eq!(
                enumerator_next(enumerator, 1, &mut custom, &mut fetched),
                S_FALSE
            );
            assert_eq!(enumerator_release(enumerator), 0);
            let filtered = allocate_enumerator(0, 1 << 19);
            let mut filtered_children = [ptr::null_mut(); 3];
            assert_eq!(
                enumerator_next(filtered, 3, filtered_children.as_mut_ptr(), &mut fetched),
                S_FALSE
            );
            assert_eq!(fetched, 2); // PDF, then custom settings; no impossible targets.
            assert_eq!(
                (*(filtered_children[0] as *mut ExplorerObject))
                    .target
                    .as_deref(),
                Some("pdf")
            );
            assert!((*(filtered_children[1] as *mut ExplorerObject))
                .target
                .is_none());
            for child in &filtered_children[..2] {
                assert_eq!(explorer_release(*child), 0);
            }
            assert_eq!(enumerator_reset(filtered), S_OK);
            assert_eq!(enumerator_skip(filtered, 1), S_OK);
            assert_eq!(
                enumerator_next(filtered, 1, &mut custom, &mut fetched),
                S_OK
            );
            assert!((*(custom as *mut ExplorerObject)).target.is_none());
            assert_eq!(explorer_release(custom), 0);
            assert_eq!(enumerator_release(filtered), 0);
            let batch = allocate_enumerator(0, PDF_TARGET | SELECTION_MULTIPLE);
            let mut commands = [ptr::null_mut(); 4];
            assert_eq!(
                enumerator_next(batch, 4, commands.as_mut_ptr(), &mut fetched),
                S_FALSE
            );
            assert_eq!(fetched, 3); // Bulk, merge, then custom settings.
            assert!(!(*(commands[0] as *mut ExplorerObject)).pdf_merge);
            assert!((*(commands[1] as *mut ExplorerObject)).pdf_merge);
            assert!((*(commands[2] as *mut ExplorerObject)).target.is_none());
            let mut bulk_guid = CLSID_CONVERTER;
            let mut merge_guid = CLSID_CONVERTER;
            assert_eq!(explorer_canonical(commands[0], &mut bulk_guid), S_OK);
            assert_eq!(explorer_canonical(commands[1], &mut merge_guid), S_OK);
            assert_eq!(bulk_guid.d[7], CLSID_CONVERTER.d[7] ^ 20);
            assert_eq!(merge_guid.d[7], CLSID_CONVERTER.d[7] ^ 46);
            for child in &commands[..3] {
                assert_eq!(explorer_release(*child), 0);
            }
            assert_eq!(enumerator_reset(batch), S_OK);
            assert_eq!(enumerator_skip(batch, 1), S_OK);
            let mut clone = ptr::null_mut();
            assert_eq!(enumerator_clone(batch, &mut clone), S_OK);
            assert_eq!(enumerator_next(clone, 1, &mut custom, &mut fetched), S_OK);
            assert!((*(custom as *mut ExplorerObject)).pdf_merge);
            assert_eq!(explorer_release(custom), 0);
            assert_eq!(enumerator_release(clone), 0);
            assert_eq!(enumerator_release(batch), 0);
            assert_eq!(explorer_release(command), 0);
            assert_eq!(factory_lock(factory, 1), S_OK);
            assert_eq!(factory_release(factory), 0);
            assert_eq!(DllCanUnloadNow(), S_FALSE);
            assert_eq!(factory_lock(ptr::null_mut(), 0), S_OK);
            assert_eq!(DllCanUnloadNow(), S_OK);
        }
    }

    #[test]
    fn target_intersections_match_conversion_capabilities() {
        let formats = |mask: u64| {
            TARGETS
                .iter()
                .enumerate()
                .filter_map(|(index, value)| (mask & (1 << index) != 0).then_some(*value))
                .collect::<Vec<_>>()
        };
        let audio = target_mask(Path::new("musica.WAV"));
        let video = target_mask(Path::new("clip.mp4"));
        let image = target_mask(Path::new("foto.png"));
        let pdf = target_mask(Path::new("documento.pdf"));
        let document = target_mask(Path::new("testo.docx"));
        assert_eq!(
            formats(audio & video),
            vec![
                "mp3", "wav", "flac", "aac", "ogg", "opus", "aiff", "m4a", "wma", "ac3", "au",
                "caf"
            ]
        );
        assert_eq!(formats(image & pdf), vec!["png", "jpg", "pdf", "txt"]);
        assert_eq!(formats(document & image), vec!["pdf"]);
        assert_eq!(audio & image, 0);
        assert_eq!(target_mask(Path::new("unknown.exe")), 0);
        for extension in [
            "txt", "doc", "docx", "odt", "rtf", "ppt", "pptx", "odp", "html", "htm",
        ] {
            assert_eq!(
                formats(target_mask(Path::new(&format!("file.{extension}")))),
                vec!["pdf"]
            );
        }
        assert_eq!(target_mask(Path::new("musica.mpga")), audio);
        assert_eq!(audio & TEXT_TARGET, 0);
        assert_eq!(audio & pdf, 0);
        let batch = [Path::new("testo.txt"), Path::new("foto.png")];
        assert_eq!(formats(common_target_mask(batch)), vec!["pdf"]);
        assert_eq!(
            common_target_mask([Path::new("testo.txt"), Path::new("audio.wav")]),
            0
        );
        assert_eq!(
            common_target_mask([Path::new("audio.wav"), Path::new("unknown.exe")]),
            0
        );
        assert_eq!(common_target_mask(std::iter::empty()), 0);
        assert_eq!(
            text_target_title(UNKNOWN_SELECTION_MASK, Language::Italian),
            "TXT"
        );
        assert_eq!(text_target_title(TEXT_TARGET, Language::Italian), "TXT");
        assert_eq!(
            text_target_title(TEXT_TARGET | SELECTION_CONTAINS_IMAGE, Language::Italian),
            "TXT · OCR"
        );
        assert_eq!(
            text_target_title(
                TEXT_TARGET | SELECTION_CONTAINS_IMAGE | SELECTION_CONTAINS_PDF,
                Language::Italian
            ),
            "TXT · OCR / testo PDF"
        );
        assert_eq!(
            pdf_target_title(UNKNOWN_SELECTION_MASK, Language::Italian),
            "PDF"
        );
        assert_eq!(
            pdf_target_title(SELECTION_CONTAINS_PDF, Language::Italian),
            "PDF · comprimi"
        );
        assert_eq!(
            pdf_target_title(SELECTION_MULTIPLE, Language::Italian),
            "PDF (Bulk)"
        );
        assert_eq!(
            pdf_target_title(
                SELECTION_MULTIPLE | SELECTION_CONTAINS_PDF,
                Language::Italian
            ),
            "PDF (Bulk)"
        );
    }

    #[test]
    fn all_registered_extensions_and_targets_match_the_application_core() {
        let core_targets: Vec<_> = core_model::OutputFormat::ALL
            .into_iter()
            .map(core_model::OutputFormat::extension)
            .collect();
        assert_eq!(core_targets, TARGETS);
        assert!(TARGETS.len() < 60); // The upper bits record selection metadata.
        let registration = include_str!("../../scripts/install-context-menu.ps1");
        let extensions: Vec<_> = registration
            .lines()
            .find(|line| line.starts_with("$classicExtensions ="))
            .unwrap()
            .split('\'')
            .skip(1)
            .step_by(2)
            .collect();
        let source = include_str!("../../src/model.rs");
        let classification = source
            .split("impl MediaKind {")
            .nth(1)
            .unwrap()
            .split("#[derive")
            .next()
            .unwrap();
        let mut core_extensions: Vec<_> = classification
            .split('"')
            .skip(1)
            .step_by(2)
            .filter(|extension| !extension.is_empty())
            .collect();
        let mut registered = extensions.clone();
        core_extensions.sort_unstable();
        registered.sort_unstable();
        assert_eq!(
            core_extensions, registered,
            "Explorer input whitelist differs from the app"
        );
        for extension in extensions {
            let path = PathBuf::from(format!("sample.{extension}"));
            let kind = core_model::MediaKind::from_path(&path);
            let expected =
                core_model::supported_formats(kind)
                    .into_iter()
                    .fold(0u64, |mask, format| {
                        mask | (1
                            << TARGETS
                                .iter()
                                .position(|target| *target == format.extension())
                                .unwrap())
                    });
            assert_eq!(target_mask(&path), expected, "{extension}");
        }
        assert_ne!(target_mask(Path::new("sample.qoi")) & (1 << 37), 0);
        assert_ne!(target_mask(Path::new("sample.pdf")) & (1 << 41), 0);
    }

    #[test]
    fn canonical_identifiers_stay_stable_and_unique_when_formats_are_added() {
        assert_eq!(target_guid_slot(None), 21);
        let mut slots = std::collections::BTreeSet::new();
        slots.insert(target_guid_slot(None));
        for (index, target) in TARGETS.iter().enumerate() {
            let slot = target_guid_slot(Some(target));
            assert!(slots.insert(slot), "duplicate GUID for {target}");
            if index < 21 {
                assert_eq!(slot, index);
            }
        }
        assert!(slots.insert(PDF_MERGE_GUID_SLOT));
        assert_eq!(PDF_MERGE_GUID_SLOT, 45);
    }
}
