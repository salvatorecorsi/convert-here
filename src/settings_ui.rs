use crate::theme::{self, FluentIcon, FormatFamily};
use eframe::egui;
use std::{collections::BTreeMap, path::PathBuf};
use the_converter::{
    i18n::tr,
    model::OutputFormat,
    output_preferences::OutputPreferences,
    runtime_install::RuntimeEngine,
    tools::{ToolPaths, executable_available},
};

pub(crate) enum EngineAction {
    Setup,
    Install(Vec<RuntimeEngine>),
    Select(&'static str),
    Reset(&'static [&'static str]),
    Website(&'static str),
    Detect,
    OpenFolder,
}

pub(crate) fn formats(ui: &mut egui::Ui, enabled: &mut OutputPreferences) {
    ui.horizontal(|ui| {
        ui.label(theme::heading(
            tr("Formati di uscita", "Output formats"),
            theme::SUBTITLE,
        ));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(tr("Più comuni", "Most common")).clicked() {
                enabled.reset_common();
            }
        });
    });
    ui.add_space(8.0);
    let width = ((ui.available_width() - 4.0 * ui.spacing().item_spacing.x) / 5.0).max(130.0);
    egui::ScrollArea::horizontal()
        .id_salt("format_columns_scroll")
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for family in FormatFamily::ALL {
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(width);
                            theme::show_card(ui, |ui| {
                                ui.spacing_mut().item_spacing.y = 4.0;
                                ui.spacing_mut().interact_size.y = 26.0;
                                ui.horizontal(|ui| {
                                    ui.label(
                                        theme::icon(family.icon(), 22.0)
                                            .color(family.color(ui.ctx())),
                                    );
                                    ui.label(theme::heading(family.label(), theme::BODY));
                                });
                                let choices: Vec<_> = OutputFormat::ALL
                                    .into_iter()
                                    .filter(|format| FormatFamily::of_output(*format) == family)
                                    .collect();
                                ui.separator();
                                for &format in &choices {
                                    let mut checked = enabled.contains(format);
                                    if ui
                                        .checkbox(
                                            &mut checked,
                                            format.extension().to_ascii_uppercase(),
                                        )
                                        .on_hover_text(format.label())
                                        .changed()
                                    {
                                        enabled.set_enabled(format, checked);
                                    }
                                }
                                ui.separator();
                                let mut all_selected =
                                    choices.iter().all(|format| enabled.contains(*format));
                                if ui
                                    .checkbox(
                                        &mut all_selected,
                                        tr("Seleziona tutti", "Select all"),
                                    )
                                    .changed()
                                {
                                    for format in choices {
                                        enabled.set_enabled(format, all_selected);
                                    }
                                }
                            });
                        },
                    );
                }
            });
        });
}

struct Engine {
    name: &'static str,
    icon: FluentIcon,
    keys: &'static [&'static str],
    install: Option<RuntimeEngine>,
    website: &'static str,
}

pub(crate) fn engines(
    ui: &mut egui::Ui,
    tools: &ToolPaths,
    overrides: &BTreeMap<String, PathBuf>,
    installing: bool,
) -> Option<EngineAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(theme::heading(tr("Motori", "Engines"), theme::SUBTITLE));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if tool_button(
                ui,
                FluentIcon::Refresh,
                tr("Rileva strumenti", "Detect tools"),
                true,
            )
            .clicked()
            {
                action = Some(EngineAction::Detect);
            }
            if tool_button(
                ui,
                FluentIcon::Folder,
                tr("Cartella strumenti", "Tools folder"),
                true,
            )
            .clicked()
            {
                action = Some(EngineAction::OpenFolder);
            }
            if tool_button(
                ui,
                FluentIcon::Download,
                tr("Configurazione guidata", "Setup wizard"),
                !installing,
            )
            .clicked()
            {
                action = Some(EngineAction::Setup);
            }
        });
    });
    ui.add_space(8.0);
    let engines = [
        Engine {
            name: "FFmpeg",
            icon: FluentIcon::Video,
            keys: &["ffmpeg", "ffprobe"],
            install: Some(RuntimeEngine::Ffmpeg),
            website: "https://ffmpeg.org/download.html",
        },
        Engine {
            name: "Poppler",
            icon: FluentIcon::Document,
            keys: &["pdftoppm", "pdftotext", "pdfinfo"],
            install: Some(RuntimeEngine::Poppler),
            website: "https://poppler.freedesktop.org/",
        },
        Engine {
            name: "LibreOffice",
            icon: FluentIcon::Document,
            keys: &["soffice"],
            install: None,
            website: "https://www.libreoffice.org/download/download-libreoffice/",
        },
        Engine {
            name: "Tesseract",
            icon: FluentIcon::Raw,
            keys: &["tesseract"],
            install: Some(RuntimeEngine::Tesseract),
            website: "https://tesseract-ocr.github.io/tessdoc/Installation.html",
        },
        Engine {
            name: "libheif",
            icon: FluentIcon::Image,
            keys: &["heif-dec", "heif-enc"],
            install: Some(RuntimeEngine::Libheif),
            website: "https://github.com/strukturag/libheif",
        },
        Engine {
            name: "Whisper · Tiny",
            icon: FluentIcon::Audio,
            keys: &["whisper-cli", "whisper-model"],
            install: Some(RuntimeEngine::Whisper),
            website: "https://github.com/ggml-org/whisper.cpp",
        },
    ];
    let columns = if ui.available_width() >= 850.0 {
        3
    } else if ui.available_width() >= 530.0 {
        2
    } else {
        1
    };
    let width = (ui.available_width() - (columns - 1) as f32 * 12.0) / columns as f32;
    for row in engines.chunks(columns) {
        ui.horizontal_top(|ui| {
            for engine in row {
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 128.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(width);
                        theme::show_card(ui, |ui| {
                            ui.set_min_height(96.0);
                            let primary = engine.keys[0];
                            let path = overrides
                                .get(primary)
                                .cloned()
                                .or_else(|| tool_path(tools, primary));
                            let valid =
                                path.as_ref().is_some_and(|path| executable_available(path))
                                    && (primary != "whisper-cli"
                                        || tools
                                            .whisper_model
                                            .as_deref()
                                            .is_some_and(the_converter::tools::model_available));
                            ui.horizontal(|ui| {
                                ui.label(
                                    theme::icon(engine.icon, 24.0)
                                        .color(theme::palette(ui.ctx()).accent),
                                );
                                ui.label(theme::heading(engine.name, theme::BODY));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            theme::icon(
                                                if valid {
                                                    FluentIcon::Check
                                                } else {
                                                    FluentIcon::Download
                                                },
                                                16.0,
                                            )
                                            .color(
                                                if valid {
                                                    theme::palette(ui.ctx()).success
                                                } else {
                                                    theme::palette(ui.ctx()).secondary
                                                },
                                            ),
                                        )
                                        .on_hover_text(
                                            path.as_ref().map_or_else(
                                                || {
                                                    tr("Da configurare", "Not configured")
                                                        .to_owned()
                                                },
                                                |path| path.display().to_string(),
                                            ),
                                        );
                                    },
                                );
                            });
                            ui.add_space(14.0);
                            let button_width = ((ui.available_width() - 24.0) / 3.0).max(50.0);
                            ui.horizontal(|ui| {
                                if ui
                                    .add_enabled(
                                        !installing,
                                        egui::Button::new(theme::button_text(
                                            FluentIcon::Download,
                                            tr("Installa", "Install"),
                                            theme::palette(ui.ctx()).text,
                                        ))
                                        .min_size(egui::vec2(button_width, 32.0)),
                                    )
                                    .clicked()
                                {
                                    action = Some(match engine.install {
                                        Some(RuntimeEngine::Whisper)
                                            if tools.ffmpeg.is_none()
                                                || tools.ffprobe.is_none() =>
                                        {
                                            EngineAction::Install(vec![
                                                RuntimeEngine::Ffmpeg,
                                                RuntimeEngine::Whisper,
                                            ])
                                        }
                                        Some(install) => EngineAction::Install(vec![install]),
                                        None => EngineAction::Website(engine.website),
                                    });
                                }
                                if ui
                                    .add(
                                        egui::Button::new(theme::button_text(
                                            FluentIcon::Folder,
                                            tr("Scegli", "Choose"),
                                            theme::palette(ui.ctx()).text,
                                        ))
                                        .min_size(egui::vec2(button_width, 32.0)),
                                    )
                                    .clicked()
                                {
                                    action = Some(EngineAction::Select(primary));
                                }
                                ui.menu_button(theme::icon(FluentIcon::More, 16.0), |ui| {
                                    for key in engine.keys {
                                        if ui
                                            .button(if *key == "whisper-model" {
                                                tr(
                                                    "Modello Whisper (.bin)…",
                                                    "Whisper model (.bin)…",
                                                )
                                                .into()
                                            } else {
                                                format!("{key}.exe…")
                                            })
                                            .clicked()
                                        {
                                            action = Some(EngineAction::Select(key));
                                            ui.close();
                                        }
                                    }
                                    ui.separator();
                                    if ui
                                        .button(tr("Rilevamento automatico", "Automatic detection"))
                                        .clicked()
                                    {
                                        action = Some(EngineAction::Reset(engine.keys));
                                        ui.close();
                                    }
                                    if ui
                                        .button(tr("Sito ufficiale", "Official website"))
                                        .clicked()
                                    {
                                        action = Some(EngineAction::Website(engine.website));
                                        ui.close();
                                    }
                                })
                                .response
                                .on_hover_text(tr("Altre opzioni", "More options"));
                            });
                        });
                    },
                );
            }
        });
        ui.add_space(8.0);
    }
    action
}

fn tool_path(tools: &ToolPaths, key: &str) -> Option<PathBuf> {
    match key {
        "ffmpeg" => tools.ffmpeg.clone(),
        "ffprobe" => tools.ffprobe.clone(),
        "pdftoppm" => tools.pdftoppm.clone(),
        "pdftotext" => tools.pdftotext.clone(),
        "pdfinfo" => tools.pdfinfo.clone(),
        "soffice" => tools.soffice.clone(),
        "tesseract" => tools.tesseract.clone(),
        "heif-enc" => tools.heif_enc.clone(),
        "heif-dec" => tools.heif_dec.clone(),
        "whisper-cli" => tools.whisper_cli.clone(),
        "whisper-model" => tools.whisper_model.clone(),
        _ => None,
    }
}

pub(crate) fn speech_controls(
    ui: &mut egui::Ui,
    settings: &mut the_converter::model::SpeechSettings,
) {
    ui.label(theme::heading(
        tr("Trascrizione", "Transcription"),
        theme::BODY,
    ));
    egui::ComboBox::from_id_salt("speech_language")
        .selected_text(match settings.language.as_str() {
            "auto" => tr("Lingua automatica", "Automatic language"),
            "it" => tr("Italiano", "Italian"),
            "en" => tr("Inglese", "English"),
            code => code,
        })
        .show_ui(ui, |ui| {
            for (code, label) in [
                ("auto", tr("Lingua automatica", "Automatic language")),
                ("it", tr("Italiano", "Italian")),
                ("en", tr("Inglese", "English")),
                ("fr", tr("Francese", "French")),
                ("de", tr("Tedesco", "German")),
                ("es", tr("Spagnolo", "Spanish")),
            ] {
                ui.selectable_value(&mut settings.language, code.into(), label);
            }
        });
    egui::CollapsingHeader::new(tr("Avanzate", "Advanced"))
        .id_salt("speech_advanced")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr("Codice lingua", "Language code"));
                ui.add(
                    egui::TextEdit::singleline(&mut settings.language)
                        .char_limit(4)
                        .desired_width(64.0),
                );
            });
            ui.add(
                egui::DragValue::new(&mut settings.threads)
                    .range(0..=32)
                    .prefix(tr("Thread (0 = auto) ", "Threads (0 = auto) ")),
            );
        });
}

pub(crate) fn tool_button(
    ui: &mut egui::Ui,
    icon: FluentIcon,
    tooltip: &str,
    enabled: bool,
) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(theme::icon(icon, 16.0)).min_size(egui::vec2(32.0, 32.0)),
    )
    .on_hover_text(tooltip)
    .on_disabled_hover_text(tooltip)
}
