//! Shared Fluent-inspired visual tokens and native Windows typography.
use eframe::egui::{self, Color32, FontDefinitions, FontFamily, FontId, RichText, Stroke};
use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

pub const CAPTION: f32 = 12.0;
pub const BODY: f32 = 14.0;
pub const SUBTITLE: f32 = 20.0;
pub const TITLE: f32 = 28.0;
pub const CONTROL_RADIUS: u8 = 4;
pub const CARD_RADIUS: u8 = 8;
pub const SEMIBOLD_FAMILY: &str = "Segoe UI Semibold";
pub const ICON_FAMILY: &str = "Segoe Fluent Icons";

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub canvas: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub stroke: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub success: Color32,
    pub error: Color32,
}

impl Palette {
    pub fn for_dark_mode(dark: bool) -> Self {
        if dark {
            Self {
                canvas: Color32::from_rgb(32, 32, 32),
                card: Color32::from_rgb(43, 43, 43),
                card_hover: Color32::from_rgb(51, 51, 51),
                text: Color32::from_rgb(245, 245, 245),
                secondary: Color32::from_rgb(206, 206, 206),
                stroke: Color32::from_rgb(70, 70, 70),
                accent: Color32::from_rgb(96, 205, 255),
                accent_text: Color32::from_rgb(0, 38, 53),
                success: Color32::from_rgb(108, 203, 95),
                error: Color32::from_rgb(255, 153, 164),
            }
        } else {
            Self {
                canvas: Color32::from_rgb(243, 243, 243),
                card: Color32::WHITE,
                card_hover: Color32::from_rgb(249, 249, 249),
                text: Color32::from_rgb(26, 26, 26),
                secondary: Color32::from_rgb(92, 92, 92),
                stroke: Color32::from_rgb(222, 222, 222),
                accent: Color32::from_rgb(0, 95, 184),
                accent_text: Color32::WHITE,
                success: Color32::from_rgb(15, 123, 15),
                error: Color32::from_rgb(196, 43, 28),
            }
        }
    }
}

pub fn palette(ctx: &egui::Context) -> Palette {
    Palette::for_dark_mode(ctx.style().visuals.dark_mode)
}

/// Keep native window chrome in step with egui. Retain SystemDefault while the
/// app follows Windows: forcing a light/dark override would suppress winit's
/// subsequent Windows theme-change notifications.
pub fn sync_window_theme(ctx: &egui::Context) {
    let dark = ctx.style().visuals.dark_mode;
    let native = match ctx.options(|options| options.theme_preference) {
        egui::ThemePreference::System => egui::SystemTheme::SystemDefault,
        egui::ThemePreference::Light => egui::SystemTheme::Light,
        egui::ThemePreference::Dark => egui::SystemTheme::Dark,
    };
    let id = egui::Id::new(("native-window-theme", ctx.viewport_id()));
    let changed = ctx.data_mut(|data| {
        let value = (dark, native);
        if data.get_temp::<(bool, egui::SystemTheme)>(id) == Some(value) {
            false
        } else {
            data.insert_temp(id, value);
            true
        }
    });
    if changed {
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(native));
    }
}

/// Configure both styles so a Windows light/dark change does not restore the
/// default egui appearance. Call once for each newly created egui context.
pub fn configure(ctx: &egui::Context) {
    static FONTS: OnceLock<FontDefinitions> = OnceLock::new();
    ctx.set_fonts(FONTS.get_or_init(windows_fonts).clone());
    ctx.set_theme(egui::ThemePreference::System);
    for (theme, dark) in [(egui::Theme::Light, false), (egui::Theme::Dark, true)] {
        let mut style = (*ctx.style_of(theme)).clone();
        let colors = Palette::for_dark_mode(dark);
        style.spacing.item_spacing = egui::vec2(12.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.interact_size = egui::vec2(40.0, 32.0);
        style.spacing.window_margin = egui::Margin::same(20);
        style.spacing.menu_margin = egui::Margin::same(8);
        style.spacing.combo_width = 180.0;
        style.animation_time = 0.12;
        style
            .text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(CAPTION));
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(BODY));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, FontId::monospace(BODY));
        style
            .text_styles
            .insert(egui::TextStyle::Button, semibold_font(BODY));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, semibold_font(SUBTITLE));
        let visuals = &mut style.visuals;
        visuals.dark_mode = dark;
        visuals.override_text_color = None;
        visuals.weak_text_color = Some(colors.secondary);
        visuals.panel_fill = colors.canvas;
        visuals.window_fill = colors.card;
        visuals.window_stroke = Stroke::new(1.0_f32, colors.stroke);
        visuals.window_corner_radius = egui::CornerRadius::same(CARD_RADIUS);
        visuals.menu_corner_radius = egui::CornerRadius::same(CARD_RADIUS);
        visuals.extreme_bg_color = colors.card;
        visuals.text_edit_bg_color = Some(colors.card);
        visuals.faint_bg_color = colors.card_hover;
        visuals.code_bg_color = colors.card_hover;
        visuals.hyperlink_color = colors.accent;
        visuals.error_fg_color = colors.error;
        visuals.warn_fg_color = if dark {
            Color32::from_rgb(252, 225, 0)
        } else {
            Color32::from_rgb(157, 93, 0)
        };
        visuals.selection.bg_fill = colors.accent;
        visuals.selection.stroke = Stroke::new(1.0_f32, colors.accent_text);
        visuals.slider_trailing_fill = true;
        visuals.interact_cursor = None;
        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 3],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(if dark { 70 } else { 22 }),
        };
        visuals.popup_shadow = visuals.window_shadow;
        let widgets = &mut visuals.widgets;
        for visual in [
            &mut widgets.noninteractive,
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            visual.corner_radius = egui::CornerRadius::same(CONTROL_RADIUS);
            visual.fg_stroke = Stroke::new(1.0_f32, colors.text);
            visual.bg_stroke = Stroke::new(1.0_f32, colors.stroke);
            visual.expansion = 0.0;
        }
        widgets.noninteractive.bg_fill = colors.card;
        widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.inactive.bg_fill = colors.card;
        widgets.inactive.weak_bg_fill = colors.card;
        widgets.hovered.bg_fill = colors.card_hover;
        widgets.hovered.weak_bg_fill = colors.card_hover;
        widgets.hovered.bg_stroke = Stroke::new(1.0_f32, colors.secondary);
        widgets.active.bg_fill = if dark {
            Color32::from_rgb(58, 58, 58)
        } else {
            Color32::from_rgb(238, 238, 238)
        };
        widgets.active.weak_bg_fill = widgets.active.bg_fill;
        widgets.active.bg_stroke = Stroke::new(1.0_f32, colors.accent);
        widgets.open = widgets.active;
        ctx.set_style_of(theme, style);
    }
}

pub fn semibold_font(size: f32) -> FontId {
    FontId::new(size.max(CAPTION), FontFamily::Name(SEMIBOLD_FAMILY.into()))
}

pub fn heading(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text).font(semibold_font(size))
}

#[derive(Clone, Copy, Debug)]
pub enum FluentIcon {
    Add,
    Folder,
    Settings,
    Convert,
    Delete,
    Cancel,
    Check,
    ChevronUp,
    ChevronDown,
    Back,
    Document,
    Info,
    Audio,
    Video,
    Image,
    Raw,
    Profiles,
    Edit,
    Import,
    Export,
    Copy,
    Download,
    Refresh,
    ExternalLink,
    More,
    History,
}

impl FluentIcon {
    pub fn glyph(self) -> char {
        match self {
            Self::Add => '\u{E710}',
            Self::Folder => '\u{E8B7}',
            Self::Settings => '\u{E713}',
            Self::Convert => '\u{E8AB}',
            Self::Delete => '\u{E74D}',
            Self::Cancel => '\u{E711}',
            Self::Check => '\u{E73E}',
            Self::ChevronUp => '\u{E70E}',
            Self::ChevronDown => '\u{E70D}',
            Self::Back => '\u{E72B}',
            Self::Document => '\u{E8A5}',
            Self::Info => '\u{E946}',
            Self::Audio => '\u{E8D6}',
            Self::Video => '\u{E714}',
            Self::Image => '\u{E8B9}',
            Self::Raw => '\u{E943}',
            Self::Profiles => '\u{E8F1}',
            Self::Edit => '\u{E70F}',
            Self::Import => '\u{E8B5}',
            Self::Export => '\u{EDE1}',
            Self::Copy => '\u{E8C8}',
            Self::Download => '\u{E896}',
            Self::Refresh => '\u{E72C}',
            Self::ExternalLink => '\u{E8A7}',
            Self::More => '\u{E712}',
            Self::History => '\u{E81C}',
        }
    }
}

/// UI groupings; these never narrow the engine's accepted input extensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatFamily {
    Audio,
    Video,
    Images,
    Documents,
    Raw,
}

impl FormatFamily {
    pub const ALL: [Self; 5] = [
        Self::Audio,
        Self::Video,
        Self::Images,
        Self::Documents,
        Self::Raw,
    ];

    pub fn label(self) -> &'static str {
        use the_converter::i18n::tr;
        match self {
            Self::Audio => "Audio",
            Self::Video => "Video",
            Self::Images => tr("Immagini", "Images"),
            Self::Documents => tr("Documenti", "Documents"),
            Self::Raw => "Raw",
        }
    }

    pub fn icon(self) -> FluentIcon {
        match self {
            Self::Audio => FluentIcon::Audio,
            Self::Video => FluentIcon::Video,
            Self::Images => FluentIcon::Image,
            Self::Documents => FluentIcon::Document,
            Self::Raw => FluentIcon::Raw,
        }
    }

    pub fn color(self, ctx: &egui::Context) -> Color32 {
        let dark = ctx.style().visuals.dark_mode;
        match (self, dark) {
            (Self::Audio, false) => Color32::from_rgb(121, 65, 186),
            (Self::Video, false) => Color32::from_rgb(196, 55, 110),
            (Self::Images, false) => Color32::from_rgb(16, 124, 95),
            (Self::Documents, false) => Color32::from_rgb(0, 95, 184),
            (Self::Raw, false) => Color32::from_rgb(162, 87, 21),
            (Self::Audio, true) => Color32::from_rgb(192, 156, 245),
            (Self::Video, true) => Color32::from_rgb(246, 156, 198),
            (Self::Images, true) => Color32::from_rgb(114, 218, 173),
            (Self::Documents, true) => Color32::from_rgb(96, 205, 255),
            (Self::Raw, true) => Color32::from_rgb(245, 190, 129),
        }
    }

    pub fn of_output(format: the_converter::model::OutputFormat) -> Self {
        use the_converter::model::{MediaKind, OutputFormat};
        match format {
            OutputFormat::Txt | OutputFormat::Html | OutputFormat::Csv => Self::Raw,
            _ => match format.kind() {
                MediaKind::Audio => Self::Audio,
                MediaKind::Video => Self::Video,
                MediaKind::Image => Self::Images,
                _ => Self::Documents,
            },
        }
    }

    pub fn of_input_extension(extension: &str) -> Self {
        use the_converter::model::MediaKind;
        let extension = extension.trim_start_matches('.').to_ascii_lowercase();
        if matches!(extension.as_str(), "html" | "htm" | "txt" | "csv") {
            return Self::Raw;
        }
        match MediaKind::from_path(std::path::Path::new(&format!("input.{extension}"))) {
            MediaKind::Audio => Self::Audio,
            MediaKind::Video => Self::Video,
            MediaKind::Image => Self::Images,
            _ => Self::Documents,
        }
    }
}

pub fn icon(icon: FluentIcon, size: f32) -> RichText {
    RichText::new(icon.glyph().to_string()).font(FontId::new(
        size.max(CAPTION),
        FontFamily::Name(ICON_FAMILY.into()),
    ))
}

/// Keep Fluent glyphs and Semibold labels in their actual font families when
/// both appear inside the same button; synthetic bold does not change weight.
pub fn button_text(icon: FluentIcon, label: &str, color: Color32) -> egui::WidgetText {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &icon.glyph().to_string(),
        0.0,
        egui::TextFormat {
            font_id: FontId::new(16.0, FontFamily::Name(ICON_FAMILY.into())),
            color,
            ..Default::default()
        },
    );
    job.append(
        label,
        8.0,
        egui::TextFormat {
            font_id: semibold_font(BODY),
            color,
            ..Default::default()
        },
    );
    job.into()
}

pub fn card(ctx: &egui::Context) -> egui::Frame {
    let colors = palette(ctx);
    egui::Frame::new()
        .fill(colors.card)
        .stroke(Stroke::new(1.0_f32, colors.stroke))
        .corner_radius(CARD_RADIUS)
        .inner_margin(16)
}

/// Cards occupy the entire column even when their contents are short or folded.
pub fn show_card<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let mut prepared = card(ui.ctx()).begin(ui);
    let width = prepared.content_ui.available_width();
    prepared.content_ui.set_width(width);
    let inner = contents(&mut prepared.content_ui);
    let response = prepared.end(ui);
    egui::InnerResponse::new(inner, response)
}

/// A small transparent version of the app's opposing-arrow mark for headers.
pub fn paint_logo(painter: &egui::Painter, rect: egui::Rect, color: Color32) {
    let point =
        |x: f32, y: f32| rect.min + egui::vec2(x / 256.0 * rect.width(), y / 256.0 * rect.height());
    let stroke = Stroke::new((rect.width() * 20.0 / 256.0).max(2.0), color);
    for coordinates in [
        [(40.0, 84.0), (96.0, 76.0), (160.0, 76.0), (216.0, 84.0)],
        [(216.0, 172.0), (160.0, 180.0), (96.0, 180.0), (40.0, 172.0)],
    ] {
        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
            coordinates.map(|(x, y)| point(x, y)),
            false,
            Color32::TRANSPARENT,
            stroke,
        ));
    }
    for (start, end) in [
        ((184.0, 52.0), (216.0, 84.0)),
        ((216.0, 84.0), (184.0, 116.0)),
        ((72.0, 140.0), (40.0, 172.0)),
        ((40.0, 172.0), (72.0, 204.0)),
    ] {
        painter.line_segment([point(start.0, start.1), point(end.0, end.1)], stroke);
    }
}

fn windows_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let root = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .map(|root| root.join("Fonts"));
    for (name, candidates) in [
        ("Segoe UI", &["segoeui.ttf"][..]),
        (
            SEMIBOLD_FAMILY,
            &["segoeuisb.ttf", "seguisb.ttf", "segoeuib.ttf"][..],
        ),
        (ICON_FAMILY, &["SegoeIcons.ttf", "segmdl2.ttf"][..]),
    ] {
        let loaded = root.as_ref().and_then(|root| {
            candidates
                .iter()
                .find_map(|file| std::fs::read(root.join(file)).ok())
        });
        let mut family = Vec::new();
        if let Some(bytes) = loaded {
            fonts
                .font_data
                .insert(name.into(), Arc::new(egui::FontData::from_owned(bytes)));
            family.push(name.to_owned());
        }
        family.extend(fallback.iter().cloned());
        if name == "Segoe UI" {
            fonts.families.insert(FontFamily::Proportional, family);
        } else {
            fonts.families.insert(FontFamily::Name(name.into()), family);
        }
    }
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_theme_keeps_both_typography_and_readable_secondary_colors() {
        let ctx = egui::Context::default();
        configure(&ctx);
        assert_eq!(
            ctx.options(|options| options.theme_preference),
            egui::ThemePreference::System
        );
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let style = ctx.style_of(theme);
            assert!(style.text_styles.values().all(|font| font.size >= 12.0));
            assert_eq!(
                style.text_styles[&egui::TextStyle::Heading].family,
                FontFamily::Name(SEMIBOLD_FAMILY.into())
            );
            assert_ne!(style.visuals.weak_text_color(), style.visuals.panel_fill);
        }
        assert_eq!(
            Palette::for_dark_mode(false).secondary,
            Color32::from_rgb(92, 92, 92)
        );
    }

    #[test]
    fn semibold_and_icon_families_have_real_font_fallbacks() {
        let fonts = windows_fonts();
        for family in [SEMIBOLD_FAMILY, ICON_FAMILY] {
            let names = &fonts.families[&FontFamily::Name(family.into())];
            assert!(!names.is_empty());
            assert!(names.iter().all(|name| fonts.font_data.contains_key(name)));
        }
    }

    #[test]
    fn short_and_collapsed_cards_fill_the_same_column_at_supported_window_widths() {
        for width in [860.0, 1060.0] {
            let ctx = egui::Context::default();
            configure(&ctx);
            let mut widths = Vec::new();
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 760.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new().inner_margin(24))
                        .show(ctx, |ui| {
                            widths.push(
                                show_card(ui, |ui| {
                                    ui.label("Aspetto");
                                })
                                .response
                                .rect
                                .width(),
                            );
                            widths.push(
                                show_card(ui, |ui| {
                                    egui::CollapsingHeader::new("Impostazioni personalizzate")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            ui.label("Contenuto");
                                        });
                                })
                                .response
                                .rect
                                .width(),
                            );
                            widths.push(
                                show_card(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut String::new())
                                            .desired_width(f32::INFINITY),
                                    );
                                })
                                .response
                                .rect
                                .width(),
                            );
                        });
                },
            );
            assert!(
                widths
                    .iter()
                    .all(|card| (*card - (width - 48.0)).abs() < 1.0),
                "window {width}: {widths:?}"
            );
        }
    }
}
