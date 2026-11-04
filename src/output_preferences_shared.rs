//! Small output-preference codec shared with the independently built Explorer DLL.
pub const ALL_OUTPUT_EXTENSIONS: [&str; 44] = [
    "mp3", "wav", "flac", "aac", "ogg", "opus", "aiff", "mp4", "mkv", "mov", "webm", "avi", "gif",
    "png", "jpg", "webp", "bmp", "tiff", "ico", "pdf", "txt", "m4a", "wma", "ac3", "au", "caf",
    "mpeg", "wmv", "flv", "ts", "m4v", "3gp", "qoi", "tga", "ff", "exr", "hdr", "pnm", "csv",
    "xlsx", "docx", "html", "avif", "heic",
];

pub const COMMON_OUTPUT_EXTENSIONS: [&str; 8] =
    ["mp3", "wav", "jpg", "png", "webp", "gif", "pdf", "mp4"];

/// Missing or malformed fields use the common defaults. An explicit empty array
/// stays empty, while unknown extensions and non-string entries cannot enable
/// any destination. Keep the saved order and remove duplicates.
pub fn decode_extensions(value: Option<&serde_json::Value>) -> Vec<&'static str> {
    let Some(extensions) = value.and_then(serde_json::Value::as_array) else {
        return COMMON_OUTPUT_EXTENSIONS.to_vec();
    };
    let mut enabled = Vec::new();
    for extension in extensions.iter().filter_map(serde_json::Value::as_str) {
        if let Some(known) = ALL_OUTPUT_EXTENSIONS
            .iter()
            .copied()
            .find(|known| *known == extension)
            .filter(|known| !enabled.contains(known))
        {
            enabled.push(known);
        }
    }
    if enabled.is_empty() && !extensions.is_empty() {
        COMMON_OUTPUT_EXTENSIONS.to_vec()
    } else {
        enabled
    }
}
