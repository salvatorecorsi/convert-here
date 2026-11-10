//! Profile management UI. The owning app performs all persistent mutations.
use crate::theme::{self, FluentIcon, FormatFamily};
use eframe::egui::{self, RichText};
use std::{collections::BTreeSet, io::Read, sync::mpsc};
use the_converter::{
    conversion_profiles::{
        ConversionProfile, ConversionProfiles, MAX_PREFERENCES_BYTES, MAX_SAVED_PROFILES,
        compatible_input_extensions,
    },
    i18n::tr,
    image_edit::{ImageEditSettings, ResizeSettings},
    model::{ConversionSettings, CsvDelimiter, MediaKind, OutputFormat, PdfAction, VideoCodec},
    output_preferences::OutputPreferences,
    page_layout::{ImageFit, PageLayoutSettings, PageOrientation, PaperSize, TextAlign},
    tools::ToolPaths,
    trf,
};

pub(crate) enum ProfileManagerAction {
    Save {
        id: Option<String>,
        name: String,
        show_in_menu: bool,
        settings: Box<ConversionSettings>,
        input_extensions: Vec<String>,
        open_image_editor: bool,
    },
    Delete(String),
    SetVisibility(String, bool),
    MakeDefault(String),
    ClearDefault,
    EditImage(String),
    Import(Vec<u8>),
    Export {
        id: Option<String>,
    },
}

struct Draft {
    id: Option<String>,
    name: String,
    show_in_menu: bool,
    settings: ConversionSettings,
    filter_inputs: bool,
    inputs: BTreeSet<String>,
    family: FormatFamily,
}
impl Draft {
    fn new(settings: ConversionSettings) -> Self {
        let mut settings = settings;
        settings.select_output_format(settings.format);
        let family = FormatFamily::of_output(settings.format);
        Self {
            id: None,
            name: String::new(),
            show_in_menu: true,
            settings,
            filter_inputs: false,
            inputs: BTreeSet::new(),
            family,
        }
    }
    fn from_profile(profile: &ConversionProfile) -> Self {
        Self {
            id: Some(profile.id.clone()),
            name: profile.name.clone(),
            show_in_menu: profile.show_in_menu,
            family: FormatFamily::of_output(profile.settings.format),
            settings: profile.settings.clone(),
            filter_inputs: !profile.input_extensions.is_empty(),
            inputs: profile.input_extensions.iter().cloned().collect(),
        }
    }
    fn select_format(&mut self, format: OutputFormat) {
        self.settings.select_output_format(format);
        self.family = FormatFamily::of_output(format);
        if format
            .audio_sample_rates()
            .is_some_and(|rates| !rates.contains(&self.settings.audio.sample_rate))
        {
            self.settings.audio.sample_rate = 0;
        }
        let allowed = allowed_inputs(&self.settings);
        self.inputs
            .retain(|input| allowed.contains(&input.as_str()));
    }
    fn save_action(&self, open_image_editor: bool) -> ProfileManagerAction {
        ProfileManagerAction::Save {
            id: self.id.clone(),
            name: self.name.trim().into(),
            show_in_menu: self.show_in_menu,
            settings: Box::new(self.settings.clone()),
            input_extensions: if self.filter_inputs {
                self.inputs.iter().cloned().collect()
            } else {
                vec![]
            },
            open_image_editor,
        }
    }
}

#[derive(Default)]
pub(crate) struct ProfileManager {
    draft: Option<Draft>,
    notice: Option<(String, bool)>,
    confirm_delete: Option<String>,
    import_receiver: Option<mpsc::Receiver<Result<Vec<u8>, String>>>,
    advanced_media: crate::media_options_ui::MediaOptionsUi,
}

impl ProfileManager {
    pub(crate) fn new_profile(&mut self, settings: ConversionSettings) {
        self.draft = Some(Draft::new(settings));
        self.notice = None;
    }
    pub(crate) fn edit_profile(&mut self, profile: &ConversionProfile) {
        self.draft = Some(Draft::from_profile(profile));
        self.notice = None;
    }
    pub(crate) fn refresh_if_current(&mut self, profile: &ConversionProfile) {
        if self
            .draft
            .as_ref()
            .is_some_and(|draft| draft.id.as_deref() == Some(profile.id.as_str()))
        {
            self.draft = Some(Draft::from_profile(profile));
        }
    }
    pub(crate) fn saved(&mut self, id: String) {
        if let Some(draft) = &mut self.draft {
            draft.id = Some(id);
        }
    }
    pub(crate) fn clear_editor(&mut self) {
        self.draft = None;
    }
    pub(crate) fn notify(&mut self, message: String, error: bool) {
        self.notice = Some((message, error));
    }

    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        profiles: &ConversionProfiles,
        default_id: Option<&str>,
        enabled: &OutputPreferences,
        tools: &ToolPaths,
    ) -> Option<ProfileManagerAction> {
        let mut action = None;
        if let Some(receiver) = &self.import_receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    self.import_receiver = None;
                    match result {
                        Ok(bytes) => action = Some(ProfileManagerAction::Import(bytes)),
                        Err(error) => self.notify(error, true),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.import_receiver = None;
                    self.notify(
                        tr(
                            "Importazione interrotta. Riprova.",
                            "Import interrupted. Try again.",
                        )
                        .into(),
                        true,
                    );
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(theme::heading(tr("Profili", "Profiles"), theme::TITLE));
            ui.label(
                RichText::new(trf!(
                    "{} di {MAX_SAVED_PROFILES}",
                    "{} of {MAX_SAVED_PROFILES}",
                    profiles.len()
                ))
                .color(theme::palette(ui.ctx()).secondary),
            );
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if button(
                ui,
                FluentIcon::Add,
                tr("Nuovo", "New"),
                profiles.len() < MAX_SAVED_PROFILES,
            )
            .clicked()
            {
                self.new_profile(ConversionSettings::default());
            }
            if button(
                ui,
                FluentIcon::Import,
                tr("Importa JSON…", "Import JSON…"),
                self.import_receiver.is_none(),
            )
            .clicked()
            {
                self.import_dialog(ui.ctx());
            }
            if button(
                ui,
                FluentIcon::Export,
                tr("Esporta tutti…", "Export all…"),
                !profiles.is_empty(),
            )
            .clicked()
            {
                action = Some(ProfileManagerAction::Export { id: None });
            }
            if self.import_receiver.is_some() {
                ui.spinner();
            }
        });
        if let Some((message, error)) = &self.notice {
            ui.add_space(8.0);
            theme::show_card(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let color = if *error {
                        theme::palette(ui.ctx()).error
                    } else {
                        theme::palette(ui.ctx()).success
                    };
                    ui.label(
                        theme::icon(
                            if *error {
                                FluentIcon::Info
                            } else {
                                FluentIcon::Check
                            },
                            16.0,
                        )
                        .color(color),
                    );
                    ui.colored_label(color, message);
                });
            });
        }
        ui.add_space(12.0);
        if let Some(draft) = &mut self.draft {
            ui.push_id("profile_draft", |ui| {
                action = draft_editor(ui, draft, enabled, &mut self.advanced_media, tools)
                    .or(action.take());
            });
            ui.add_space(12.0);
            if button(
                ui,
                FluentIcon::Back,
                tr("Chiudi modifica", "Close editor"),
                true,
            )
            .clicked()
            {
                self.clear_editor();
            }
            ui.add_space(16.0);
        }
        if profiles.is_empty() {
            theme::show_card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        theme::icon(FluentIcon::Profiles, 32.0)
                            .color(theme::palette(ui.ctx()).secondary),
                    );
                    ui.label(tr(
                        "Nessun profilo. Crea una ricetta o importa un file JSON.",
                        "No profiles yet. Create a recipe or import a JSON file.",
                    ));
                });
            });
        }
        for profile in profiles.iter() {
            ui.push_id(&profile.id, |ui| {
                theme::show_card(ui, |ui| {
                    let family = FormatFamily::of_output(profile.settings.format);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(theme::icon(family.icon(), 24.0).color(family.color(ui.ctx())));
                        ui.label(theme::heading(&profile.name, theme::BODY));
                        ui.label(profile.settings.format.label());
                        if default_id == Some(profile.id.as_str()) {
                            ui.colored_label(
                                theme::palette(ui.ctx()).accent,
                                tr("Predefinito", "Default"),
                            );
                        }
                        if !enabled.contains(profile.settings.format) {
                            ui.colored_label(
                                theme::palette(ui.ctx()).secondary,
                                tr("Formato disattivato", "Format disabled"),
                            );
                        }
                    });
                    ui.label(
                        RichText::new(input_summary(profile))
                            .color(theme::palette(ui.ctx()).secondary),
                    );
                    ui.horizontal_wrapped(|ui| {
                        if button(ui, FluentIcon::Edit, tr("Modifica", "Edit"), true).clicked() {
                            self.edit_profile(profile);
                        }
                        if can_open_image_editor(&profile.settings, &profile.input_extensions)
                            && button(
                                ui,
                                FluentIcon::Image,
                                tr("Editor visivo…", "Visual editor…"),
                                true,
                            )
                            .clicked()
                        {
                            action = Some(ProfileManagerAction::EditImage(profile.id.clone()));
                        }
                        if button(
                            ui,
                            FluentIcon::Copy,
                            tr("Duplica", "Duplicate"),
                            profiles.len() < MAX_SAVED_PROFILES,
                        )
                        .clicked()
                        {
                            let mut draft = Draft::from_profile(profile);
                            draft.id = None;
                            draft.name = duplicate_name(&profile.name);
                            self.draft = Some(draft);
                        }
                        if button(ui, FluentIcon::Export, tr("Esporta…", "Export…"), true).clicked()
                        {
                            action = Some(ProfileManagerAction::Export {
                                id: Some(profile.id.clone()),
                            });
                        }
                        if button(ui, FluentIcon::Delete, tr("Elimina", "Delete"), true).clicked() {
                            self.confirm_delete = Some(profile.id.clone());
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        let mut show = profile.show_in_menu;
                        if ui
                            .checkbox(&mut show, tr("Mostra in Converti in", "Show in Convert to"))
                            .changed()
                        {
                            action = Some(ProfileManagerAction::SetVisibility(
                                profile.id.clone(),
                                show,
                            ));
                        }
                        if default_id == Some(profile.id.as_str()) {
                            if ui
                                .button(tr("Rimuovi predefinito", "Remove default"))
                                .clicked()
                            {
                                action = Some(ProfileManagerAction::ClearDefault);
                            }
                        } else if ui
                            .button(tr("Usa come predefinito", "Make default"))
                            .clicked()
                        {
                            action = Some(ProfileManagerAction::MakeDefault(profile.id.clone()));
                        }
                    });
                });
            });
            ui.add_space(8.0);
        }
        if let Some(id) = self.confirm_delete.clone() {
            if let Some(profile) = profiles.get(&id) {
                egui::Modal::new(ui.id().with("delete_profile")).show(ui.ctx(), |ui| {
                    ui.label(theme::heading(
                        tr("Elimina profilo", "Delete profile"),
                        theme::SUBTITLE,
                    ));
                    ui.label(trf!("Eliminare «{}»?", "Delete ‘{}’?", profile.name));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Annulla", "Cancel")).clicked() {
                            self.confirm_delete = None;
                        }
                        if ui.button(tr("Elimina", "Delete")).clicked() {
                            action = Some(ProfileManagerAction::Delete(id.clone()));
                            self.confirm_delete = None;
                            if self
                                .draft
                                .as_ref()
                                .is_some_and(|draft| draft.id.as_deref() == Some(id.as_str()))
                            {
                                self.draft = None;
                            }
                        }
                    });
                });
            } else {
                self.confirm_delete = None;
            }
        }
        action
    }

    fn import_dialog(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .set_title(tr("Importa profili", "Import profiles"))
            .add_filter("JSON", &["json"])
            .pick_file()
        else {
            return;
        };
        let (sender, receiver) = mpsc::channel();
        self.import_receiver = Some(receiver);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = (|| -> anyhow::Result<Vec<u8>> {
                let mut bytes = Vec::new();
                std::fs::File::open(path)?
                    .take(MAX_PREFERENCES_BYTES + 1)
                    .read_to_end(&mut bytes)?;
                anyhow::ensure!(
                    bytes.len() as u64 <= MAX_PREFERENCES_BYTES,
                    tr(
                        "Il file dei profili supera 256 KiB",
                        "The profiles file exceeds 256 KiB"
                    )
                );
                Ok(bytes)
            })()
            .map_err(|error| format!("{error:#}"));
            let _ = sender.send(result);
            ctx.request_repaint();
        });
    }
}

fn button(ui: &mut egui::Ui, icon: FluentIcon, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(theme::button_text(
            icon,
            label,
            theme::palette(ui.ctx()).text,
        )),
    )
}
fn duplicate_name(name: &str) -> String {
    let suffix = tr(" · copia", " · copy");
    name.chars()
        .take(40 - suffix.chars().count())
        .collect::<String>()
        + suffix
}
fn input_summary(profile: &ConversionProfile) -> String {
    if profile.input_extensions.is_empty() {
        tr(
            "Ingressi: tutti i formati compatibili",
            "Inputs: all compatible formats",
        )
        .into()
    } else {
        trf!(
            "Ingressi: {}",
            "Inputs: {}",
            profile
                .input_extensions
                .iter()
                .map(|ext| format!(".{ext}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
pub(crate) fn allowed_inputs(settings: &ConversionSettings) -> Vec<&'static str> {
    compatible_input_extensions(settings.format)
        .into_iter()
        .filter(|ext| {
            let kind = MediaKind::from_path(std::path::Path::new(&format!("input.{ext}")));
            if !settings.media_options.is_empty() {
                return matches!(kind, MediaKind::Audio | MediaKind::Video);
            }
            if settings.format == OutputFormat::Pdf && settings.pdf.action == PdfAction::Split {
                return kind == MediaKind::Pdf;
            }
            if settings.image.edit.is_some() {
                return kind == MediaKind::Image;
            }
            if settings.image.page.is_some() {
                return kind == MediaKind::Image || *ext == "txt";
            }
            true
        })
        .collect()
}

fn draft_editor(
    ui: &mut egui::Ui,
    draft: &mut Draft,
    enabled: &OutputPreferences,
    advanced_media: &mut crate::media_options_ui::MediaOptionsUi,
    tools: &ToolPaths,
) -> Option<ProfileManagerAction> {
    let mut action = None;
    theme::show_card(ui, |ui| {
        ui.label(theme::heading(
            if draft.id.is_some() {
                tr("Modifica profilo", "Edit profile")
            } else {
                tr("Nuovo profilo", "New profile")
            },
            theme::SUBTITLE,
        ));
        ui.add(
            egui::TextEdit::singleline(&mut draft.name)
                .hint_text(tr("Nome del profilo", "Profile name"))
                .char_limit(40)
                .desired_width(f32::INFINITY),
        );
        ui.checkbox(
            &mut draft.show_in_menu,
            tr("Mostra nel menu Converti in", "Show in the Convert to menu"),
        );
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for family in FormatFamily::ALL {
                if ui
                    .selectable_label(
                        draft.family == family,
                        theme::button_text(
                            family.icon(),
                            family.label(),
                            if draft.family == family {
                                ui.visuals().selection.stroke.color
                            } else {
                                family.color(ui.ctx())
                            },
                        ),
                    )
                    .clicked()
                    && draft.family != family
                    && let Some(format) = OutputFormat::ALL
                        .into_iter()
                        .find(|format| FormatFamily::of_output(*format) == family)
                {
                    draft.select_format(format);
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            for format in OutputFormat::ALL
                .into_iter()
                .filter(|format| FormatFamily::of_output(*format) == draft.family)
                .collect::<Vec<_>>()
            {
                if ui
                    .selectable_label(draft.settings.format == format, format.label())
                    .clicked()
                {
                    draft.select_format(format);
                }
            }
        });
        if !enabled.contains(draft.settings.format) {
            ui.colored_label(
                theme::palette(ui.ctx()).secondary,
                tr(
                    "Formato disattivato nelle Impostazioni: abilitalo per usarlo da Converti in.",
                    "This format is disabled in Settings: enable it to use it from Convert to.",
                ),
            );
        }
    });
    ui.add_space(8.0);
    input_controls(ui, draft);
    ui.add_space(8.0);
    theme::show_card(ui, |ui| {
        ui.label(theme::heading(tr("Opzioni", "Options"), theme::BODY));
        let format = draft.settings.format;
        let ffmpeg = matches!(format.kind(), MediaKind::Audio | MediaKind::Video)
            || format == OutputFormat::Gif;
        if !ffmpeg && !draft.settings.media_options.is_empty() {
            ui.colored_label(
                theme::palette(ui.ctx()).error,
                tr(
                    "Il profilo conserva opzioni FFmpeg che non si applicano a questo formato.",
                    "This profile retains FFmpeg options that do not apply to this format.",
                ),
            );
            if button(
                ui,
                FluentIcon::Refresh,
                tr("Rimuovi opzioni FFmpeg", "Remove FFmpeg options"),
                true,
            )
            .clicked()
            {
                draft.settings.media_options.clear();
            }
        }
        match format.kind() {
            MediaKind::Audio => audio_controls(ui, &mut draft.settings),
            MediaKind::Video => {
                video_controls(ui, &mut draft.settings);
                audio_controls(ui, &mut draft.settings);
            }
            MediaKind::Image => image_controls(ui, &mut draft.settings),
            MediaKind::Pdf => pdf_controls(ui, &mut draft.settings),
            _ => {}
        }
        if format == OutputFormat::Gif {
            video_controls(ui, &mut draft.settings);
        }
        if format == OutputFormat::Txt {
            crate::settings_ui::speech_controls(ui, &mut draft.settings.speech);
        }
        if ffmpeg {
            egui::CollapsingHeader::new(tr("Avanzate", "Advanced"))
                .id_salt("profile_media_advanced")
                .show(ui, |ui| {
                    advanced_media.show(ui, &mut draft.settings, tools);
                });
        }
        let allowed = allowed_inputs(&draft.settings);
        if allowed.contains(&"pdf") && matches!(format, OutputFormat::Png | OutputFormat::Jpg) {
            ui.add(
                egui::DragValue::new(&mut draft.settings.pdf.dpi)
                    .range(36..=600)
                    .prefix(tr("DPI pagine PDF ", "PDF page DPI ")),
            );
        }
        if allowed.contains(&"csv") || matches!(format, OutputFormat::Csv | OutputFormat::Xlsx) {
            egui::ComboBox::from_id_salt("profile_csv_separator")
                .selected_text(csv_label(draft.settings.csv.delimiter))
                .show_ui(ui, |ui| {
                    for value in [
                        CsvDelimiter::Comma,
                        CsvDelimiter::Semicolon,
                        CsvDelimiter::Tab,
                        CsvDelimiter::Pipe,
                    ] {
                        ui.selectable_value(
                            &mut draft.settings.csv.delimiter,
                            value,
                            csv_label(value),
                        );
                    }
                });
        }
        if matches!(
            format,
            OutputFormat::Txt | OutputFormat::Docx | OutputFormat::Html
        ) {
            ui.checkbox(
                &mut draft.settings.ocr.enabled,
                tr(
                    "Riconosci testo nelle scansioni con OCR locale",
                    "Recognize scanned text with local OCR",
                ),
            );
            ui.horizontal(|ui| {
                ui.label(tr("Lingue OCR", "OCR languages"));
                ui.add(
                    egui::TextEdit::singleline(&mut draft.settings.ocr.language)
                        .hint_text("ita+eng")
                        .char_limit(80),
                );
            });
        }
    });
    ui.add_space(8.0);
    limits_controls(ui, &mut draft.settings);
    let allowed = allowed_inputs(&draft.settings);
    draft
        .inputs
        .retain(|input| allowed.contains(&input.as_str()));
    let has_inputs = !draft.filter_inputs || !draft.inputs.is_empty();
    let valid_name = !draft.name.trim().is_empty();
    if !has_inputs {
        ui.colored_label(
            theme::palette(ui.ctx()).error,
            tr(
                "Seleziona almeno un ingresso o scegli tutti i compatibili.",
                "Select at least one input or choose all compatible formats.",
            ),
        );
    }
    ui.horizontal_wrapped(|ui| {
        if button(
            ui,
            FluentIcon::Check,
            tr("Salva profilo", "Save profile"),
            valid_name && has_inputs,
        )
        .clicked()
        {
            action = Some(draft.save_action(false));
        }
        let explicit_inputs: Vec<_> = if draft.filter_inputs {
            draft.inputs.iter().cloned().collect()
        } else {
            vec![]
        };
        let image_editor = can_open_image_editor(&draft.settings, &explicit_inputs);
        if button(
            ui,
            FluentIcon::Image,
            tr("Editor immagine e pagina…", "Image and page editor…"),
            image_editor && valid_name && has_inputs,
        )
        .clicked()
        {
            action = Some(draft.save_action(true));
        }
    });
    action
}

fn can_open_image_editor(settings: &ConversionSettings, explicit_inputs: &[String]) -> bool {
    matches!(
        settings.format,
        OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp | OutputFormat::Pdf
    ) && !(settings.format == OutputFormat::Pdf && settings.pdf.action == PdfAction::Split)
        && allowed_inputs(settings).into_iter().any(|ext| {
            (explicit_inputs.is_empty() || explicit_inputs.iter().any(|input| input == ext))
                && (MediaKind::from_path(std::path::Path::new(&format!("input.{ext}")))
                    == MediaKind::Image
                    || ext == "txt")
        })
}

fn input_controls(ui: &mut egui::Ui, draft: &mut Draft) {
    theme::show_card(ui, |ui| {
        ui.label(theme::heading(
            tr("Ingressi consentiti", "Allowed inputs"),
            theme::BODY,
        ));
        let allowed = allowed_inputs(&draft.settings);
        let mut all = !draft.filter_inputs;
        if ui
            .checkbox(
                &mut all,
                tr("Tutti i formati compatibili", "All compatible formats"),
            )
            .changed()
        {
            draft.filter_inputs = !all;
            if !all && draft.inputs.is_empty() {
                draft.inputs = allowed.iter().map(|ext| (*ext).into()).collect();
            }
        }
        ui.label(
            RichText::new(tr(
                "Il profilo compare soltanto per selezioni che rispettano questi ingressi.",
                "The profile is shown only for selections matching these inputs.",
            ))
            .color(theme::palette(ui.ctx()).secondary),
        );
        ui.add_enabled_ui(draft.filter_inputs, |ui| {
            for family in FormatFamily::ALL {
                let extensions: Vec<_> = allowed
                    .iter()
                    .copied()
                    .filter(|ext| FormatFamily::of_input_extension(ext) == family)
                    .collect();
                if extensions.is_empty() {
                    continue;
                }
                ui.push_id(family.label(), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut selected = !draft.filter_inputs
                            || extensions.iter().all(|ext| draft.inputs.contains(*ext));
                        let partial =
                            !selected && extensions.iter().any(|ext| draft.inputs.contains(*ext));
                        if ui
                            .add(
                                egui::Checkbox::new(
                                    &mut selected,
                                    theme::button_text(
                                        family.icon(),
                                        family.label(),
                                        family.color(ui.ctx()),
                                    ),
                                )
                                .indeterminate(partial),
                            )
                            .changed()
                        {
                            for ext in &extensions {
                                if selected {
                                    draft.inputs.insert((*ext).into());
                                } else {
                                    draft.inputs.remove(*ext);
                                }
                            }
                        }
                    });
                    egui::CollapsingHeader::new(tr("Estensioni", "Extensions"))
                        .id_salt("input_extensions")
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for ext in extensions {
                                    let mut selected =
                                        !draft.filter_inputs || draft.inputs.contains(ext);
                                    if ui.checkbox(&mut selected, format!(".{ext}")).changed() {
                                        if selected {
                                            draft.inputs.insert(ext.into());
                                        } else {
                                            draft.inputs.remove(ext);
                                        }
                                    }
                                }
                            });
                        });
                });
            }
        });
    });
}

fn audio_controls(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    let lossless = matches!(
        settings.format,
        OutputFormat::Wav
            | OutputFormat::Flac
            | OutputFormat::Aiff
            | OutputFormat::Au
            | OutputFormat::Caf
    );
    ui.add_enabled_ui(!lossless, |ui| {
        ui.add(
            egui::Slider::new(&mut settings.audio.bitrate_kbps, 8..=512)
                .text(tr("Bitrate audio (kbps)", "Audio bitrate (kbps)")),
        );
    });
    if lossless {
        ui.label(tr(
            "Formato senza perdita: il bitrate non viene applicato.",
            "Lossless format: bitrate does not apply.",
        ));
    }
    let rates = settings.format.audio_sample_rates().unwrap_or(&[
        0, 8000, 11025, 12000, 16000, 22050, 24000, 32000, 44100, 48000, 88200, 96000, 192000,
    ]);
    egui::ComboBox::from_id_salt("profile_audio_rate")
        .selected_text(if settings.audio.sample_rate == 0 {
            tr("Frequenza sorgente", "Source sample rate").into()
        } else {
            format!("{} Hz", settings.audio.sample_rate)
        })
        .show_ui(ui, |ui| {
            for rate in rates {
                ui.selectable_value(
                    &mut settings.audio.sample_rate,
                    *rate,
                    if *rate == 0 {
                        tr("Frequenza sorgente", "Source sample rate").into()
                    } else {
                        format!("{rate} Hz")
                    },
                );
            }
        });
    ui.horizontal_wrapped(|ui| {
        for (channels, label) in [
            (0, tr("Canali sorgente", "Source channels")),
            (1, "Mono"),
            (2, "Stereo"),
        ] {
            ui.radio_value(&mut settings.audio.channels, channels, label);
        }
    });
    ui.checkbox(
        &mut settings.audio.normalize,
        tr("Normalizza volume", "Normalize volume"),
    );
    trim_controls(ui, settings);
}
fn video_controls(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    if matches!(
        settings.format,
        OutputFormat::Mp4 | OutputFormat::Mkv | OutputFormat::Mov | OutputFormat::M4v
    ) {
        ui.horizontal(|ui| {
            ui.radio_value(&mut settings.video.codec, VideoCodec::H264, "H.264");
            ui.radio_value(&mut settings.video.codec, VideoCodec::H265, "H.265 / HEVC");
        });
    }
    if settings.format != OutputFormat::Gif {
        ui.add(egui::Slider::new(&mut settings.video.crf, 0..=51).text(
            if matches!(
                settings.format,
                OutputFormat::Mp4 | OutputFormat::Mkv | OutputFormat::Mov | OutputFormat::M4v
            ) {
                "CRF"
            } else {
                tr("Qualità", "Quality")
            },
        ));
    }
    optional_u32(
        ui,
        tr("Larghezza video", "Video width"),
        &mut settings.video.width,
        1920,
        16..=16384,
        " px",
    );
    optional_u32(
        ui,
        tr("Fotogrammi al secondo", "Frames per second"),
        &mut settings.video.fps,
        30,
        1..=240,
        " FPS",
    );
    if settings.format == OutputFormat::Gif {
        trim_controls(ui, settings);
    }
}
fn trim_controls(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    optional_f64(
        ui,
        tr("Inizio", "Start"),
        &mut settings.video.trim_start,
        0.0,
        0.0..=86400.0,
        " s",
    );
    optional_f64(
        ui,
        tr("Fine", "End"),
        &mut settings.video.trim_end,
        10.0,
        0.01..=86400.0,
        " s",
    );
}
fn image_controls(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    let editable = matches!(
        settings.format,
        OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp
    );
    if editable {
        let mut enabled = settings.image.edit.is_some();
        if ui
            .checkbox(&mut enabled, tr("Modifiche immagine", "Image edits"))
            .changed()
        {
            settings.image.edit = enabled.then(|| ImageEditSettings {
                resize: settings.image.width.map(|width| ResizeSettings {
                    width: Some(width),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
    }
    if matches!(
        settings.format,
        OutputFormat::Jpg | OutputFormat::Avif | OutputFormat::Heic
    ) || settings.format == OutputFormat::Webp && settings.image.edit.is_some()
    {
        ui.add(
            egui::Slider::new(&mut settings.image.quality, 1..=100)
                .text(tr("Qualità immagine", "Image quality")),
        );
    } else if settings.format == OutputFormat::Webp {
        ui.label(tr(
            "WebP senza perdita. Attiva le modifiche immagine per regolare la qualità.",
            "Lossless WebP. Enable image edits to adjust quality.",
        ));
    }
    if let Some(edit) = &mut settings.image.edit {
        let mut scale = edit.resize.is_some();
        if ui
            .checkbox(&mut scale, tr("Ridimensiona", "Resize"))
            .changed()
        {
            edit.resize = scale.then(|| ResizeSettings {
                width: Some(1920),
                ..Default::default()
            });
        }
        if let Some(resize) = &mut edit.resize {
            optional_u32(
                ui,
                tr("Larghezza", "Width"),
                &mut resize.width,
                1920,
                1..=32768,
                " px",
            );
            optional_u32(
                ui,
                tr("Altezza", "Height"),
                &mut resize.height,
                1080,
                1..=32768,
                " px",
            );
            ui.checkbox(
                &mut resize.keep_aspect,
                tr("Mantieni proporzioni", "Keep aspect ratio"),
            );
        }
        if edit.crop.is_some() {
            ui.label(tr(
                "Ritaglio configurato nell’editor visivo",
                "Crop configured in the visual editor",
            ));
        }
        if edit.canvas.is_some() {
            ui.label(tr(
                "Tela configurata nell’editor visivo",
                "Canvas configured in the visual editor",
            ));
        }
        ui.label(tr(
            "Usa l’editor immagine e pagina per ritaglio, tela, sfondo e anteprima.",
            "Use the image and page editor for crop, canvas, background and preview.",
        ));
    } else {
        optional_u32(
            ui,
            tr("Larghezza immagine", "Image width"),
            &mut settings.image.width,
            1920,
            1..=32768,
            " px",
        );
    }
}
fn pdf_controls(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    ui.horizontal_wrapped(|ui| {
        ui.radio_value(
            &mut settings.pdf.action,
            PdfAction::Compress,
            tr("Converti / comprimi", "Convert / compress"),
        );
        if ui
            .radio_value(
                &mut settings.pdf.action,
                PdfAction::Split,
                tr("Dividi pagine PDF", "Split PDF pages"),
            )
            .changed()
        {
            settings.image.edit = None;
            settings.image.page = None;
        }
        ui.radio_value(
            &mut settings.pdf.action,
            PdfAction::Merge,
            tr("Unisci e ordina pagine", "Merge and arrange pages"),
        );
    });
    if settings.pdf.action == PdfAction::Split {
        return;
    }
    ui.add(
        egui::Slider::new(&mut settings.image.quality, 1..=100)
            .text(tr("Qualità immagini PDF", "PDF image quality")),
    );
    if settings.image.page.is_none() {
        ui.add(
            egui::DragValue::new(&mut settings.pdf.dpi)
                .range(36..=600)
                .prefix(tr("Risoluzione PDF ", "PDF resolution "))
                .suffix(" DPI"),
        );
    }
    let mut page = settings.image.page.is_some();
    if ui
        .checkbox(
            &mut page,
            tr("Impagina immagini e TXT", "Lay out images and TXT"),
        )
        .changed()
    {
        settings.image.page = page.then(PageLayoutSettings::default);
    }
    if let Some(page) = &mut settings.image.page {
        ui.horizontal(|ui| {
            ui.radio_value(&mut page.paper, PaperSize::A4, "A4");
            ui.radio_value(&mut page.paper, PaperSize::A5, "A5");
        });
        ui.horizontal_wrapped(|ui| {
            ui.radio_value(
                &mut page.orientation,
                PageOrientation::Portrait,
                tr("Verticale", "Portrait"),
            );
            ui.radio_value(
                &mut page.orientation,
                PageOrientation::Landscape,
                tr("Orizzontale", "Landscape"),
            );
        });
        ui.add(
            egui::DragValue::new(&mut page.margin_mm)
                .range(0.0..=70.0)
                .prefix(tr("Margini ", "Margins "))
                .suffix(" mm"),
        );
        ui.add(
            egui::DragValue::new(&mut page.dpi)
                .range(72..=600)
                .prefix(tr("Risoluzione ", "Resolution "))
                .suffix(" DPI"),
        );
        ui.add(
            egui::DragValue::new(&mut page.font_size_pt)
                .range(6.0..=96.0)
                .prefix(tr("Testo ", "Text "))
                .suffix(" pt"),
        );
        ui.add(
            egui::DragValue::new(&mut page.line_height)
                .range(1.0..=3.0)
                .prefix(tr("Interlinea ", "Line spacing ")),
        );
        ui.horizontal_wrapped(|ui| {
            ui.radio_value(&mut page.align, TextAlign::Left, tr("Sinistra", "Left"));
            ui.radio_value(&mut page.align, TextAlign::Center, tr("Centro", "Center"));
            ui.radio_value(&mut page.align, TextAlign::Right, tr("Destra", "Right"));
        });
        ui.horizontal_wrapped(|ui| {
            ui.radio_value(
                &mut page.fit,
                ImageFit::Contain,
                tr("Immagine intera", "Entire image"),
            );
            ui.radio_value(
                &mut page.fit,
                ImageFit::Cover,
                tr("Riempi foglio", "Fill page"),
            );
        });
    }
}
fn limits_controls(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    theme::show_card(ui, |ui| {
        ui.label(theme::heading(tr("Limiti", "Limits"), theme::BODY));
        let legacy = settings.image.edit.as_ref().and_then(|edit| edit.max_bytes);
        let effective = match (settings.limits.max_bytes, legacy) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let mut limited = effective.is_some();
        if ui
            .checkbox(
                &mut limited,
                tr("Peso massimo per file", "Maximum size per file"),
            )
            .changed()
        {
            settings.limits.max_bytes = limited.then_some(300_000);
            if let Some(edit) = &mut settings.image.edit {
                edit.max_bytes = None;
            }
        }
        if limited {
            let legacy = settings.image.edit.as_ref().and_then(|edit| edit.max_bytes);
            let effective = match (settings.limits.max_bytes, legacy) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            let mut kb = effective.unwrap_or(300_000) as f64 / 1000.0;
            if ui
                .add(
                    egui::DragValue::new(&mut kb)
                        .range(0.001..=u64::MAX as f64 / 1000.0)
                        .speed(1.0)
                        .suffix(" kB"),
                )
                .changed()
            {
                settings.limits.max_bytes = Some((kb * 1000.0).round() as u64);
                if let Some(edit) = &mut settings.image.edit {
                    edit.max_bytes = None;
                }
            }
            ui.label(tr(
                "1 kB = 1.000 byte. Un file oltre il limite non viene pubblicato.",
                "1 kB = 1,000 bytes. A file over the limit is not published.",
            ));
            if matches!(
                settings.format,
                OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp
            ) && let Some(edit) = &mut settings.image.edit
            {
                ui.checkbox(
                    &mut edit.allow_downsize,
                    tr(
                        "Riduci dimensioni se necessario",
                        "Reduce dimensions if necessary",
                    ),
                );
            }
        }
        let av = allowed_inputs(settings).into_iter().any(|ext| {
            matches!(
                MediaKind::from_path(std::path::Path::new(&format!("input.{ext}"))),
                MediaKind::Audio | MediaKind::Video
            )
        });
        if av {
            optional_f64(
                ui,
                tr("Durata massima audio/video", "Maximum audio/video duration"),
                &mut settings.limits.max_duration,
                30.0,
                0.01..=86400.0,
                " s",
            );
        }
    });
}
fn optional_u32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut Option<u32>,
    default: u32,
    range: std::ops::RangeInclusive<u32>,
    suffix: &str,
) {
    ui.horizontal(|ui| {
        let mut active = value.is_some();
        if ui.checkbox(&mut active, label).changed() {
            *value = active.then_some(default);
        }
        if let Some(value) = value {
            ui.add(egui::DragValue::new(value).range(range).suffix(suffix));
        }
    });
}
fn optional_f64(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut Option<f64>,
    default: f64,
    range: std::ops::RangeInclusive<f64>,
    suffix: &str,
) {
    ui.horizontal(|ui| {
        let mut active = value.is_some();
        if ui.checkbox(&mut active, label).changed() {
            *value = active.then_some(default);
        }
        if let Some(value) = value {
            ui.add(
                egui::DragValue::new(value)
                    .range(range)
                    .speed(0.1)
                    .suffix(suffix),
            );
        }
    });
}
fn csv_label(value: CsvDelimiter) -> &'static str {
    match value {
        CsvDelimiter::Comma => tr("CSV: virgola", "CSV: comma"),
        CsvDelimiter::Semicolon => tr("CSV: punto e virgola", "CSV: semicolon"),
        CsvDelimiter::Tab => "CSV: Tab",
        CsvDelimiter::Pipe => "CSV: |",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draft_target_changes_keep_quality_and_do_not_turn_empty_filters_into_all() {
        let mut settings = ConversionSettings::default();
        settings.select_output_format(OutputFormat::Png);
        settings.image.quality = 77;
        settings.image.edit = Some(Default::default());
        let mut draft = Draft::new(settings);
        draft.filter_inputs = true;
        draft.inputs.insert("png".into());
        draft.select_format(OutputFormat::Mp3);
        assert_eq!(draft.settings.image.quality, 77);
        assert!(draft.settings.image.edit.is_none());
        assert!(draft.filter_inputs);
        assert!(draft.inputs.is_empty());
        assert!(allowed_inputs(&draft.settings).contains(&"mp3"));
        assert!(!allowed_inputs(&draft.settings).contains(&"png"));
    }
    #[test]
    fn allowed_inputs_follow_page_editor_and_pdf_split_policies() {
        let mut settings = ConversionSettings::default();
        settings.select_output_format(OutputFormat::Pdf);
        settings.image.page = Some(Default::default());
        let page_inputs = allowed_inputs(&settings);
        assert!(page_inputs.contains(&"txt"));
        assert!(page_inputs.contains(&"png"));
        assert!(!page_inputs.contains(&"pdf"));
        assert!(!page_inputs.contains(&"docx"));
        settings.image.edit = Some(Default::default());
        assert!(!allowed_inputs(&settings).contains(&"txt"));
        settings.pdf.action = PdfAction::Split;
        assert_eq!(allowed_inputs(&settings), vec!["pdf"]);
        assert!(!can_open_image_editor(&settings, &[]));
        settings.select_output_format(OutputFormat::Png);
        assert!(can_open_image_editor(&settings, &[]));
    }
    #[test]
    fn merge_recipe_preserves_operation_and_visual_editor_respects_input_filter() {
        let mut settings = ConversionSettings::default();
        settings.select_output_format(OutputFormat::Pdf);
        settings.pdf.action = PdfAction::Merge;
        let mut draft = Draft::new(settings);
        assert_eq!(draft.settings.pdf.action, PdfAction::Merge);
        draft.select_format(OutputFormat::Pdf);
        assert_eq!(draft.settings.pdf.action, PdfAction::Merge);
        assert!(allowed_inputs(&draft.settings).contains(&"pdf"));
        assert!(can_open_image_editor(&draft.settings, &["png".into()]));
        assert!(can_open_image_editor(&draft.settings, &["txt".into()]));
        assert!(!can_open_image_editor(&draft.settings, &["pdf".into()]));
        draft.select_format(OutputFormat::Png);
        assert!(!can_open_image_editor(&draft.settings, &["pdf".into()]));
        assert!(can_open_image_editor(&draft.settings, &["jpg".into()]));
    }
    #[test]
    fn visual_edit_refresh_updates_only_the_same_profile_draft() {
        let mut profiles = ConversionProfiles::default();
        let first = profiles
            .create("First".into(), false, ConversionSettings::default())
            .unwrap();
        let second = profiles
            .create("Second".into(), false, ConversionSettings::default())
            .unwrap();
        let mut manager = ProfileManager::default();
        manager.edit_profile(profiles.get(&first).unwrap());
        manager.draft.as_mut().unwrap().name = "Unsaved draft".into();
        manager.refresh_if_current(profiles.get(&second).unwrap());
        assert_eq!(manager.draft.as_ref().unwrap().name, "Unsaved draft");
        profiles
            .update(
                &first,
                "Updated first".into(),
                false,
                ConversionSettings::default(),
            )
            .unwrap();
        manager.refresh_if_current(profiles.get(&first).unwrap());
        assert_eq!(manager.draft.as_ref().unwrap().name, "Updated first");
        assert_eq!(
            manager.draft.as_ref().unwrap().id.as_deref(),
            Some(first.as_str())
        );
    }
    #[test]
    fn gif_with_ffmpeg_options_lists_only_sources_that_use_ffmpeg() {
        let mut settings = ConversionSettings::default();
        settings.select_output_format(OutputFormat::Gif);
        assert!(allowed_inputs(&settings).contains(&"png"));
        settings.media_options.insert("threads".into(), "2".into());
        let allowed = allowed_inputs(&settings);
        assert!(allowed.contains(&"mp4"));
        assert!(!allowed.contains(&"png"));
        assert!(!allowed.contains(&"gif"));
        assert!(allowed.iter().all(|extension| matches!(
            MediaKind::from_path(std::path::Path::new(&format!("input.{extension}"))),
            MediaKind::Audio | MediaKind::Video
        )));
    }
    #[test]
    fn duplicate_names_stay_bounded_and_saved_actions_own_the_recipe() {
        let name = duplicate_name(&"è".repeat(40));
        assert_eq!(name.chars().count(), 40);
        let mut draft = Draft::new(ConversionSettings::default());
        draft.name = "Audio".into();
        draft.settings.audio.bitrate_kbps = 160;
        let ProfileManagerAction::Save { settings, .. } = draft.save_action(false) else {
            panic!("save action")
        };
        draft.settings.audio.bitrate_kbps = 320;
        assert_eq!(settings.audio.bitrate_kbps, 160);
    }
}
