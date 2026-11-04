//! User-visible destinations. Input detection and conversion capabilities stay
//! independent from this list.
use crate::model::OutputFormat;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::output_preferences_shared as shared;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputPreferences {
    enabled: Vec<OutputFormat>,
}

impl Default for OutputPreferences {
    fn default() -> Self {
        Self {
            enabled: shared::COMMON_OUTPUT_EXTENSIONS
                .into_iter()
                .filter_map(format_from_extension)
                .collect(),
        }
    }
}

impl OutputPreferences {
    pub fn contains(&self, format: OutputFormat) -> bool {
        self.enabled.contains(&format)
    }

    pub fn set_enabled(&mut self, format: OutputFormat, enabled: bool) {
        if enabled {
            if !self.contains(format) {
                self.enabled.push(format);
            }
        } else {
            self.enabled.retain(|candidate| *candidate != format);
        }
    }

    pub fn filter<'a, I>(&'a self, formats: I) -> impl Iterator<Item = OutputFormat> + 'a
    where
        I: IntoIterator<Item = OutputFormat>,
        I::IntoIter: 'a,
    {
        formats.into_iter().filter(|format| self.contains(*format))
    }

    pub fn reset_common(&mut self) {
        *self = Self::default();
    }

    pub fn enable_all(&mut self) {
        self.enabled = OutputFormat::ALL.to_vec();
    }

    pub fn disable_all(&mut self) {
        self.enabled.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.enabled.is_empty()
    }
}

impl Serialize for OutputPreferences {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.enabled
            .iter()
            .map(|format| format.extension())
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for OutputPreferences {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self {
            enabled: shared::decode_extensions(Some(&value))
                .into_iter()
                .filter_map(format_from_extension)
                .collect(),
        })
    }
}

fn format_from_extension(extension: &str) -> Option<OutputFormat> {
    OutputFormat::ALL
        .into_iter()
        .find(|format| format.extension() == extension)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MediaKind, common_formats, supported_formats};
    use std::path::Path;

    #[test]
    fn defaults_have_exact_extensions_and_preserve_capability_order() {
        assert_eq!(
            OutputFormat::ALL.map(OutputFormat::extension),
            shared::ALL_OUTPUT_EXTENSIONS
        );
        let preferences = OutputPreferences::default();
        assert_eq!(
            serde_json::to_string(&preferences).unwrap(),
            r#"["mp3","wav","jpg","png","webp","gif","pdf","mp4"]"#
        );
        assert_eq!(
            preferences
                .filter(supported_formats(MediaKind::Audio))
                .collect::<Vec<_>>(),
            vec![OutputFormat::Mp3, OutputFormat::Wav]
        );
        assert_eq!(
            preferences
                .filter(common_formats([
                    Path::new("foto.heif"),
                    Path::new("testo.pdf")
                ]))
                .collect::<Vec<_>>(),
            vec![OutputFormat::Png, OutputFormat::Jpg, OutputFormat::Pdf]
        );
        assert!(
            preferences
                .filter(common_formats([Path::new("file.exe")]))
                .next()
                .is_none()
        );
        assert!(supported_formats(MediaKind::Audio).contains(&OutputFormat::Flac));
        assert!(!preferences.contains(OutputFormat::Flac));
    }

    #[test]
    fn old_settings_default_and_invalid_entries_cannot_enable_destinations() {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Saved {
            enabled_output_formats: OutputPreferences,
        }
        let old: Saved = serde_json::from_str(r#"{"settings":{"format":"Flac"}}"#).unwrap();
        assert_eq!(old.enabled_output_formats, OutputPreferences::default());
        for value in [r#"null"#, r#"17"#, r#"{"mp3":true}"#, r#""mp3""#] {
            assert_eq!(
                serde_json::from_str::<OutputPreferences>(value).unwrap(),
                OutputPreferences::default()
            );
        }
        let sanitized: OutputPreferences =
            serde_json::from_str(r#"["flac","flac","unknown","MP3","jpeg",true,23,null,"avif"]"#)
                .unwrap();
        assert_eq!(
            serde_json::to_string(&sanitized).unwrap(),
            r#"["flac","avif"]"#
        );
        let empty: OutputPreferences = serde_json::from_str("[]").unwrap();
        assert!(empty.is_empty());
        assert_eq!(serde_json::to_string(&empty).unwrap(), "[]");
    }

    #[test]
    fn toggles_bulk_actions_and_roundtrip_preserve_every_known_format() {
        let mut preferences = OutputPreferences::default();
        preferences.disable_all();
        assert!(preferences.is_empty());
        preferences.set_enabled(OutputFormat::Flac, true);
        preferences.set_enabled(OutputFormat::Flac, true);
        assert_eq!(serde_json::to_string(&preferences).unwrap(), r#"["flac"]"#);
        preferences.set_enabled(OutputFormat::Flac, false);
        assert!(preferences.is_empty());
        preferences.enable_all();
        assert_eq!(
            preferences.filter(OutputFormat::ALL).count(),
            OutputFormat::ALL.len()
        );
        let restored: OutputPreferences =
            serde_json::from_slice(&serde_json::to_vec(&preferences).unwrap()).unwrap();
        assert_eq!(restored, preferences);
        preferences.reset_common();
        assert_eq!(preferences, OutputPreferences::default());
    }
}
