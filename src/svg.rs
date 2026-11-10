//! Local, bounded SVG rasterization. Document references never open files or URLs.
use crate::i18n::tr;
use anyhow::{Context, Result, ensure};
use image::RgbaImage;
use resvg::{tiny_skia, usvg};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::Path,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

pub const MAX_INPUT_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_PIXELS: u64 = 40_000_000;
pub const MAX_SIDE: u32 = 32_768;
const MAX_NODES: u32 = 20_000;
const MAX_DEPTH: usize = 64;
const MAX_TEXT_BYTES: usize = 8192;
const MAX_PATH_BYTES: usize = 512 * 1024;
const MAX_WORK_BYTES: u64 = 512 * 1024 * 1024;

pub fn is_svg_input(input: &Path) -> bool {
    input
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("svg"))
}

fn cancelled(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Relaxed),
        tr("Rendering SVG annullato", "SVG rendering cancelled")
    );
    Ok(())
}

fn limit_error() -> &'static str {
    tr(
        "L’SVG supera i limiti di dimensioni o complessità del renderer locale",
        "The SVG exceeds the local renderer's size or complexity limits",
    )
}

fn reference_error() -> &'static str {
    tr(
        "L’SVG contiene risorse esterne o dati incorporati non consentiti: usa un SVG autonomo con tracciati e testo",
        "The SVG contains unsupported external or embedded resources: use a self-contained SVG with paths and text",
    )
}

fn embedded_font() -> &'static [u8] {
    static FONT: OnceLock<Vec<u8>> = OnceLock::new();
    FONT.get_or_init(|| {
        eframe::egui::FontDefinitions::default().font_data["Ubuntu-Light"]
            .font
            .as_ref()
            .to_vec()
    })
}

fn check_css(value: &str) -> Result<()> {
    let lower = value.to_ascii_lowercase();
    // Escape sequences are not needed for local fragment references and can
    // otherwise disguise @import/url tokens from this resource policy.
    ensure!(
        !lower.contains('\\') && !lower.contains("@import") && !lower.contains("@font-face"),
        reference_error()
    );
    let mut remaining = lower.as_str();
    while let Some(start) = remaining.find("url(") {
        remaining = &remaining[start + 4..];
        let end = remaining.find(')').context(reference_error())?;
        let target = remaining[..end].trim().trim_matches(['\'', '"']).trim();
        ensure!(
            target.starts_with('#') && target.len() > 1,
            reference_error()
        );
        remaining = &remaining[end + 1..];
    }
    Ok(())
}

fn expanded_nodes<'a>(
    node: usvg::roxmltree::Node<'a, 'a>,
    ids: &HashMap<&'a str, usvg::roxmltree::Node<'a, 'a>>,
    active: &mut HashSet<usvg::roxmltree::NodeId>,
    memo: &mut HashMap<usvg::roxmltree::NodeId, u64>,
    depth: usize,
) -> Result<u64> {
    ensure!(depth <= MAX_DEPTH, limit_error());
    if let Some(count) = memo.get(&node.id()) {
        return Ok(*count);
    }
    ensure!(
        active.insert(node.id()),
        tr(
            "L’SVG contiene riferimenti ricorsivi",
            "The SVG contains recursive references"
        )
    );
    let mut count = 1u64;
    for child in node.children().filter(|n| n.is_element()) {
        count = count.saturating_add(expanded_nodes(child, ids, active, memo, depth + 1)?);
        ensure!(count <= u64::from(MAX_NODES), limit_error());
    }
    if node.has_tag_name("use")
        && let Some(href) = node.attributes().find(|a| a.name() == "href")
        && let Some(target) = ids.get(href.value().trim().trim_start_matches('#'))
    {
        count = count.saturating_add(expanded_nodes(*target, ids, active, memo, depth + 1)?);
    }
    ensure!(count <= u64::from(MAX_NODES), limit_error());
    active.remove(&node.id());
    memo.insert(node.id(), count);
    Ok(count)
}

fn preflight<'a>(text: &'a str, cancel: &AtomicBool) -> Result<usvg::roxmltree::Document<'a>> {
    ensure!(
        !text.to_ascii_lowercase().contains("<!doctype"),
        reference_error()
    );
    let doc = usvg::roxmltree::Document::parse_with_options(
        text,
        usvg::roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: MAX_NODES,
            ..Default::default()
        },
    )
    .context(tr(
        "SVG XML non valido o non consentito",
        "Invalid or unsupported SVG XML",
    ))?;
    ensure!(
        doc.root_element().tag_name().name() == "svg",
        tr(
            "Il file non contiene un documento SVG",
            "The file does not contain an SVG document"
        )
    );
    let mut ids = HashMap::new();
    let mut path_bytes = 0usize;
    let mut text_bytes = 0usize;
    let face = ttf_parser::Face::parse(embedded_font(), 0).context("Embedded Ubuntu font")?;
    for node in doc.descendants() {
        cancelled(cancel)?;
        ensure!(
            node.ancestors().take(MAX_DEPTH + 2).count() <= MAX_DEPTH,
            limit_error()
        );
        if let Some(pi) = node.pi() {
            ensure!(pi.target != "xml-stylesheet", reference_error());
        }
        if !node.is_element() {
            continue;
        }
        ensure!(
            !matches!(
                node.tag_name().name(),
                "image" | "feImage" | "script" | "foreignObject"
            ),
            reference_error()
        );
        // These controls can multiply CPU work independently of image size.
        if node.has_tag_name("feTurbulence") {
            let octaves = node
                .attribute("numOctaves")
                .unwrap_or("1")
                .parse::<u32>()
                .unwrap_or(u32::MAX);
            ensure!(octaves <= 8, limit_error());
        }
        if node.has_tag_name("feConvolveMatrix") {
            let orders = node
                .attribute("order")
                .unwrap_or("3")
                .split([',', ' '])
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            ensure!(
                !orders.is_empty()
                    && orders.len() <= 2
                    && orders
                        .iter()
                        .all(|s| s.parse::<u32>().is_ok_and(|n| (1..=7).contains(&n))),
                limit_error()
            );
        }
        if node.has_tag_name("feMorphology") {
            ensure!(
                node.attribute("radius")
                    .unwrap_or("0")
                    .split([',', ' '])
                    .filter(|s| !s.is_empty())
                    .all(|s| s
                        .parse::<f32>()
                        .is_ok_and(|n| n.is_finite() && (0.0..=32.0).contains(&n))),
                limit_error()
            );
        }
        if let Some(id) = node.attribute("id") {
            ensure!(
                ids.insert(id, node).is_none(),
                tr(
                    "L’SVG contiene identificatori duplicati",
                    "The SVG contains duplicate identifiers"
                )
            );
        }
        for attribute in node.attributes() {
            if attribute.name() == "href" {
                let value = attribute.value().trim();
                ensure!(value.starts_with('#') && value.len() > 1, reference_error());
            }
            if attribute.name() == "base" {
                return Err(anyhow::anyhow!(reference_error()));
            }
            check_css(attribute.value())?;
            if matches!(attribute.name(), "d" | "points") {
                path_bytes = path_bytes.saturating_add(attribute.value().len());
            }
            if attribute.name().starts_with("on") {
                return Err(anyhow::anyhow!(reference_error()));
            }
        }
        if node.has_tag_name("style") {
            for child in node.children().filter(|n| n.is_text()) {
                check_css(child.text().unwrap_or_default())?;
            }
        }
        if node
            .ancestors()
            .any(|ancestor| ancestor.has_tag_name("text"))
        {
            for child in node.children().filter(|n| n.is_text()) {
                let value = child.text().unwrap_or_default();
                text_bytes = text_bytes.saturating_add(value.len());
                ensure!(
                    value
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .all(|c| face.glyph_index(c).is_some()),
                    tr(
                        "Il testo SVG contiene caratteri non disponibili nel font Ubuntu incorporato",
                        "SVG text contains characters unavailable in the embedded Ubuntu font"
                    )
                );
            }
        }
        ensure!(
            path_bytes <= MAX_PATH_BYTES && text_bytes <= MAX_TEXT_BYTES,
            limit_error()
        );
    }
    expanded_nodes(
        doc.root_element(),
        &ids,
        &mut HashSet::new(),
        &mut HashMap::new(),
        0,
    )?;
    Ok(doc)
}

struct Budget<'a> {
    bytes: u64,
    visits: u32,
    cancel: &'a AtomicBool,
}
impl Budget<'_> {
    fn add(&mut self, pixels: u64, bytes_per_pixel: u64) -> Result<()> {
        self.bytes = self
            .bytes
            .saturating_add(pixels.saturating_mul(bytes_per_pixel));
        ensure!(self.bytes <= MAX_WORK_BYTES, limit_error());
        Ok(())
    }
    fn visit(&mut self, depth: usize) -> Result<()> {
        cancelled(self.cancel)?;
        self.visits = self.visits.saturating_add(1);
        ensure!(
            depth <= MAX_DEPTH && self.visits <= MAX_NODES,
            limit_error()
        );
        Ok(())
    }
}

fn area(width: f32, height: f32, padding: f32) -> Result<u64> {
    let width = width.ceil() + padding;
    let height = height.ceil() + padding;
    ensure!(
        width.is_finite()
            && height.is_finite()
            && width > 0.0
            && height > 0.0
            && width <= MAX_SIDE as f32
            && height <= MAX_SIDE as f32,
        limit_error()
    );
    Ok((width as u64).saturating_mul(height as u64))
}

fn paint(
    paint: &usvg::Paint,
    ts: tiny_skia::Transform,
    budget: &mut Budget<'_>,
    depth: usize,
) -> Result<()> {
    if let usvg::Paint::Pattern(pattern) = paint {
        let (sx, sy) = ts.pre_concat(pattern.transform()).get_scale();
        let rect = pattern.rect();
        let pixels = area(
            (rect.width() * sx).round(),
            (rect.height() * sy).round(),
            0.0,
        )?;
        budget.add(pixels, 4)?;
        nodes(
            pattern.root(),
            tiny_skia::Transform::from_scale(sx, sy),
            pixels,
            budget,
            depth + 1,
        )?;
    }
    Ok(())
}

fn clip(
    clip: &usvg::ClipPath,
    ts: tiny_skia::Transform,
    pixels: u64,
    budget: &mut Budget<'_>,
    depth: usize,
) -> Result<()> {
    budget.visit(depth)?;
    budget.add(pixels, 12)?;
    nodes(
        clip.root(),
        ts.pre_concat(clip.transform()),
        pixels,
        budget,
        depth + 1,
    )?;
    if let Some(next) = clip.clip_path() {
        self::clip(next, ts, pixels, budget, depth + 1)?;
    }
    Ok(())
}

fn mask(
    mask: &usvg::Mask,
    ts: tiny_skia::Transform,
    pixels: u64,
    budget: &mut Budget<'_>,
    depth: usize,
) -> Result<()> {
    budget.visit(depth)?;
    budget.add(pixels, 12)?;
    nodes(mask.root(), ts, pixels, budget, depth + 1)?;
    if let Some(next) = mask.mask() {
        self::mask(next, ts, pixels, budget, depth + 1)?;
    }
    Ok(())
}

fn group(
    group: &usvg::Group,
    ts: tiny_skia::Transform,
    outer_pixels: u64,
    budget: &mut Budget<'_>,
    depth: usize,
) -> Result<()> {
    budget.visit(depth)?;
    let mut ts = ts.pre_concat(group.transform());
    let mut pixels = outer_pixels;
    if group.should_isolate() {
        // Mirror resvg's layer coordinate system before allocation. Ignoring
        // clipping overestimates rather than underestimates the required layer.
        if !group.filters().is_empty() {
            let sx = ts.sx.hypot(ts.ky);
            let sy = ts.kx.hypot(ts.sy);
            let rest = tiny_skia::Transform::from_row(
                ts.sx / sx,
                ts.ky / sx,
                ts.kx / sy,
                ts.sy / sy,
                0.0,
                0.0,
            );
            let inverse = rest.invert().context(limit_error())?;
            ts = inverse.pre_concat(ts);
        }
        let bbox = group
            .layer_bounding_box()
            .transform(ts)
            .context(limit_error())?;
        pixels = area(
            bbox.width(),
            bbox.height(),
            if group.filters().is_empty() { 4.0 } else { 0.0 },
        )?;
        budget.add(pixels, 4)?;
        for filter in group.filters() {
            ensure!(filter.primitives().len() <= 32, limit_error());
            // Includes retained intermediate filter images and floating-point
            // working channels, not just the final RGBA surface.
            budget.add(pixels, 128 * (filter.primitives().len() as u64 + 1))?;
        }
    }
    nodes(group, ts, pixels, budget, depth + 1)?;
    if let Some(path) = group.clip_path() {
        budget.add(outer_pixels, 8)?; // nested clip groups can use the whole parent
        clip(path, ts, pixels.max(outer_pixels), budget, depth + 1)?;
    }
    if let Some(value) = group.mask() {
        mask(value, ts, pixels, budget, depth + 1)?;
    }
    Ok(())
}

fn nodes(
    root: &usvg::Group,
    ts: tiny_skia::Transform,
    pixels: u64,
    budget: &mut Budget<'_>,
    depth: usize,
) -> Result<()> {
    for node in root.children() {
        budget.visit(depth)?;
        match node {
            usvg::Node::Group(value) => group(value, ts, pixels, budget, depth + 1)?,
            usvg::Node::Path(path) => {
                if let Some(fill) = path.fill() {
                    paint(fill.paint(), ts, budget, depth + 1)?;
                }
                if let Some(stroke) = path.stroke() {
                    paint(stroke.paint(), ts, budget, depth + 1)?;
                }
            }
            usvg::Node::Text(text) => group(text.flattened(), ts, pixels, budget, depth + 1)?,
            usvg::Node::Image(_) => anyhow::bail!(reference_error()),
        }
    }
    Ok(())
}

/// Rasterize a standalone UTF-8 SVG at its declared size (96 DPI for CSS units).
/// SVGZ, external resources, embedded images and document-provided fonts are refused.
pub fn render(input: &Path, cancel: &AtomicBool) -> Result<RgbaImage> {
    cancelled(cancel)?;
    let mut file = File::open(input).context(tr("SVG non leggibile", "Unable to read SVG"))?;
    ensure!(file.metadata()?.len() <= MAX_INPUT_BYTES, limit_error());
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= MAX_INPUT_BYTES, limit_error());
    let text = std::str::from_utf8(&bytes).context(tr(
        "Sono supportati solo SVG XML UTF-8 non compressi",
        "Only uncompressed UTF-8 SVG XML is supported",
    ))?;
    let doc = preflight(text, cancel)?;
    let mut fonts = usvg::fontdb::Database::new();
    fonts.load_font_data(embedded_font().to_vec());
    let family = fonts
        .faces()
        .next()
        .and_then(|face| face.families.first())
        .map(|family| family.0.clone())
        .context(tr(
            "Font Ubuntu incorporato non disponibile",
            "The embedded Ubuntu font is unavailable",
        ))?;
    fonts.set_sans_serif_family(family.clone());
    fonts.set_serif_family(family.clone());
    fonts.set_monospace_family(family.clone());
    let options = usvg::Options {
        resources_dir: None,
        font_family: family,
        fontdb: Arc::new(fonts),
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    let tree = usvg::Tree::from_xmltree(&doc, &options).context(tr(
        "SVG non valido o non renderizzabile",
        "Invalid or unrenderable SVG",
    ))?;
    cancelled(cancel)?;
    let width = tree.size().width().ceil();
    let height = tree.size().height().ceil();
    let pixels = area(width, height, 0.0)?;
    ensure!(pixels <= MAX_PIXELS, limit_error());
    let mut budget = Budget {
        bytes: 0,
        visits: 0,
        cancel,
    };
    budget.add(pixels, 8)?;
    nodes(
        tree.root(),
        tiny_skia::Transform::identity(),
        pixels,
        &mut budget,
        0,
    )?;
    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32).context(limit_error())?;
    cancelled(cancel)?;
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    cancelled(cancel)?;
    let output = RgbaImage::from_raw(width as u32, height as u32, pixmap.take_demultiplied())
        .context(limit_error())?;
    cancelled(cancel)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raster(source: &str) -> Result<RgbaImage> {
        let dir = tempfile::tempdir()?;
        let input = dir.path().join("test.svg");
        std::fs::write(&input, source)?;
        render(&input, &AtomicBool::new(false))
    }

    #[test]
    fn vector_paths_gradient_use_and_straight_alpha_are_rendered() {
        let pixels = raster(r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="48">
          <defs><linearGradient id="g"><stop stop-color="red"/><stop offset="1" stop-color="blue"/></linearGradient>
          <path id="p" d="M0 0h40v24H0z" fill="url(#g)"/></defs>
          <g transform="translate(8 4)"><use href="#p"/></g>
          <rect x="8" y="32" width="20" height="8" fill="red" opacity="0.5"/>
        </svg>"##).unwrap();
        assert_eq!(pixels.dimensions(), (64, 48));
        assert_eq!(pixels.get_pixel(0, 0).0, [0, 0, 0, 0]);
        let left = pixels.get_pixel(10, 12).0;
        let right = pixels.get_pixel(44, 12).0;
        assert!(left[0] > left[2] && right[2] > right[0]);
        let alpha = pixels.get_pixel(16, 36).0;
        assert_eq!(&alpha[..3], &[255, 0, 0]);
        assert!((126..=129).contains(&alpha[3]));
    }

    #[test]
    fn embedded_font_text_physical_units_and_bounded_filters_work() {
        let pixels = raster(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="2in" height="1in">
          <defs><filter id="f"><feGaussianBlur stdDeviation="0.5"/></filter></defs>
          <rect width="192" height="96" fill="white"/>
          <text x="4" y="42" font-size="24">Città €</text>
          <rect x="150" y="60" width="20" height="20" fill="blue" filter="url(#f)"/>
        </svg>"##,
        )
        .unwrap();
        assert_eq!(pixels.dimensions(), (192, 96));
        assert!(
            pixels
                .enumerate_pixels()
                .filter(|(x, y, p)| *x < 140 && *y < 50 && p[0] < 100 && p[3] > 200)
                .count()
                > 80
        );
        assert!(pixels.get_pixel(160, 70)[2] > 200);
    }

    #[test]
    fn document_references_do_not_contact_a_server() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let href = format!("http://{}/private.png", listener.local_addr().unwrap());
        for body in [
            format!(r#"<image href="{href}" width="10" height="10"/>"#),
            format!(r#"<use href="{href}"/>"#),
            format!(r#"<rect width="10" height="10" fill="url({href})"/>"#),
            format!(r#"<style>@import url({href});</style>"#),
            r#"<image href="data:image/png;base64,aGVsbG8="/>"#.into(),
            r#"<use href="file:///C:/private.svg#x"/>"#.into(),
        ] {
            assert!(raster(&format!(r#"<svg width="16" height="16">{body}</svg>"#)).is_err());
        }
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn malformed_entities_recursion_and_resource_bombs_are_refused() {
        let sources = [
            "<!DOCTYPE svg [<!ENTITY x SYSTEM 'file:///private'>]><svg width='10' height='10'>&x;</svg>".into(),
            "<svg width='100000' height='100000'><rect width='1' height='1'/></svg>".into(),
            "<svg width='10' height='10'><defs><g id='x'><use href='#x'/></g></defs><use href='#x'/></svg>".into(),
            "<svg width='10' height='10'><defs><pattern id='p' width='1e9' height='1e9' patternUnits='userSpaceOnUse'><rect width='1' height='1'/></pattern></defs><rect width='10' height='10' fill='url(#p)'/></svg>".into(),
            "<svg width='4000' height='4000'><defs><filter id='f'><feGaussianBlur stdDeviation='10'/></filter></defs><rect width='4000' height='4000' filter='url(#f)'/></svg>".into(),
            format!("<svg width='10' height='10'>{}<rect width='1' height='1'/>{}</svg>", "<g>".repeat(70), "</g>".repeat(70)),
        ];
        for source in sources {
            assert!(raster(&source).is_err(), "Unsafe SVG was accepted");
        }
    }

    #[test]
    fn use_expansion_limit_is_checked_before_usvg() {
        let mut source = String::from(
            "<svg width='16' height='16'><defs><g id='n0'><rect width='1' height='1'/></g>",
        );
        for index in 1..20 {
            source.push_str(&format!(
                "<g id='n{index}'><use href='#n{}'/><use href='#n{}'/></g>",
                index - 1,
                index - 1
            ));
        }
        source.push_str("</defs><use href='#n19'/></svg>");
        assert!(raster(&source).is_err());
    }

    #[test]
    fn cancellation_and_input_bound_precede_rendering() {
        let cancel = AtomicBool::new(true);
        let error = render(Path::new("not-existing.svg"), &cancel)
            .unwrap_err()
            .to_string()
            .to_lowercase();
        assert!(error.contains("cancel") || error.contains("annull"));
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("large.svg");
        let file = File::create(&input).unwrap();
        file.set_len(MAX_INPUT_BYTES + 1).unwrap();
        assert!(render(&input, &AtomicBool::new(false)).is_err());
    }
}
