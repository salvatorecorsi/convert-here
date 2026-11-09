//! Bounded, dependency-light profile descriptors shared with Explorer's COM DLL.
use serde_json::Value;
use std::collections::BTreeSet;

pub const MAX_SAVED_PROFILES: usize = 32;
pub const MAX_MENU_PROFILES: usize = 8;
pub const MAX_PROFILE_NAME_CHARS: usize = 40;
pub const MAX_PREFERENCES_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileInputPolicy {
    Compatible,
    Images,
    ImagesOrText,
    Pdf,
}

impl ProfileInputPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::Images => "images",
            Self::ImagesOrText => "images_or_text",
            Self::Pdf => "pdf",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileMenuItem {
    pub id: String,
    pub revision: u64,
    pub name: String,
    pub show_in_menu: bool,
    pub output_extension: &'static str,
    pub input_policy: ProfileInputPolicy,
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.trim() == name
        && name.chars().count() <= MAX_PROFILE_NAME_CHARS
        && !name.chars().any(char::is_control)
}

/// Use all 128 UUID bits in a dedicated bijective namespace. Reject every
/// existing command GUID, including the parent and all legacy reserved slots.
pub fn profile_guid_bytes(id: &str) -> Option<[u8; 16]> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    let mut bytes = [0; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&id[index * 2..index * 2 + 2], 16).ok()?;
    }
    if bytes == [0; 16] {
        return None;
    }
    bytes[0] ^= 0xd7;
    bytes[1] ^= 0x39;
    const LEGACY: [u8; 16] = [
        0x67, 0xb5, 0x1b, 0xa1, 0x8a, 0x78, 0x4b, 0x4b, 0xa5, 0xf9, 0x08, 0xa8, 0x25, 0xbe, 0x91,
        0xf8,
    ];
    if bytes[..15] == LEGACY[..15] && (0..=46u8).any(|slot| bytes[15] == LEGACY[15] ^ slot) {
        return None;
    }
    Some(bytes)
}

fn output_identifier(extension: &str) -> String {
    match extension {
        "3gp" => "ThreeGp".into(),
        "ff" => "Farbfeld".into(),
        "exr" => "OpenExr".into(),
        _ => {
            let mut value = extension.to_owned();
            value[..1].make_ascii_uppercase();
            value
        }
    }
}

pub fn decode_profile(value: &Value) -> Option<ProfileMenuItem> {
    let id = value.get("id")?.as_str()?;
    profile_guid_bytes(id)?;
    let revision = value.get("revision")?.as_u64()?;
    let name = value.get("name")?.as_str()?;
    if revision == 0 || !valid_name(name) {
        return None;
    }
    let show_in_menu = value.get("show_in_menu")?.as_bool()?;
    let extension = value.get("output_extension")?.as_str()?;
    let output_extension = crate::output_preferences_shared::ALL_OUTPUT_EXTENSIONS
        .iter()
        .copied()
        .find(|known| *known == extension)?;
    let settings = value.get("settings")?;
    if settings.get("format")?.as_str()? != output_identifier(output_extension) {
        return None;
    }
    if output_extension == "pdf"
        && settings
            .get("pdf")
            .and_then(|v| v.get("action"))
            .and_then(Value::as_str)
            == Some("Merge")
    {
        return None;
    }
    let image = settings.get("image");
    let edit = image
        .and_then(|v| v.get("edit"))
        .is_some_and(|v| !v.is_null());
    let page = image
        .and_then(|v| v.get("page"))
        .is_some_and(|v| !v.is_null());
    if edit && !matches!(output_extension, "png" | "jpg" | "webp" | "pdf")
        || page && output_extension != "pdf"
    {
        return None;
    }
    let split = output_extension == "pdf"
        && settings
            .get("pdf")
            .and_then(|v| v.get("action"))
            .and_then(Value::as_str)
            == Some("Split");
    if split && (edit || page) {
        return None;
    }
    let derived = if split {
        ProfileInputPolicy::Pdf
    } else if edit {
        ProfileInputPolicy::Images
    } else if page {
        ProfileInputPolicy::ImagesOrText
    } else {
        ProfileInputPolicy::Compatible
    };
    if value.get("input_policy")?.as_str()? != derived.as_str() {
        return None;
    }
    Some(ProfileMenuItem {
        id: id.into(),
        revision,
        name: name.into(),
        show_in_menu,
        output_extension,
        input_policy: derived,
    })
}

// This entry point is used by the independently compiled Explorer DLL.
#[allow(dead_code)]
pub fn decode_profiles(value: Option<&Value>) -> Vec<ProfileMenuItem> {
    let Some(values) = value.and_then(Value::as_array) else {
        return vec![];
    };
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for id in values
        .iter()
        .filter_map(|v| v.get("id"))
        .filter_map(Value::as_str)
    {
        if !seen.insert(id) {
            duplicates.insert(id);
        }
    }
    let mut visible = 0;
    values
        .iter()
        .filter_map(decode_profile)
        .filter(|p| !duplicates.contains(p.id.as_str()))
        .take(MAX_SAVED_PROFILES)
        .map(|mut p| {
            if p.show_in_menu {
                visible += 1;
                if visible > MAX_MENU_PROFILES {
                    p.show_in_menu = false;
                }
            }
            p
        })
        .collect()
}
