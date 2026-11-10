// Exact upstream notices; each SHA-256/provenance is in portable-packages.json.
pub(super) fn bytes(name: &str) -> Option<&'static [u8]> {
    match name {
        "giflib-COPYING" => Some(include_bytes!("licenses/giflib-COPYING")),
        "giflib-SOURCE-NOTICES.txt" => Some(include_bytes!("licenses/giflib-SOURCE-NOTICES.txt")),
        "libidn2-AUTHORS" => Some(include_bytes!("licenses/libidn2-AUTHORS")),
        "libidn2-COPYING" => Some(include_bytes!("licenses/libidn2-COPYING")),
        "libidn2-COPYING.LESSERv3" => Some(include_bytes!("licenses/libidn2-COPYING.LESSERv3")),
        "libidn2-COPYING.unicode" => Some(include_bytes!("licenses/libidn2-COPYING.unicode")),
        "libidn2-COPYINGv2" => Some(include_bytes!("licenses/libidn2-COPYINGv2")),
        "lz4-LICENSE" => Some(include_bytes!("licenses/lz4-LICENSE")),
        "lz4-contrib_djgpp_LICENSE" => Some(include_bytes!("licenses/lz4-contrib_djgpp_LICENSE")),
        "lz4-examples_COPYING" => Some(include_bytes!("licenses/lz4-examples_COPYING")),
        "lz4-lib_LICENSE" => Some(include_bytes!("licenses/lz4-lib_LICENSE")),
        "lz4-programs_COPYING" => Some(include_bytes!("licenses/lz4-programs_COPYING")),
        "lz4-tests_data_COPYING" => Some(include_bytes!("licenses/lz4-tests_data_COPYING")),
        "x264-COPYING" => Some(include_bytes!("licenses/x264-COPYING")),
        "x265-COPYING" => Some(include_bytes!("licenses/x265-COPYING")),
        "x265-source_dynamicHDR10_LICENSE.txt" => Some(include_bytes!(
            "licenses/x265-source_dynamicHDR10_LICENSE.txt"
        )),
        "x265-source_dynamicHDR10_json11_LICENSE.txt" => Some(include_bytes!(
            "licenses/x265-source_dynamicHDR10_json11_LICENSE.txt"
        )),
        _ => None,
    }
}
