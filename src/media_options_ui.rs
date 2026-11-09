//! Typed FFmpeg options; the core owns validation, supported flags and probing.
use crate::theme::{self, FluentIcon};
use eframe::egui;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use the_converter::{
    i18n::tr,
    media_options::{self, EncoderOption, OptionKind, OptionSpec},
    model::{ConversionSettings, MediaKind},
    tools::ToolPaths,
    trf,
};

#[derive(Clone, PartialEq, Eq)]
struct DiscoveryKey {
    executable: Option<PathBuf>,
    encoder: String,
}

struct PendingDiscovery {
    key: DiscoveryKey,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Result<Vec<EncoderOption>, String>>,
}

#[derive(Default)]
pub(crate) struct MediaOptionsUi {
    search: String,
    audio_stream: bool,
    pending: Option<PendingDiscovery>,
    discovered: Option<(DiscoveryKey, Vec<EncoderOption>)>,
    current: Option<DiscoveryKey>,
    error: Option<String>,
}

impl Drop for MediaOptionsUi {
    fn drop(&mut self) {
        if let Some(pending) = &self.pending {
            pending.cancel.store(true, Ordering::Relaxed);
        }
    }
}

impl MediaOptionsUi {
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut ConversionSettings,
        tools: &ToolPaths,
    ) {
        ui.push_id("typed_media_options", |ui| {
            ui.label(tr(
                "Attiva un’opzione per sostituire il valore automatico.",
                "Enable an option to override its automatic value.",
            ));
            ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text(tr("Cerca opzione", "Search options"))
                    .desired_width(f32::INFINITY),
            );
            if !settings.media_options.is_empty()
                && ui
                    .button(theme::button_text(
                        FluentIcon::Refresh,
                        tr("Ripristina opzioni avanzate", "Reset advanced options"),
                        theme::palette(ui.ctx()).text,
                    ))
                    .clicked()
            {
                settings.media_options.clear();
            }
            let specs = media_options::options_for(settings.format);
            let query = self.search.trim().to_lowercase();
            for group in [
                "codec",
                "video",
                "audio",
                "filters",
                "metadata",
                "performance",
            ] {
                let visible: Vec<_> = specs
                    .iter()
                    .copied()
                    .filter(|spec| {
                        spec.group == group
                            && applicable_spec(spec, settings)
                            && matches_search(
                                &query,
                                &[spec.key, spec.label_it, spec.label_en, group_label(group)],
                            )
                    })
                    .collect();
                if visible.is_empty() {
                    continue;
                }
                egui::CollapsingHeader::new(group_label(group))
                    .id_salt(("media_group", group))
                    .default_open(group == "codec" || !query.is_empty())
                    .open((!query.is_empty()).then_some(true))
                    .show(ui, |ui| {
                        for spec in visible {
                            let encoder = media_options::encoder_for(settings);
                            let specific =
                                self.discovered.as_ref().and_then(|(loaded, options)| {
                                    if loaded.executable == tools.ffmpeg
                                        && active_encoders(settings).contains(&loaded.encoder)
                                    {
                                        options.iter().find(|option| option.key == spec.key)
                                    } else {
                                        None
                                    }
                                });
                            let kind = match spec.key {
                                "video_encoder" => OptionKind::Choice(
                                    media_options::video_encoder_choices(settings.format),
                                ),
                                "audio_encoder" => OptionKind::Choice(
                                    media_options::audio_encoder_choices(settings.format),
                                ),
                                key if scoped_key(key) => {
                                    match media_options::encoder_specific_choices(key, &encoder) {
                                        Some(choices) if !choices.is_empty() => {
                                            OptionKind::Choice(choices)
                                        }
                                        _ => specific.map_or(spec.kind, |option| option.kind),
                                    }
                                }
                                _ => specific.map_or(spec.kind, |option| option.kind),
                            };
                            option_row(
                                ui,
                                spec.key,
                                tr(spec.label_it, spec.label_en),
                                &kind,
                                specific.map_or(&[], |option| option.choices.as_slice()),
                                specific.and_then(|option| option.default.as_deref()),
                                &mut settings.media_options,
                            );
                        }
                    });
            }
            ui.add_space(8.0);
            self.codec_options(ui, settings, tools, &query);
            self.active_codec_options(ui, settings, &query);
            let unsupported: Vec<_> = settings
                .media_options
                .keys()
                .filter(|key| {
                    !key.starts_with("codec:")
                        && !specs
                            .iter()
                            .any(|spec| spec.key == key.as_str() && applicable_spec(spec, settings))
                })
                .cloned()
                .collect();
            if !unsupported.is_empty() {
                ui.add_space(8.0);
                ui.colored_label(
                    theme::palette(ui.ctx()).error,
                    tr(
                        "Queste opzioni non si applicano al formato o ai codec selezionati:",
                        "These options do not apply to the selected format or codecs:",
                    ),
                );
                for key in unsupported {
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(&key);
                        if ui.small_button(tr("Rimuovi", "Remove")).clicked() {
                            settings.media_options.remove(&key);
                        }
                    });
                }
            }
        });
    }

    fn active_codec_options(
        &self,
        ui: &mut egui::Ui,
        settings: &mut ConversionSettings,
        query: &str,
    ) {
        let active: Vec<_> = settings
            .media_options
            .iter()
            .filter(|(key, value)| {
                key.starts_with("codec:") && matches_search(query, &[key, value])
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        if active.is_empty() {
            return;
        }
        let encoders = active_encoders(settings);
        egui::CollapsingHeader::new(tr("Opzioni codec attive", "Active codec options"))
            .id_salt("active_codec_options")
            .default_open(true)
            .show(ui, |ui| {
                for (key, value) in active {
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(format!("{key} = {value}"));
                        if ui.small_button(tr("Rimuovi", "Remove")).clicked() {
                            settings.media_options.remove(&key);
                        }
                    });
                    let available_encoder = encoders
                        .iter()
                        .any(|encoder| key.starts_with(&format!("codec:{encoder}:")));
                    if !available_encoder {
                        ui.colored_label(
                            theme::palette(ui.ctx()).error,
                            tr("Questo codec non è selezionato.", "This codec is not selected."),
                        );
                    } else if self.discovered.as_ref().is_some_and(|(loaded, options)| {
                        self.current.as_ref() == Some(loaded)
                            && key.starts_with(&format!("codec:{}:", loaded.encoder))
                            && !options.iter().any(|option| option.key == key)
                    }) {
                        ui.colored_label(
                            theme::palette(ui.ctx()).error,
                            tr(
                                "Opzione non disponibile in questo FFmpeg: rimuovila o seleziona un’altra installazione.",
                                "This option is unavailable in this FFmpeg: remove it or select another installation.",
                            ),
                        );
                    }
                }
            });
    }

    fn codec_options(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut ConversionSettings,
        tools: &ToolPaths,
        query: &str,
    ) {
        ui.label(theme::heading(
            tr("Opzioni del codec installato", "Installed codec options"),
            theme::BODY,
        ));
        if settings.format.kind() == MediaKind::Video {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.audio_stream,
                    false,
                    tr("Codec video", "Video codec"),
                );
                ui.selectable_value(
                    &mut self.audio_stream,
                    true,
                    tr("Codec audio", "Audio codec"),
                );
            });
        }
        let encoder = if settings.format.kind() == MediaKind::Video && self.audio_stream {
            media_options::audio_encoder_for(settings)
        } else {
            media_options::encoder_for(settings)
        };
        let key = DiscoveryKey {
            executable: tools.ffmpeg.clone(),
            encoder,
        };
        self.poll(ui.ctx(), &key);
        ui.horizontal_wrapped(|ui| {
            ui.monospace(&key.encoder);
            if self.pending.is_some() {
                ui.spinner();
                ui.label(tr("Lettura opzioni…", "Reading options…"));
                if ui.button(tr("Annulla", "Cancel")).clicked()
                    && let Some(pending) = &self.pending
                {
                    pending.cancel.store(true, Ordering::Relaxed);
                }
            } else if ui
                .add_enabled(
                    tools.ffmpeg.is_some() && !key.encoder.is_empty(),
                    egui::Button::new(theme::button_text(
                        FluentIcon::Refresh,
                        tr("Leggi opzioni da FFmpeg", "Read options from FFmpeg"),
                        theme::palette(ui.ctx()).text,
                    )),
                )
                .clicked()
            {
                self.discover(ui.ctx(), tools.clone(), key.clone());
            }
        });
        if tools.ffmpeg.is_none() {
            ui.label(tr(
                "Seleziona FFmpeg in Impostazioni → Motori per leggere le opzioni disponibili.",
                "Select FFmpeg in Settings → Conversion engines to read the available options.",
            ));
        }
        if let Some(error) = &self.error {
            ui.colored_label(theme::palette(ui.ctx()).error, error);
        }
        if let Some((loaded, options)) = &self.discovered
            && *loaded == key
        {
            ui.label(trf!(
                "{} opzioni disponibili in questo encoder",
                "{} options available in this encoder",
                options.len()
            ));
            if options.is_empty() {
                ui.label(tr(
                    "Questo encoder non espone altre opzioni configurabili.",
                    "This encoder exposes no additional configurable options.",
                ));
            }
            for option in options.iter().filter(|option| {
                option.key.starts_with("codec:")
                    && matches_search(query, &[&option.name, &option.description, &option.key])
            }) {
                ui.push_id(&option.key, |ui| {
                    option_row(
                        ui,
                        &option.key,
                        &option.name,
                        &option.kind,
                        &option.choices,
                        option.default.as_deref(),
                        &mut settings.media_options,
                    );
                    if !option.description.is_empty() {
                        ui.label(
                            egui::RichText::new(&option.description)
                                .color(theme::palette(ui.ctx()).secondary),
                        );
                    }
                });
            }
        }
    }

    fn discover(&mut self, ctx: &egui::Context, tools: ToolPaths, key: DiscoveryKey) {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        self.pending = Some(PendingDiscovery {
            key: key.clone(),
            cancel: cancel.clone(),
            receiver,
        });
        self.error = None;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = media_options::discover_encoder_options(&tools, &key.encoder, &cancel)
                .map_err(|error| format!("{error:#}"));
            let _ = sender.send(result);
            ctx.request_repaint();
        });
    }

    fn poll(&mut self, ctx: &egui::Context, key: &DiscoveryKey) {
        if self.current.as_ref() != Some(key) {
            self.current = Some(key.clone());
            self.error = None;
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.key != *key)
            && let Some(pending) = self.pending.take()
        {
            pending.cancel.store(true, Ordering::Relaxed);
        }
        let Some(result) = self
            .pending
            .as_ref()
            .map(|pending| pending.receiver.try_recv())
        else {
            return;
        };
        match result {
            Ok(result) => {
                let pending = self.pending.take().expect("pending discovery");
                if pending.key == *key && !pending.cancel.load(Ordering::Relaxed) {
                    match result {
                        Ok(options) => self.discovered = Some((pending.key, options)),
                        Err(error) => self.error = Some(error),
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                self.error = Some(
                    tr(
                        "Lettura delle opzioni interrotta",
                        "Reading options was interrupted",
                    )
                    .into(),
                );
            }
        }
    }
}

fn active_encoders(settings: &ConversionSettings) -> Vec<String> {
    let mut encoders = vec![media_options::encoder_for(settings)];
    if settings.format.kind() == MediaKind::Video {
        encoders.push(media_options::audio_encoder_for(settings));
    }
    encoders
}

fn scoped_key(key: &str) -> bool {
    matches!(key, "preset" | "tune" | "profile" | "level" | "crf")
}

fn applicable_spec(spec: &OptionSpec, settings: &ConversionSettings) -> bool {
    !scoped_key(spec.key)
        || media_options::encoder_specific_choices(spec.key, &media_options::encoder_for(settings))
            .is_some()
}

fn matches_search(query: &str, values: &[&str]) -> bool {
    query.is_empty()
        || values
            .iter()
            .any(|value| value.to_lowercase().contains(query))
}

fn group_label(group: &str) -> &'static str {
    match group {
        "codec" => tr("Codec e profilo", "Codec and profile"),
        "video" => tr("Video e frequenza", "Video and frame rate"),
        "audio" => tr("Audio e canali", "Audio and channels"),
        "filters" => tr("Filtri", "Filters"),
        "metadata" => tr("Metadati", "Metadata"),
        "performance" => tr("Prestazioni", "Performance"),
        _ => tr("Altre opzioni", "Other options"),
    }
}

fn initial_value(kind: &OptionKind, choices: &[String], default: Option<&str>) -> String {
    if let Some(default) = default {
        return default.into();
    }
    if let Some(first) = choices.first() {
        return first.clone();
    }
    match kind {
        OptionKind::Bool => "1".into(),
        OptionKind::Int { min, max } => 0_i64.clamp(*min, *max).to_string(),
        OptionKind::Float { min, max } => 0.0_f64.clamp(*min, *max).to_string(),
        OptionKind::Choice(values) => values.first().copied().unwrap_or_default().into(),
        OptionKind::Text => String::new(),
    }
}

fn compatible_value(
    kind: &OptionKind,
    choices: &[String],
    default: Option<&str>,
    value: &str,
) -> bool {
    if !choices.is_empty() {
        return choices.iter().any(|choice| choice == value)
            || default == Some(value)
            || value == "-1"
                && matches!(kind, OptionKind::Int { min: -1, max: 1 })
                && choices.iter().any(|choice| choice == "auto")
            || compatible_number(kind, value);
    }
    match kind {
        OptionKind::Bool => matches!(value, "0" | "1" | "true" | "false") || default == Some(value),
        OptionKind::Int { min, max } => value
            .parse::<i64>()
            .is_ok_and(|number| (*min..=*max).contains(&number)),
        OptionKind::Float { min, max } => value
            .parse::<f64>()
            .is_ok_and(|number| number.is_finite() && (*min..=*max).contains(&number)),
        OptionKind::Choice(choices) => choices.contains(&value),
        OptionKind::Text => true,
    }
}

fn compatible_number(kind: &OptionKind, value: &str) -> bool {
    match kind {
        OptionKind::Int { min, max } => value
            .parse::<i64>()
            .is_ok_and(|number| (*min..=*max).contains(&number)),
        OptionKind::Float { min, max } => value
            .parse::<f64>()
            .is_ok_and(|number| number.is_finite() && (*min..=*max).contains(&number)),
        _ => false,
    }
}

/// Named numeric aliases remain selectable, while explicit numbers use the
/// same bounded numeric widgets as options without aliases.
fn numeric_alias_input(ui: &mut egui::Ui, kind: &OptionKind, value: &mut String) -> bool {
    let mut changed = false;
    match kind {
        OptionKind::Int { min, max } => {
            if let Ok(mut number) = value.parse::<i64>() {
                if ui
                    .add(egui::DragValue::new(&mut number).range(*min..=*max))
                    .changed()
                {
                    *value = number.to_string();
                    changed = true;
                }
            } else if ui.button(tr("Numero…", "Number…")).clicked() {
                *value = 0_i64.clamp(*min, *max).to_string();
                changed = true;
            }
        }
        OptionKind::Float { min, max } => {
            if let Some(mut number) = value
                .parse::<f64>()
                .ok()
                .filter(|number| number.is_finite())
            {
                if ui
                    .add(
                        egui::DragValue::new(&mut number)
                            .range(*min..=*max)
                            .speed(0.01),
                    )
                    .changed()
                {
                    *value = number.to_string();
                    changed = true;
                }
            } else if ui.button(tr("Numero…", "Number…")).clicked() {
                *value = 0.0_f64.clamp(*min, *max).to_string();
                changed = true;
            }
        }
        _ => {}
    }
    changed
}

fn option_row(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    kind: &OptionKind,
    choices: &[String],
    default: Option<&str>,
    options: &mut BTreeMap<String, String>,
) {
    ui.push_id(key, |ui| {
        ui.horizontal_wrapped(|ui| {
            let mut enabled = options.contains_key(key);
            if ui
                .checkbox(&mut enabled, label)
                .on_hover_text(key)
                .changed()
            {
                if enabled {
                    options.insert(key.into(), initial_value(kind, choices, default));
                } else {
                    options.remove(key);
                }
            }
            ui.add_enabled_ui(enabled, |ui| {
                let mut value = options
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| initial_value(kind, choices, default));
                let mut changed = false;
                let flags = key.starts_with("codec:")
                    && matches!(kind, OptionKind::Text)
                    && !choices.is_empty();
                if flags {
                    changed |= ui
                        .add(
                            egui::TextEdit::singleline(&mut value)
                                .char_limit(512)
                                .desired_width(220.0),
                        )
                        .on_hover_text(tr(
                            "Combina i nomi disponibili con + o -. Usa 0 per disattivare tutti i flag.",
                            "Combine the available names with + or -. Use 0 to disable all flags.",
                        ))
                        .changed();
                    ui.menu_button(tr("Flag disponibili", "Available flags"), |ui| {
                        if ui.button(tr("Nessuno", "None")).clicked() {
                            value = "0".into();
                            changed = true;
                            ui.close();
                        }
                        egui::ScrollArea::vertical()
                            .max_height(240.0)
                            .show(ui, |ui| {
                                for choice in choices {
                                    if ui.button(choice).clicked() {
                                        if value == "0" || value.is_empty() {
                                            value = choice.clone();
                                        } else {
                                            value.push('+');
                                            value.push_str(choice);
                                        }
                                        changed = true;
                                        ui.close();
                                    }
                                }
                            });
                    });
                } else if !choices.is_empty() {
                    let automatic = matches!(kind, OptionKind::Int { min: -1, max: 1 })
                        && choices.iter().any(|choice| choice == "auto");
                    egui::ComboBox::from_id_salt("value")
                        .selected_text(if automatic && matches!(value.as_str(), "auto" | "-1") {
                            tr("Automatico", "Automatic")
                        } else {
                            &value
                        })
                        .show_ui(ui, |ui| {
                            if let Some(default) = default
                                && !choices.iter().any(|choice| choice == default)
                            {
                                changed |= ui
                                    .selectable_value(&mut value, default.into(), default)
                                    .changed();
                            }
                            for choice in choices {
                                let label = if automatic {
                                    match choice.as_str() {
                                        "auto" | "-1" => tr("Automatico", "Automatic"),
                                        "0" => tr("Disattivo", "Off"),
                                        "1" => tr("Attivo", "On"),
                                        _ => choice,
                                    }
                                } else {
                                    choice
                                };
                                changed |= ui
                                    .selectable_value(&mut value, choice.clone(), label)
                                    .changed();
                            }
                        });
                    changed |= numeric_alias_input(ui, kind, &mut value);
                } else {
                    match kind {
                        OptionKind::Bool => {
                            let label = match value.as_str() {
                                "1" | "true" => tr("Attivo", "On"),
                                "0" | "false" => tr("Disattivo", "Off"),
                                "auto" | "-1" => tr("Automatico", "Automatic"),
                                _ => &value,
                            };
                            let extra = default
                                .filter(|value| !matches!(*value, "0" | "1" | "true" | "false"))
                                .map(str::to_string)
                                .or_else(|| {
                                    matches!(value.as_str(), "auto" | "-1").then(|| value.clone())
                                });
                            egui::ComboBox::from_id_salt("value")
                                .selected_text(label)
                                .show_ui(ui, |ui| {
                                    for (choice, label) in
                                        [("1", tr("Attivo", "On")), ("0", tr("Disattivo", "Off"))]
                                    {
                                        changed |= ui
                                            .selectable_value(&mut value, choice.into(), label)
                                            .changed();
                                    }
                                    if let Some(extra) = extra {
                                        let label = if matches!(extra.as_str(), "auto" | "-1") {
                                            tr("Automatico", "Automatic").to_string()
                                        } else {
                                            trf!("Predefinito ({extra})", "Default ({extra})")
                                        };
                                        changed |=
                                            ui.selectable_value(&mut value, extra, label).changed();
                                    }
                                });
                        }
                        OptionKind::Int { min, max } => {
                            let mut number =
                                value.parse::<i64>().unwrap_or(0_i64.clamp(*min, *max));
                            if ui
                                .add(egui::DragValue::new(&mut number).range(*min..=*max))
                                .changed()
                            {
                                value = number.to_string();
                                changed = true;
                            }
                        }
                        OptionKind::Float { min, max } => {
                            let mut number =
                                value.parse::<f64>().unwrap_or(0.0_f64.clamp(*min, *max));
                            if ui
                                .add(
                                    egui::DragValue::new(&mut number)
                                        .range(*min..=*max)
                                        .speed(0.01),
                                )
                                .changed()
                            {
                                value = number.to_string();
                                changed = true;
                            }
                        }
                        OptionKind::Choice(values) => {
                            egui::ComboBox::from_id_salt("value")
                                .selected_text(&value)
                                .show_ui(ui, |ui| {
                                    for choice in *values {
                                        changed |= ui
                                            .selectable_value(&mut value, (*choice).into(), *choice)
                                            .changed();
                                    }
                                });
                        }
                        OptionKind::Text => {
                            changed = ui
                                .add(
                                    egui::TextEdit::singleline(&mut value)
                                        .char_limit(512)
                                        .desired_width(220.0),
                                )
                                .changed();
                        }
                    }
                }
                let compatible = if flags {
                    value == "0"
                        || value
                            .trim_start_matches(['+', '-'])
                            .split(['+', '-', '|'])
                            .all(|part| choices.iter().any(|choice| choice == part))
                } else {
                    compatible_value(kind, choices, default, &value)
                };
                if enabled && changed {
                    options.insert(key.into(), value);
                }
                if enabled && !compatible {
                    ui.colored_label(
                        theme::palette(ui.ctx()).error,
                        tr("Valore non compatibile", "Incompatible value"),
                    );
                }
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_aliases_also_accept_in_range_numeric_values_without_paths() {
        let kind = OptionKind::Int { min: 0, max: 3 };
        let choices = vec!["none".into(), "variance".into(), "autovariance".into()];
        assert!(compatible_value(&kind, &choices, None, "variance"));
        assert!(compatible_value(&kind, &choices, None, "2"));
        assert!(compatible_value(&kind, &choices, None, "0"));
        assert!(compatible_value(&kind, &choices, None, "3"));
        for value in ["-1", "4", "2.5", "2;file", "../../file", "C:/file"] {
            assert!(!compatible_value(&kind, &choices, None, value), "{value}");
        }
        let deadline = OptionKind::Int {
            min: 0,
            max: i64::MAX,
        };
        assert!(compatible_value(
            &deadline,
            &["good".into(), "realtime".into()],
            None,
            "200000"
        ));
    }

    #[test]
    fn floating_aliases_accept_finite_numbers_and_preserve_auto_boolean_aliases() {
        let kind = OptionKind::Float {
            min: -1.0,
            max: 10.0,
        };
        let choices = vec!["automatic".into()];
        assert!(compatible_value(&kind, &choices, None, "automatic"));
        assert!(compatible_value(&kind, &choices, None, "0.25"));
        assert!(compatible_value(&kind, &choices, None, "10"));
        for value in ["-1.1", "10.1", "NaN", "inf", "1/2", "file:/input"] {
            assert!(!compatible_value(&kind, &choices, None, value), "{value}");
        }
        let automatic = OptionKind::Int { min: -1, max: 1 };
        let choices = vec!["auto".into(), "0".into(), "1".into()];
        assert!(compatible_value(&automatic, &choices, Some("auto"), "-1"));
        assert!(compatible_value(&automatic, &choices, Some("auto"), "auto"));
        assert!(!compatible_value(&automatic, &choices, Some("auto"), "2"));
    }

    #[test]
    fn changing_encoder_cancels_pending_probe_and_does_not_apply_stale_options() {
        let old = DiscoveryKey {
            executable: Some(PathBuf::from("ffmpeg.exe")),
            encoder: "libx264".into(),
        };
        let new = DiscoveryKey {
            encoder: "libx265".into(),
            ..old.clone()
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::channel();
        let mut state = MediaOptionsUi {
            search: String::new(),
            audio_stream: false,
            pending: Some(PendingDiscovery {
                key: old,
                cancel: cancel.clone(),
                receiver,
            }),
            discovered: None,
            current: None,
            error: None,
        };
        sender.send(Ok(vec![])).unwrap();
        state.poll(&egui::Context::default(), &new);
        assert!(cancel.load(Ordering::Relaxed));
        assert!(state.pending.is_none());
        assert!(state.discovered.is_none());
        assert!(state.current.as_ref() == Some(&new));
    }

    #[test]
    fn cancelled_probe_cannot_replace_previously_loaded_results() {
        let key = DiscoveryKey {
            executable: Some(PathBuf::from("ffmpeg.exe")),
            encoder: "libopus".into(),
        };
        let (sender, receiver) = mpsc::channel();
        let mut state = MediaOptionsUi {
            search: String::new(),
            audio_stream: false,
            pending: Some(PendingDiscovery {
                key: key.clone(),
                cancel: Arc::new(AtomicBool::new(true)),
                receiver,
            }),
            discovered: Some((
                key.clone(),
                vec![EncoderOption {
                    key: "codec:libopus:compression_level".into(),
                    name: "compression_level".into(),
                    description: String::new(),
                    kind: OptionKind::Int { min: 0, max: 10 },
                    default: Some("10".into()),
                    choices: vec![],
                }],
            )),
            current: None,
            error: None,
        };
        sender.send(Ok(vec![])).unwrap();
        state.poll(&egui::Context::default(), &key);
        assert!(state.pending.is_none());
        let (_, options) = state.discovered.as_ref().unwrap();
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].name, "compression_level");
        assert!(state.error.is_none());
    }
}
