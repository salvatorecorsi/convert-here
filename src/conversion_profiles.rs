//! Durable user recipes. A queued job receives the complete settings snapshot.
use crate::i18n::tr;
use crate::{
    engine,
    model::{ConversionSettings, MediaKind, OutputFormat, PdfAction},
    profile_menu_shared::{self, ProfileMenuItem},
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

pub use crate::profile_menu_shared::{
    MAX_MENU_PROFILES, MAX_PREFERENCES_BYTES, MAX_PROFILE_NAME_CHARS, MAX_SAVED_PROFILES,
    ProfileInputPolicy,
};

pub fn valid_profile_id(id: &str) -> bool {
    profile_menu_shared::profile_guid_bytes(id).is_some()
}

#[derive(Debug, Clone)]
pub struct ConversionProfile {
    pub id: String,
    pub revision: u64,
    pub name: String,
    pub show_in_menu: bool,
    pub settings: ConversionSettings,
    // Preserve the producer's settings representation. Newly introduced serde
    // defaults and unknown future fields must not invalidate an existing recipe.
    settings_wire: serde_json::Value,
}

impl ConversionProfile {
    pub fn input_policy(&self) -> ProfileInputPolicy {
        if self.settings.format == OutputFormat::Pdf && self.settings.pdf.action == PdfAction::Split
        {
            ProfileInputPolicy::Pdf
        } else if self.settings.image.edit.is_some() {
            ProfileInputPolicy::Images
        } else if self.settings.image.page.is_some() {
            ProfileInputPolicy::ImagesOrText
        } else {
            ProfileInputPolicy::Compatible
        }
    }

    pub fn menu_item(&self) -> ProfileMenuItem {
        ProfileMenuItem {
            id: self.id.clone(),
            revision: self.revision,
            name: self.name.clone(),
            show_in_menu: self.show_in_menu,
            output_extension: self.settings.format.extension(),
            input_policy: self.input_policy(),
        }
    }

    pub fn validate_for_inputs<'a>(
        &self,
        inputs: impl IntoIterator<Item = &'a Path>,
    ) -> Result<()> {
        validate_profile(self)?;
        let mut count = 0;
        for input in inputs {
            count += 1;
            ensure!(
                input.is_file(),
                tr(
                    "Il profilo richiede file esistenti",
                    "The profile requires existing files"
                )
            );
            let kind = MediaKind::from_path(input);
            ensure!(
                crate::model::supported_formats(kind).contains(&self.settings.format),
                tr(
                    "Il profilo non è compatibile con tutti i file selezionati",
                    "The profile is not compatible with all selected files"
                )
            );
            match self.input_policy() {
                ProfileInputPolicy::Images => ensure!(
                    kind == MediaKind::Image,
                    tr(
                        "Il profilo di modifica richiede immagini",
                        "The editing profile requires images"
                    )
                ),
                ProfileInputPolicy::ImagesOrText => ensure!(
                    kind == MediaKind::Image
                        || input
                            .extension()
                            .and_then(|e| e.to_str())
                            .is_some_and(|e| e.eq_ignore_ascii_case("txt")),
                    tr(
                        "Il profilo pagina richiede immagini o file TXT",
                        "The page profile requires images or TXT files"
                    )
                ),
                ProfileInputPolicy::Compatible => {}
                ProfileInputPolicy::Pdf => ensure!(
                    kind == MediaKind::Pdf,
                    tr(
                        "Il profilo PDF Split richiede file PDF",
                        "The PDF Split profile requires PDF files"
                    )
                ),
            }
        }
        ensure!(
            count > 0 && count <= 10_000,
            tr(
                "Seleziona da 1 a 10000 file per il profilo",
                "Select 1 to 10000 files for the profile"
            )
        );
        Ok(())
    }
}

fn validate_profile(profile: &ConversionProfile) -> Result<()> {
    ensure!(
        profile_menu_shared::profile_guid_bytes(&profile.id).is_some(),
        tr(
            "Identificatore del profilo non valido",
            "Invalid profile identifier"
        )
    );
    ensure!(
        profile.revision > 0,
        tr(
            "Revisione del profilo non valida",
            "Invalid profile revision"
        )
    );
    ensure!(
        profile.revision == revision_for(profile)?,
        tr(
            "Il contenuto del profilo non corrisponde alla revisione salvata",
            "The profile content does not match the saved revision"
        )
    );
    let materialized: ConversionSettings = serde_json::from_value(profile.settings_wire.clone())?;
    ensure!(
        settings_equal(&materialized, &profile.settings)?,
        tr(
            "Le impostazioni del profilo non corrispondono al contenuto salvato",
            "The profile settings do not match the saved content"
        )
    );
    ensure!(
        profile_menu_shared::valid_name(&profile.name),
        tr(
            "Il nome del profilo deve contenere da 1 a 40 caratteri, senza caratteri di controllo",
            "The profile name must contain 1 to 40 characters, without control characters"
        )
    );
    ensure!(
        profile.settings.format != OutputFormat::Pdf
            || profile.settings.pdf.action != PdfAction::Merge,
        tr(
            "PDF Merge richiede la finestra per ordinare le pagine",
            "PDF Merge requires the page ordering window"
        )
    );
    engine::validate(&profile.settings)?;
    if let Some(edit) = &profile.settings.image.edit {
        crate::image_edit::validate(edit)?;
        ensure!(
            matches!(
                profile.settings.format,
                OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp | OutputFormat::Pdf
            ),
            tr(
                "Le modifiche immagini richiedono PNG, JPEG, WebP o PDF",
                "Image edits require PNG, JPEG, WebP or PDF"
            )
        );
    }
    if let Some(page) = &profile.settings.image.page {
        page.validate()?;
        ensure!(
            profile.settings.format == OutputFormat::Pdf,
            tr("Il formato pagina richiede PDF", "Page layout requires PDF")
        );
    }
    Ok(())
}

fn revision_for(profile: &ConversionProfile) -> Result<u64> {
    revision_for_wire(
        &profile.id,
        &profile.name,
        profile.show_in_menu,
        &profile.settings_wire,
    )
}

fn revision_for_wire(
    id: &str,
    name: &str,
    show: bool,
    settings: &serde_json::Value,
) -> Result<u64> {
    let value = serde_json::json!([id, name, show, settings]);
    let bytes = serde_json::to_vec(&canonical_json(&value))?;
    let digest = Sha256::digest(bytes);
    let value = u64::from_be_bytes(digest[..8].try_into().unwrap());
    Ok(value.max(1))
}

fn canonical_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(fields) => {
            let mut entries: Vec<_> = fields.iter().collect();
            entries.sort_by_key(|(name, _)| *name);
            serde_json::Value::Object(
                entries
                    .into_iter()
                    .map(|(name, value)| (name.clone(), canonical_json(value)))
                    .collect(),
            )
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(canonical_json).collect())
        }
        _ => value.clone(),
    }
}

fn settings_equal(left: &ConversionSettings, right: &ConversionSettings) -> Result<bool> {
    Ok(serde_json::to_value(left)? == serde_json::to_value(right)?)
}

fn canonical_settings(mut settings: ConversionSettings) -> ConversionSettings {
    if settings.format == OutputFormat::Pdf && settings.pdf.action == PdfAction::Split {
        settings.image.edit = None;
        settings.image.page = None;
    }
    if !matches!(
        settings.format,
        OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp | OutputFormat::Pdf
    ) {
        settings.image.edit = None;
    }
    if settings.format != OutputFormat::Pdf {
        settings.image.page = None;
    }
    settings
}

impl Serialize for ConversionProfile {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Wire<'a> {
            id: &'a str,
            revision: u64,
            name: &'a str,
            show_in_menu: bool,
            output_extension: &'static str,
            input_policy: &'static str,
            settings: &'a serde_json::Value,
        }
        validate_profile(self).map_err(serde::ser::Error::custom)?;
        let revision = revision_for(self).map_err(serde::ser::Error::custom)?;
        Wire {
            id: &self.id,
            revision,
            name: &self.name,
            show_in_menu: self.show_in_menu,
            output_extension: self.settings.format.extension(),
            input_policy: self.input_policy().as_str(),
            settings: &self.settings_wire,
        }
        .serialize(serializer)
    }
}

fn decode_profile(value: serde_json::Value) -> Option<ConversionProfile> {
    let header = profile_menu_shared::decode_profile(&value)?;
    let settings_wire = value.get("settings")?.clone();
    if revision_for_wire(
        &header.id,
        &header.name,
        header.show_in_menu,
        &settings_wire,
    )
    .ok()?
        != header.revision
    {
        return None;
    }
    let profile = ConversionProfile {
        id: header.id,
        revision: header.revision,
        name: header.name,
        show_in_menu: header.show_in_menu,
        settings: serde_json::from_value(settings_wire.clone()).ok()?,
        settings_wire,
    };
    validate_profile(&profile).ok()?;
    (profile.settings.format.extension() == header.output_extension
        && profile.input_policy() == header.input_policy)
        .then_some(profile)
}

#[derive(Debug, Clone, Default)]
pub struct ConversionProfiles {
    profiles: Vec<ConversionProfile>,
}

impl ConversionProfiles {
    pub fn len(&self) -> usize {
        self.profiles.len()
    }
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &ConversionProfile> {
        self.profiles.iter()
    }
    pub fn get(&self, id: &str) -> Option<&ConversionProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }
    pub fn resolve(&self, id: &str, revision: Option<u64>) -> Result<&ConversionProfile> {
        ensure!(
            profile_menu_shared::profile_guid_bytes(id).is_some(),
            tr(
                "Identificatore del profilo non valido",
                "Invalid profile identifier"
            )
        );
        let profile = self.get(id).ok_or_else(|| {
            anyhow::anyhow!(tr(
                "Il profilo non esiste più. Riapri il menu Converti in.",
                "The profile no longer exists. Reopen the Convert to menu."
            ))
        })?;
        ensure!(
            revision.is_none_or(|value| value == profile.revision),
            tr(
                "Il profilo è stato modificato. Riapri il menu Converti in.",
                "The profile has changed. Reopen the Convert to menu."
            )
        );
        validate_profile(profile)?;
        Ok(profile)
    }
    pub fn create(
        &mut self,
        name: String,
        show_in_menu: bool,
        settings: ConversionSettings,
    ) -> Result<String> {
        ensure!(
            !name.chars().any(char::is_control),
            tr(
                "Il nome del profilo non può contenere caratteri di controllo",
                "The profile name cannot contain control characters"
            )
        );
        ensure!(
            self.len() < MAX_SAVED_PROFILES,
            tr(
                "Puoi salvare al massimo 32 profili",
                "You can save up to 32 profiles"
            )
        );
        self.check_menu_limit(None, show_in_menu)?;
        let id = new_id()?;
        ensure!(
            self.get(&id).is_none(),
            tr(
                "Identificatore del profilo già presente; riprova",
                "The profile identifier already exists; try again"
            )
        );
        let settings = canonical_settings(settings);
        let settings_wire = serde_json::to_value(&settings)?;
        let mut profile = ConversionProfile {
            id: id.clone(),
            revision: 0,
            name: name.trim().into(),
            show_in_menu,
            settings,
            settings_wire,
        };
        profile.revision = revision_for(&profile)?;
        validate_profile(&profile)?;
        self.profiles.push(profile);
        Ok(id)
    }
    pub fn update(
        &mut self,
        id: &str,
        name: String,
        show_in_menu: bool,
        settings: ConversionSettings,
    ) -> Result<()> {
        ensure!(
            !name.chars().any(char::is_control),
            tr(
                "Il nome del profilo non può contenere caratteri di controllo",
                "The profile name cannot contain control characters"
            )
        );
        self.check_menu_limit(Some(id), show_in_menu)?;
        let previous = self.resolve(id, None)?;
        let settings = canonical_settings(settings);
        let settings_wire = if settings_equal(&previous.settings, &settings)? {
            previous.settings_wire.clone()
        } else {
            serde_json::to_value(&settings)?
        };
        let mut profile = ConversionProfile {
            id: id.into(),
            revision: 0,
            name: name.trim().into(),
            show_in_menu,
            settings,
            settings_wire,
        };
        profile.revision = revision_for(&profile)?;
        validate_profile(&profile)?;
        *self.profiles.iter_mut().find(|p| p.id == id).unwrap() = profile;
        Ok(())
    }
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.len();
        self.profiles.retain(|p| p.id != id);
        before != self.len()
    }
    pub fn set_show_in_menu(&mut self, id: &str, value: bool) -> Result<()> {
        let p = self.resolve(id, None)?.clone();
        self.update(id, p.name, value, p.settings)
    }
    fn check_menu_limit(&self, except: Option<&str>, show: bool) -> Result<()> {
        ensure!(
            !show
                || self
                    .iter()
                    .filter(|p| p.show_in_menu && Some(p.id.as_str()) != except)
                    .count()
                    < MAX_MENU_PROFILES,
            tr(
                "Puoi mostrare al massimo 8 profili nel menu",
                "You can show up to 8 profiles in the menu"
            )
        );
        Ok(())
    }
}

impl Serialize for ConversionProfiles {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.profiles.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ConversionProfiles {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let Some(values) = value.as_array() else {
            return Ok(Self::default());
        };
        let mut seen = BTreeSet::new();
        let mut duplicates = BTreeSet::new();
        for id in values
            .iter()
            .filter_map(|v| v.get("id"))
            .filter_map(serde_json::Value::as_str)
        {
            if !seen.insert(id) {
                duplicates.insert(id);
            }
        }
        let mut profiles: Vec<_> = values
            .iter()
            .cloned()
            .filter_map(decode_profile)
            .filter(|p| !duplicates.contains(p.id.as_str()))
            .take(MAX_SAVED_PROFILES)
            .collect();
        let mut visible = 0;
        for profile in &mut profiles {
            if profile.show_in_menu {
                visible += 1;
                if visible > MAX_MENU_PROFILES {
                    profile.show_in_menu = false;
                    profile.revision = revision_for(profile).map_err(serde::de::Error::custom)?;
                }
            }
        }
        Ok(Self { profiles })
    }
}

fn new_id() -> Result<String> {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct Uuid {
            a: u32,
            b: u16,
            c: u16,
            d: [u8; 8],
        }
        #[link(name = "ole32")]
        unsafe extern "system" {
            fn CoCreateGuid(value: *mut Uuid) -> i32;
        }
        let mut uuid = Uuid {
            a: 0,
            b: 0,
            c: 0,
            d: [0; 8],
        };
        ensure!(
            unsafe { CoCreateGuid(&mut uuid) } >= 0,
            tr(
                "Impossible creare l’identificatore del profilo",
                "Could not create the profile identifier"
            )
        );
        let id = format!(
            "{:08x}{:04x}{:04x}{}",
            uuid.a,
            uuid.b,
            uuid.c,
            uuid.d.map(|b| format!("{b:02x}")).join("")
        );
        ensure!(
            profile_menu_shared::profile_guid_bytes(&id).is_some(),
            tr(
                "Identificatore riservato; riprova",
                "Reserved identifier; try again"
            )
        );
        Ok(id)
    }
    #[cfg(not(windows))]
    {
        use std::io::Read;
        let mut bytes = [0; 16];
        std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
        Ok(bytes.map(|b| format!("{b:02x}")).join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_output_changes_remove_incompatible_recipes_and_keep_quality() {
        let mut settings = ConversionSettings::default();
        settings.image.quality = 73;
        settings.image.width = Some(640);
        settings.audio.sample_rate = 44_100;
        settings.image.edit = Some(crate::image_edit::ImageEditSettings {
            crop: Some(crate::image_edit::NormalizedCrop {
                left: 0.1,
                top: 0.2,
                right: 0.9,
                bottom: 0.8,
            }),
            max_bytes: Some(300_000),
            ..Default::default()
        });
        settings.image.page = Some(crate::page_layout::PageLayoutSettings {
            paper: crate::page_layout::PaperSize::A5,
            ..Default::default()
        });
        for format in OutputFormat::ALL {
            let mut selected = settings.clone();
            selected.select_output_format(format);
            assert_eq!(selected.format, format);
            assert_eq!(selected.image.quality, 73);
            assert_eq!(selected.image.width, Some(640));
            assert_eq!(selected.audio.sample_rate, 44_100);
            if matches!(
                format,
                OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp | OutputFormat::Pdf
            ) {
                assert_eq!(selected.image.edit, settings.image.edit);
            } else {
                assert!(selected.image.edit.is_none(), "{format:?}");
            }
            if format == OutputFormat::Pdf {
                assert_eq!(selected.image.page, settings.image.page);
            } else {
                assert!(selected.image.page.is_none(), "{format:?}");
            }
        }
        settings.select_output_format(OutputFormat::Bmp);
        settings.select_output_format(OutputFormat::Webp);
        assert!(settings.image.edit.is_none());
        assert_eq!(settings.image.quality, 73);
    }

    #[test]
    fn invalid_image_and_page_recipes_cannot_be_created_or_replace_a_valid_profile() {
        use crate::image_edit::{ImageEditSettings, NormalizedCrop, ResizeSettings};
        let mut profiles = ConversionProfiles::default();
        let id = profiles.create("Valido".into(), true, settings()).unwrap();
        let previous = serde_json::to_vec(&profiles).unwrap();
        let invalid_edits = [
            ImageEditSettings {
                crop: Some(NormalizedCrop {
                    left: 0.8,
                    top: 0.0,
                    right: 0.2,
                    bottom: 1.0,
                }),
                ..Default::default()
            },
            ImageEditSettings {
                resize: Some(ResizeSettings::default()),
                ..Default::default()
            },
            ImageEditSettings {
                max_bytes: Some(0),
                ..Default::default()
            },
        ];
        for edit in invalid_edits {
            let mut invalid = settings();
            invalid.image.edit = Some(edit);
            assert!(
                profiles
                    .create("Non valido".into(), false, invalid.clone())
                    .is_err()
            );
            assert!(
                profiles
                    .update(&id, "Sostituzione".into(), true, invalid)
                    .is_err()
            );
            assert_eq!(serde_json::to_vec(&profiles).unwrap(), previous);
        }
        let mut invalid_page = settings();
        invalid_page.format = OutputFormat::Pdf;
        invalid_page.image.page = Some(crate::page_layout::PageLayoutSettings {
            margin_mm: 150.0,
            ..Default::default()
        });
        assert!(
            profiles
                .create("Pagina non valida".into(), false, invalid_page.clone())
                .is_err()
        );
        assert!(
            profiles
                .update(&id, "Sostituzione".into(), true, invalid_page)
                .is_err()
        );
        assert_eq!(serde_json::to_vec(&profiles).unwrap(), previous);
        // A positive byte limit is a valid request even when a particular image cannot meet it.
        let mut small_target = settings();
        small_target.image.edit = Some(ImageEditSettings {
            max_bytes: Some(1),
            ..Default::default()
        });
        assert!(
            profiles
                .create("Limite esplicito".into(), false, small_target)
                .is_ok()
        );
    }

    fn settings() -> ConversionSettings {
        ConversionSettings {
            format: OutputFormat::Png,
            ..Default::default()
        }
    }

    fn raw_recipe(settings: serde_json::Value) -> serde_json::Value {
        let id = "0123456789abcdef0123456789abcdef";
        let revision = revision_for_wire(id, "Compatibile", true, &settings).unwrap();
        serde_json::json!({"id":id,"revision":revision,"name":"Compatibile","show_in_menu":true,"output_extension":"png","input_policy":"compatible","settings":settings})
    }

    #[test]
    fn profiles_with_missing_defaults_keep_their_wire_and_revision_when_loaded() {
        let wire = serde_json::json!({"format":"Png"});
        let raw = raw_recipe(wire.clone());
        let revision = raw["revision"].as_u64().unwrap();
        let mut profiles: ConversionProfiles =
            serde_json::from_value(serde_json::json!([raw])).unwrap();
        let profile = profiles.iter().next().unwrap().clone();
        assert_eq!(profile.revision, revision);
        assert_eq!(
            profile.settings.image.quality,
            ConversionSettings::default().image.quality
        );
        assert_eq!(
            serde_json::to_value(&profiles).unwrap()[0]["settings"],
            wire
        );
        profiles
            .update(&profile.id, "Nome nuovo".into(), false, profile.settings)
            .unwrap();
        assert_ne!(profiles.get(&profile.id).unwrap().revision, revision);
        assert_eq!(
            serde_json::to_value(&profiles).unwrap()[0]["settings"],
            wire
        );
    }

    #[test]
    fn unknown_future_settings_survive_cosmetic_changes_but_typed_tampering_fails() {
        let wire = serde_json::json!({"format":"Png","image":{"width":16,"future_crop_mode":"unchanged"},"future_option":{"enabled":true,"values":[1,"x"]}});
        let mut profiles: ConversionProfiles =
            serde_json::from_value(serde_json::json!([raw_recipe(wire.clone())])).unwrap();
        let profile = profiles.iter().next().unwrap().clone();
        profiles.set_show_in_menu(&profile.id, false).unwrap();
        assert_eq!(
            serde_json::to_value(&profiles).unwrap()[0]["settings"],
            wire
        );
        let roundtrip: ConversionProfiles =
            serde_json::from_slice(&serde_json::to_vec(&profiles).unwrap()).unwrap();
        assert_eq!(
            roundtrip.get(&profile.id).unwrap().settings.image.width,
            Some(16)
        );
        let mut tampered = profile.clone();
        tampered.settings.image.width = Some(9);
        assert!(serde_json::to_value(tampered).is_err());
        let mut raw = serde_json::to_value(&profiles).unwrap();
        raw[0]["settings"]["image"]["width"] = serde_json::json!(9);
        assert!(
            serde_json::from_value::<ConversionProfiles>(raw)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn profiles_roundtrip_and_payload_changes_invalidate_old_revision() {
        let mut profiles = ConversionProfiles::default();
        let id = profiles
            .create("Salva per web & città".into(), true, settings())
            .unwrap();
        let old = profiles.get(&id).unwrap().clone();
        profiles
            .update(&id, old.name.clone(), true, old.settings.clone())
            .unwrap();
        assert_eq!(profiles.get(&id).unwrap().revision, old.revision);
        let mut changed = old.settings.clone();
        changed.image.width = Some(37);
        profiles
            .update(&id, "Salva per web & città".into(), true, changed)
            .unwrap();
        assert!(profiles.resolve(&id, Some(old.revision)).is_err());
        let json = serde_json::to_value(&profiles).unwrap();
        let loaded: ConversionProfiles = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            loaded.get(&id).unwrap().revision,
            profiles.get(&id).unwrap().revision
        );
        let mut tampered = json;
        tampered[0]["settings"]["image"]["width"] = serde_json::json!(11);
        assert!(
            serde_json::from_value::<ConversionProfiles>(tampered)
                .unwrap()
                .is_empty()
        );
        assert!(profiles.remove(&id));
        assert!(profiles.resolve(&id, None).is_err());
    }

    #[test]
    fn malformed_profiles_do_not_discard_other_preferences_or_valid_recipes() {
        let mut profiles = ConversionProfiles::default();
        let id = profiles.create("Valido".into(), true, settings()).unwrap();
        let valid = serde_json::to_value(profiles.get(&id).unwrap()).unwrap();
        #[derive(Deserialize)]
        struct Saved {
            other: u32,
            #[serde(default)]
            conversion_profiles: ConversionProfiles,
        }
        let saved: Saved = serde_json::from_value(
            serde_json::json!({"other":42,"conversion_profiles":[null,{"id":"bad"},valid.clone()]}),
        )
        .unwrap();
        assert_eq!(saved.other, 42);
        assert_eq!(saved.conversion_profiles.len(), 1);
        let mut bad_header = valid.clone();
        bad_header["output_extension"] = serde_json::json!("webp");
        assert!(
            serde_json::from_value::<ConversionProfiles>(serde_json::json!([bad_header]))
                .unwrap()
                .is_empty()
        );
        assert!(
            serde_json::from_value::<ConversionProfiles>(serde_json::json!([valid.clone(), valid]))
                .unwrap()
                .is_empty()
        );
        assert!(
            serde_json::from_value::<ConversionProfiles>(serde_json::json!({}))
                .unwrap()
                .is_empty()
        );
        assert!(
            serde_json::from_str::<Saved>(r#"{"other":12}"#)
                .unwrap()
                .conversion_profiles
                .is_empty()
        );
    }

    #[test]
    fn saved_and_visible_limits_names_ids_and_quick_merge_are_validated() {
        let mut profiles = ConversionProfiles::default();
        let mut ids = BTreeSet::new();
        for index in 0..MAX_SAVED_PROFILES {
            let id = profiles
                .create(
                    format!("Profilo {index}"),
                    index < MAX_MENU_PROFILES,
                    settings(),
                )
                .unwrap();
            assert!(ids.insert(id));
        }
        assert_eq!(profiles.len(), MAX_SAVED_PROFILES);
        assert!(profiles.create("Extra".into(), false, settings()).is_err());
        let hidden = profiles
            .iter()
            .find(|p| !p.show_in_menu)
            .unwrap()
            .id
            .clone();
        assert!(profiles.set_show_in_menu(&hidden, true).is_err());
        let mut fresh = ConversionProfiles::default();
        assert!(
            fresh
                .create("\nNon valido".into(), true, settings())
                .is_err()
        );
        assert!(fresh.create("x".repeat(41), true, settings()).is_err());
        assert!(fresh.create("é".repeat(40), true, settings()).is_ok());
        assert!(!valid_profile_id(&"0".repeat(32)));
        assert!(!valid_profile_id("../../preferences.json"));
        let mut merge = settings();
        merge.format = OutputFormat::Pdf;
        merge.pdf.action = PdfAction::Merge;
        assert!(fresh.create("Merge".into(), false, merge).is_err());
    }

    #[test]
    fn a_prepared_profile_keeps_its_recipe_after_profile_update_or_removal() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("original.png");
        image::RgbaImage::new(32, 16).save(&input).unwrap();
        let original = std::fs::read(&input).unwrap();
        let mut recipe = settings();
        recipe.image.width = Some(8);
        recipe.output_dir = Some(directory.path().join("outputs"));
        let mut profiles = ConversionProfiles::default();
        let id = profiles
            .create("Immagine piccola".into(), true, recipe)
            .unwrap();
        let snapshot = profiles.resolve(&id, None).unwrap().clone();
        snapshot.validate_for_inputs([input.as_path()]).unwrap();
        assert!(snapshot.validate_for_inputs([directory.path()]).is_err());
        let tools = crate::tools::ToolPaths::default();
        let plan = engine::prepare(&input, &snapshot.settings, &tools).unwrap();
        let durable: engine::PreparedConversion =
            serde_json::from_slice(&serde_json::to_vec(&plan).unwrap()).unwrap();
        let mut changed = snapshot.settings.clone();
        changed.image.width = Some(4);
        profiles
            .update(&id, "Diverso".into(), true, changed)
            .unwrap();
        profiles.remove(&id);
        let outputs = engine::convert_prepared(
            &durable,
            &tools,
            &std::sync::atomic::AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert_eq!(image::open(&outputs[0]).unwrap().width(), 8);
        assert_eq!(std::fs::read(&input).unwrap(), original);
    }

    #[test]
    fn editor_page_and_split_profiles_gate_the_actual_input_domain() {
        let directory = tempfile::tempdir().unwrap();
        let image = directory.path().join("image.png");
        let pdf = directory.path().join("pages.PDF");
        let text = directory.path().join("text.TXT");
        let word = directory.path().join("document.docx");
        for path in [&image, &pdf, &text, &word] {
            std::fs::write(path, []).unwrap();
        }
        let mut profiles = ConversionProfiles::default();
        let mut editor = settings();
        editor.image.edit = Some(Default::default());
        let id = profiles.create("Editor".into(), true, editor).unwrap();
        let profile = profiles.get(&id).unwrap();
        profile.validate_for_inputs([image.as_path()]).unwrap();
        assert!(profile.validate_for_inputs([pdf.as_path()]).is_err());
        let mut page = settings();
        page.format = OutputFormat::Pdf;
        page.image.page = Some(Default::default());
        let id = profiles.create("Pagina".into(), true, page).unwrap();
        let profile = profiles.get(&id).unwrap();
        profile
            .validate_for_inputs([image.as_path(), text.as_path()])
            .unwrap();
        assert!(profile.validate_for_inputs([word.as_path()]).is_err());
        assert!(profile.validate_for_inputs([pdf.as_path()]).is_err());
        let mut split = settings();
        split.format = OutputFormat::Pdf;
        split.pdf.action = PdfAction::Split;
        let id = profiles.create("Split".into(), true, split).unwrap();
        let profile = profiles.get(&id).unwrap();
        assert_eq!(profile.input_policy(), ProfileInputPolicy::Pdf);
        profile.validate_for_inputs([pdf.as_path()]).unwrap();
        assert!(profile.validate_for_inputs([image.as_path()]).is_err());
        let mut audio = settings();
        audio.format = OutputFormat::Mp3;
        audio.image.edit = Some(Default::default());
        let id = profiles.create("Audio".into(), false, audio).unwrap();
        assert!(profiles.get(&id).unwrap().settings.image.edit.is_none());
    }
}
