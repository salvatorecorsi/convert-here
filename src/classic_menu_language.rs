//! Refresh an existing, owned per-user classic Explorer menu after a UI-language
//! change. This never creates a menu or modifies another installation's keys.
use crate::language_shared::Language;

pub fn refresh_classic_menu_language(language: Language) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        windows::refresh(language)
    }
    #[cfg(not(windows))]
    {
        let _ = language;
        Ok(())
    }
}

#[cfg(windows)]
mod windows {
    use super::Language;
    use std::{ffi::c_void, io, path::Path, ptr};

    type HKey = isize;
    const HKCU: HKey = 0x8000_0001u32 as i32 as isize;
    const QUERY_VALUE: u32 = 0x0001;
    const SET_VALUE: u32 = 0x0002;
    const REG_SZ: u32 = 1;
    const CLASS_ID: &str = "{67B51BA1-8A78-4B4B-A5F9-08A825BE91F8}";
    const CLASS_KEY: &str =
        r"Software\Classes\CLSID\{67B51BA1-8A78-4B4B-A5F9-08A825BE91F8}\InprocServer32";
    const MENU_KEY: &str = r"Software\Classes\*\shell\TheConverter";

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegOpenKeyExW(
            key: HKey,
            name: *const u16,
            options: u32,
            access: u32,
            out: *mut HKey,
        ) -> i32;
        fn RegQueryValueExW(
            key: HKey,
            name: *const u16,
            reserved: *mut u32,
            kind: *mut u32,
            data: *mut u8,
            bytes: *mut u32,
        ) -> i32;
        fn RegSetValueExW(
            key: HKey,
            name: *const u16,
            reserved: u32,
            kind: u32,
            data: *const u8,
            bytes: u32,
        ) -> i32;
        fn RegCloseKey(key: HKey) -> i32;
    }
    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHChangeNotify(event: i32, flags: u32, item1: *const c_void, item2: *const c_void);
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(Some(0)).collect()
    }

    struct Key(HKey);
    impl Drop for Key {
        fn drop(&mut self) {
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }
    impl Key {
        fn open(name: &str, access: u32) -> io::Result<Option<Self>> {
            let mut key = 0;
            let status = unsafe { RegOpenKeyExW(HKCU, wide(name).as_ptr(), 0, access, &mut key) };
            match status {
                0 => Ok(Some(Self(key))),
                2 | 3 => Ok(None),
                error => Err(io::Error::from_raw_os_error(error)),
            }
        }
        fn string(&self, name: Option<&str>) -> io::Result<Option<String>> {
            let name = name.map(wide);
            let name = name.as_ref().map_or(ptr::null(), |name| name.as_ptr());
            let mut kind = 0;
            let mut bytes = 0;
            let status = unsafe {
                RegQueryValueExW(
                    self.0,
                    name,
                    ptr::null_mut(),
                    &mut kind,
                    ptr::null_mut(),
                    &mut bytes,
                )
            };
            if status == 2 {
                return Ok(None);
            }
            if status != 0 {
                return Err(io::Error::from_raw_os_error(status));
            }
            if kind != REG_SZ || bytes > 65536 || !bytes.is_multiple_of(2) {
                return Ok(None);
            }
            let mut data = vec![0u16; bytes as usize / 2 + 1];
            let status = unsafe {
                RegQueryValueExW(
                    self.0,
                    name,
                    ptr::null_mut(),
                    &mut kind,
                    data.as_mut_ptr().cast(),
                    &mut bytes,
                )
            };
            if status != 0 {
                return Err(io::Error::from_raw_os_error(status));
            }
            if kind != REG_SZ {
                return Ok(None);
            }
            let length = data
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(data.len());
            Ok(String::from_utf16(&data[..length]).ok())
        }
        fn set_string(&self, name: &str, value: &str) -> io::Result<()> {
            let value = wide(value);
            let status = unsafe {
                RegSetValueExW(
                    self.0,
                    wide(name).as_ptr(),
                    0,
                    REG_SZ,
                    value.as_ptr().cast(),
                    (value.len() * 2) as u32,
                )
            };
            if status != 0 {
                return Err(io::Error::from_raw_os_error(status));
            }
            Ok(())
        }
    }

    fn same_existing_file(registered: &str, expected: &Path) -> bool {
        let Ok(registered) = Path::new(registered).canonicalize() else {
            return false;
        };
        let Ok(expected) = expected.canonicalize() else {
            return false;
        };
        registered
            .as_os_str()
            .eq_ignore_ascii_case(expected.as_os_str())
    }

    pub(super) fn refresh(language: Language) -> io::Result<()> {
        let executable = std::env::current_exe()?;
        let Some(directory) = executable.parent() else {
            return Ok(());
        };
        let Some(server) = Key::open(CLASS_KEY, QUERY_VALUE)? else {
            return Ok(());
        };
        let Some(registered) = server.string(None)? else {
            return Ok(());
        };
        if !same_existing_file(&registered, &directory.join("the_converter_shell.dll")) {
            return Ok(());
        }
        let Some(menu) = Key::open(MENU_KEY, QUERY_VALUE | SET_VALUE)? else {
            return Ok(());
        };
        if !menu
            .string(Some("ExplorerCommandHandler"))?
            .is_some_and(|value| value.eq_ignore_ascii_case(CLASS_ID))
        {
            return Ok(());
        }
        let title = if language.is_english() {
            "Convert to"
        } else {
            "Converti in"
        };
        if menu.string(Some("MUIVerb"))?.as_deref() == Some(title) {
            return Ok(());
        }
        menu.set_string("MUIVerb", title)?;
        unsafe {
            SHChangeNotify(0x0800_0000, 0, ptr::null(), ptr::null());
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn ownership_requires_the_exact_existing_library() {
            let directory = tempfile::tempdir().unwrap();
            let expected = directory.path().join("the_converter_shell.dll");
            let other = directory.path().join("another.dll");
            std::fs::write(&expected, []).unwrap();
            std::fs::write(&other, []).unwrap();
            assert!(same_existing_file(expected.to_str().unwrap(), &expected));
            assert!(!same_existing_file(other.to_str().unwrap(), &expected));
            assert!(!same_existing_file("missing.dll", &expected));
        }
    }
}
