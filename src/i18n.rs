//! Small bilingual layer. Format strings remain compile-time checked in both
//! languages; persisted identifiers, paths and conversion settings stay stable.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cell::Cell,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};

pub use crate::language_shared::Language;

// Library users and existing unit tests retain the original language until an
// application entry point explicitly initializes the user's preference.
static CHOICE: AtomicU8 = AtomicU8::new(1);
static ACTIVE: AtomicU8 = AtomicU8::new(1);
static OVERRIDE: AtomicBool = AtomicBool::new(false);
thread_local! { static SCOPED: Cell<Option<Language>> = const { Cell::new(None) }; }

impl Serialize for Language {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.code())
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self::from_code(value.as_str().unwrap_or("system")))
    }
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => tr("Sistema", "System"),
            Self::Italian => "Italiano",
            Self::English => "English",
        }
    }
}

fn number(language: Language) -> u8 {
    match language {
        Language::System => 0,
        Language::Italian => 1,
        Language::English => 2,
    }
}
fn language(number: u8) -> Language {
    match number {
        1 => Language::Italian,
        2 => Language::English,
        _ => Language::System,
    }
}

pub fn current_language() -> Language {
    language(CHOICE.load(Ordering::Relaxed))
}

pub fn set_language(value: Language) {
    CHOICE.store(number(value), Ordering::Relaxed);
    ACTIVE.store(number(value.resolve()), Ordering::Relaxed);
}

pub fn is_english() -> bool {
    SCOPED
        .with(|value| value.get())
        .map(Language::is_english)
        .unwrap_or_else(|| ACTIVE.load(Ordering::Relaxed) == 2)
}

pub fn tr(italian: &'static str, english: &'static str) -> &'static str {
    if is_english() { english } else { italian }
}

/// Per-invocation selection; does not overwrite the saved GUI preference.
pub fn set_language_override(value: Language) {
    OVERRIDE.store(true, Ordering::Relaxed);
    set_language(value);
}

pub fn has_language_override() -> bool {
    OVERRIDE.load(Ordering::Relaxed)
}

pub fn preferences_path() -> Option<PathBuf> {
    // Enables isolated UI/CLI verification without touching personal settings.
    if let Some(directory) = std::env::var_os("THE_CONVERTER_CONFIG_DIR") {
        return Some(PathBuf::from(directory).join("preferences.json"));
    }
    directories::ProjectDirs::from("com", "SalvatoreCorsi", "TheConverter")
        .map(|dirs| dirs.config_dir().join("preferences.json"))
}

pub fn initialize_from_preferences() -> Language {
    let value = preferences_path()
        .and_then(|path| preference_language_at(&path))
        .unwrap_or_default();
    if !has_language_override() {
        set_language(value);
    }
    value
}

fn preference_language_at(path: &std::path::Path) -> Option<Language> {
    use std::io::Read;
    let limit = crate::profile_menu_shared::MAX_PREFERENCES_BYTES;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > limit {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    serde_json::from_value(value.get("ui_language")?.clone()).ok()
}

pub fn refresh_explorer_language() {
    let _ = crate::classic_menu_language::refresh_classic_menu_language(current_language());
}

/// Scoped language for deterministic, parallel tests. Restored even on panic.
pub fn with_language<T>(value: Language, action: impl FnOnce() -> T) -> T {
    struct Restore(Option<Language>);
    impl Drop for Restore {
        fn drop(&mut self) {
            SCOPED.with(|slot| slot.set(self.0));
        }
    }
    let previous = SCOPED.with(|slot| slot.replace(Some(value)));
    let _restore = Restore(previous);
    action()
}

#[macro_export]
macro_rules! trf {
    ($italian:literal, $english:literal $(, $args:expr)* $(,)?) => {
        if $crate::i18n::is_english() {
            format!($english $(, $args)*)
        } else {
            format!($italian $(, $args)*)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_reader_obeys_the_shared_preferences_size_limit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let mut bytes = br#"{"ui_language":"en","conversion_profiles":[]}"#.to_vec();
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(preference_language_at(&path), Some(Language::English));
        bytes.resize(
            crate::profile_menu_shared::MAX_PREFERENCES_BYTES as usize + 1,
            b' ',
        );
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(preference_language_at(&path), None);
    }

    #[test]
    fn preferences_are_backward_compatible_and_ocr_is_independent() {
        for (json, expected) in [
            ("\"it\"", Language::Italian),
            ("\"en\"", Language::English),
            ("\"system\"", Language::System),
            ("\"fr\"", Language::System),
            ("null", Language::System),
            ("17", Language::System),
        ] {
            assert_eq!(serde_json::from_str::<Language>(json).unwrap(), expected);
        }
        assert_eq!(serde_json::to_string(&Language::English).unwrap(), "\"en\"");
        let settings = crate::model::ConversionSettings::default();
        with_language(Language::English, || {
            assert_eq!(settings.ocr.language, "ita+eng");
            assert_eq!(settings.format.extension(), "mp3");
        });
    }

    #[test]
    fn nested_and_parallel_language_scopes_preserve_formatting() {
        let handles: Vec<_> = [Language::Italian, Language::English]
            .into_iter()
            .map(|selected| {
                std::thread::spawn(move || {
                    with_language(selected, || {
                        let count = 3;
                        let rendered = crate::trf!("Pagina {} di {count}", "Page {} of {count}", 1);
                        assert_eq!(
                            rendered,
                            if selected == Language::English {
                                "Page 1 of 3"
                            } else {
                                "Pagina 1 di 3"
                            }
                        );
                        with_language(Language::Italian, || {
                            assert_eq!(tr("File", "Files"), "File")
                        });
                        assert_eq!(is_english(), selected == Language::English);
                    })
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
