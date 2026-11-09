//! Language identifiers shared by the application and the Explorer extension.
//! Keep these independent of serialization and UI libraries.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    System,
    Italian,
    English,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code {
            "it" | "Italian" => Self::Italian,
            "en" | "English" => Self::English,
            _ => Self::System,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Italian => "it",
            Self::English => "en",
        }
    }

    pub fn resolve(self) -> Self {
        if self != Self::System {
            return self;
        }
        system_language()
    }

    pub fn is_english(self) -> bool {
        self.resolve() == Self::English
    }
}

#[cfg(windows)]
fn system_language() -> Language {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }
    language_from_windows_id(unsafe { GetUserDefaultUILanguage() })
}

#[cfg(any(windows, test))]
fn language_from_windows_id(id: u16) -> Language {
    // PRIMARYLANGID: Italian in any region; English is the supported fallback.
    if id & 0x03ff == 0x10 {
        Language::Italian
    } else {
        Language::English
    }
}

#[cfg(not(windows))]
fn system_language() -> Language {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    if locale.split(['_', '-', '.', '@']).next() == Some("it") {
        Language::Italian
    } else {
        Language::English
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_and_windows_regions_resolve_without_changing_identifiers() {
        for language in [Language::System, Language::Italian, Language::English] {
            assert_eq!(Language::from_code(language.code()), language);
        }
        assert_eq!(Language::from_code("unsupported"), Language::System);
        assert_eq!(language_from_windows_id(0x0410), Language::Italian);
        assert_eq!(language_from_windows_id(0x0810), Language::Italian);
        assert_eq!(language_from_windows_id(0x0409), Language::English);
        assert_eq!(language_from_windows_id(0x040c), Language::English);
        assert_eq!(language_from_windows_id(0), Language::English);
    }
}
