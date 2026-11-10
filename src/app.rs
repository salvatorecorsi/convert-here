use crate::{
    file_icons::FileIcons,
    image_editor::{self, EditorAction, ImageEditor},
    media_editor::{MediaEditor, MediaEditorAction},
    profile_manager::{ProfileManager, ProfileManagerAction},
    settings_ui::{self, EngineAction},
    theme::{self, FluentIcon},
};
use eframe::egui::{self, Color32, RichText};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use the_converter::platform::display_path;
use the_converter::{
    conversion_profiles::{ConversionProfile, ConversionProfiles, MAX_PREFERENCES_BYTES},
    engine,
    model::{
        ConversionSettings, CsvDelimiter, MediaKind, OutputFormat, PdfAction, VideoCodec,
        supported_formats,
    },
    output_preferences::OutputPreferences,
    platform,
    queue::{self, JobState as QuickJobState, QueueJob},
    runtime_install::{self, InstallPhase, InstallProgress, RuntimeEngine},
    tools::{ToolPaths, executable_available},
};
use the_converter::{
    i18n::{self, Language, tr},
    trf,
};

const PREFERENCES_SAVE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);

fn editable(path: &Path) -> bool {
    image_editor::editable(path)
        || matches!(
            MediaKind::from_path(path),
            MediaKind::Audio | MediaKind::Video
        )
}

pub fn launch(
    files: Vec<PathBuf>,
    target: Option<OutputFormat>,
    startup_warning: Option<String>,
) -> eframe::Result {
    launch_mode(files, target, startup_warning, false)
}

pub fn launch_custom(
    files: Vec<PathBuf>,
    target: Option<OutputFormat>,
    startup_warning: Option<String>,
) -> eframe::Result {
    launch_mode(files, target, startup_warning, true)
}

fn launch_mode(
    files: Vec<PathBuf>,
    target: Option<OutputFormat>,
    startup_warning: Option<String>,
    custom: bool,
) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Convert Here")
            .with_icon(app_icon())
            .with_inner_size([1060.0, 760.0])
            .with_min_inner_size([860.0, 620.0])
            .with_drag_and_drop(true),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Convert Here",
        options,
        Box::new(move |cc| {
            configure_style(&cc.egui_ctx);
            let mut app = ConverterApp::new(files, target);
            if custom {
                app.advanced = true;
                if app.items.len() == 1 && editable(&app.items[0].path) {
                    app.open_editor(Some(app.items[0].path.clone()), None);
                }
            }
            if let Some(warning) = startup_warning {
                app.notice = Some(match app.notice.take() {
                    Some(notice) => format!("{warning}\n{notice}"),
                    None => warning,
                });
            }
            Ok(Box::new(app))
        }),
    )
}

pub fn launch_queue_status(job_id: String, startup_warning: Option<String>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(tr(
                "Convert Here · Stato conversione",
                "Convert Here · Conversion status",
            ))
            .with_icon(app_icon())
            .with_inner_size([650.0, 470.0])
            .with_min_inner_size([500.0, 360.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        tr(
            "Convert Here · Stato conversione",
            "Convert Here · Conversion status",
        ),
        options,
        Box::new(move |cc| {
            configure_style(&cc.egui_ctx);
            let mut app = QueueStatusApp::new(job_id);
            app.action_error = startup_warning;
            Ok(Box::new(app))
        }),
    )
}

pub(crate) fn app_icon() -> egui::IconData {
    let icon = image::load_from_memory(include_bytes!("../assets/app.png"))
        .expect("L’icona incorporata deve essere valida")
        .into_rgba8();
    let (width, height) = icon.dimensions();
    egui::IconData {
        rgba: icon.into_raw(),
        width,
        height,
    }
}

fn configure_style(ctx: &egui::Context) {
    theme::configure(ctx);
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct SavedState {
    ui_language: Language,
    settings: ConversionSettings,
    overrides: BTreeMap<String, PathBuf>,
    enabled_output_formats: OutputPreferences,
    conversion_profiles: ConversionProfiles,
    default_profile_id: Option<String>,
    preset: usize,
}

#[derive(Clone, Deserialize)]
struct SavedReport {
    timestamp: String,
    format: String,
    successes: Vec<PathBuf>,
    errors: Vec<String>,
}

fn load_saved() -> SavedState {
    settings_path()
        .and_then(|path| {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)
                .ok()?
                .take(MAX_PREFERENCES_BYTES + 1)
                .read_to_end(&mut bytes)
                .ok()?;
            (bytes.len() as u64 <= MAX_PREFERENCES_BYTES).then_some(bytes)
        })
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn persist_preferences(
    path: &Path,
    saved: &SavedState,
    last_persist: &mut Option<std::time::Instant>,
    now: std::time::Instant,
    force: bool,
) -> std::io::Result<bool> {
    if !force
        && last_persist.is_some_and(|last| now.duration_since(last) < PREFERENCES_SAVE_INTERVAL)
    {
        return Ok(false);
    }
    // Failed writes are throttled too, while the dirty state remains pending.
    *last_persist = Some(now);
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::other(tr(
            "Cartella delle preferenze non disponibile",
            "Preferences folder is unavailable",
        ))
    })?;
    std::fs::create_dir_all(parent)?;
    use std::io::Write;
    let bytes = serde_json::to_vec_pretty(saved).map_err(std::io::Error::other)?;
    if bytes.len() as u64 > MAX_PREFERENCES_BYTES {
        return Err(std::io::Error::other(tr(
            "Le preferenze superano il limite di 256 KiB. Riduci il numero o la dimensione dei profili.",
            "Preferences exceed the 256 KiB limit. Reduce the number or size of your profiles.",
        )));
    }
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(&bytes)?;
    temporary.flush()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(true)
}

pub fn quick_configuration() -> (ConversionSettings, ToolPaths) {
    let saved = load_saved();
    let tools = ToolPaths::configured(&saved.overrides);
    (saved.settings, tools)
}

pub fn quick_profile_configuration(
    id: &str,
    revision: Option<u64>,
) -> anyhow::Result<(ConversionProfile, ToolPaths)> {
    profile_configuration(&load_saved(), id, revision)
}

fn profile_configuration(
    saved: &SavedState,
    id: &str,
    revision: Option<u64>,
) -> anyhow::Result<(ConversionProfile, ToolPaths)> {
    let profile = saved.conversion_profiles.resolve(id, revision)?;
    anyhow::ensure!(
        profile.show_in_menu,
        tr(
            "Il profilo non è attivo nel menu Converti in. Riapri il menu dopo averlo abilitato.",
            "This profile is hidden from Convert to. Enable it, then reopen the menu."
        )
    );
    anyhow::ensure!(
        saved
            .enabled_output_formats
            .contains(profile.settings.format),
        tr(
            "Il formato di uscita del profilo è disattivato. Abilitalo nelle Impostazioni.",
            "This profile's output format is disabled. Enable it in Settings."
        )
    );
    Ok((profile.clone(), ToolPaths::configured(&saved.overrides)))
}

pub fn report_path() -> Option<PathBuf> {
    queue::last_report_path()
}

fn load_report() -> Option<SavedReport> {
    report_path()
        .and_then(|path| std::fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

impl Default for SavedState {
    fn default() -> Self {
        Self {
            ui_language: Language::System,
            settings: ConversionSettings::default(),
            overrides: BTreeMap::new(),
            enabled_output_formats: OutputPreferences::default(),
            conversion_profiles: ConversionProfiles::default(),
            default_profile_id: None,
            preset: 1,
        }
    }
}

#[derive(Clone)]
enum JobState {
    Waiting,
    Running(f32, String),
    Complete(Vec<PathBuf>),
    Failed(String),
    Cancelled,
}

struct QueueItem {
    path: PathBuf,
    size: u64,
    state: JobState,
}

enum Event {
    Started(usize),
    Progress(usize, f32, String),
    Complete(usize, Vec<PathBuf>),
    Failed(usize, String),
    Cancelled(usize),
    End,
}

enum RuntimeInstallEvent {
    Progress(InstallProgress),
    Installed(RuntimeEngine, PathBuf),
    Finished(Result<(), String>),
}

struct RuntimeInstallTask {
    progress: InstallProgress,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<RuntimeInstallEvent>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum MainView {
    #[default]
    Files,
    History,
    Settings,
    Profiles,
}

pub struct ConverterApp {
    pdf_merge: Option<crate::pdf_merge_app::PdfMergeApp>,
    image_editor: Option<ImageEditor>,
    media_editor: Option<MediaEditor>,
    profile_manager: ProfileManager,
    view: MainView,
    conversion_started: Option<std::time::Instant>,
    conversion_format: OutputFormat,
    conversion_elapsed: std::time::Duration,
    file_icons: FileIcons,
    saved: SavedState,
    tools: ToolPaths,
    items: Vec<QueueItem>,
    gui_jobs: Vec<QueueJob>,
    gui_history: Option<crate::gui_history::AsyncHistory>,
    history_revision: u64,
    history_refresh: Option<std::time::Instant>,
    history_error: Option<String>,
    running: bool,
    cancel: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<Event>>,
    pending_runtime_install: Option<Vec<RuntimeEngine>>,
    runtime_install: Option<RuntimeInstallTask>,
    runtime_notice: Option<(String, bool)>,
    show_license: bool,
    advanced: bool,
    notice: Option<String>,
    settings_dirty: bool,
    language_dirty: bool,
    last_persist: Option<std::time::Instant>,
    quick_report: Option<SavedReport>,
    quick_queue: QuickQueueView,
}

impl ConverterApp {
    fn new(files: Vec<PathBuf>, target: Option<OutputFormat>) -> Self {
        let mut saved = load_saved();
        if target.is_none()
            && !files.is_empty()
            && let Some(profile) = saved
                .default_profile_id
                .as_deref()
                .and_then(|id| saved.conversion_profiles.get(id))
            && saved
                .enabled_output_formats
                .contains(profile.settings.format)
            && profile
                .validate_for_inputs(files.iter().map(PathBuf::as_path))
                .is_ok()
        {
            saved.settings = profile.settings.clone();
        }
        if let Some(format) = target {
            select_format_defaults(&mut saved.settings, format);
        }
        let tools = ToolPaths::configured(&saved.overrides);
        let gui_history = crate::gui_history::path()
            .map(crate::gui_history::AsyncHistory::open)
            .transpose();
        let (gui_history, history_error) = match gui_history {
            Ok(history) => (history, None),
            Err(error) => (None, Some(format!("{error:#}"))),
        };
        let mut app = Self {
            pdf_merge: None,
            image_editor: None,
            media_editor: None,
            profile_manager: ProfileManager::default(),
            view: MainView::Files,
            conversion_started: None,
            conversion_format: saved.settings.format,
            conversion_elapsed: std::time::Duration::ZERO,
            file_icons: FileIcons::default(),
            saved,
            tools,
            items: Vec::new(),
            gui_jobs: Vec::new(),
            gui_history,
            history_revision: 0,
            history_refresh: None,
            history_error,
            running: false,
            cancel: Arc::new(AtomicBool::new(false)),
            receiver: None,
            pending_runtime_install: None,
            runtime_install: None,
            runtime_notice: None,
            show_license: false,
            advanced: false,
            notice: None,
            settings_dirty: false,
            language_dirty: false,
            last_persist: None,
            quick_report: load_report(),
            quick_queue: QuickQueueView::default(),
        };
        app.add_files(files);
        app
    }

    fn persist(&mut self) {
        self.persist_with_force(false);
    }

    fn persist_with_force(&mut self, force: bool) {
        if let Some(path) = settings_path() {
            match persist_preferences(
                &path,
                &self.saved,
                &mut self.last_persist,
                std::time::Instant::now(),
                force,
            ) {
                Ok(true) => {
                    self.settings_dirty = false;
                    if self.language_dirty {
                        i18n::refresh_explorer_language();
                        self.language_dirty = false;
                    }
                }
                Ok(false) => {}
                Err(err) => {
                    self.notice = Some(trf!(
                        "Impossibile salvare le preferenze: {err}",
                        "Unable to save preferences: {err}"
                    ));
                }
            }
        }
    }

    fn profile_manager_action(&mut self, action: ProfileManagerAction) {
        if let ProfileManagerAction::Export { id } = &action {
            let result = (|| -> anyhow::Result<()> {
                use std::io::Write;
                let bytes = self.saved.conversion_profiles.export_json(id.as_deref())?;
                let name = if id.is_some() {
                    "ConvertHere-profile.json"
                } else {
                    "ConvertHere-profiles.json"
                };
                let Some(mut path) = rfd::FileDialog::new()
                    .set_file_name(name)
                    .add_filter("JSON", &["json"])
                    .save_file()
                else {
                    return Ok(());
                };
                if path.extension().is_none() {
                    path.set_extension("json");
                }
                anyhow::ensure!(
                    path.extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("json")),
                    tr("Scegli un file .json.", "Choose a .json file.")
                );
                let parent = path.parent().unwrap_or_else(|| Path::new("."));
                let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
                temporary.write_all(&bytes)?;
                temporary.as_file().sync_all()?;
                temporary.persist(&path).map_err(|error| error.error)?;
                self.profile_manager
                    .notify(tr("Profili esportati.", "Profiles exported.").into(), false);
                Ok(())
            })();
            if let Err(error) = result {
                self.profile_manager.notify(format!("{error:#}"), true);
            }
            return;
        }
        let editor_id = match &action {
            ProfileManagerAction::EditImage(id) => Some(id.clone()),
            _ => None,
        };
        if let Some(id) = editor_id {
            self.open_profile_editor(&id);
            return;
        }
        let previous = self.saved.clone();
        let previous_dirty = self.settings_dirty;
        let mut saved_id = None;
        let mut open_editor = false;
        let result = (|| -> anyhow::Result<String> {
            match action {
                ProfileManagerAction::Save {
                    id,
                    name,
                    show_in_menu,
                    settings,
                    input_extensions,
                    open_image_editor,
                } => {
                    let id = if let Some(id) = id {
                        self.saved.conversion_profiles.update_with_inputs(
                            &id,
                            name,
                            show_in_menu,
                            *settings,
                            input_extensions,
                        )?;
                        id
                    } else {
                        self.saved.conversion_profiles.create_with_inputs(
                            name,
                            show_in_menu,
                            *settings,
                            input_extensions,
                        )?
                    };
                    saved_id = Some(id);
                    open_editor = open_image_editor;
                    Ok(tr("Profilo salvato.", "Profile saved.").into())
                }
                ProfileManagerAction::SetVisibility(id, visible) => {
                    self.saved
                        .conversion_profiles
                        .set_show_in_menu(&id, visible)?;
                    Ok(String::new())
                }
                ProfileManagerAction::Delete(id) => {
                    self.saved.conversion_profiles.remove(&id);
                    if self.saved.default_profile_id.as_deref() == Some(id.as_str()) {
                        self.saved.default_profile_id = None;
                    }
                    Ok(tr("Profilo eliminato.", "Profile deleted.").into())
                }
                ProfileManagerAction::MakeDefault(id) => {
                    let profile = self.saved.conversion_profiles.resolve(&id, None)?;
                    self.saved.settings = profile.settings.clone();
                    self.saved.default_profile_id = Some(id);
                    self.saved.preset = 3;
                    self.sync_output_choice();
                    Ok(tr("Predefinito aggiornato.", "Default updated.").into())
                }
                ProfileManagerAction::ClearDefault => {
                    self.saved.default_profile_id = None;
                    Ok(tr("Predefinito rimosso.", "Default cleared.").into())
                }
                ProfileManagerAction::Import(bytes) => {
                    let imported = self.saved.conversion_profiles.import_json(&bytes)?;
                    Ok(trf!(
                        "{} profili importati · {} fuori dal menu",
                        "{} profiles imported · {} hidden from menu",
                        imported.imported,
                        imported.hidden_from_menu
                    ))
                }
                ProfileManagerAction::Export { .. } | ProfileManagerAction::EditImage(_) => {
                    unreachable!()
                }
            }
        })();
        match result {
            Ok(message) => {
                self.settings_dirty = true;
                self.persist_with_force(true);
                if self.settings_dirty {
                    self.saved = previous;
                    self.settings_dirty = previous_dirty;
                    self.profile_manager.notify(
                        self.notice.clone().unwrap_or_else(|| {
                            tr("Salvataggio non riuscito.", "Could not save.").into()
                        }),
                        true,
                    );
                    return;
                }
                if let Some(id) = saved_id {
                    self.profile_manager.saved(id.clone());
                    if open_editor {
                        self.open_profile_editor(&id);
                    }
                }
                if !message.is_empty() {
                    self.profile_manager.notify(message, false);
                }
            }
            Err(error) => {
                self.saved = previous;
                self.settings_dirty = previous_dirty;
                self.profile_manager.notify(format!("{error:#}"), true);
            }
        }
    }

    fn open_profile_editor(&mut self, id: &str) {
        if let Some(profile) = self.saved.conversion_profiles.get(id).cloned() {
            let input = self
                .items
                .iter()
                .find(|item| {
                    editable(&item.path)
                        && profile.validate_for_inputs([item.path.as_path()]).is_ok()
                })
                .map(|item| item.path.clone());
            self.open_editor(input, Some(profile));
        }
    }

    fn open_editor(&mut self, input: Option<PathBuf>, profile: Option<ConversionProfile>) {
        if self.running {
            return;
        }
        let editing_profile = profile.is_some();
        if profile.as_ref().is_some_and(|profile| {
            !self
                .saved
                .enabled_output_formats
                .contains(profile.settings.format)
        }) {
            self.notice=Some(tr("Il formato di questo profilo è disattivato. Riabilitalo in Formati di uscita prima di modificarne le opzioni.", "This profile's output format is disabled. Enable it in Output formats before editing its options.").into());
            return;
        }
        let profile = profile.or_else(|| {
            let input = input.as_ref()?;
            self.saved
                .default_profile_id
                .as_deref()
                .and_then(|id| self.saved.conversion_profiles.get(id))
                .filter(|profile| {
                    self.saved
                        .enabled_output_formats
                        .contains(profile.settings.format)
                })
                .filter(|profile| profile.validate_for_inputs([input.as_path()]).is_ok())
                .cloned()
        });
        let text = input.as_ref().is_some_and(|path| {
            path.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
        }) || (input.is_none()
            && profile.as_ref().is_none_or(|profile| {
                profile.settings.image.page.is_some() && profile.settings.image.edit.is_none()
            }));
        let settings = profile.as_ref().map_or_else(
            || self.saved.settings.clone(),
            |profile| profile.settings.clone(),
        );
        let formats = self
            .saved
            .enabled_output_formats
            .filter(OutputFormat::ALL)
            .collect();
        if let Some(input) = input.as_ref().filter(|path| {
            matches!(
                MediaKind::from_path(path),
                MediaKind::Audio | MediaKind::Video
            )
        }) {
            let mut editor = MediaEditor::new(input.clone(), settings, self.tools.clone(), formats);
            if editing_profile && let Some(profile) = profile {
                editor.editing_profile(profile.id, profile.name, profile.show_in_menu);
            }
            self.media_editor = Some(editor);
            return;
        }
        let mut editor = ImageEditor::new(input, text, settings, self.tools.clone(), formats);
        if editing_profile && let Some(profile) = profile {
            editor.editing_profile(profile.id, profile.name, profile.show_in_menu);
        }
        self.image_editor = Some(editor);
    }

    fn apply_profile(&mut self, profile: &ConversionProfile) {
        if !self.items.is_empty()
            && let Err(error) =
                profile.validate_for_inputs(self.items.iter().map(|item| item.path.as_path()))
        {
            self.notice = Some(format!("{error:#}"));
            return;
        }
        if !self
            .saved
            .enabled_output_formats
            .contains(profile.settings.format)
        {
            self.notice = Some(
                tr(
                    "Abilita il formato di uscita del profilo nelle Impostazioni.",
                    "Enable this profile's output format in Settings.",
                )
                .into(),
            );
            return;
        }
        self.saved.settings = profile.settings.clone();
        self.saved.preset = 3;
        self.settings_dirty = true;
        self.persist_with_force(true);
    }

    fn editor_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::Back => {
                self.image_editor = None;
                self.media_editor = None;
            }
            EditorAction::SetDefaults(settings) => {
                self.saved.settings = settings;
                self.saved.preset = 3;
                self.saved.default_profile_id = None;
                self.settings_dirty = true;
                self.persist_with_force(true);
                let error = self.settings_dirty;
                let message = if error {
                    self.notice.clone().unwrap_or_else(|| {
                        tr(
                            "Impossibile salvare le impostazioni.",
                            "Could not save the settings.",
                        )
                        .into()
                    })
                } else {
                    tr(
                        "Impostazioni predefinite salvate.",
                        "Default settings saved.",
                    )
                    .into()
                };
                if let Some(editor) = &mut self.image_editor {
                    editor.notify(message.clone(), error);
                }
                if let Some(editor) = &mut self.media_editor {
                    editor.notify(message, error);
                }
            }
            EditorAction::SaveProfile {
                id,
                name,
                show_in_menu,
                mut settings,
            } => {
                let previous = self.saved.clone();
                let previous_dirty = self.settings_dirty;
                if id
                    .as_deref()
                    .and_then(|id| previous.conversion_profiles.get(id))
                    .is_some_and(|profile| profile.settings.pdf.action == PdfAction::Merge)
                    && settings.format == OutputFormat::Pdf
                {
                    settings.pdf.action = PdfAction::Merge;
                }
                let old_inputs = id
                    .as_deref()
                    .and_then(|id| previous.conversion_profiles.get(id))
                    .map(|profile| profile.input_extensions.clone())
                    .unwrap_or_default();
                let allowed = crate::profile_manager::allowed_inputs(&settings);
                let inputs: Vec<_> = old_inputs
                    .iter()
                    .filter(|extension| allowed.contains(&extension.as_str()))
                    .cloned()
                    .collect();
                if !old_inputs.is_empty() && inputs.is_empty() {
                    if let Some(editor) = &mut self.image_editor {
                        editor.notify(tr(
                            "Queste modifiche non sono compatibili con gli ingressi del profilo. Modifica gli ingressi nella scheda Profili.",
                            "These edits are incompatible with the profile's inputs. Change its allowed inputs in Profiles.",
                        ).into(), true);
                    }
                    if let Some(editor) = &mut self.media_editor {
                        editor.notify(
                            tr(
                                "Modifica gli ingressi del profilo nella scheda Profili.",
                                "Change this profile's allowed inputs in Profiles.",
                            )
                            .into(),
                            true,
                        );
                    }
                    return;
                }
                let narrowed = inputs.len() != old_inputs.len();
                let result = if let Some(id) = id {
                    self.saved
                        .conversion_profiles
                        .update_with_inputs(&id, name.clone(), show_in_menu, settings, inputs)
                        .map(|()| id)
                } else {
                    self.saved
                        .conversion_profiles
                        .create(name.clone(), show_in_menu, settings)
                };
                match result {
                    Ok(id) => {
                        self.settings_dirty = true;
                        self.persist_with_force(true);
                        let error = self.settings_dirty;
                        let message = if error {
                            self.notice.clone().unwrap_or_else(|| {
                                tr(
                                    "Impossibile salvare il profilo.",
                                    "Could not save the profile.",
                                )
                                .into()
                            })
                        } else {
                            let mut message =
                                trf!("Profilo «{}» salvato.", "Profile ‘{}’ saved.", name.trim());
                            if narrowed {
                                message.push_str(tr(
                                    " Ingressi limitati ai formati compatibili con le modifiche.",
                                    " Inputs restricted to formats compatible with the edits.",
                                ));
                            }
                            message
                        };
                        if error {
                            self.saved = previous;
                            self.settings_dirty = previous_dirty;
                        } else if let Some(profile) = self.saved.conversion_profiles.get(&id) {
                            self.profile_manager.refresh_if_current(profile);
                        }
                        if let Some(editor) = &mut self.image_editor {
                            if !error {
                                editor.editing_profile(
                                    id.clone(),
                                    name.trim().into(),
                                    show_in_menu,
                                );
                            }
                            editor.notify(message.clone(), error);
                        }
                        if let Some(editor) = &mut self.media_editor {
                            if !error {
                                editor.editing_profile(id, name.trim().into(), show_in_menu);
                            }
                            editor.notify(message, error);
                        }
                    }
                    Err(error) => {
                        if let Some(editor) = &mut self.image_editor {
                            editor.notify(format!("{error:#}"), true);
                        }
                        if let Some(editor) = &mut self.media_editor {
                            editor.notify(format!("{error:#}"), true);
                        }
                    }
                }
            }
        }
    }

    fn add_files(&mut self, paths: Vec<PathBuf>) {
        let mut rejected = 0;
        for path in paths {
            if !path.is_file() || supported_formats(MediaKind::from_path(&path)).is_empty() {
                rejected += 1;
                continue;
            }
            let path = path.canonicalize().unwrap_or(path);
            if !self.items.iter().any(|item| item.path == path) {
                let size = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
                self.items.push(QueueItem {
                    path,
                    size,
                    state: JobState::Waiting,
                });
            }
        }
        if rejected > 0 {
            self.notice = Some(trf!(
                "{rejected} elementi ignorati: cartelle o formati non supportati.",
                "Ignored items: {rejected} (folders or unsupported formats)."
            ));
        }
        let common = self.common_formats();
        if !common.is_empty() && !common.contains(&self.saved.settings.format) {
            select_format_defaults(&mut self.saved.settings, common[0]);
            self.settings_dirty = true;
        }
        if (self.saved.settings.pdf.action == PdfAction::Split
            && self
                .items
                .iter()
                .any(|item| MediaKind::from_path(&item.path) != MediaKind::Pdf))
            || (self.saved.settings.pdf.action == PdfAction::Merge
                && (self.items.len() < 2 || !common.contains(&OutputFormat::Pdf)))
        {
            self.saved.settings.pdf.action = PdfAction::Compress;
            self.settings_dirty = true;
        }
    }

    fn common_formats(&self) -> Vec<OutputFormat> {
        self.saved
            .enabled_output_formats
            .filter(the_converter::model::common_formats(
                self.items.iter().map(|item| item.path.as_path()),
            ))
            .collect()
    }

    fn sync_output_choice(&mut self) {
        let available = if self.items.is_empty() {
            self.saved
                .enabled_output_formats
                .filter(OutputFormat::ALL)
                .collect()
        } else {
            self.common_formats()
        };
        if !available.contains(&self.saved.settings.format)
            && let Some(format) = available.first()
        {
            select_format_defaults(&mut self.saved.settings, *format);
            self.saved.settings.audio.sample_rate = 0;
        }
    }

    fn begin_runtime_install(&mut self, engines: Vec<RuntimeEngine>, ctx: &egui::Context) {
        if self.runtime_install.is_some() || engines.is_empty() {
            return;
        }
        let engine = engines[0];
        let cancel = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::channel();
        self.runtime_notice = None;
        self.runtime_install = Some(RuntimeInstallTask {
            progress: InstallProgress {
                engine,
                phase: InstallPhase::Connecting,
                fraction: 0.0,
                downloaded_bytes: 0,
                total_bytes: engine.manifest().download_bytes,
                message: tr(
                    "Connessione al distributore…",
                    "Connecting to the distributor…",
                )
                .into(),
            },
            cancel: cancel.clone(),
            receiver,
        });
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = (|| {
                for engine in engines {
                    let directory = runtime_install::install(engine, &cancel, |progress| {
                        let _ = sender.send(RuntimeInstallEvent::Progress(progress));
                        ctx.request_repaint();
                    })
                    .map_err(|error| format!("{error:#}"))?;
                    let _ = sender.send(RuntimeInstallEvent::Installed(engine, directory));
                    ctx.request_repaint();
                }
                Ok(())
            })();
            let _ = sender.send(RuntimeInstallEvent::Finished(result));
            ctx.request_repaint();
        });
    }

    fn receive_runtime_events(&mut self) {
        let mut installed = Vec::new();
        let mut finished = None;
        if let Some(task) = &mut self.runtime_install {
            loop {
                match task.receiver.try_recv() {
                    Ok(RuntimeInstallEvent::Progress(progress)) => task.progress = progress,
                    Ok(RuntimeInstallEvent::Installed(engine, directory)) => {
                        installed.push((engine, directory));
                    }
                    Ok(RuntimeInstallEvent::Finished(result)) => finished = Some(result),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        if finished.is_none() {
                            finished = Some(Err(tr(
                                "L’installazione si è interrotta. Puoi riprovare.",
                                "Installation was interrupted. You can try again.",
                            )
                            .into()));
                        }
                        break;
                    }
                }
            }
        }
        if !installed.is_empty() {
            for (engine, directory) in installed {
                for (key, path) in engine.tool_paths(&directory) {
                    if executable_available(&path)
                        || key == "whisper-model" && the_converter::tools::model_available(&path)
                    {
                        self.saved.overrides.insert(key.into(), path);
                    }
                }
            }
            self.tools = ToolPaths::configured(&self.saved.overrides);
            self.settings_dirty = true;
            self.persist_with_force(true);
        }
        if let Some(result) = finished {
            self.runtime_install = None;
            self.runtime_notice = Some(match result {
                Ok(()) => (
                    tr("Installazione completata.", "Installation complete.").into(),
                    false,
                ),
                Err(error) => (error, true),
            });
        }
    }

    fn output_format_settings(&mut self, ui: &mut egui::Ui) {
        let before = self.saved.enabled_output_formats.clone();
        settings_ui::formats(ui, &mut self.saved.enabled_output_formats);
        if self.saved.enabled_output_formats != before {
            self.sync_output_choice();
            self.settings_dirty = true;
            self.persist_with_force(true);
        }
    }

    fn runtime_install_progress(&mut self, ui: &mut egui::Ui) {
        if let Some((message, error)) = &self.runtime_notice {
            info_bar(
                ui,
                message,
                if *error {
                    InfoKind::Warning
                } else {
                    InfoKind::Info
                },
            );
            if ui
                .small_button(tr("Chiudi messaggio", "Dismiss message"))
                .clicked()
            {
                self.runtime_notice = None;
            }
        }
        if let Some(task) = &self.runtime_install {
            theme::show_card(ui, |ui| {
                ui.label(theme::heading(
                    task.progress.engine.manifest().name,
                    theme::BODY,
                ));
                ui.label(&task.progress.message);
                ui.add(
                    egui::ProgressBar::new(task.progress.fraction.clamp(0.0, 1.0))
                        .desired_height(3.0),
                );
                if matches!(task.progress.phase, InstallPhase::Downloading) {
                    ui.label(format!(
                        "{:.1} / {:.1} MB",
                        task.progress.downloaded_bytes as f64 / 1_000_000.0,
                        task.progress.total_bytes as f64 / 1_000_000.0
                    ));
                }
                let cancelling = task.cancel.load(Ordering::Relaxed);
                if ui
                    .add_enabled(
                        !cancelling,
                        egui::Button::new(if cancelling {
                            tr("Annullamento…", "Cancelling…")
                        } else {
                            tr("Annulla installazione", "Cancel installation")
                        }),
                    )
                    .clicked()
                {
                    task.cancel.store(true, Ordering::Relaxed);
                }
            });
        }
    }

    fn runtime_install_confirmation(&mut self, ctx: &egui::Context) {
        let Some(engines) = self.pending_runtime_install.clone() else {
            return;
        };
        let mut confirmed = false;
        let mut dismissed = false;
        egui::Modal::new(egui::Id::new("confirm_runtime_install")).show(ctx, |ui| {
            ui.set_width(490.0);
            ui.label(theme::heading(tr("Installa motori opzionali", "Install optional engines"), theme::SUBTITLE));
            ui.label(tr("Scarica i motori dalle distribuzioni indicate e li verifica prima di estrarli. Il download richiede Internet; i tuoi file da convertire restano sul PC.", "Downloads engines from the listed distributions and verifies them before extraction. The download requires Internet; the files you convert stay on your PC."));
            ui.add_space(8.0);
            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
            for engine in &engines {
                let manifest = engine.manifest();
                ui.label(format!("{} {} · {:.1} MB · {}", manifest.name, manifest.version, manifest.download_bytes as f64 / 1_000_000.0, manifest.license_label()));
                ui.label(trf!("Spazio dopo l’estrazione: {:.1} MB", "Space after extraction: {:.1} MB", manifest.expanded_bytes as f64 / 1_000_000.0));
                let source = match engine {
                    RuntimeEngine::Ffmpeg => "https://www.gyan.dev/ffmpeg/builds/",
                    RuntimeEngine::Poppler => "https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0",
                    RuntimeEngine::Tesseract => "https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-tesseract-ocr",
                    RuntimeEngine::Libheif => "https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-libheif",
                    RuntimeEngine::Whisper => "https://github.com/ggml-org/whisper.cpp",
                };
                if ui.link(tr("Distribuzione e licenze", "Distribution and licenses")).on_hover_text(source).clicked()
                    && let Err(error) = platform::open_website(source)
                {
                    self.notice = Some(format!("{error:#}"));
                }
                if *engine == RuntimeEngine::Tesseract {
                    ui.label(tr("Lingue incluse: italiano e inglese", "Included languages: Italian and English"));
                }
                if *engine == RuntimeEngine::Whisper {
                    ui.label(tr("Modello Tiny multilingue incluso (77,7 MB). Per leggere audio e video serve anche FFmpeg.", "Includes the multilingual Tiny model (77.7 MB). FFmpeg is also required to read audio and video."));
                }
                egui::CollapsingHeader::new(tr("File, provenienza e licenze", "Files, sources and licenses"))
                    .id_salt(format!("runtime-artifacts-{engine:?}"))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                            for artifact in engine.artifacts() {
                                ui.label(format!("{} {} · {:.1} MB", artifact.name, artifact.version, artifact.download_bytes as f64 / 1_000_000.0));
                                ui.add(egui::Label::new(&artifact.license).wrap());
                                ui.add(egui::Label::new(&artifact.source_url).wrap());
                                ui.add(egui::Label::new(format!("SHA-256: {}", artifact.sha256)).wrap());
                                ui.separator();
                            }
                        });
                    });
            }
            });
            if let Some(directory) = the_converter::tools::user_tools_dir() {
                ui.add_space(8.0);
                ui.label(tr("Cartella di installazione:", "Installation folder:"));
                ui.add(egui::Label::new(display_path(&directory)).wrap());
            }
            ui.label(RichText::new(tr("Le licenze dei motori restano incluse. Non occorrono abbonamenti o pagamenti.", "Engine licenses remain included. No subscription or payment is required.")).small().weak());
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                confirmed = ui.button(tr("Scarica e installa", "Download and install")).clicked();
                dismissed = ui.button(tr("Annulla", "Cancel")).clicked();
            });
        });
        if confirmed {
            self.pending_runtime_install = None;
            self.begin_runtime_install(engines, ctx);
        } else if dismissed || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.pending_runtime_install = None;
        }
    }

    fn license_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_license {
            return;
        }
        let mut close = false;
        egui::Modal::new(egui::Id::new("application_license")).show(ctx, |ui| {
            ui.set_width(600.0_f32.min((ctx.content_rect().width() - 70.0).max(300.0)));
            ui.label(theme::heading(
                tr("GNU GPL versione 3", "GNU GPL version 3"),
                theme::SUBTITLE,
            ));
            ui.label("Convert Here · Copyright © 2026 Salvatore Corsi · GPL-3.0-only");
            egui::ScrollArea::vertical()
                .id_salt("application_license_scroll")
                .max_height((ctx.content_rect().height() - 230.0).max(180.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(include_str!("../LICENSE"))
                                .monospace()
                                .size(12.0),
                        )
                        .wrap(),
                    );
                });
            ui.horizontal(|ui| {
                if ui.button(tr("Copia testo", "Copy text")).clicked() {
                    ctx.copy_text(include_str!("../LICENSE").into());
                }
                close = ui.button(tr("Chiudi", "Close")).clicked();
            });
        });
        if close || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.show_license = false;
        }
    }

    fn prerequisite(&self) -> Option<String> {
        if self.items.is_empty() {
            return None;
        }
        if self
            .saved
            .settings
            .video
            .trim_end
            .is_some_and(|end| end <= self.saved.settings.video.trim_start.unwrap_or(0.0))
        {
            return Some(
                tr(
                    "La fine dell’intervallo deve essere successiva all’inizio.",
                    "The end of the range must be after its start.",
                )
                .into(),
            );
        }
        if self.common_formats().is_empty() {
            if !the_converter::model::common_formats(
                self.items.iter().map(|item| item.path.as_path()),
            )
            .is_empty()
            {
                return Some(tr("Per questi file non ci sono formati di uscita attivi. Abilitali nelle Impostazioni; i formati dei file in ingresso restano supportati.", "No output formats are enabled for these files. Enable them in Settings; all input file formats remain supported.").into());
            }
            return Some(tr("Questi file richiedono formati di uscita diversi. Crea una coda con file compatibili.", "These files require different output formats. Create a queue with compatible files.").into());
        }
        if !self.common_formats().contains(&self.saved.settings.format) {
            return Some(trf!(
                "Il formato {} non è compatibile con tutti i file. Scegli un altro formato di uscita.",
                "The {} format is not compatible with every file. Choose another output format.",
                self.saved.settings.format.label()
            ));
        }
        if self.saved.settings.format == OutputFormat::Pdf
            && self.saved.settings.pdf.action == PdfAction::Merge
            && (self.items.len() < 2 || !self.common_formats().contains(&OutputFormat::Pdf))
        {
            return Some(tr("Per PDF (Merge) aggiungi almeno due file convertibili in PDF. Potrai ordinare le singole pagine prima di creare il PDF unico.", "For PDF (Merge), add at least two files that can be converted to PDF. You can arrange individual pages before creating the merged PDF.").into());
        }
        for item in &self.items {
            if let Some(missing) = self
                .tools
                .missing_for_input(&item.path, &self.saved.settings)
            {
                return Some(missing);
            }
        }
        if self
            .saved
            .settings
            .format
            .audio_sample_rates()
            .is_some_and(|rates| !rates.contains(&self.saved.settings.audio.sample_rate))
        {
            return Some(tr("Il formato scelto richiede una frequenza audio diversa. Scegli una frequenza compatibile nelle impostazioni.", "The selected format requires a different audio sample rate. Choose a compatible rate in the settings.").into());
        }
        None
    }

    fn start(&mut self, ctx: &egui::Context) {
        if self.running || self.items.is_empty() {
            return;
        }
        if let Some(message) = self.prerequisite() {
            self.notice = Some(message);
            return;
        }
        self.persist();
        if self.saved.settings.format == OutputFormat::Pdf
            && self.saved.settings.pdf.action == PdfAction::Merge
        {
            let paths = self.items.iter().map(|item| item.path.clone()).collect();
            self.pdf_merge = Some(crate::pdf_merge_app::PdfMergeApp::new(
                paths,
                self.saved.settings.clone(),
                self.tools.clone(),
                None,
            ));
            return;
        }
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        let paths: Vec<_> = self.items.iter().map(|item| item.path.clone()).collect();
        for item in &mut self.items {
            item.state = JobState::Waiting;
        }
        let settings = self.saved.settings.clone();
        let tools = self.tools.clone();
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.running = true;
        self.conversion_format = settings.format;
        self.conversion_started = Some(std::time::Instant::now());
        self.conversion_elapsed = std::time::Duration::ZERO;
        self.notice = None;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            for (index, path) in paths.into_iter().enumerate() {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let _ = sender.send(Event::Started(index));
                ctx.request_repaint();
                let result = engine::convert(&path, &settings, &tools, &cancel, |progress| {
                    let _ =
                        sender.send(Event::Progress(index, progress.fraction, progress.message));
                    ctx.request_repaint();
                });
                let event = match result {
                    Ok(outputs) => Event::Complete(index, outputs),
                    Err(_) if cancel.load(Ordering::Relaxed) => Event::Cancelled(index),
                    Err(error) => Event::Failed(index, format!("{error:#}")),
                };
                let _ = sender.send(event);
                ctx.request_repaint();
            }
            let _ = sender.send(Event::End);
            ctx.request_repaint();
        });
    }

    fn receive_events(&mut self) {
        let events: Vec<_> = self
            .receiver
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                Event::Started(index) => {
                    self.items[index].state =
                        JobState::Running(0.0, tr("Preparazione…", "Preparing…").into())
                }
                Event::Progress(index, fraction, message) => {
                    self.items[index].state = JobState::Running(fraction.clamp(0.0, 1.0), message)
                }
                Event::Complete(index, outputs) => {
                    self.items[index].state = JobState::Complete(outputs);
                    self.record_history(index);
                }
                Event::Failed(index, error) => {
                    self.items[index].state = JobState::Failed(error);
                    self.record_history(index);
                }
                Event::Cancelled(index) => {
                    self.items[index].state = JobState::Cancelled;
                    self.record_history(index);
                }
                Event::End => {
                    self.running = false;
                    if let Some(started) = self.conversion_started.take() {
                        self.conversion_elapsed = started.elapsed();
                    }
                    self.receiver = None;
                    if self.cancel.load(Ordering::Relaxed) {
                        let waiting: Vec<_> = self
                            .items
                            .iter()
                            .enumerate()
                            .filter_map(|(index, item)| {
                                matches!(item.state, JobState::Waiting | JobState::Running(..))
                                    .then_some(index)
                            })
                            .collect();
                        for index in waiting {
                            self.items[index].state = JobState::Cancelled;
                            self.record_history(index);
                        }
                    }
                }
            }
        }
    }

    fn apply_preset(&mut self, preset: usize) {
        self.saved.preset = preset;
        let (bitrate, crf, quality) = match preset {
            0 => (128, 28, 75),
            2 => (320, 18, 95),
            _ => (192, 23, 90),
        };
        self.saved.settings.audio.bitrate_kbps = bitrate;
        self.saved.settings.video.crf = crf;
        self.saved.settings.image.quality = quality;
        self.settings_dirty = true;
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            navigation_tabs(ui, &mut self.view);
        });
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui) {
        let colors = theme::palette(ui.ctx());
        ui.label(theme::heading(tr("Opzioni", "Options"), theme::SUBTITLE));
        ui.add_space(8.0);
        let profiles: Vec<_> = self
            .saved
            .conversion_profiles
            .iter()
            .filter(|profile| {
                self.saved
                    .enabled_output_formats
                    .contains(profile.settings.format)
                    && (self.items.is_empty()
                        || profile
                            .validate_for_inputs(self.items.iter().map(|item| item.path.as_path()))
                            .is_ok())
            })
            .cloned()
            .collect();
        let mut selected_profile = None;
        if !profiles.is_empty() {
            egui::ComboBox::from_id_salt("conversion_profile_choice")
                .selected_text(tr("Applica un profilo…", "Apply a profile…"))
                .show_ui(ui, |ui| {
                    for profile in profiles {
                        if ui.selectable_label(false, &profile.name).clicked() {
                            selected_profile = Some(profile);
                        }
                    }
                });
        }
        if let Some(profile) = selected_profile {
            self.apply_profile(&profile);
        }
        if ui
            .small_button(tr("Salva profilo…", "Save profile…"))
            .clicked()
        {
            self.profile_manager
                .new_profile(self.saved.settings.clone());
            self.view = MainView::Profiles;
        }
        if let Some(path) = self
            .items
            .iter()
            .find(|item| editable(&item.path))
            .map(|item| item.path.clone())
            && ui
                .button(match MediaKind::from_path(&path) {
                    MediaKind::Image => tr("Modifica immagine…", "Edit image…"),
                    MediaKind::Audio | MediaKind::Video => tr("Taglia / guadagno…", "Trim / gain…"),
                    _ => tr("Impagina testo…", "Lay out text…"),
                })
                .clicked()
        {
            self.open_editor(Some(path), None);
        }
        ui.add_space(8.0);
        let formats = self.common_formats();
        let before = self.saved.settings.format;
        setting_card(
            ui,
            tr("Formato", "Format"),
            tr("Scegli il file da creare", "Choose the file to create"),
            |ui| {
                ui.add_enabled_ui(!formats.is_empty(), |ui| {
                    egui::ComboBox::from_id_salt("target_format")
                        .width(142.0)
                        .selected_text(if self.items.is_empty() {
                            tr("Aggiungi un file", "Add a file")
                        } else if formats.is_empty() {
                            tr("Nessuna uscita attiva", "No output formats enabled")
                        } else {
                            self.saved.settings.format.label()
                        })
                        .show_ui(ui, |ui| {
                            for format in formats {
                                let label = if format == OutputFormat::Txt
                                    && self.items.iter().all(|item| {
                                        matches!(
                                            MediaKind::from_path(&item.path),
                                            MediaKind::Audio | MediaKind::Video
                                        )
                                    }) {
                                    tr("TXT · Trascrizione", "TXT · Transcription")
                                } else if format == OutputFormat::Txt
                                    && self.items.iter().any(|item| {
                                        MediaKind::from_path(&item.path) == MediaKind::Image
                                    })
                                {
                                    tr("TXT · OCR locale", "TXT · Local OCR")
                                } else if format == OutputFormat::Pdf
                                    && self.items.iter().all(|item| {
                                        MediaKind::from_path(&item.path) == MediaKind::Pdf
                                    })
                                {
                                    tr("PDF · operazioni", "PDF · operations")
                                } else {
                                    format.label()
                                };
                                ui.selectable_value(&mut self.saved.settings.format, format, label);
                            }
                        });
                });
            },
        );
        if ui
            .link(tr("Scegli i formati di uscita…", "Choose output formats…"))
            .clicked()
        {
            self.view = MainView::Settings;
        }
        if before != self.saved.settings.format {
            self.saved.settings.media_options.clear();
            self.saved
                .settings
                .select_output_format(self.saved.settings.format);
            self.settings_dirty = true;
            if self
                .saved
                .settings
                .format
                .audio_sample_rates()
                .is_some_and(|rates| !rates.contains(&self.saved.settings.audio.sample_rate))
            {
                self.saved.settings.audio.sample_rate = 0;
            }
        }
        ui.add_space(6.0);
        let quality = quality_applicable(self.saved.settings.format)
            && !(self.saved.settings.format == OutputFormat::Pdf
                && !self.items.is_empty()
                && self
                    .items
                    .iter()
                    .all(|item| MediaKind::from_path(&item.path) == MediaKind::Pdf));
        if quality {
            let detail = match self.saved.preset {
                0 => tr("File più piccoli", "Smaller files"),
                2 => tr("Più dettaglio", "More detail"),
                3 => tr("Valori personalizzati", "Custom values"),
                _ => tr(
                    "Qualità e dimensioni bilanciate",
                    "Balanced quality and size",
                ),
            };
            setting_card(ui, tr("Qualità", "Quality"), detail, |ui| {
                ui.vertical(|ui| {
                    for (index, title) in [
                        (0, tr("Leggero", "Compact")),
                        (1, tr("Bilanciato", "Balanced")),
                        (2, tr("Alta qualità", "High quality")),
                    ] {
                        if ui.radio(self.saved.preset == index, title).clicked() {
                            self.apply_preset(index);
                        }
                    }
                });
            });
            ui.add_space(6.0);
        }
        theme::show_card(ui, |ui| {
            ui.label(theme::heading(
                tr("Destinazione", "Destination"),
                theme::BODY,
            ));
            let mut beside = self.saved.settings.output_dir.is_none();
            if ui
                .radio_value(
                    &mut beside,
                    true,
                    tr("Accanto all’originale", "Next to the original"),
                )
                .changed()
            {
                self.saved.settings.output_dir = None;
                self.settings_dirty = true;
            }
            if ui
                .radio_value(&mut beside, false, tr("Cartella a scelta", "Custom folder"))
                .clicked()
                && self.saved.settings.output_dir.is_none()
                && let Some(path) = rfd::FileDialog::new()
                    .set_title(tr("Cartella di destinazione", "Output folder"))
                    .pick_folder()
            {
                self.saved.settings.output_dir = Some(path);
                self.settings_dirty = true;
            }
            if let Some(path) = self.saved.settings.output_dir.clone() {
                ui.add(
                    egui::Label::new(
                        RichText::new(display_path(&path))
                            .size(theme::CAPTION)
                            .color(colors.secondary),
                    )
                    .truncate(),
                )
                .on_hover_text(path.display().to_string());
                if ui
                    .add(egui::Button::new(theme::button_text(
                        FluentIcon::Folder,
                        tr("Cambia cartella…", "Change folder…"),
                        colors.text,
                    )))
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new().set_directory(path).pick_folder()
                {
                    self.saved.settings.output_dir = Some(path);
                    self.settings_dirty = true;
                }
            }
        });
        ui.add_space(6.0);
        theme::show_card(ui, |ui| {
            egui::CollapsingHeader::new(theme::heading(
                tr("Impostazioni personalizzate", "Custom settings"),
                theme::BODY,
            ))
            .id_salt("advanced_conversion_settings")
            .default_open(self.advanced)
            .show(ui, |ui| {
                let before = serde_json::to_vec(&self.saved.settings).unwrap_or_default();
                self.advanced_settings(ui);
                if serde_json::to_vec(&self.saved.settings).unwrap_or_default() != before {
                    self.saved.preset = 3;
                    self.settings_dirty = true;
                }
            });
        });
        if self.items.iter().any(|item| {
            item.path.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("gif")
                    || ext.eq_ignore_ascii_case("tif")
                    || ext.eq_ignore_ascii_case("tiff")
            })
        }) {
            ui.add_space(6.0);
            info_bar(
                ui,
                tr(
                    "GIF: primo fotogramma. TIFF: prima pagina.",
                    "GIF: first frame. TIFF: first page.",
                ),
                InfoKind::Info,
            );
        }
        if let Some(message) = self.prerequisite() {
            ui.add_space(6.0);
            info_bar(ui, &message, InfoKind::Warning);
            if ui
                .button(tr("Configura strumenti…", "Configure tools…"))
                .clicked()
            {
                self.view = MainView::Settings;
            }
        }
    }

    fn advanced_settings(&mut self, ui: &mut egui::Ui) {
        if self.saved.settings.format == OutputFormat::Txt
            && self.items.iter().any(|item| {
                matches!(
                    MediaKind::from_path(&item.path),
                    MediaKind::Audio | MediaKind::Video
                )
            })
        {
            settings_ui::speech_controls(ui, &mut self.saved.settings.speech);
            if self.items.iter().all(|item| {
                matches!(
                    MediaKind::from_path(&item.path),
                    MediaKind::Audio | MediaKind::Video
                )
            }) {
                return;
            }
        }
        if self.items.iter().any(|item| {
            item.path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
        }) || self.saved.settings.format == OutputFormat::Csv
        {
            ui.label(tr(
                "Separatore CSV in ingresso e uscita",
                "CSV separator for input and output",
            ));
            let delimiter = &mut self.saved.settings.csv.delimiter;
            egui::ComboBox::from_id_salt("csv_delimiter")
                .selected_text(match delimiter {
                    CsvDelimiter::Comma => tr("Virgola (,)", "Comma (,)"),
                    CsvDelimiter::Semicolon => tr("Punto e virgola (;)", "Semicolon (;)"),
                    CsvDelimiter::Tab => tr("Tabulazione", "Tab"),
                    CsvDelimiter::Pipe => tr("Barra verticale (|)", "Pipe (|)"),
                })
                .show_ui(ui, |ui| {
                    for (value, label) in [
                        (CsvDelimiter::Comma, tr("Virgola (,)", "Comma (,)")),
                        (
                            CsvDelimiter::Semicolon,
                            tr("Punto e virgola (;)", "Semicolon (;)"),
                        ),
                        (CsvDelimiter::Tab, tr("Tabulazione", "Tab")),
                        (CsvDelimiter::Pipe, tr("Barra verticale (|)", "Pipe (|)")),
                    ] {
                        ui.selectable_value(delimiter, value, label);
                    }
                });
            ui.label(RichText::new(tr("UTF-8 con virgolette. Il separatore scelto viene usato anche per leggere i CSV.", "UTF-8 with quoted fields. The selected separator is also used when reading CSV files.")).small().weak());
            ui.add_space(7.0);
        }
        let image_ocr = self.saved.settings.format == OutputFormat::Txt
            && self
                .items
                .iter()
                .any(|item| MediaKind::from_path(&item.path) == MediaKind::Image);
        let pdf_text = matches!(
            self.saved.settings.format,
            OutputFormat::Txt | OutputFormat::Docx | OutputFormat::Html
        ) && self
            .items
            .iter()
            .any(|item| MediaKind::from_path(&item.path) == MediaKind::Pdf);
        if pdf_text
            && ui
                .checkbox(
                    &mut self.saved.settings.ocr.enabled,
                    tr(
                        "Riconosci le scansioni con OCR locale",
                        "Recognize scanned pages with local OCR",
                    ),
                )
                .changed()
            && self.saved.settings.ocr.enabled
        {
            self.saved.settings.pdf.dpi = self.saved.settings.pdf.dpi.max(300);
        }
        if image_ocr || pdf_text && self.saved.settings.ocr.enabled {
            ui.label(tr("Lingue OCR installate", "Installed OCR languages"));
            egui::ComboBox::from_id_salt("ocr_languages")
                .selected_text(&self.saved.settings.ocr.language)
                .show_ui(ui, |ui| {
                    for (code, name) in [
                        ("ita+eng", tr("Italiano e inglese", "Italian and English")),
                        ("ita", tr("Italiano", "Italian")),
                        ("eng", tr("Inglese", "English")),
                        ("fra", tr("Francese", "French")),
                        ("deu", tr("Tedesco", "German")),
                        ("spa", tr("Spagnolo", "Spanish")),
                    ] {
                        ui.selectable_value(
                            &mut self.saved.settings.ocr.language,
                            code.into(),
                            name,
                        );
                    }
                });
            ui.text_edit_singleline(&mut self.saved.settings.ocr.language);
            ui.label(RichText::new(tr("Tesseract riconosce il testo sul PC. Occorrono i dati delle lingue indicate; rileggi il risultato.", "Tesseract recognizes text on your PC. The selected language data is required; review the result.")).small().weak());
            if pdf_text {
                ui.add(
                    egui::Slider::new(&mut self.saved.settings.pdf.dpi, 150..=600)
                        .suffix(" DPI OCR"),
                );
            }
            if image_ocr {
                return;
            }
        }
        let kind = if self.saved.settings.format == OutputFormat::Gif
            && self
                .items
                .iter()
                .any(|item| MediaKind::from_path(&item.path) == MediaKind::Video)
        {
            MediaKind::Video
        } else {
            self.saved.settings.format.kind()
        };
        match kind {
            MediaKind::Audio => {
                if quality_applicable(self.saved.settings.format) {
                    ui.label(tr("Bitrate audio", "Audio bitrate"));
                    ui.add(
                        egui::Slider::new(&mut self.saved.settings.audio.bitrate_kbps, 48..=320)
                            .suffix(" kbps"),
                    );
                }
                let rates: &[u32] = self
                    .saved
                    .settings
                    .format
                    .audio_sample_rates()
                    .unwrap_or(&[0, 22050, 44100, 48000, 96000]);
                let rate = &mut self.saved.settings.audio.sample_rate;
                egui::ComboBox::from_id_salt("sample_rate")
                    .selected_text(if *rate == 0 {
                        tr("Frequenza originale", "Original sample rate").into()
                    } else {
                        format!("{rate} Hz")
                    })
                    .show_ui(ui, |ui| {
                        for &value in rates {
                            ui.selectable_value(
                                rate,
                                value,
                                if value == 0 {
                                    tr("Originale", "Original").into()
                                } else {
                                    format!("{value} Hz")
                                },
                            );
                        }
                    });
                let channels = &mut self.saved.settings.audio.channels;
                egui::ComboBox::from_id_salt("channels")
                    .selected_text(match channels {
                        1 => "Mono",
                        2 => "Stereo",
                        _ => tr("Canali originali", "Original channels"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(channels, 0, tr("Originali", "Original"));
                        ui.selectable_value(channels, 1, "Mono");
                        ui.selectable_value(channels, 2, "Stereo");
                    });
                ui.checkbox(
                    &mut self.saved.settings.audio.normalize,
                    tr("Normalizza volume", "Normalize volume"),
                );
                if matches!(
                    self.saved.settings.format,
                    OutputFormat::Wav
                        | OutputFormat::Flac
                        | OutputFormat::Aiff
                        | OutputFormat::Au
                        | OutputFormat::Caf
                ) {
                    ui.label(
                        RichText::new(tr(
                            "Formato senza perdita: il bitrate non viene applicato.",
                            "Lossless format: bitrate does not apply.",
                        ))
                        .small()
                        .weak(),
                    );
                }
                ui.label(tr("Ritaglia intervallo (secondi)", "Trim range (seconds)"));
                optional_seconds(
                    ui,
                    tr("Da", "From"),
                    &mut self.saved.settings.video.trim_start,
                    0.0,
                );
                optional_seconds(
                    ui,
                    tr("A", "To"),
                    &mut self.saved.settings.video.trim_end,
                    1.0,
                );
            }
            MediaKind::Video => {
                if matches!(
                    self.saved.settings.format,
                    OutputFormat::Mp4 | OutputFormat::Mkv | OutputFormat::Mov | OutputFormat::M4v
                ) {
                    egui::ComboBox::from_id_salt("codec")
                        .selected_text(match self.saved.settings.video.codec {
                            VideoCodec::H264 => {
                                tr("H.264 · compatibilità", "H.264 · compatibility")
                            }
                            VideoCodec::H265 => {
                                tr("H.265 · file più piccoli", "H.265 · smaller files")
                            }
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.saved.settings.video.codec,
                                VideoCodec::H264,
                                tr("H.264 · compatibilità", "H.264 · compatibility"),
                            );
                            ui.selectable_value(
                                &mut self.saved.settings.video.codec,
                                VideoCodec::H265,
                                tr("H.265 · file più piccoli", "H.265 · smaller files"),
                            );
                        });
                }
                if self.saved.settings.format != OutputFormat::Gif {
                    ui.label(tr(
                        "Qualità video · valori minori = più dettaglio",
                        "Video quality · lower values = more detail",
                    ));
                    let quality_label = if matches!(
                        self.saved.settings.format,
                        OutputFormat::Webm
                            | OutputFormat::Avi
                            | OutputFormat::Mpeg
                            | OutputFormat::Wmv
                            | OutputFormat::Flv
                            | OutputFormat::ThreeGp
                    ) {
                        tr("Qualità ", "Quality ")
                    } else {
                        "CRF "
                    };
                    ui.add(
                        egui::Slider::new(&mut self.saved.settings.video.crf, 16..=35)
                            .prefix(quality_label),
                    );
                }
                width_setting(ui, "video_width", &mut self.saved.settings.video.width);
                let fps = &mut self.saved.settings.video.fps;
                egui::ComboBox::from_id_salt("fps")
                    .selected_text(
                        fps.map(|n| format!("{n} fps"))
                            .unwrap_or(tr("Fotogrammi originali", "Original frame rate").into()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(fps, None, tr("Originali", "Original"));
                        for n in [24, 25, 30, 50, 60] {
                            ui.selectable_value(fps, Some(n), format!("{n} fps"));
                        }
                    });
                ui.label(tr("Ritaglia intervallo (secondi)", "Trim range (seconds)"));
                optional_seconds(
                    ui,
                    tr("Da", "From"),
                    &mut self.saved.settings.video.trim_start,
                    0.0,
                );
                optional_seconds(
                    ui,
                    tr("A", "To"),
                    &mut self.saved.settings.video.trim_end,
                    1.0,
                );
                if self.saved.settings.format != OutputFormat::Gif {
                    ui.label(
                        RichText::new(tr(
                            "La traccia audio viene conservata quando disponibile.",
                            "The audio track is preserved when available.",
                        ))
                        .small()
                        .weak(),
                    );
                    egui::CollapsingHeader::new(tr("Audio del video", "Video audio"))
                        .id_salt("video_audio_settings")
                        .show(ui, |ui| {
                            ui.label(tr("Bitrate audio", "Audio bitrate"));
                            ui.add(
                                egui::Slider::new(
                                    &mut self.saved.settings.audio.bitrate_kbps,
                                    48..=320,
                                )
                                .suffix(" kbps"),
                            );
                            let rates: &[u32] = self
                                .saved
                                .settings
                                .format
                                .audio_sample_rates()
                                .unwrap_or(&[0, 44100, 48000]);
                            let rate = &mut self.saved.settings.audio.sample_rate;
                            egui::ComboBox::from_id_salt("video_sample_rate")
                                .selected_text(if *rate == 0 {
                                    tr("Frequenza originale", "Original sample rate").into()
                                } else {
                                    format!("{rate} Hz")
                                })
                                .show_ui(ui, |ui| {
                                    for &value in rates {
                                        ui.selectable_value(
                                            rate,
                                            value,
                                            if value == 0 {
                                                tr("Originale", "Original").into()
                                            } else {
                                                format!("{value} Hz")
                                            },
                                        );
                                    }
                                });
                            let channels = &mut self.saved.settings.audio.channels;
                            egui::ComboBox::from_id_salt("video_channels")
                                .selected_text(match channels {
                                    1 => "Mono",
                                    2 => "Stereo",
                                    _ => tr("Canali originali", "Original channels"),
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(channels, 0, tr("Originali", "Original"));
                                    ui.selectable_value(channels, 1, "Mono");
                                    ui.selectable_value(channels, 2, "Stereo");
                                });
                            ui.checkbox(
                                &mut self.saved.settings.audio.normalize,
                                tr("Normalizza volume", "Normalize volume"),
                            );
                        });
                } else {
                    ui.label(
                        RichText::new(tr(
                            "Il GIF animato non contiene audio.",
                            "Animated GIF does not contain audio.",
                        ))
                        .small()
                        .weak(),
                    );
                }
            }
            MediaKind::Image => {
                if self
                    .items
                    .iter()
                    .any(|item| MediaKind::from_path(&item.path) == MediaKind::Pdf)
                {
                    ui.label(tr("Risoluzione delle pagine PDF", "PDF page resolution"));
                    ui.add(
                        egui::Slider::new(&mut self.saved.settings.pdf.dpi, 72..=600)
                            .suffix(" DPI"),
                    );
                    ui.label(
                        RichText::new(tr(
                            "Un’immagine per pagina, nella cartella di destinazione.",
                            "One image per page, saved in the output folder.",
                        ))
                        .small()
                        .weak(),
                    );
                } else {
                    if matches!(
                        self.saved.settings.format,
                        OutputFormat::Jpg | OutputFormat::Avif | OutputFormat::Heic
                    ) {
                        ui.label(tr("Qualità dell'immagine", "Image quality"));
                        ui.add(
                            egui::Slider::new(&mut self.saved.settings.image.quality, 20..=100)
                                .suffix("%"),
                        );
                    }
                    width_setting(ui, "image_width", &mut self.saved.settings.image.width);
                    if self.saved.settings.format == OutputFormat::Webp {
                        ui.label(
                            RichText::new("WebP viene codificato senza perdita.")
                                .small()
                                .weak(),
                        );
                    }
                }
            }
            MediaKind::Pdf => {
                let all_pdf = !self.items.is_empty()
                    && self
                        .items
                        .iter()
                        .all(|item| MediaKind::from_path(&item.path) == MediaKind::Pdf);
                let can_merge =
                    self.items.len() >= 2 && self.common_formats().contains(&OutputFormat::Pdf);
                if all_pdf || can_merge {
                    let action = &mut self.saved.settings.pdf.action;
                    egui::ComboBox::from_id_salt("pdf_action")
                        .selected_text(match action {
                            PdfAction::Compress if all_pdf => {
                                tr("Comprimi senza perdita", "Lossless compression")
                            }
                            PdfAction::Compress => "PDF (Bulk)",
                            PdfAction::Split => {
                                tr("Dividi: una pagina per file", "Split: one page per file")
                            }
                            PdfAction::Merge => {
                                tr("PDF (Merge) · ordina pagine", "PDF (Merge) · arrange pages")
                            }
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                action,
                                PdfAction::Compress,
                                if all_pdf {
                                    tr("Comprimi senza perdita", "Lossless compression")
                                } else {
                                    "PDF (Bulk)"
                                },
                            );
                            if all_pdf {
                                ui.selectable_value(
                                    action,
                                    PdfAction::Split,
                                    tr("Dividi: una pagina per file", "Split: one page per file"),
                                );
                            }
                            ui.add_enabled_ui(can_merge, |ui| {
                                ui.selectable_value(
                                    action,
                                    PdfAction::Merge,
                                    tr(
                                        "PDF (Merge) · ordina pagine",
                                        "PDF (Merge) · arrange pages",
                                    ),
                                );
                            });
                        });
                    ui.label(RichText::new(match action {
                        PdfAction::Compress if all_pdf => tr("Ottimizza ogni PDF senza ridurre la qualità. Un PDF già ottimizzato può mantenere le stesse dimensioni.", "Optimizes each PDF without reducing quality. An already optimized PDF may keep the same size."),
                        PdfAction::Compress => tr("Crea un PDF separato per ciascun file della lista.", "Creates a separate PDF for each file in the list."),
                        PdfAction::Split => tr("Crea un PDF per pagina. Segnalibri, struttura accessibile e metadati del documento non vengono trasferiti; documenti firmati o con moduli compilabili richiedono una copia statica.", "Creates one PDF per page. Bookmarks, accessibility structure and document metadata are not transferred; signed documents or fillable forms require a static copy."),
                        PdfAction::Merge => tr("Apre l'ordine delle singole pagine: trascinale prima di creare un unico PDF. Segnalibri, struttura accessibile e metadati non vengono trasferiti; documenti firmati o con moduli compilabili richiedono una copia statica.", "Opens the individual page order: drag pages before creating one PDF. Bookmarks, accessibility structure and metadata are not transferred; signed documents or fillable forms require a static copy."),
                    }).small().weak());
                }
                if self
                    .items
                    .iter()
                    .any(|item| MediaKind::from_path(&item.path) == MediaKind::Image)
                {
                    ui.label(tr(
                        "Qualità delle immagini nel PDF",
                        "Image quality in the PDF",
                    ));
                    ui.add(
                        egui::Slider::new(&mut self.saved.settings.image.quality, 20..=100)
                            .suffix("%"),
                    );
                    width_setting(ui, "pdf_image_width", &mut self.saved.settings.image.width);
                    ui.label(RichText::new(tr("Una pagina A4 per immagine, con proporzioni mantenute e margine di 24 pt.", "One A4 page per image, preserving proportions with a 24 pt margin.")).small().weak());
                } else if !all_pdf {
                    ui.label(
                        RichText::new(
                            tr("Documenti Office, fogli di calcolo e HTML vengono esportati in PDF con LibreOffice. HTML/CSS complessi possono cambiare impaginazione; JavaScript non viene eseguito. Le risorse collegate nell'HTML possono essere lette dal motore.", "Office documents, spreadsheets and HTML are exported to PDF with LibreOffice. Complex HTML/CSS may change layout; JavaScript is not executed. The engine may read resources linked in HTML."),
                        )
                        .small()
                        .weak(),
                    );
                }
            }
            MediaKind::Spreadsheet => {
                let text = if self.saved.settings.format == OutputFormat::Csv {
                    tr(
                        "Esporta il primo foglio in CSV UTF-8 con il separatore scelto e virgolette. Conserva i valori delle celle; formattazione, formule e altri fogli non sono inclusi.",
                        "Exports the first sheet as UTF-8 CSV with the selected separator and quoted fields. Preserves cell values; formatting, formulas and other sheets are not included.",
                    )
                } else {
                    tr(
                        "Crea un file Excel XLSX con LibreOffice. I CSV in ingresso devono essere UTF-8 e usare il separatore scelto; il contenuto che inizia con = resta testo.",
                        "Creates an Excel XLSX file with LibreOffice. Input CSV files must use UTF-8 and the selected separator; content starting with = stays as text.",
                    )
                };
                ui.label(RichText::new(text).small().weak());
            }
            MediaKind::Document | MediaKind::Html => {
                let text = if self.saved.settings.format == OutputFormat::Txt {
                    tr(
                        "Estrae il testo incorporato nel PDF; per le scansioni abilita OCR locale qui sopra.",
                        "Extracts embedded PDF text; enable local OCR above for scanned pages.",
                    )
                } else {
                    tr(
                        "Ricostruisce il testo del PDF in un file modificabile. Per le scansioni abilita OCR locale qui sopra; immagini, tabelle e impaginazione originali non vengono ricostruite.",
                        "Reconstructs PDF text in an editable file. Enable local OCR above for scanned pages; original images, tables and layout are not reconstructed.",
                    )
                };
                ui.label(RichText::new(text).small().weak());
            }
            _ => {
                ui.label(RichText::new(tr("Il testo viene estratto dal PDF. I PDF scansionati richiedono OCR, qui non incluso.", "Text is extracted from the PDF. Scanned PDFs require OCR, which is not included here.")).small().weak());
            }
        }
    }

    fn queue_panel(&mut self, ui: &mut egui::Ui) {
        let colors = theme::palette(ui.ctx());
        ui.horizontal(|ui| {
            ui.label(theme::heading(
                trf!(
                    "File da convertire{}",
                    "Files to convert{}",
                    if self.items.is_empty() {
                        String::new()
                    } else {
                        format!(" · {}", self.items.len())
                    }
                ),
                theme::SUBTITLE,
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        !self.running,
                        egui::Button::new(tr("Nuovo foglio…", "New sheet…")),
                    )
                    .clicked()
                {
                    self.open_editor(None, None);
                }
                if ui
                    .add_enabled(
                        !self.running,
                        egui::Button::new(theme::button_text(
                            FluentIcon::Add,
                            tr("Aggiungi file", "Add files"),
                            colors.text,
                        )),
                    )
                    .clicked()
                    && let Some(paths) = rfd::FileDialog::new()
                        .set_title(tr("Scegli i file da convertire", "Choose files to convert"))
                        .pick_files()
                {
                    self.add_files(paths);
                }
            });
        });
        ui.add_space(12.0);
        if self.items.is_empty() {
            theme::show_card(ui, |ui| {
                ui.set_min_height((ui.available_height() - 40.0).max(260.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(54.0);
                    ui.label(theme::icon(FluentIcon::Document, 36.0).color(colors.secondary));
                    ui.add_space(12.0);
                    ui.label(theme::heading(
                        tr("Trascina qui i tuoi file", "Drop your files here"),
                        theme::SUBTITLE,
                    ));
                    ui.add_space(16.0);
                    if ui
                        .add(
                            egui::Button::new(theme::button_text(
                                FluentIcon::Add,
                                tr("Scegli file…", "Choose files…"),
                                colors.accent_text,
                            ))
                            .fill(colors.accent),
                        )
                        .clicked()
                        && let Some(paths) = rfd::FileDialog::new().pick_files()
                    {
                        self.add_files(paths);
                    }
                    ui.add_space(18.0);
                });
            });
            return;
        }
        let mut remove = None;
        let mut edit_requested = None;
        let mut reorder = None;
        let merging = self.saved.settings.format == OutputFormat::Pdf
            && self.saved.settings.pdf.action == PdfAction::Merge;
        let icons = &mut self.file_icons;
        let mut action_error = None;
        egui::ScrollArea::vertical()
            .id_salt("queue")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, item) in self.items.iter().enumerate() {
                    theme::show_card(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            file_icon(ui, icons, &item.path);
                            let name_width = (ui.available_width()
                                - if merging { 150.0 } else { 105.0 }
                                - if editable(&item.path) { 80.0 } else { 0.0 })
                            .max(80.0);
                            ui.allocate_ui_with_layout(
                                egui::vec2(name_width, 42.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.add(
                                        egui::Label::new(theme::heading(
                                            item.path
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy(),
                                            theme::BODY,
                                        ))
                                        .truncate(),
                                    )
                                    .on_hover_text(display_path(&item.path));
                                    ui.label(
                                        RichText::new(format!(
                                            "{} · {}",
                                            human_size(item.size),
                                            kind_label(MediaKind::from_path(&item.path))
                                        ))
                                        .size(theme::CAPTION)
                                        .color(colors.secondary),
                                    );
                                },
                            );
                            if editable(&item.path)
                                && ui
                                    .add_enabled(
                                        !self.running,
                                        egui::Button::new(tr("Modifica…", "Edit…")).small(),
                                    )
                                    .clicked()
                            {
                                edit_requested = Some(item.path.clone());
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if !self.running
                                        && icon_button(
                                            ui,
                                            FluentIcon::Delete,
                                            tr("Rimuovi dalla lista", "Remove from list"),
                                        )
                                        .clicked()
                                    {
                                        remove = Some(index);
                                    }
                                    if merging && !self.running {
                                        if ui
                                            .add_enabled(
                                                index + 1 < self.items.len(),
                                                egui::Button::new(theme::icon(
                                                    FluentIcon::ChevronDown,
                                                    16.0,
                                                ))
                                                .frame(false),
                                            )
                                            .on_hover_text(tr(
                                                "Sposta dopo nell’unione",
                                                "Move later in the merge",
                                            ))
                                            .clicked()
                                        {
                                            reorder = Some((index, index + 1));
                                        }
                                        if ui
                                            .add_enabled(
                                                index > 0,
                                                egui::Button::new(theme::icon(
                                                    FluentIcon::ChevronUp,
                                                    16.0,
                                                ))
                                                .frame(false),
                                            )
                                            .on_hover_text(tr(
                                                "Sposta prima nell’unione",
                                                "Move earlier in the merge",
                                            ))
                                            .clicked()
                                        {
                                            reorder = Some((index, index - 1));
                                        }
                                    }
                                    match &item.state {
                                        JobState::Complete(outputs) => {
                                            if icon_button(
                                                ui,
                                                FluentIcon::Folder,
                                                tr(
                                                    "Apri cartella di destinazione",
                                                    "Open output folder",
                                                ),
                                            )
                                            .clicked()
                                                && let Some(parent) =
                                                    outputs.first().and_then(|p| p.parent())
                                                && let Err(error) = platform::open_folder(parent)
                                            {
                                                action_error = Some(format!("{error:#}"));
                                            }
                                        }
                                        JobState::Running(..) => {
                                            ui.spinner();
                                        }
                                        JobState::Waiting => {
                                            ui.label(
                                                RichText::new(tr("Pronto", "Ready"))
                                                    .size(theme::CAPTION)
                                                    .color(colors.secondary),
                                            );
                                        }
                                        JobState::Cancelled => {
                                            ui.label(
                                                RichText::new(tr("Annullato", "Cancelled"))
                                                    .size(theme::CAPTION)
                                                    .color(colors.secondary),
                                            );
                                        }
                                        JobState::Failed(_) => {
                                            ui.label(
                                                theme::icon(FluentIcon::Info, 16.0)
                                                    .color(colors.error),
                                            );
                                        }
                                    }
                                },
                            );
                        });
                        match &item.state {
                            JobState::Running(fraction, message) => {
                                ui.add_space(8.0);
                                progress_display(ui, *fraction, message);
                            }
                            JobState::Complete(outputs) => {
                                ui.label(
                                    RichText::new(trf!(
                                        "Completato · {} file creati",
                                        "Completed · Output files: {}",
                                        outputs.len()
                                    ))
                                    .size(theme::CAPTION)
                                    .color(colors.success),
                                );
                            }
                            JobState::Failed(message) => {
                                info_bar(
                                    ui,
                                    tr("Conversione non riuscita", "Conversion failed"),
                                    InfoKind::Error,
                                );
                                egui::CollapsingHeader::new(tr(
                                    "Dettagli e copia errore",
                                    "Error details and copy",
                                ))
                                .id_salt(index)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(message).size(theme::CAPTION),
                                        )
                                        .wrap(),
                                    );
                                    if ui.button(tr("Copia errore", "Copy error")).clicked() {
                                        ui.ctx().copy_text(message.clone());
                                    }
                                });
                            }
                            _ => {}
                        }
                    });
                    ui.add_space(6.0);
                }
                if !self.running && ui.button(tr("Svuota lista", "Clear list")).clicked() {
                    remove = Some(usize::MAX);
                }
            });
        if let Some(error) = action_error {
            self.notice = Some(error);
        }
        if let Some(path) = edit_requested {
            self.open_editor(Some(path), None);
        }
        if let Some(index) = remove {
            if index == usize::MAX {
                self.items.clear();
            } else {
                self.items.remove(index);
            }
            if self.items.len() < 2 && self.saved.settings.pdf.action == PdfAction::Merge {
                self.saved.settings.pdf.action = PdfAction::Compress;
                self.settings_dirty = true;
            }
        } else if let Some((from, to)) = reorder {
            self.items.swap(from, to);
        }
    }

    fn record_history(&mut self, index: usize) {
        let mut job = session_job(&self.items[index], index, self.conversion_format);
        job.id = format!(
            "gui-{}-{index}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.gui_jobs.insert(0, job.clone());
        self.gui_jobs.truncate(300);
        if let Some(path) = crate::gui_history::path()
            && let Err(error) = crate::gui_history::record_async(path, job)
        {
            self.history_error = Some(format!("{error:#}"));
        }
    }

    fn refresh_history(&mut self) {
        if let Some(history) = &self.gui_history {
            if self
                .history_refresh
                .is_none_or(|last| last.elapsed() >= QUEUE_REFRESH)
            {
                self.history_refresh = Some(std::time::Instant::now());
                history.refresh();
            }
            if let Some(snapshot) = history.snapshot_after(self.history_revision) {
                self.history_revision = snapshot.revision;
                self.gui_jobs = snapshot.jobs;
                self.history_error = snapshot.error;
            }
        }
    }

    fn quick_report_panel(&mut self, ui: &mut egui::Ui) {
        let Some(report) = self.quick_report.clone() else {
            return;
        };
        egui::CollapsingHeader::new(trf!(
            "Ultima conversione rapida · {} file creati",
            "Last quick conversion · Output files: {}",
            report.successes.len()
        ))
        .id_salt("last_quick_report")
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("{} · {}", report.timestamp, report.format))
                    .small()
                    .weak(),
            );
            egui::ScrollArea::vertical()
                .id_salt("quick_outputs")
                .max_height(145.0)
                .show(ui, |ui| {
                    for path in &report.successes {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(
                                        path.file_name().unwrap_or_default().to_string_lossy(),
                                    )
                                    .small(),
                                )
                                .truncate(),
                            )
                            .on_hover_text(display_path(path));
                            if ui
                                .small_button(tr("Apri cartella", "Open folder"))
                                .clicked()
                                && let Some(parent) = path.parent()
                                && let Err(error) = platform::open_folder(parent)
                            {
                                self.notice = Some(format!("{error:#}"));
                            }
                        });
                    }
                    for error in &report.errors {
                        ui.add(
                            egui::Label::new(
                                RichText::new(error)
                                    .small()
                                    .color(ui.visuals().error_fg_color),
                            )
                            .wrap(),
                        );
                    }
                });
            if ui
                .small_button(tr("Aggiorna resoconto", "Refresh report"))
                .clicked()
            {
                self.quick_report = load_report();
            }
        });
        ui.add_space(4.0);
    }

    fn footer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let colors = theme::palette(ctx);
        let done = self
            .items
            .iter()
            .filter(|item| matches!(item.state, JobState::Complete(_)))
            .count();
        let failed = self
            .items
            .iter()
            .filter(|item| matches!(item.state, JobState::Failed(_)))
            .count();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let summary = if self.running {
                    trf!(
                        "Conversione in corso · {done}/{} completati",
                        "Converting · {done}/{} completed",
                        self.items.len()
                    )
                } else if done > 0 || failed > 0 {
                    trf!(
                        "{done} completati · {failed} errori",
                        "Completed: {done} · Errors: {failed}"
                    )
                } else if self.items.is_empty() {
                    tr("0 file", "0 files").into()
                } else {
                    trf!(
                        "{} file · {}",
                        "Files: {} · {}",
                        self.items.len(),
                        human_size(self.items.iter().map(|item| item.size).sum())
                    )
                };
                ui.label(theme::heading(summary, theme::BODY));
            });
            if self.running || !self.conversion_elapsed.is_zero() {
                let elapsed = self
                    .conversion_started
                    .map(|started| started.elapsed())
                    .unwrap_or(self.conversion_elapsed);
                ui.label(
                    RichText::new(trf!(
                        "Tempo trascorso {}",
                        "Elapsed time {}",
                        duration_hms(elapsed)
                    ))
                    .size(theme::CAPTION)
                    .color(colors.secondary),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.running {
                    let cancelling = self.cancel.load(Ordering::Relaxed);
                    if ui
                        .add_enabled(
                            !cancelling,
                            egui::Button::new(theme::button_text(
                                FluentIcon::Cancel,
                                if cancelling {
                                    tr("Annullamento…", "Cancelling…")
                                } else {
                                    tr("Annulla", "Cancel")
                                },
                                colors.text,
                            ))
                            .min_size(egui::vec2(160.0, 36.0)),
                        )
                        .clicked()
                    {
                        self.cancel.store(true, Ordering::Relaxed);
                    }
                } else {
                    let label = if self.saved.settings.format == OutputFormat::Pdf
                        && self.saved.settings.pdf.action == PdfAction::Merge
                    {
                        tr("Ordina e unisci PDF", "Arrange and merge PDF")
                    } else if self.saved.settings.format == OutputFormat::Pdf
                        && !self.items.is_empty()
                        && self
                            .items
                            .iter()
                            .all(|item| MediaKind::from_path(&item.path) == MediaKind::Pdf)
                    {
                        match self.saved.settings.pdf.action {
                            PdfAction::Compress => tr("Comprimi PDF", "Compress PDF"),
                            PdfAction::Split => tr("Dividi PDF", "Split PDF"),
                            PdfAction::Merge => tr("Unisci PDF", "Merge PDF"),
                        }
                    } else if self.items.iter().any(|item| {
                        matches!(item.state, JobState::Complete(_) | JobState::Failed(_))
                    }) {
                        tr("Converti di nuovo", "Convert again")
                    } else {
                        tr("Converti file", "Convert files")
                    };
                    if ui
                        .add_enabled(
                            !self.items.is_empty() && self.prerequisite().is_none(),
                            egui::Button::new(theme::button_text(
                                FluentIcon::Convert,
                                label,
                                colors.accent_text,
                            ))
                            .fill(colors.accent)
                            .min_size(egui::vec2(174.0, 36.0)),
                        )
                        .clicked()
                    {
                        self.start(ctx);
                    }
                }
            });
        });
    }

    fn tools_window(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::palette(ctx).canvas).inner_margin(24)).show(ctx, |ui| {
            egui::ScrollArea::vertical().id_salt("settings_page").show(ui, |ui| {
                self.output_format_settings(ui);
                ui.add_space(24.0);
                if let Some(action) = settings_ui::engines(ui, &self.tools, &self.saved.overrides, self.runtime_install.is_some()) {
                    match action {
                        EngineAction::Install(engines) => self.pending_runtime_install = Some(engines),
                        EngineAction::Select(key) => {
                            let dialog = if key == "whisper-model" {
                                rfd::FileDialog::new().set_title(tr("Seleziona modello Whisper", "Select Whisper model")).set_file_name("ggml-tiny.bin").add_filter("Whisper", &["bin"])
                            } else {
                                rfd::FileDialog::new().set_title(trf!("Seleziona {key}.exe", "Select {key}.exe")).set_file_name(format!("{key}.exe")).add_filter("EXE", &["exe"])
                            };
                            if let Some(path) = dialog.pick_file() {
                                if (key == "whisper-model" && the_converter::tools::model_available(&path)) || (key != "whisper-model" && executable_available(&path)) {
                                    self.saved.overrides.insert(key.into(), path);
                                    self.tools = ToolPaths::configured(&self.saved.overrides);
                                    self.settings_dirty = true;
                                } else { self.notice = Some(if key == "whisper-model" { tr("Seleziona un modello Whisper .bin esistente.", "Select an existing Whisper .bin model.") } else { tr("Seleziona un programma .exe esistente.", "Select an existing .exe program.") }.into()); }
                            }
                        }
                        EngineAction::Reset(keys) => {
                            for key in keys { self.saved.overrides.remove(*key); }
                            self.tools = ToolPaths::configured(&self.saved.overrides);
                            self.settings_dirty = true;
                        }
                        EngineAction::Website(url) => { if let Err(error) = platform::open_website(url) { self.notice = Some(format!("{error:#}")); } }
                        EngineAction::Detect => self.tools = ToolPaths::configured(&self.saved.overrides),
                        EngineAction::OpenFolder => {
                            if let Some(path) = the_converter::tools::user_tools_dir()
                                && let Err(error) = platform::open_folder(&path) { self.notice = Some(format!("{error:#}"));
                            }
                        }
                    }
                }
                self.runtime_install_progress(ui);
                ui.add_space(16.0);
                ui.columns(2, |columns| {
                    theme::show_card(&mut columns[0], |ui| {
                        ui.label(theme::heading(tr("Lingua", "Language"), theme::BODY));
                        let mut language = if i18n::has_language_override() { i18n::current_language() } else { self.saved.ui_language };
                        egui::ComboBox::from_id_salt("ui_language").selected_text(language.label()).show_ui(ui, |ui| {
                            for choice in [Language::System, Language::Italian, Language::English] {
                                if ui.selectable_value(&mut language, choice, choice.label()).changed() {
                                    self.saved.ui_language = language;
                                    i18n::set_language(language);
                                    self.settings_dirty = true;
                                    self.language_dirty = true;
                                }
                            }
                        });
                    });
                    theme::show_card(&mut columns[1], |ui| {
                        ui.label(theme::heading(tr("Aspetto", "Appearance"), theme::BODY));
                        let mut preference=ui.ctx().options(|options| options.theme_preference);
                        let label=match preference { egui::ThemePreference::System => tr("Sistema", "System"), egui::ThemePreference::Light => tr("Chiaro", "Light"), egui::ThemePreference::Dark => tr("Scuro", "Dark") };
                        egui::ComboBox::from_id_salt("ui_theme").selected_text(label).show_ui(ui, |ui| {
                            for (value,label) in [(egui::ThemePreference::System,tr("Sistema", "System")),(egui::ThemePreference::Light,tr("Chiaro", "Light")),(egui::ThemePreference::Dark,tr("Scuro", "Dark"))] {
                                if ui.selectable_value(&mut preference,value,label).changed() { ui.ctx().set_theme(preference); }
                            }
                        });
                    });
                });
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Convert Here {} · Salvatore Corsi", env!("CARGO_PKG_VERSION"))).color(theme::palette(ui.ctx()).secondary));
                    if ui.button("GPL-3.0").clicked() { self.show_license=true; }
                    if settings_ui::tool_button(ui,FluentIcon::Raw,tr("Sorgenti", "Source"),true).clicked()
                        && let Err(error)=platform::open_website(&format!("https://github.com/salvatorecorsi/convert-here/releases/tag/v{}", env!("CARGO_PKG_VERSION"))) {self.notice=Some(format!("{error:#}"));
                    }
                });
                if let Some(notice)=self.notice.clone() {
                    info_bar(ui,&notice,InfoKind::Warning);
                    if settings_ui::tool_button(ui,FluentIcon::Cancel,tr("Chiudi avviso", "Dismiss notice"),true).clicked() {self.notice=None;}
                }
            });
        });
    }
}

impl eframe::App for ConverterApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(task) = &self.runtime_install {
            task.cancel.store(true, Ordering::Relaxed);
        }
        if self.settings_dirty {
            self.persist_with_force(true);
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.receive_runtime_events();
        self.refresh_history();
        if let Some(task) = &self.runtime_install
            && ctx.input(|input| input.viewport().close_requested())
        {
            task.cancel.store(true, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.runtime_notice = Some((
                tr(
                    "Annullamento dell’installazione. Chiudi la finestra al termine.",
                    "Cancelling installation. Close the window when it finishes.",
                )
                .into(),
                false,
            ));
        }
        if let Some(editor) = &mut self.image_editor {
            let action = editor.show(ctx);
            if let Some(action) = action {
                self.editor_action(action);
            }
            return;
        }
        if let Some(editor) = &mut self.media_editor {
            let action = editor.show(ctx);
            if let Some(action) = action {
                self.editor_action(match action {
                    MediaEditorAction::Back => EditorAction::Back,
                    MediaEditorAction::SetDefaults(settings) => EditorAction::SetDefaults(settings),
                    MediaEditorAction::SaveProfile {
                        id,
                        name,
                        show_in_menu,
                        settings,
                    } => EditorAction::SaveProfile {
                        id,
                        name,
                        show_in_menu,
                        settings,
                    },
                });
            }
            return;
        }
        if self.pdf_merge.is_some() {
            let busy = self.pdf_merge.as_ref().is_some_and(|merge| merge.is_busy());
            let mut go_back = false;
            egui::TopBottomPanel::top("merge_return_to_files").show(ctx, |ui| {
                if self
                    .runtime_install
                    .as_ref()
                    .is_some_and(|task| task.cancel.load(Ordering::Relaxed))
                {
                    ui.label(tr(
                        "Annullamento dell’installazione. Chiudi la finestra al termine.",
                        "Cancelling installation. Close the window when it finishes.",
                    ));
                }
                go_back = ui
                    .add_enabled(
                        !busy,
                        egui::Button::new(tr(
                            "← Torna ai file da convertire",
                            "← Back to files to convert",
                        )),
                    )
                    .clicked();
            });
            if go_back {
                self.pdf_merge = None;
                ctx.request_repaint();
            } else if let Some(merge) = &mut self.pdf_merge {
                eframe::App::update(merge, ctx, frame);
            }
            return;
        }
        theme::sync_window_theme(ctx);
        self.receive_events();
        self.quick_queue.refresh();
        if !self.running {
            let dropped: Vec<_> = ctx.input(|input| {
                input
                    .raw
                    .dropped_files
                    .iter()
                    .filter_map(|file| file.path.clone())
                    .collect()
            });
            if !dropped.is_empty() {
                self.add_files(dropped);
                self.view = MainView::Files;
            }
        }
        let colors = theme::palette(ctx);
        egui::TopBottomPanel::top("navigation")
            .frame(
                egui::Frame::new()
                    .fill(colors.canvas)
                    .inner_margin(egui::Margin::symmetric(20, 12)),
            )
            .show(ctx, |ui| self.header(ui));
        if self.view == MainView::Settings {
            self.tools_window(ctx);
        } else if self.view == MainView::Profiles {
            let mut action = None;
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(colors.canvas).inner_margin(24))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("profiles_scroll")
                        .show(ui, |ui| {
                            action = self.profile_manager.show(
                                ui,
                                &self.saved.conversion_profiles,
                                self.saved.default_profile_id.as_deref(),
                                &self.saved.enabled_output_formats,
                                &self.tools,
                            );
                        });
                });
            if let Some(action) = action {
                self.profile_manager_action(action);
            }
        } else {
            if self.view == MainView::Files {
                egui::TopBottomPanel::bottom("footer")
                    .frame(
                        egui::Frame::new()
                            .fill(colors.card)
                            .stroke(egui::Stroke::new(1.0_f32, colors.stroke))
                            .inner_margin(egui::Margin::symmetric(24, 16)),
                    )
                    .show(ctx, |ui| self.footer(ui, ctx));
            }
            if self.view == MainView::Files {
                egui::SidePanel::right("settings")
                    .exact_width(370.0)
                    .resizable(false)
                    .frame(
                        egui::Frame::new()
                            .fill(colors.canvas)
                            .inner_margin(egui::Margin::symmetric(16, 20)),
                    )
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("settings_scroll")
                            .show(ui, |ui| {
                                ui.add_enabled_ui(!self.running, |ui| self.settings_panel(ui))
                            });
                    });
            }
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(colors.canvas).inner_margin(20))
                .show(ctx, |ui| {
                    if let Some(notice) = self.notice.clone() {
                        info_bar(ui, &notice, InfoKind::Warning);
                        if ui
                            .small_button(tr("Chiudi avviso", "Dismiss notice"))
                            .clicked()
                        {
                            self.notice = None;
                        }
                        ui.add_space(8.0);
                    }
                    match self.view {
                        MainView::Files => self.queue_panel(ui),
                        MainView::History => {
                            egui::ScrollArea::vertical()
                                .id_salt("unified_history")
                                .show(ui, |ui| {
                                    if let Some(error) = &self.history_error {
                                        info_bar(ui, error, InfoKind::Error);
                                    }
                                    let mut local = self.gui_jobs.clone();
                                    if self.running {
                                        local.extend(
                                            self.items
                                                .iter()
                                                .enumerate()
                                                .filter(|(_, item)| {
                                                    matches!(
                                                        item.state,
                                                        JobState::Waiting | JobState::Running(..)
                                                    )
                                                })
                                                .map(|(index, item)| {
                                                    session_job(item, index, self.conversion_format)
                                                }),
                                        );
                                    }
                                    if self.quick_queue.jobs.is_empty() && local.is_empty() {
                                        self.quick_report_panel(ui);
                                    }
                                    if self.quick_queue.show(ui, &mut self.file_icons, &local) {
                                        self.cancel.store(true, Ordering::Relaxed);
                                    }
                                });
                        }
                        MainView::Settings | MainView::Profiles => {}
                    }
                });
        }
        if self.settings_dirty && (!self.running || self.language_dirty) {
            self.persist_with_force(self.language_dirty);
        }
        self.runtime_install_confirmation(ctx);
        self.license_dialog(ctx);
        if self.running {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        ctx.request_repaint_after(QUEUE_REFRESH);
        if ctx.input(|input| !input.raw.hovered_files.is_empty()) {
            let screen = ctx.content_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("drop_overlay"),
            ));
            painter.rect_filled(screen, 0.0, Color32::from_rgba_unmultiplied(0, 45, 75, 235));
            painter.text(
                screen.center(),
                egui::Align2::CENTER_CENTER,
                if self.running {
                    tr(
                        "Attendi il termine della conversione",
                        "Wait for the conversion to finish",
                    )
                } else {
                    tr(
                        "Rilascia per aggiungere alla lista",
                        "Drop to add to the list",
                    )
                },
                theme::semibold_font(theme::TITLE),
                Color32::WHITE,
            );
        }
        if self.running && ctx.input(|input| input.viewport().close_requested()) {
            self.cancel.store(true, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.notice = Some(
                tr(
                    "Annullamento della conversione. Chiudi la finestra al termine.",
                    "Cancelling conversion. Close the window when it finishes.",
                )
                .into(),
            );
        }
        if let Some((message, _)) = crate::gui_history::guard_close(
            ctx,
            self.running || self.runtime_install.is_some(),
            false,
        ) {
            self.notice = Some(message);
        }
    }
}

impl Drop for ConverterApp {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

fn select_format_defaults(settings: &mut ConversionSettings, format: OutputFormat) {
    if settings.format != format {
        settings.media_options.clear();
    }
    settings.select_output_format(format);
}

fn session_job(item: &QueueItem, index: usize, format: OutputFormat) -> QueueJob {
    let (state, progress, message, outputs, error) = match &item.state {
        JobState::Waiting => (QuickJobState::Queued, 0.0, String::new(), Vec::new(), None),
        JobState::Running(progress, message) => (
            QuickJobState::Running,
            *progress,
            message.clone(),
            Vec::new(),
            None,
        ),
        JobState::Complete(outputs) => (
            QuickJobState::Completed,
            1.0,
            String::new(),
            outputs.clone(),
            None,
        ),
        JobState::Failed(error) => (
            QuickJobState::Failed,
            0.0,
            String::new(),
            Vec::new(),
            Some(error.clone()),
        ),
        JobState::Cancelled => (
            QuickJobState::Cancelled,
            0.0,
            String::new(),
            Vec::new(),
            None,
        ),
    };
    QueueJob {
        id: format!("session-{index}"),
        input: item.path.clone(),
        format: format.label().into(),
        outputs,
        state,
        progress,
        message,
        error,
    }
}

const QUEUE_REFRESH: std::time::Duration = std::time::Duration::from_millis(500);

fn background_reader<T: Send + 'static>(
    mut read: impl FnMut() -> anyhow::Result<T> + Send + 'static,
) -> (mpsc::SyncSender<()>, mpsc::Receiver<Result<T, String>>) {
    let (requests, triggers) = mpsc::sync_channel(1);
    let (results, snapshots) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        while triggers.recv().is_ok() {
            let snapshot = read().map_err(|error| format!("{error:#}"));
            if results.send(snapshot).is_err() {
                break;
            }
        }
    });
    (requests, snapshots)
}

fn poll_background<T>(
    requests: &mpsc::SyncSender<()>,
    snapshots: &mpsc::Receiver<Result<T, String>>,
    last_refresh: &mut Option<std::time::Instant>,
) -> Option<Result<T, String>> {
    let mut latest = None;
    loop {
        match snapshots.try_recv() {
            Ok(snapshot) => latest = Some(snapshot),
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                if latest.is_none() {
                    latest = Some(Err(tr(
                        "La lettura in background della coda è stata interrotta",
                        "Background queue reading was interrupted",
                    )
                    .into()));
                }
                break;
            }
        }
    }
    if last_refresh.is_none_or(|time| time.elapsed() >= QUEUE_REFRESH) {
        *last_refresh = Some(std::time::Instant::now());
        // One pending request is enough; a slow disk must not create a backlog.
        let _ = requests.try_send(());
    }
    latest
}

type QueueSnapshot = (Vec<QueueJob>, Vec<String>);

struct QuickQueueView {
    jobs: Vec<QueueJob>,
    last_refresh: Option<std::time::Instant>,
    error: Option<String>,
    action_error: Option<String>,
    warnings: Vec<String>,
    requests: mpsc::SyncSender<()>,
    snapshots: mpsc::Receiver<Result<QueueSnapshot, String>>,
}

impl Default for QuickQueueView {
    fn default() -> Self {
        let (requests, snapshots) = background_reader(queue::list_jobs_with_warnings);
        Self {
            jobs: Vec::new(),
            last_refresh: None,
            error: None,
            action_error: None,
            warnings: Vec::new(),
            requests,
            snapshots,
        }
    }
}

impl QuickQueueView {
    fn refresh(&mut self) {
        let Some(snapshot) =
            poll_background(&self.requests, &self.snapshots, &mut self.last_refresh)
        else {
            return;
        };
        match snapshot {
            Ok((jobs, warnings)) => {
                self.jobs = jobs;
                self.warnings = warnings;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(trf!(
                    "Impossibile leggere la coda: {error:#}",
                    "Unable to read the queue: {error:#}"
                ))
            }
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, icons: &mut FileIcons, local: &[QueueJob]) -> bool {
        for warning in &self.warnings {
            info_bar(ui, warning, InfoKind::Warning);
        }
        for error in self.error.iter().chain(self.action_error.iter()) {
            info_bar(ui, error, InfoKind::Error);
        }
        let mut jobs: Vec<_> = self.jobs.iter().chain(local.iter()).collect();
        jobs.sort_by_key(|job| match job.state {
            QuickJobState::Running => 0,
            QuickJobState::Queued => 1,
            _ => 2,
        });
        let mut cancel = None;
        for job in jobs {
            theme::show_card(ui, |ui| {
                ui.horizontal(|ui| {
                    file_icon(ui, icons, &job.input);
                    ui.allocate_ui_with_layout(
                        egui::vec2((ui.available_width() - 110.0).max(160.0), 42.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.add(
                                egui::Label::new(theme::heading(
                                    job.input.file_name().unwrap_or_default().to_string_lossy(),
                                    theme::BODY,
                                ))
                                .truncate(),
                            )
                            .on_hover_text(display_path(&job.input));
                            ui.label(
                                RichText::new(format!(
                                    "{} · {}",
                                    job.format,
                                    quick_state_label(&job.state)
                                ))
                                .color(quick_state_color(&job.state, ui)),
                            );
                        },
                    );
                    if quick_job_active(job)
                        && settings_ui::tool_button(
                            ui,
                            FluentIcon::Cancel,
                            tr("Annulla", "Cancel"),
                            true,
                        )
                        .clicked()
                    {
                        cancel = Some(job.id.clone());
                    }
                    if settings_ui::tool_button(
                        ui,
                        FluentIcon::Folder,
                        tr("Apri cartella", "Open folder"),
                        true,
                    )
                    .clicked()
                    {
                        self.action_error =
                            open_job_folder(job).err().map(|error| format!("{error:#}"));
                    }
                });
                if matches!(job.state, QuickJobState::Running) {
                    progress_display(ui, job.progress, &job.message);
                }
                if let Some(error) = &job.error {
                    info_bar(ui, error, InfoKind::Error);
                    if settings_ui::tool_button(
                        ui,
                        FluentIcon::Copy,
                        tr("Copia errore", "Copy error"),
                        true,
                    )
                    .clicked()
                    {
                        ui.ctx().copy_text(error.clone());
                    }
                }
            });
            ui.add_space(8.0);
        }
        if let Some(id) = cancel {
            if id.starts_with("session-") {
                return true;
            }
            self.action_error = queue::cancel_job(&id)
                .err()
                .map(|error| format!("{error:#}"));
            self.last_refresh = None;
        }
        false
    }
}

struct QueueStatusApp {
    id: String,
    job: Option<QueueJob>,
    last_refresh: Option<std::time::Instant>,
    error: Option<String>,
    action_error: Option<String>,
    cancel_sent: bool,
    previews_removed: bool,
    removal_result: Option<mpsc::Receiver<Result<(), String>>>,
    file_icons: FileIcons,
    requests: mpsc::SyncSender<()>,
    snapshots: mpsc::Receiver<Result<QueueJob, String>>,
}

impl QueueStatusApp {
    fn new(id: String) -> Self {
        let requested_id = id.clone();
        let (requests, snapshots) = background_reader(move || queue::get_job(&requested_id));
        Self {
            id,
            job: None,
            last_refresh: None,
            error: None,
            action_error: None,
            cancel_sent: false,
            previews_removed: false,
            removal_result: None,
            file_icons: FileIcons::default(),
            requests,
            snapshots,
        }
    }

    fn refresh(&mut self) {
        let Some(snapshot) =
            poll_background(&self.requests, &self.snapshots, &mut self.last_refresh)
        else {
            return;
        };
        match snapshot {
            Ok(job) => {
                self.job = Some(job);
                self.error = None;
            }
            Err(error) => {
                self.error = Some(trf!(
                    "Stato non disponibile: {error:#}",
                    "Status unavailable: {error:#}"
                ))
            }
        }
    }
}

impl eframe::App for QueueStatusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::sync_window_theme(ctx);
        self.refresh();
        if let Some(receiver) = &self.removal_result {
            let result = match receiver.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err(tr("Rimozione interrotta", "Removal interrupted").into()))
                }
            };
            if let Some(result) = result {
                self.removal_result = None;
                match result {
                    Ok(()) => {
                        self.previews_removed = true;
                        self.action_error = None;
                    }
                    Err(error) => {
                        self.action_error = Some(trf!(
                            "Impossibile rimuovere i segnaposto: {error}",
                            "Unable to remove placeholders: {error}"
                        ))
                    }
                }
            }
        }

        let colors = theme::palette(ctx);
        let mut cancel_requested = false;
        let mut remove_requested = false;
        egui::TopBottomPanel::bottom("queue_status_actions")
            .frame(
                egui::Frame::new()
                    .fill(colors.card)
                    .stroke(egui::Stroke::new(1.0_f32, colors.stroke))
                    .inner_margin(18),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(tr(
                            "La coda continua anche chiudendo questa finestra.",
                            "The queue keeps running when you close this window.",
                        ))
                        .size(theme::CAPTION)
                        .color(colors.secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(tr("Chiudi", "Close")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        if self
                            .job
                            .as_ref()
                            .is_some_and(|job| job.state == QuickJobState::Failed)
                            && ui
                                .add_enabled(
                                    !self.previews_removed && self.removal_result.is_none(),
                                    egui::Button::new(if self.previews_removed {
                                        tr("Segnaposto rimossi", "Placeholders removed")
                                    } else if self.removal_result.is_some() {
                                        tr("Rimozione…", "Removing…")
                                    } else {
                                        tr("Rimuovi segnaposto", "Remove placeholders")
                                    }),
                                )
                                .clicked()
                        {
                            remove_requested = true;
                        }

                        if self.job.as_ref().is_some_and(quick_job_active)
                            && ui
                                .add_enabled(
                                    !self.cancel_sent,
                                    egui::Button::new(theme::button_text(
                                        FluentIcon::Cancel,
                                        if self.cancel_sent {
                                            tr("Annullamento…", "Cancelling…")
                                        } else {
                                            tr("Annulla", "Cancel")
                                        },
                                        colors.text,
                                    )),
                                )
                                .clicked()
                        {
                            cancel_requested = true;
                        }
                    });
                });
            });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(colors.canvas).inner_margin(24)).show(ctx,|ui| {
            egui::ScrollArea::vertical().id_salt("queue_status_content").show(ui,|ui| {
                ui.label(theme::heading(tr("Stato della conversione", "Conversion status"),theme::SUBTITLE));
                ui.add_space(14.0);
                for error in self.error.iter().chain(self.action_error.iter()) { info_bar(ui,error,InfoKind::Error); }
                if let Some(job)=&self.job {
                    theme::show_card(ui, |ui| {
                        ui.horizontal(|ui| {
                            file_icon(ui,&mut self.file_icons,&job.input);
                            ui.vertical(|ui| {
                                ui.add(egui::Label::new(theme::heading(job.input.file_name().unwrap_or_default().to_string_lossy(),theme::BODY)).wrap());
                                ui.label(RichText::new(trf!("Destinazione: {}", "Target: {}",job.format)).size(theme::CAPTION).color(colors.secondary));
                            });
                        });
                        ui.add_space(12.0);
                        ui.label(theme::heading(quick_state_label(&job.state),theme::BODY).color(quick_state_color(&job.state,ui)));
                        if matches!(job.state,QuickJobState::Running) { progress_display(ui,job.progress,&job.message); }
                        else if !job.message.is_empty() { ui.add(egui::Label::new(RichText::new(&job.message).size(theme::CAPTION).color(colors.secondary)).wrap()); }
                        if self.cancel_sent && quick_job_active(job) { info_bar(ui,tr("Annullamento richiesto. Attendi la pulizia dei file temporanei.", "Cancellation requested. Wait for temporary files to be cleaned up."),InfoKind::Info); }
                        if let Some(error)=&job.error { info_bar(ui,error,InfoKind::Error); if ui.button(tr("Copia errore", "Copy error")).clicked() { ctx.copy_text(error.clone()); } }
                    });
                    ui.add_space(10.0);
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading(tr("File di destinazione", "Output files"),theme::BODY));
                        for output in &job.outputs { ui.add(egui::Label::new(RichText::new(display_path(output)).size(theme::CAPTION).color(colors.secondary)).wrap()); }
                        if ui.add(egui::Button::new(theme::button_text(FluentIcon::Folder,tr("Apri cartella", "Open folder"),colors.text))).clicked() { self.action_error=open_job_folder(job).err().map(|error|format!("{error:#}")); }
                    });
                    if quick_job_active(job) { ui.add_space(8.0); info_bar(ui,tr("Il segnaposto in Esplora file viene sostituito dal risultato al termine.", "The placeholder in File Explorer is replaced by the result when complete."),InfoKind::Info); }
                } else if self.error.is_none() { ui.spinner(); ui.label(tr("Lettura della coda…", "Reading queue…")); }
            });
        });
        if cancel_requested {
            match queue::cancel_job(&self.id) {
                Ok(()) => {
                    self.cancel_sent = true;
                    self.action_error = None;
                }
                Err(error) => {
                    self.action_error = Some(trf!(
                        "Impossibile annullare: {error:#}",
                        "Unable to cancel: {error:#}"
                    ))
                }
            }
            self.last_refresh = None;
        }
        if remove_requested {
            let id = self.id.clone();
            let (sender, receiver) = mpsc::channel();
            match std::thread::Builder::new()
                .name("remove-queue-previews".into())
                .spawn(move || {
                    let result = queue::remove_failed_placeholders(&id)
                        .map_err(|error| format!("{error:#}"));
                    let _ = sender.send(result);
                }) {
                Ok(_) => self.removal_result = Some(receiver),
                Err(error) => {
                    self.action_error = Some(trf!(
                        "Impossibile rimuovere i segnaposto: {error}",
                        "Unable to remove placeholders: {error}"
                    ))
                }
            }
        }
        ctx.request_repaint_after(QUEUE_REFRESH);
    }
}

fn quick_job_active(job: &QueueJob) -> bool {
    matches!(job.state, QuickJobState::Queued | QuickJobState::Running)
}

fn quick_state_label(state: &QuickJobState) -> &'static str {
    match state {
        QuickJobState::Queued => tr("In coda", "Queued"),
        QuickJobState::Running => tr("Conversione in corso", "Converting"),
        QuickJobState::Completed => tr("Completato", "Completed"),
        QuickJobState::Failed => tr("Conversione non riuscita", "Conversion failed"),
        QuickJobState::Cancelled => tr("Annullato", "Cancelled"),
    }
}

fn quick_state_color(state: &QuickJobState, ui: &egui::Ui) -> Color32 {
    match state {
        QuickJobState::Completed => theme::palette(ui.ctx()).success,
        QuickJobState::Failed => ui.visuals().error_fg_color,
        QuickJobState::Running => theme::palette(ui.ctx()).accent,
        _ => ui.visuals().weak_text_color(),
    }
}

fn quick_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn open_job_folder(job: &QueueJob) -> anyhow::Result<()> {
    let folder = job
        .outputs
        .first()
        .and_then(|path| path.parent())
        .or_else(|| job.input.parent())
        .ok_or_else(|| {
            anyhow::anyhow!(tr(
                "La cartella della conversione non è disponibile",
                "The conversion folder is unavailable"
            ))
        })?;
    platform::open_folder(folder)
}

fn settings_path() -> Option<PathBuf> {
    i18n::preferences_path()
}

fn navigation_tabs(ui: &mut egui::Ui, selected: &mut MainView) -> [egui::Rect; 4] {
    let mut rectangles = [egui::Rect::NOTHING; 4];
    for (index, (view, label, icon)) in [
        (
            MainView::Files,
            tr("File da convertire", "Files to convert"),
            FluentIcon::Folder,
        ),
        (
            MainView::History,
            tr("Cronologia", "History"),
            FluentIcon::History,
        ),
        (
            MainView::Profiles,
            tr("Profili", "Profiles"),
            FluentIcon::Profiles,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        rectangles[index] = navigation_tab(ui, selected, view, label, icon);
    }
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        rectangles[3] = navigation_tab(
            ui,
            selected,
            MainView::Settings,
            tr("Impostazioni", "Settings"),
            FluentIcon::Settings,
        );
    });
    rectangles
}

fn navigation_tab(
    ui: &mut egui::Ui,
    selected: &mut MainView,
    view: MainView,
    label: &str,
    icon: FluentIcon,
) -> egui::Rect {
    let colors = theme::palette(ui.ctx());
    let response = ui.add(
        egui::Button::new(theme::button_text(
            icon,
            label,
            if *selected == view {
                colors.accent
            } else {
                colors.text
            },
        ))
        .frame(false),
    );
    if response.clicked() {
        *selected = view;
    }
    if *selected == view {
        let y = response.rect.bottom() + 4.0;
        ui.painter().line_segment(
            [
                egui::pos2(response.rect.left() + 8.0, y),
                egui::pos2(response.rect.right() - 8.0, y),
            ],
            egui::Stroke::new(2.0_f32, colors.accent),
        );
    }
    response.rect
}

fn icon_button(ui: &mut egui::Ui, icon: FluentIcon, tooltip: &str) -> egui::Response {
    ui.add(
        egui::Button::new(theme::icon(icon, 16.0))
            .frame(false)
            .min_size(egui::vec2(30.0, 30.0)),
    )
    .on_hover_text(tooltip)
}

fn file_icon(ui: &mut egui::Ui, icons: &mut FileIcons, path: &Path) {
    if let Some(texture) = icons.get(ui.ctx(), path) {
        ui.add(egui::Image::new((texture, egui::vec2(28.0, 28.0))));
    } else {
        ui.label(theme::icon(FluentIcon::Document, 28.0).color(theme::palette(ui.ctx()).secondary));
    }
}

#[derive(Clone, Copy)]
enum InfoKind {
    Info,
    Warning,
    Error,
}

fn info_bar(ui: &mut egui::Ui, message: &str, kind: InfoKind) {
    let colors = theme::palette(ui.ctx());
    let tone = match kind {
        InfoKind::Info => colors.accent,
        InfoKind::Warning => ui.visuals().warn_fg_color,
        InfoKind::Error => colors.error,
    };
    egui::Frame::new()
        .fill(tone.gamma_multiply(if ui.visuals().dark_mode { 0.13 } else { 0.07 }))
        .stroke(egui::Stroke::new(1.0_f32, tone.gamma_multiply(0.45)))
        .corner_radius(theme::CONTROL_RADIUS)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.label(theme::icon(FluentIcon::Info, 16.0).color(tone));
                ui.add(
                    egui::Label::new(
                        RichText::new(message)
                            .size(theme::CAPTION)
                            .color(colors.text),
                    )
                    .wrap(),
                );
            });
        });
    ui.add_space(6.0);
}

fn setting_card(
    ui: &mut egui::Ui,
    title: &str,
    detail: &str,
    controls: impl FnOnce(&mut egui::Ui),
) {
    theme::show_card(ui, |ui| {
        ui.horizontal(|ui| {
            let left = (ui.available_width() - 155.0).max(90.0);
            ui.allocate_ui_with_layout(
                egui::vec2(left, 40.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(theme::heading(title, theme::BODY))
                        .on_hover_text(detail);
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(145.0, 40.0),
                egui::Layout::right_to_left(egui::Align::Center),
                controls,
            );
        });
    });
}

fn duration_hms(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60
    )
}

fn progress_display(ui: &mut egui::Ui, fraction: f32, message: &str) {
    let colors = theme::palette(ui.ctx());
    let fraction = quick_progress(fraction);
    ui.horizontal(|ui| {
        ui.add(
            egui::Label::new(
                RichText::new(message)
                    .size(theme::CAPTION)
                    .color(colors.secondary),
            )
            .wrap(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::heading(
                format!("{:.0}%", fraction * 100.0),
                theme::CAPTION,
            ));
        });
    });
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 3.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 1.5, colors.stroke);
    if fraction > 0.0 {
        let mut fill = rect;
        fill.max.x = fill.min.x + fill.width() * fraction;
        ui.painter().rect_filled(fill, 1.5, colors.accent);
    }
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn kind_label(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Audio => "Audio",
        MediaKind::Video => "Video",
        MediaKind::Image => tr("Immagine", "Image"),
        MediaKind::Pdf => tr("Documento PDF", "PDF document"),
        MediaKind::Document => tr("Documento", "Document"),
        MediaKind::Spreadsheet => tr("Foglio di calcolo", "Spreadsheet"),
        MediaKind::Html => tr("Pagina HTML", "HTML page"),
        MediaKind::Unknown => "File",
    }
}

fn width_setting(ui: &mut egui::Ui, id: &str, width: &mut Option<u32>) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(
            width
                .map(|n| trf!("Larghezza {n} px", "Width {n} px"))
                .unwrap_or(tr("Dimensioni originali", "Original dimensions").into()),
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(width, None, tr("Originali", "Original"));
            for n in [320, 640, 1280, 1920, 2560, 3840] {
                ui.selectable_value(width, Some(n), trf!("{n} px di larghezza", "{n} px wide"));
            }
        });
    if let Some(width) = width {
        let minimum = if id == "video_width" { 16 } else { 1 };
        ui.horizontal(|ui| {
            ui.label(tr("Larghezza", "Width"));
            ui.add(
                egui::DragValue::new(width)
                    .range(minimum..=16384)
                    .suffix(" px"),
            );
        });
    }
    ui.label(
        RichText::new(tr(
            "Le proporzioni vengono mantenute.",
            "Aspect ratio is preserved.",
        ))
        .small()
        .weak(),
    );
}

fn quality_applicable(format: OutputFormat) -> bool {
    matches!(
        format,
        OutputFormat::Mp3
            | OutputFormat::Aac
            | OutputFormat::Ogg
            | OutputFormat::Opus
            | OutputFormat::M4a
            | OutputFormat::Wma
            | OutputFormat::Ac3
            | OutputFormat::Mp4
            | OutputFormat::Mkv
            | OutputFormat::Mov
            | OutputFormat::Webm
            | OutputFormat::Avi
            | OutputFormat::Mpeg
            | OutputFormat::Wmv
            | OutputFormat::Flv
            | OutputFormat::Ts
            | OutputFormat::M4v
            | OutputFormat::ThreeGp
            | OutputFormat::Jpg
            | OutputFormat::Avif
            | OutputFormat::Heic
            | OutputFormat::Pdf
    )
}

fn optional_seconds(ui: &mut egui::Ui, label: &str, value: &mut Option<f64>, default_seconds: f64) {
    ui.horizontal(|ui| {
        let mut enabled = value.is_some();
        if ui.checkbox(&mut enabled, label).changed() {
            *value = if enabled { Some(default_seconds) } else { None };
        }
        if let Some(seconds) = value {
            ui.add(
                egui::DragValue::new(seconds)
                    .speed(0.1)
                    .range(0.0..=86400.0)
                    .suffix(" s"),
            );
        }
    });
}

#[cfg(test)]
mod queue_reader_tests {
    use super::*;

    #[test]
    fn saved_web_profile_roundtrips_without_losing_legacy_defaults_and_obeys_menu_settings() {
        let mut saved: SavedState=serde_json::from_str(r#"{"settings":{"format":"Jpg","image":{"quality":73,"width":640}},"overrides":{},"preset":3}"#).unwrap();
        assert!(saved.conversion_profiles.is_empty());
        assert_eq!(saved.settings.image.quality, 73);
        assert_eq!(saved.settings.image.width, Some(640));
        let mut recipe = ConversionSettings {
            format: OutputFormat::Webp,
            ..Default::default()
        };
        recipe.image.edit = Some(the_converter::image_edit::ImageEditSettings {
            max_bytes: Some(300_000),
            allow_downsize: true,
            ..Default::default()
        });
        let id = saved
            .conversion_profiles
            .create("Salva per web".into(), true, recipe)
            .unwrap();
        saved.default_profile_id = Some(id.clone());
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        persist_preferences(&path, &saved, &mut None, std::time::Instant::now(), true).unwrap();
        let mut loaded: SavedState =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded.default_profile_id.as_deref(), Some(id.as_str()));
        let revision = loaded.conversion_profiles.get(&id).unwrap().revision;
        let (profile, _) = profile_configuration(&loaded, &id, Some(revision)).unwrap();
        assert_eq!(
            profile.settings.image.edit.unwrap().max_bytes,
            Some(300_000)
        );
        loaded
            .enabled_output_formats
            .set_enabled(OutputFormat::Webp, false);
        assert!(profile_configuration(&loaded, &id, Some(revision)).is_err());
        loaded
            .enabled_output_formats
            .set_enabled(OutputFormat::Webp, true);
        loaded
            .conversion_profiles
            .set_show_in_menu(&id, false)
            .unwrap();
        assert!(profile_configuration(&loaded, &id, None).is_err());
    }

    #[test]
    fn oversized_preferences_never_replace_the_last_valid_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let mut saved = SavedState::default();
        persist_preferences(&path, &saved, &mut None, std::time::Instant::now(), true).unwrap();
        let previous = std::fs::read(&path).unwrap();
        saved.overrides.insert(
            "ffmpeg".into(),
            PathBuf::from("x".repeat(MAX_PREFERENCES_BYTES as usize)),
        );
        assert!(
            persist_preferences(&path, &saved, &mut None, std::time::Instant::now(), true).is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), previous);
    }

    #[test]
    fn language_preferences_migrate_without_resetting_conversion_or_ocr_settings() {
        let old: SavedState = serde_json::from_str(
            r#"{"settings":{"format":"Flac","ocr":{"enabled":true,"language":"fra"}},"preset":2}"#,
        )
        .unwrap();
        assert_eq!(old.ui_language, Language::System);
        assert_eq!(old.settings.format, OutputFormat::Flac);
        assert_eq!(old.settings.ocr.language, "fra");
        assert!(old.settings.ocr.enabled);
        assert_eq!(old.preset, 2);

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let mut saved = old;
        let mut last = None;
        for language in [Language::English, Language::Italian, Language::System] {
            saved.ui_language = language;
            persist_preferences(&path, &saved, &mut last, std::time::Instant::now(), true).unwrap();
            let restored: SavedState =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            assert_eq!(restored.ui_language, language);
            assert_eq!(restored.settings.format, OutputFormat::Flac);
            assert_eq!(restored.settings.ocr.language, "fra");
            assert!(restored.settings.ocr.enabled);
            assert_eq!(restored.preset, 2);
        }
    }

    #[test]
    fn queue_and_media_labels_follow_the_ui_language() {
        for (language, image, queued, complete) in [
            (Language::Italian, "Immagine", "In coda", "Completato"),
            (Language::English, "Image", "Queued", "Completed"),
        ] {
            i18n::with_language(language, || {
                assert_eq!(kind_label(MediaKind::Image), image);
                assert_eq!(quick_state_label(&QuickJobState::Queued), queued);
                assert_eq!(quick_state_label(&QuickJobState::Completed), complete);
            });
        }
    }

    #[test]
    fn preference_drag_updates_are_throttled_and_close_flushes_latest_values() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let mut saved = SavedState::default();
        let mut last = None;
        let now = std::time::Instant::now();
        assert!(persist_preferences(&path, &saved, &mut last, now, false).unwrap());
        let original = std::fs::read(&path).unwrap();
        saved.settings.image.quality = 42;
        for milliseconds in [0, 1, 25, 100, 250, 499] {
            let next_frame = now + std::time::Duration::from_millis(milliseconds);
            assert!(!persist_preferences(&path, &saved, &mut last, next_frame, false).unwrap());
        }
        assert_eq!(std::fs::read(&path).unwrap(), original);

        let next_save = now + PREFERENCES_SAVE_INTERVAL;
        assert!(persist_preferences(&path, &saved, &mut last, next_save, false).unwrap());
        let stored: SavedState = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(stored.settings.image.quality, 42);

        saved.settings.image.quality = 73;
        assert!(persist_preferences(&path, &saved, &mut last, next_save, true).unwrap());
        let stored: SavedState = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(stored.settings.image.quality, 73);
    }

    #[test]
    fn failed_preference_writes_are_throttled_and_can_be_flushed_again() {
        let directory = tempfile::tempdir().unwrap();
        let parent = directory.path().join("settings");
        let path = parent.join("preferences.json");
        std::fs::write(&parent, b"blocked").unwrap();
        let saved = SavedState::default();
        let mut last = None;
        let now = std::time::Instant::now();
        assert!(persist_preferences(&path, &saved, &mut last, now, false).is_err());
        std::fs::remove_file(&parent).unwrap();
        assert!(!persist_preferences(&path, &saved, &mut last, now, false).unwrap());
        assert!(!path.exists());
        assert!(persist_preferences(&path, &saved, &mut last, now, true).unwrap());
        assert!(path.is_file());
    }

    #[test]
    fn slow_queue_io_runs_off_the_ui_thread_and_polling_remains_nonblocking() {
        let ui_thread = std::thread::current().id();
        let (started, ready) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let (requests, snapshots) = background_reader(move || {
            assert_ne!(std::thread::current().id(), ui_thread);
            started.send(()).unwrap();
            gate.recv().unwrap();
            Ok(17)
        });
        let mut last_refresh = None;
        assert!(poll_background(&requests, &snapshots, &mut last_refresh).is_none());
        ready
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        assert!(poll_background(&requests, &snapshots, &mut last_refresh).is_none());
        release.send(()).unwrap();
        assert_eq!(
            snapshots
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap()
                .unwrap(),
            17
        );
    }

    #[test]
    fn closing_a_queue_view_disconnects_and_releases_its_reader() {
        let (lifetime, released) = mpsc::channel::<()>();
        let (requests, snapshots) = background_reader(move || Ok(lifetime.clone()));
        requests.send(()).unwrap();
        drop(
            snapshots
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap()
                .unwrap(),
        );
        drop(requests);
        drop(snapshots);
        assert!(matches!(
            released.recv_timeout(std::time::Duration::from_secs(2)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }
}

#[cfg(test)]
mod fluent_ui_tests {
    use super::*;

    #[test]
    fn selector_buttons_switch_views_without_creating_an_app_or_touching_userdata() {
        let ctx = egui::Context::default();
        theme::configure(&ctx);
        let mut selected = MainView::Files;
        let mut rectangles = [egui::Rect::NOTHING; 4];
        let input = |events| egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(700.0, 100.0),
            )),
            events,
            ..Default::default()
        };
        let _ = ctx.run(input(Vec::new()), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| rectangles = navigation_tabs(ui, &mut selected));
            });
        });
        assert!(rectangles[0].right() < rectangles[1].left());
        assert!(rectangles[1].right() < rectangles[2].left());
        assert!(rectangles[2].right() < rectangles[3].left());
        assert!(
            rectangles[3].right() > 650.0,
            "Settings stays at the right edge"
        );
        for (index, expected) in [
            (1, MainView::History),
            (2, MainView::Profiles),
            (3, MainView::Settings),
            (0, MainView::Files),
        ] {
            let point = rectangles[index].center();
            for pressed in [true, false] {
                let _ = ctx.run(
                    input(vec![
                        egui::Event::PointerMoved(point),
                        egui::Event::PointerButton {
                            pos: point,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::default(),
                        },
                    ]),
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.horizontal(|ui| rectangles = navigation_tabs(ui, &mut selected));
                        });
                    },
                );
            }
            assert_eq!(selected, expected);
        }
    }
}
