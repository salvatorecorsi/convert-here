use crate::{
    file_icons::FileIcons,
    image_editor::{self, EditorAction, ImageEditor},
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

const PREFERENCES_SAVE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);

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
                if app.items.len() == 1 && image_editor::editable(&app.items[0].path) {
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
            .with_title("Convert Here · Stato conversione")
            .with_icon(app_icon())
            .with_inner_size([650.0, 470.0])
            .with_min_inner_size([500.0, 360.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Convert Here · Stato conversione",
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
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("Cartella delle preferenze non disponibile"))?;
    std::fs::create_dir_all(parent)?;
    use std::io::Write;
    let bytes = serde_json::to_vec_pretty(saved).map_err(std::io::Error::other)?;
    if bytes.len() as u64 > MAX_PREFERENCES_BYTES {
        return Err(std::io::Error::other(
            "Le preferenze superano il limite di 256 KiB. Riduci il numero o la dimensione dei profili.",
        ));
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
        "Il profilo non è attivo nel menu Converti in. Riapri il menu dopo averlo abilitato."
    );
    anyhow::ensure!(
        saved
            .enabled_output_formats
            .contains(profile.settings.format),
        "Il formato di uscita del profilo è disattivato. Abilitalo nelle Impostazioni."
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
    QuickQueue,
    History,
}

struct ProfileDialog {
    id: Option<String>,
    name: String,
    show_in_menu: bool,
    settings: ConversionSettings,
}

pub struct ConverterApp {
    pdf_merge: Option<crate::pdf_merge_app::PdfMergeApp>,
    image_editor: Option<ImageEditor>,
    profile_dialog: Option<ProfileDialog>,
    view: MainView,
    conversion_started: Option<std::time::Instant>,
    conversion_elapsed: std::time::Duration,
    file_icons: FileIcons,
    saved: SavedState,
    tools: ToolPaths,
    items: Vec<QueueItem>,
    running: bool,
    cancel: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<Event>>,
    show_tools: bool,
    pending_runtime_install: Option<Vec<RuntimeEngine>>,
    runtime_install: Option<RuntimeInstallTask>,
    runtime_notice: Option<(String, bool)>,
    show_license: bool,
    advanced: bool,
    notice: Option<String>,
    settings_dirty: bool,
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
            saved.settings.select_output_format(format);
        }
        let tools = ToolPaths::configured(&saved.overrides);
        let mut app = Self {
            pdf_merge: None,
            image_editor: None,
            profile_dialog: None,
            view: MainView::Files,
            conversion_started: None,
            conversion_elapsed: std::time::Duration::ZERO,
            file_icons: FileIcons::default(),
            saved,
            tools,
            items: Vec::new(),
            running: false,
            cancel: Arc::new(AtomicBool::new(false)),
            receiver: None,
            show_tools: false,
            pending_runtime_install: None,
            runtime_install: None,
            runtime_notice: None,
            show_license: false,
            advanced: false,
            notice: None,
            settings_dirty: false,
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
                Ok(true) => self.settings_dirty = false,
                Ok(false) => {}
                Err(err) => {
                    self.notice = Some(format!("Impossibile salvare le preferenze: {err}"));
                }
            }
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
            self.notice=Some("Il formato di questo profilo è disattivato. Riabilitalo in Formati di uscita prima di modificarne le opzioni.".into());
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
        let mut editor = ImageEditor::new(input, text, settings, self.tools.clone(), formats);
        if editing_profile && let Some(profile) = profile {
            editor.editing_profile(profile.id, profile.name, profile.show_in_menu);
        }
        self.image_editor = Some(editor);
        self.show_tools = false;
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
            self.notice =
                Some("Abilita il formato di uscita del profilo nelle Impostazioni.".into());
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
            }
            EditorAction::SetDefaults(settings) => {
                self.saved.settings = settings;
                self.saved.preset = 3;
                self.saved.default_profile_id = None;
                self.settings_dirty = true;
                self.persist_with_force(true);
                let error = self.settings_dirty;
                let message = if error {
                    self.notice
                        .clone()
                        .unwrap_or_else(|| "Impossibile salvare le impostazioni.".into())
                } else {
                    "Impostazioni predefinite salvate. Gli originali restano invariati.".into()
                };
                if let Some(editor) = &mut self.image_editor {
                    editor.notify(message, error);
                }
            }
            EditorAction::SaveProfile {
                id,
                name,
                show_in_menu,
                settings,
            } => {
                let result = if let Some(id) = id {
                    self.saved
                        .conversion_profiles
                        .update(&id, name.clone(), show_in_menu, settings)
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
                            self.notice
                                .clone()
                                .unwrap_or_else(|| "Impossibile salvare il profilo.".into())
                        } else {
                            format!(
                                "Profilo «{}» salvato. Il menu Converti in si aggiorna alla prossima apertura.",
                                name.trim()
                            )
                        };
                        if let Some(editor) = &mut self.image_editor {
                            editor.editing_profile(id, name.trim().into(), show_in_menu);
                            editor.notify(message, error);
                        }
                    }
                    Err(error) => {
                        if let Some(editor) = &mut self.image_editor {
                            editor.notify(format!("{error:#}"), true);
                        }
                    }
                }
            }
        }
    }

    fn profile_settings(&mut self, ui: &mut egui::Ui) {
        let profiles: Vec<_> = self.saved.conversion_profiles.iter().cloned().collect();
        let mut edit = None;
        let mut apply = None;
        let mut rename = None;
        let mut remove = None;
        let mut update = None;
        let mut visibility = None;
        let mut make_default = None;
        let mut create_web = false;
        theme::show_card(ui, |ui| {
            ui.label(theme::heading("I miei profili", theme::SUBTITLE));
            ui.label("Salva formato, qualità e opzioni. I profili visibili compaiono in «Converti in» ed eseguono subito la conversione.");
            ui.label(format!(
                "{} di 32 salvati · massimo 8 nel menu",
                profiles.len()
            ));
            if ui.button("Crea «Salva per web»").clicked() {
                create_web = true;
            }
            ui.label(RichText::new("WebP, massimo 300 kB: adatta qualità e, se necessario, riduce le dimensioni. Potrai modificarlo nell’editor.").small().weak());
            if profiles.is_empty() {
                ui.label("Puoi creare altri profili dalle opzioni di conversione o dall’editor.");
            }
            for profile in &profiles {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label(theme::heading(&profile.name, theme::BODY));
                    ui.label(profile_summary(profile));
                    if self.saved.default_profile_id.as_deref() == Some(profile.id.as_str()) {
                        ui.label("Predefinito");
                    }
                });
                let mut visible = profile.show_in_menu;
                if ui.checkbox(&mut visible, "Mostra in Converti in").changed() {
                    visibility = Some((profile.id.clone(), visible));
                }
                if !self
                    .saved
                    .enabled_output_formats
                    .contains(profile.settings.format)
                {
                    ui.label(RichText::new("Il formato è disattivato: riabilitalo in Formati di uscita per vedere questo profilo nel menu.").color(theme::palette(ui.ctx()).secondary));
                }
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Applica").clicked() {
                        apply = Some(profile.clone());
                    }
                    if ui.button("Modifica opzioni…").clicked() {
                        edit = Some(profile.clone());
                    }
                    if ui.button("Rinomina…").clicked() {
                        rename = Some(profile.clone());
                    }
                    if ui.button("Aggiorna con le opzioni correnti").clicked() {
                        update = Some(profile.clone());
                    }
                    if ui.button("Usa come predefinito").clicked() {
                        make_default = Some(profile.clone());
                    }
                    if ui.button("Elimina").clicked() {
                        remove = Some(profile.id.clone());
                    }
                });
            }
        });
        if create_web {
            let mut settings = ConversionSettings {
                format: OutputFormat::Webp,
                ..Default::default()
            };
            settings.output_dir = self.saved.settings.output_dir.clone();
            settings.image.quality = 90;
            settings.image.edit = Some(the_converter::image_edit::ImageEditSettings {
                max_bytes: Some(300_000),
                allow_downsize: true,
                ..Default::default()
            });
            match self
                .saved
                .conversion_profiles
                .create("Salva per web".into(), true, settings)
            {
                Ok(id) => {
                    self.settings_dirty = true;
                    self.persist_with_force(true);
                    if let Some(profile) = self.saved.conversion_profiles.get(&id).cloned() {
                        edit = Some(profile);
                    }
                }
                Err(error) => self.notice = Some(format!("{error:#}")),
            }
        }
        if let Some((id, visible)) = visibility {
            match self
                .saved
                .conversion_profiles
                .set_show_in_menu(&id, visible)
            {
                Ok(()) => self.settings_dirty = true,
                Err(error) => self.notice = Some(format!("{error:#}")),
            }
        }
        if let Some(id) = remove {
            self.saved.conversion_profiles.remove(&id);
            if self.saved.default_profile_id.as_deref() == Some(id.as_str()) {
                self.saved.default_profile_id = None;
            }
            self.settings_dirty = true;
        }
        if let Some(profile) = update {
            match self.saved.conversion_profiles.update(
                &profile.id,
                profile.name,
                profile.show_in_menu,
                self.saved.settings.clone(),
            ) {
                Ok(()) => self.settings_dirty = true,
                Err(error) => self.notice = Some(format!("{error:#}")),
            }
        }
        if let Some(profile) = rename {
            self.profile_dialog = Some(ProfileDialog {
                id: Some(profile.id),
                name: profile.name,
                show_in_menu: profile.show_in_menu,
                settings: profile.settings,
            });
        }
        if let Some(profile) = make_default {
            self.saved.default_profile_id = Some(profile.id.clone());
            self.saved.settings = profile.settings;
            self.saved.preset = 3;
            self.sync_output_choice();
            self.settings_dirty = true;
        }
        if let Some(profile) = apply {
            self.apply_profile(&profile);
            self.show_tools = false;
        }
        if let Some(profile) = edit {
            if matches!(
                profile.settings.format,
                OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp
            ) || profile.settings.image.edit.is_some()
                || profile.settings.image.page.is_some()
            {
                let input = self
                    .items
                    .iter()
                    .find(|item| {
                        image_editor::editable(&item.path)
                            && profile.validate_for_inputs([item.path.as_path()]).is_ok()
                    })
                    .map(|item| item.path.clone());
                self.open_editor(input, Some(profile));
            } else {
                self.apply_profile(&profile);
                self.advanced = true;
                self.show_tools = false;
                self.notice =
                    Some("Modifica le opzioni, poi aggiorna il profilo nelle Impostazioni.".into());
            }
        }
        if self.settings_dirty {
            self.persist_with_force(true);
        }
    }

    fn profile_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.profile_dialog.take() else {
            return;
        };
        let mut save = false;
        let mut close = false;
        egui::Modal::new(egui::Id::new("save_conversion_profile")).show(ctx,|ui| {
            ui.set_width(430.0);
            ui.label(theme::heading(if dialog.id.is_some(){"Aggiorna profilo"}else{"Salva profilo"},theme::SUBTITLE));
            ui.label(format!("Formato: {}. Le opzioni correnti saranno riutilizzate per ogni conversione del profilo.",dialog.settings.format.label()));
            ui.add(egui::TextEdit::singleline(&mut dialog.name).hint_text("Nome profilo").char_limit(40));
            ui.checkbox(&mut dialog.show_in_menu,"Mostra in Converti in");
            ui.horizontal(|ui|{
                save=ui.add_enabled(!dialog.name.trim().is_empty(),egui::Button::new("Salva")).clicked();
                close=ui.button("Annulla").clicked();
            });
        });
        if save {
            let result = if let Some(id) = &dialog.id {
                self.saved
                    .conversion_profiles
                    .update(
                        id,
                        dialog.name.clone(),
                        dialog.show_in_menu,
                        dialog.settings.clone(),
                    )
                    .map(|()| id.clone())
            } else {
                self.saved.conversion_profiles.create(
                    dialog.name.clone(),
                    dialog.show_in_menu,
                    dialog.settings.clone(),
                )
            };
            match result {
                Ok(_) => {
                    self.settings_dirty = true;
                    self.persist_with_force(true);
                }
                Err(error) => {
                    self.notice = Some(format!("{error:#}"));
                    self.profile_dialog = Some(dialog);
                }
            }
        } else if !close && !ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.profile_dialog = Some(dialog);
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
            self.notice = Some(format!(
                "{rejected} elementi ignorati: cartelle o formati non supportati."
            ));
        }
        let common = self.common_formats();
        if !common.is_empty() && !common.contains(&self.saved.settings.format) {
            self.saved.settings.select_output_format(common[0]);
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
            self.saved.settings.select_output_format(*format);
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
                message: "Connessione al distributore…".into(),
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
                            finished = Some(Err(
                                "L’installazione si è interrotta. Puoi riprovare.".into(),
                            ));
                        }
                        break;
                    }
                }
            }
        }
        if !installed.is_empty() {
            for (engine, directory) in installed {
                for (key, path) in engine.tool_paths(&directory) {
                    if executable_available(&path) {
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
                    "Installazione completata. I motori sono pronti per le conversioni locali."
                        .into(),
                    false,
                ),
                Err(error) => (error, true),
            });
        }
    }

    fn output_format_settings(&mut self, ui: &mut egui::Ui) {
        let before = self.saved.enabled_output_formats.clone();
        theme::show_card(ui, |ui| {
            ui.label(theme::heading("Formati di uscita", theme::SUBTITLE));
            ui.label("Scegli i formati da mostrare nell’app e in «Converti in». Tutti i formati in ingresso restano supportati.");
            ui.horizontal_wrapped(|ui| {
                let count = self
                    .saved
                    .enabled_output_formats
                    .filter(OutputFormat::ALL)
                    .count();
                ui.label(format!("{count} di {} attivi", OutputFormat::ALL.len()));
                if ui.button("Più comuni").clicked() {
                    self.saved.enabled_output_formats.reset_common();
                }
                if ui.button("Attiva tutti").clicked() {
                    self.saved.enabled_output_formats.enable_all();
                }
                if ui.button("Disattiva tutti").clicked() {
                    self.saved.enabled_output_formats.disable_all();
                }
            });
            for (index, title) in ["Audio", "Video", "Immagini", "Documenti e PDF"]
                .into_iter()
                .enumerate()
            {
                let formats: Vec<_> = OutputFormat::ALL
                    .into_iter()
                    .filter(|format| match index {
                        0 => format.kind() == MediaKind::Audio,
                        1 => format.kind() == MediaKind::Video,
                        2 => format.kind() == MediaKind::Image,
                        _ => matches!(
                            format.kind(),
                            MediaKind::Pdf
                                | MediaKind::Document
                                | MediaKind::Spreadsheet
                                | MediaKind::Html
                        ),
                    })
                    .collect();
                let enabled = self
                    .saved
                    .enabled_output_formats
                    .filter(formats.iter().copied())
                    .count();
                egui::CollapsingHeader::new(format!("{title} · {enabled}/{}", formats.len()))
                    .id_salt(("enabled_output_category", index))
                    .default_open(index == 0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button("Tutti").clicked() {
                                for format in &formats {
                                    self.saved.enabled_output_formats.set_enabled(*format, true);
                                }
                            }
                            if ui.small_button("Nessuno").clicked() {
                                for format in &formats {
                                    self.saved
                                        .enabled_output_formats
                                        .set_enabled(*format, false);
                                }
                            }
                        });
                        ui.horizontal_wrapped(|ui| {
                            for format in formats {
                                let mut enabled =
                                    self.saved.enabled_output_formats.contains(format);
                                if ui
                                    .checkbox(
                                        &mut enabled,
                                        format!("{} (.{})", format.label(), format.extension()),
                                    )
                                    .changed()
                                {
                                    self.saved
                                        .enabled_output_formats
                                        .set_enabled(format, enabled);
                                }
                            }
                        });
                    });
            }
            if self.saved.enabled_output_formats.is_empty() {
                info_bar(
                    ui,
                    "Non ci sono uscite attive. Abilita almeno un formato per convertire; puoi comunque aggiungere i file.",
                    InfoKind::Info,
                );
            }
            ui.label(RichText::new("Per esempio: un FLAC resta selezionabile e offre MP3 e WAV con le impostazioni iniziali.").small().weak());
        });
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
            if ui.small_button("Chiudi messaggio").clicked() {
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
                            "Annullamento…"
                        } else {
                            "Annulla installazione"
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
            ui.label(theme::heading("Installa motori opzionali", theme::SUBTITLE));
            ui.label("Scarica i motori dalle distribuzioni indicate e li verifica prima di estrarli. Il download richiede Internet; i tuoi file da convertire restano sul PC.");
            ui.add_space(8.0);
            for engine in &engines {
                let manifest = engine.manifest();
                ui.label(format!("{} {} · {:.1} MB · {}", manifest.name, manifest.version, manifest.download_bytes as f64 / 1_000_000.0, manifest.license));
                ui.label(format!("Spazio dopo l’estrazione: {:.1} MB", manifest.expanded_bytes as f64 / 1_000_000.0));
                let source = match engine {
                    RuntimeEngine::Ffmpeg => "https://www.gyan.dev/ffmpeg/builds/",
                    RuntimeEngine::Poppler => "https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0",
                };
                if ui.link("Distribuzione e licenze").on_hover_text(source).clicked()
                    && let Err(error) = platform::open_website(source)
                {
                    self.notice = Some(format!("{error:#}"));
                }
            }
            if let Some(directory) = the_converter::tools::user_tools_dir() {
                ui.add_space(8.0);
                ui.label("Cartella di installazione:");
                ui.add(egui::Label::new(display_path(&directory)).wrap());
            }
            ui.label(RichText::new("Le licenze dei motori restano incluse. Non occorrono abbonamenti o pagamenti.").small().weak());
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                confirmed = ui.button("Scarica e installa").clicked();
                dismissed = ui.button("Annulla").clicked();
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
            ui.label(theme::heading("GNU GPL versione 3", theme::SUBTITLE));
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
                if ui.button("Copia testo").clicked() {
                    ctx.copy_text(include_str!("../LICENSE").into());
                }
                close = ui.button("Chiudi").clicked();
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
            return Some("La fine dell’intervallo deve essere successiva all’inizio.".into());
        }
        if self.common_formats().is_empty() {
            if !the_converter::model::common_formats(
                self.items.iter().map(|item| item.path.as_path()),
            )
            .is_empty()
            {
                return Some("Per questi file non ci sono formati di uscita attivi. Abilitali nelle Impostazioni; i formati dei file in ingresso restano supportati.".into());
            }
            return Some("Questi file richiedono formati di uscita diversi. Crea una coda con file compatibili.".into());
        }
        if !self.common_formats().contains(&self.saved.settings.format) {
            return Some(format!(
                "Il formato {} non è compatibile con tutti i file. Scegli un altro formato di uscita.",
                self.saved.settings.format.label()
            ));
        }
        if self.saved.settings.format == OutputFormat::Pdf
            && self.saved.settings.pdf.action == PdfAction::Merge
            && (self.items.len() < 2 || !self.common_formats().contains(&OutputFormat::Pdf))
        {
            return Some("Per PDF (Merge) aggiungi almeno due file convertibili in PDF. Potrai ordinare le singole pagine prima di creare il PDF unico.".into());
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
            return Some("Il formato scelto richiede una frequenza audio diversa. Scegli una frequenza compatibile nelle impostazioni.".into());
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
                    self.items[index].state = JobState::Running(0.0, "Preparazione…".into())
                }
                Event::Progress(index, fraction, message) => {
                    self.items[index].state = JobState::Running(fraction.clamp(0.0, 1.0), message)
                }
                Event::Complete(index, outputs) => {
                    self.items[index].state = JobState::Complete(outputs)
                }
                Event::Failed(index, error) => self.items[index].state = JobState::Failed(error),
                Event::Cancelled(index) => self.items[index].state = JobState::Cancelled,
                Event::End => {
                    self.running = false;
                    if let Some(started) = self.conversion_started.take() {
                        self.conversion_elapsed = started.elapsed();
                    }
                    self.receiver = None;
                    if self.cancel.load(Ordering::Relaxed) {
                        for item in &mut self.items {
                            if matches!(item.state, JobState::Waiting | JobState::Running(..)) {
                                item.state = JobState::Cancelled;
                            }
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
        let colors = theme::palette(ui.ctx());
        ui.horizontal(|ui| {
            if self.show_tools {
                if ui
                    .add(
                        egui::Button::new(theme::button_text(
                            FluentIcon::Back,
                            "Indietro",
                            colors.text,
                        ))
                        .frame(false),
                    )
                    .clicked()
                {
                    self.show_tools = false;
                }
                ui.label(theme::heading("Impostazioni", theme::SUBTITLE));
            } else {
                navigation_tabs(ui, &mut self.view);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(theme::button_text(
                                FluentIcon::Settings,
                                "Impostazioni",
                                colors.text,
                            ))
                            .frame(false),
                        )
                        .clicked()
                    {
                        self.show_tools = true;
                    }
                });
            }
        });
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui) {
        let colors = theme::palette(ui.ctx());
        ui.label(theme::heading("Opzioni", theme::SUBTITLE));
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
                .selected_text("Applica un profilo…")
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
            .small_button("Salva queste opzioni come profilo…")
            .clicked()
        {
            self.profile_dialog = Some(ProfileDialog {
                id: None,
                name: String::new(),
                show_in_menu: true,
                settings: self.saved.settings.clone(),
            });
        }
        if let Some(path) = self
            .items
            .iter()
            .find(|item| image_editor::editable(&item.path))
            .map(|item| item.path.clone())
            && ui
                .button(if MediaKind::from_path(&path) == MediaKind::Image {
                    "Modifica immagine…"
                } else {
                    "Impagina testo…"
                })
                .clicked()
        {
            self.open_editor(Some(path), None);
        }
        ui.add_space(8.0);
        let formats = self.common_formats();
        let before = self.saved.settings.format;
        setting_card(ui, "Formato", "Scegli il file da creare", |ui| {
            ui.add_enabled_ui(!formats.is_empty(), |ui| {
                egui::ComboBox::from_id_salt("target_format")
                    .width(142.0)
                    .selected_text(if self.items.is_empty() {
                        "Aggiungi un file"
                    } else if formats.is_empty() {
                        "Nessuna uscita attiva"
                    } else {
                        self.saved.settings.format.label()
                    })
                    .show_ui(ui, |ui| {
                        for format in formats {
                            let label = if format == OutputFormat::Txt
                                && self.items.iter().any(|item| {
                                    MediaKind::from_path(&item.path) == MediaKind::Image
                                }) {
                                "TXT · OCR locale"
                            } else if format == OutputFormat::Pdf
                                && self
                                    .items
                                    .iter()
                                    .all(|item| MediaKind::from_path(&item.path) == MediaKind::Pdf)
                            {
                                "PDF · operazioni"
                            } else {
                                format.label()
                            };
                            ui.selectable_value(&mut self.saved.settings.format, format, label);
                        }
                    });
            });
        });
        if ui.link("Scegli i formati di uscita…").clicked() {
            self.show_tools = true;
        }
        if before != self.saved.settings.format {
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
                0 => "File più piccoli",
                2 => "Più dettaglio",
                3 => "Valori personalizzati",
                _ => "Qualità e dimensioni bilanciate",
            };
            setting_card(ui, "Qualità", detail, |ui| {
                ui.vertical(|ui| {
                    for (index, title) in [(0, "Leggero"), (1, "Bilanciato"), (2, "Alta qualità")]
                    {
                        if ui.radio(self.saved.preset == index, title).clicked() {
                            self.apply_preset(index);
                        }
                    }
                });
            });
            ui.add_space(6.0);
        }
        theme::show_card(ui, |ui| {
            ui.label(theme::heading("Destinazione", theme::BODY));
            let mut beside = self.saved.settings.output_dir.is_none();
            if ui
                .radio_value(&mut beside, true, "Accanto all’originale")
                .changed()
            {
                self.saved.settings.output_dir = None;
                self.settings_dirty = true;
            }
            if ui
                .radio_value(&mut beside, false, "Cartella a scelta")
                .clicked()
                && self.saved.settings.output_dir.is_none()
                && let Some(path) = rfd::FileDialog::new()
                    .set_title("Cartella di destinazione")
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
                        "Cambia cartella…",
                        colors.text,
                    )))
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new().set_directory(path).pick_folder()
                {
                    self.saved.settings.output_dir = Some(path);
                    self.settings_dirty = true;
                }
            }
            ui.label(
                RichText::new("Gli originali e i risultati già presenti restano al sicuro.")
                    .size(theme::CAPTION)
                    .color(colors.secondary),
            );
        });
        ui.add_space(6.0);
        theme::show_card(ui, |ui| {
            egui::CollapsingHeader::new(theme::heading("Impostazioni personalizzate", theme::BODY))
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
                "GIF: primo fotogramma. TIFF: prima pagina.",
                InfoKind::Info,
            );
        }
        if let Some(message) = self.prerequisite() {
            ui.add_space(6.0);
            info_bar(ui, &message, InfoKind::Warning);
            if ui.button("Configura strumenti…").clicked() {
                self.show_tools = true;
            }
        }
    }

    fn advanced_settings(&mut self, ui: &mut egui::Ui) {
        if self.items.iter().any(|item| {
            item.path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
        }) || self.saved.settings.format == OutputFormat::Csv
        {
            ui.label("Separatore CSV in ingresso e uscita");
            let delimiter = &mut self.saved.settings.csv.delimiter;
            egui::ComboBox::from_id_salt("csv_delimiter")
                .selected_text(match delimiter {
                    CsvDelimiter::Comma => "Virgola (,)",
                    CsvDelimiter::Semicolon => "Punto e virgola (;)",
                    CsvDelimiter::Tab => "Tabulazione",
                    CsvDelimiter::Pipe => "Barra verticale (|)",
                })
                .show_ui(ui, |ui| {
                    for (value, label) in [
                        (CsvDelimiter::Comma, "Virgola (,)"),
                        (CsvDelimiter::Semicolon, "Punto e virgola (;)"),
                        (CsvDelimiter::Tab, "Tabulazione"),
                        (CsvDelimiter::Pipe, "Barra verticale (|)"),
                    ] {
                        ui.selectable_value(delimiter, value, label);
                    }
                });
            ui.label(RichText::new("UTF-8 con virgolette. Il separatore scelto viene usato anche per leggere i CSV.").small().weak());
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
                    "Riconosci le scansioni con OCR locale",
                )
                .changed()
            && self.saved.settings.ocr.enabled
        {
            self.saved.settings.pdf.dpi = self.saved.settings.pdf.dpi.max(300);
        }
        if image_ocr || pdf_text && self.saved.settings.ocr.enabled {
            ui.label("Lingue OCR installate");
            egui::ComboBox::from_id_salt("ocr_languages")
                .selected_text(&self.saved.settings.ocr.language)
                .show_ui(ui, |ui| {
                    for (code, name) in [
                        ("ita+eng", "Italiano e inglese"),
                        ("ita", "Italiano"),
                        ("eng", "Inglese"),
                        ("fra", "Francese"),
                        ("deu", "Tedesco"),
                        ("spa", "Spagnolo"),
                    ] {
                        ui.selectable_value(
                            &mut self.saved.settings.ocr.language,
                            code.into(),
                            name,
                        );
                    }
                });
            ui.text_edit_singleline(&mut self.saved.settings.ocr.language);
            ui.label(RichText::new("Tesseract riconosce il testo sul PC. Occorrono i dati delle lingue indicate; rileggi il risultato.").small().weak());
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
                    ui.label("Bitrate audio");
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
                        "Frequenza originale".into()
                    } else {
                        format!("{rate} Hz")
                    })
                    .show_ui(ui, |ui| {
                        for &value in rates {
                            ui.selectable_value(
                                rate,
                                value,
                                if value == 0 {
                                    "Originale".into()
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
                        _ => "Canali originali",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(channels, 0, "Originali");
                        ui.selectable_value(channels, 1, "Mono");
                        ui.selectable_value(channels, 2, "Stereo");
                    });
                ui.checkbox(
                    &mut self.saved.settings.audio.normalize,
                    "Normalizza volume",
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
                        RichText::new("Formato senza perdita: il bitrate non viene applicato.")
                            .small()
                            .weak(),
                    );
                }
                ui.label("Ritaglia intervallo (secondi)");
                optional_seconds(ui, "Da", &mut self.saved.settings.video.trim_start);
                optional_seconds(ui, "A", &mut self.saved.settings.video.trim_end);
            }
            MediaKind::Video => {
                if matches!(
                    self.saved.settings.format,
                    OutputFormat::Mp4 | OutputFormat::Mkv | OutputFormat::Mov | OutputFormat::M4v
                ) {
                    egui::ComboBox::from_id_salt("codec")
                        .selected_text(match self.saved.settings.video.codec {
                            VideoCodec::H264 => "H.264 · compatibilità",
                            VideoCodec::H265 => "H.265 · file più piccoli",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.saved.settings.video.codec,
                                VideoCodec::H264,
                                "H.264 · compatibilità",
                            );
                            ui.selectable_value(
                                &mut self.saved.settings.video.codec,
                                VideoCodec::H265,
                                "H.265 · file più piccoli",
                            );
                        });
                }
                if self.saved.settings.format != OutputFormat::Gif {
                    ui.label("Qualità video · valori minori = più dettaglio");
                    let quality_label = if matches!(
                        self.saved.settings.format,
                        OutputFormat::Webm
                            | OutputFormat::Avi
                            | OutputFormat::Mpeg
                            | OutputFormat::Wmv
                            | OutputFormat::Flv
                            | OutputFormat::ThreeGp
                    ) {
                        "Qualità "
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
                            .unwrap_or("Fotogrammi originali".into()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(fps, None, "Originali");
                        for n in [24, 25, 30, 50, 60] {
                            ui.selectable_value(fps, Some(n), format!("{n} fps"));
                        }
                    });
                ui.label("Ritaglia intervallo (secondi)");
                optional_seconds(ui, "Da", &mut self.saved.settings.video.trim_start);
                optional_seconds(ui, "A", &mut self.saved.settings.video.trim_end);
                if self.saved.settings.format != OutputFormat::Gif {
                    ui.label(
                        RichText::new("La traccia audio viene conservata quando disponibile.")
                            .small()
                            .weak(),
                    );
                    egui::CollapsingHeader::new("Audio del video").show(ui, |ui| {
                        ui.label("Bitrate audio");
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
                                "Frequenza originale".into()
                            } else {
                                format!("{rate} Hz")
                            })
                            .show_ui(ui, |ui| {
                                for &value in rates {
                                    ui.selectable_value(
                                        rate,
                                        value,
                                        if value == 0 {
                                            "Originale".into()
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
                                _ => "Canali originali",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(channels, 0, "Originali");
                                ui.selectable_value(channels, 1, "Mono");
                                ui.selectable_value(channels, 2, "Stereo");
                            });
                        ui.checkbox(
                            &mut self.saved.settings.audio.normalize,
                            "Normalizza volume",
                        );
                    });
                } else {
                    ui.label(
                        RichText::new("Il GIF animato non contiene audio.")
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
                    ui.label("Risoluzione delle pagine PDF");
                    ui.add(
                        egui::Slider::new(&mut self.saved.settings.pdf.dpi, 72..=600)
                            .suffix(" DPI"),
                    );
                    ui.label(
                        RichText::new("Un’immagine per pagina, nella cartella di destinazione.")
                            .small()
                            .weak(),
                    );
                } else {
                    if matches!(
                        self.saved.settings.format,
                        OutputFormat::Jpg | OutputFormat::Avif | OutputFormat::Heic
                    ) {
                        ui.label("Qualità dell'immagine");
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
                            PdfAction::Compress if all_pdf => "Comprimi senza perdita",
                            PdfAction::Compress => "PDF (Bulk)",
                            PdfAction::Split => "Dividi: una pagina per file",
                            PdfAction::Merge => "PDF (Merge) · ordina pagine",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                action,
                                PdfAction::Compress,
                                if all_pdf {
                                    "Comprimi senza perdita"
                                } else {
                                    "PDF (Bulk)"
                                },
                            );
                            if all_pdf {
                                ui.selectable_value(
                                    action,
                                    PdfAction::Split,
                                    "Dividi: una pagina per file",
                                );
                            }
                            ui.add_enabled_ui(can_merge, |ui| {
                                ui.selectable_value(
                                    action,
                                    PdfAction::Merge,
                                    "PDF (Merge) · ordina pagine",
                                );
                            });
                        });
                    ui.label(RichText::new(match action {
                        PdfAction::Compress if all_pdf => "Ottimizza ogni PDF senza ridurre la qualità. Un PDF già ottimizzato può mantenere le stesse dimensioni.",
                        PdfAction::Compress => "Crea un PDF separato per ciascun file della lista.",
                        PdfAction::Split => "Crea un PDF per pagina. Segnalibri, struttura accessibile e metadati del documento non vengono trasferiti; documenti firmati o con moduli compilabili richiedono una copia statica.",
                        PdfAction::Merge => "Apre l'ordine delle singole pagine: trascinale prima di creare un unico PDF. Segnalibri, struttura accessibile e metadati non vengono trasferiti; documenti firmati o con moduli compilabili richiedono una copia statica.",
                    }).small().weak());
                }
                if self
                    .items
                    .iter()
                    .any(|item| MediaKind::from_path(&item.path) == MediaKind::Image)
                {
                    ui.label("Qualità delle immagini nel PDF");
                    ui.add(
                        egui::Slider::new(&mut self.saved.settings.image.quality, 20..=100)
                            .suffix("%"),
                    );
                    width_setting(ui, "pdf_image_width", &mut self.saved.settings.image.width);
                    ui.label(RichText::new("Una pagina A4 per immagine, con proporzioni mantenute e margine di 24 pt.").small().weak());
                } else if !all_pdf {
                    ui.label(
                        RichText::new(
                            "Documenti Office, fogli di calcolo e HTML vengono esportati in PDF con LibreOffice. HTML/CSS complessi possono cambiare impaginazione; JavaScript non viene eseguito. Le risorse collegate nell'HTML possono essere lette dal motore.",
                        )
                        .small()
                        .weak(),
                    );
                }
            }
            MediaKind::Spreadsheet => {
                let text = if self.saved.settings.format == OutputFormat::Csv {
                    "Esporta il primo foglio in CSV UTF-8 con il separatore scelto e virgolette. Conserva i valori delle celle; formattazione, formule e altri fogli non sono inclusi."
                } else {
                    "Crea un file Excel XLSX con LibreOffice. I CSV in ingresso devono essere UTF-8 e usare il separatore scelto; il contenuto che inizia con = resta testo."
                };
                ui.label(RichText::new(text).small().weak());
            }
            MediaKind::Document | MediaKind::Html => {
                let text = if self.saved.settings.format == OutputFormat::Txt {
                    "Estrae il testo incorporato nel PDF; per le scansioni abilita OCR locale qui sopra."
                } else {
                    "Ricostruisce il testo del PDF in un file modificabile. Per le scansioni abilita OCR locale qui sopra; immagini, tabelle e impaginazione originali non vengono ricostruite."
                };
                ui.label(RichText::new(text).small().weak());
            }
            _ => {
                ui.label(RichText::new("Il testo viene estratto dal PDF. I PDF scansionati richiedono OCR, qui non incluso.").small().weak());
            }
        }
    }

    fn queue_panel(&mut self, ui: &mut egui::Ui) {
        let colors = theme::palette(ui.ctx());
        ui.horizontal(|ui| {
            ui.label(theme::heading(
                format!(
                    "File da convertire{}",
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
                    .add_enabled(!self.running, egui::Button::new("Nuovo foglio…"))
                    .clicked()
                {
                    self.open_editor(None, None);
                }
                if ui
                    .add_enabled(
                        !self.running,
                        egui::Button::new(theme::button_text(
                            FluentIcon::Add,
                            "Aggiungi file",
                            colors.text,
                        )),
                    )
                    .clicked()
                    && let Some(paths) = rfd::FileDialog::new()
                        .set_title("Scegli i file da convertire")
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
                    ui.label(theme::heading("Trascina qui i tuoi file", theme::SUBTITLE));
                    ui.label(
                        RichText::new("Audio, video, immagini e documenti.")
                            .color(colors.secondary),
                    );
                    ui.add_space(16.0);
                    if ui
                        .add(
                            egui::Button::new(theme::button_text(
                                FluentIcon::Add,
                                "Scegli file…",
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
                    ui.label(
                        RichText::new("Puoi anche usare il tasto destro in Esplora file.")
                            .size(theme::CAPTION)
                            .color(colors.secondary),
                    );
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
                                - if image_editor::editable(&item.path) {
                                    80.0
                                } else {
                                    0.0
                                })
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
                            if image_editor::editable(&item.path)
                                && ui
                                    .add_enabled(
                                        !self.running,
                                        egui::Button::new("Modifica…").small(),
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
                                            "Rimuovi dalla lista",
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
                                            .on_hover_text("Sposta dopo nell’unione")
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
                                            .on_hover_text("Sposta prima nell’unione")
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
                                                "Apri cartella di destinazione",
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
                                                RichText::new("Pronto")
                                                    .size(theme::CAPTION)
                                                    .color(colors.secondary),
                                            );
                                        }
                                        JobState::Cancelled => {
                                            ui.label(
                                                RichText::new("Annullato")
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
                                    RichText::new(format!(
                                        "Completato · {} file creati",
                                        outputs.len()
                                    ))
                                    .size(theme::CAPTION)
                                    .color(colors.success),
                                );
                            }
                            JobState::Failed(message) => {
                                info_bar(ui, "Conversione non riuscita", InfoKind::Error);
                                egui::CollapsingHeader::new("Dettagli e copia errore")
                                    .id_salt(index)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(message).size(theme::CAPTION),
                                            )
                                            .wrap(),
                                        );
                                        if ui.button("Copia errore").clicked() {
                                            ui.ctx().copy_text(message.clone());
                                        }
                                    });
                            }
                            _ => {}
                        }
                    });
                    ui.add_space(6.0);
                }
                if !self.running && ui.button("Svuota lista").clicked() {
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

    fn session_history(&mut self, ui: &mut egui::Ui) {
        let count = self
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item.state,
                    JobState::Complete(_) | JobState::Failed(_) | JobState::Cancelled
                )
            })
            .count();
        if count == 0 {
            return;
        }
        egui::CollapsingHeader::new(theme::heading(
            format!("Questa sessione · {count} file"),
            theme::BODY,
        ))
        .default_open(false)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("session_history")
                .max_height(200.0)
                .show(ui, |ui| {
                    for item in self.items.iter().filter(|item| {
                        matches!(
                            item.state,
                            JobState::Complete(_) | JobState::Failed(_) | JobState::Cancelled
                        )
                    }) {
                        theme::show_card(ui, |ui| {
                            ui.horizontal(|ui| {
                                file_icon(ui, &mut self.file_icons, &item.path);
                                ui.add(
                                    egui::Label::new(theme::heading(
                                        item.path.file_name().unwrap_or_default().to_string_lossy(),
                                        theme::BODY,
                                    ))
                                    .truncate(),
                                );
                            });
                            match &item.state {
                                JobState::Complete(outputs) => {
                                    ui.label(
                                        RichText::new(format!(
                                            "Completato · {} file creati",
                                            outputs.len()
                                        ))
                                        .color(theme::palette(ui.ctx()).success),
                                    );
                                }
                                JobState::Failed(error) => info_bar(ui, error, InfoKind::Error),
                                JobState::Cancelled => {
                                    ui.label("Annullato");
                                }
                                _ => {}
                            }
                        });
                        ui.add_space(6.0);
                    }
                });
        });
    }

    fn quick_report_panel(&mut self, ui: &mut egui::Ui) {
        let Some(report) = self.quick_report.clone() else {
            return;
        };
        egui::CollapsingHeader::new(format!(
            "Ultima conversione rapida · {} file creati",
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
                            if ui.small_button("Apri cartella").clicked()
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
            if ui.small_button("Aggiorna resoconto").clicked() {
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
                    format!(
                        "Conversione in corso · {done}/{} completati",
                        self.items.len()
                    )
                } else if done > 0 || failed > 0 {
                    format!("{done} completati · {failed} errori")
                } else if self.items.is_empty() {
                    "Pronto quando vuoi".into()
                } else {
                    format!(
                        "{} file · {}",
                        self.items.len(),
                        human_size(self.items.iter().map(|item| item.size).sum())
                    )
                };
                ui.label(theme::heading(summary, theme::BODY));
                ui.label(
                    RichText::new("Conversioni locali · originali conservati")
                        .size(theme::CAPTION)
                        .color(colors.secondary),
                );
            });
            if self.running || !self.conversion_elapsed.is_zero() {
                let elapsed = self
                    .conversion_started
                    .map(|started| started.elapsed())
                    .unwrap_or(self.conversion_elapsed);
                ui.label(
                    RichText::new(format!("Tempo trascorso {}", duration_hms(elapsed)))
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
                                    "Annullamento…"
                                } else {
                                    "Annulla"
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
                        "Ordina e unisci PDF"
                    } else if self.saved.settings.format == OutputFormat::Pdf
                        && !self.items.is_empty()
                        && self
                            .items
                            .iter()
                            .all(|item| MediaKind::from_path(&item.path) == MediaKind::Pdf)
                    {
                        match self.saved.settings.pdf.action {
                            PdfAction::Compress => "Comprimi PDF",
                            PdfAction::Split => "Dividi PDF",
                            PdfAction::Merge => "Unisci PDF",
                        }
                    } else if self.items.iter().any(|item| {
                        matches!(item.state, JobState::Complete(_) | JobState::Failed(_))
                    }) {
                        "Converti di nuovo"
                    } else {
                        "Converti file"
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
            if let Some(notice)=self.notice.clone() {
                info_bar(ui,&notice,InfoKind::Warning);
                if ui.small_button("Chiudi avviso").clicked() { self.notice=None; }
            }
            centered_column(ui, 1000.0, |ui| {
                ui.label(RichText::new("Immagini e PDF da immagini, subito disponibili.").family(egui::FontFamily::Name(theme::SEMIBOLD_FAMILY.into())));
                ui.label("Audio, video, pagine PDF e documenti Office usano motori opzionali. Puoi installare FFmpeg e Poppler con un clic oppure selezionare programmi già presenti.");
                ui.label("Convert Here è gratuito, senza abbonamenti, acquisti in-app o costi a consumo.");
                ui.label(RichText::new("Le conversioni avvengono sul PC con i motori locali. I documenti HTML possono caricare risorse collegate; i siti ufficiali si aprono nel browser.").small().weak());
                ui.add_space(5.0);
                egui::ScrollArea::vertical().id_salt("tools_scroll").max_height((ui.available_height()-60.0).max(120.0)).show(ui, |ui| {
                    ui.add_enabled_ui(!self.running, |ui| self.output_format_settings(ui));
                    ui.add_space(8.0);
                    ui.add_enabled_ui(!self.running, |ui| self.profile_settings(ui));
                    ui.add_space(8.0);
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading("Aspetto",theme::SUBTITLE));
                        ui.label(RichText::new("Segui Windows oppure prova un tema.").color(theme::palette(ui.ctx()).secondary));
                        let mut preference=ui.ctx().options(|options| options.theme_preference);
                        ui.horizontal(|ui| {
                            for (value,label) in [(egui::ThemePreference::System,"Sistema"),(egui::ThemePreference::Light,"Chiaro"),(egui::ThemePreference::Dark,"Scuro")] {
                                if ui.radio_value(&mut preference,value,label).changed() { ui.ctx().set_theme(preference); }
                            }
                        });
                    });
                    ui.add_space(8.0);
                    ui.label(theme::heading("Motori di conversione", theme::SUBTITLE));
                    ui.label("FFmpeg include FFprobe; Poppler include gli strumenti per immagini, testo e conteggio pagine PDF.");
                    if ui.add_enabled(self.runtime_install.is_none(), egui::Button::new("Installa FFmpeg e Poppler…")).clicked() {
                        self.pending_runtime_install = Some(vec![RuntimeEngine::Ffmpeg, RuntimeEngine::Poppler]);
                    }
                    self.runtime_install_progress(ui);
                    ui.add_space(8.0);
                    for (key, label, description, website) in [
                        ("ffmpeg", "FFmpeg", "Necessario per audio, video e GIF da video.", "https://ffmpeg.org/download.html"),
                        ("ffprobe", "FFprobe", "Opzionale: durata dei media e avanzamento più preciso. Di solito è accanto a ffmpeg.exe.", "https://ffmpeg.org/ffprobe.html"),
                        ("pdftoppm", "Poppler · immagini", "Necessario per esportare ogni pagina PDF in PNG o JPEG. Seleziona pdftoppm.exe.", "https://poppler.freedesktop.org/"),
                        ("pdfinfo", "Poppler · conteggio pagine", "Usa lo stesso motore Poppler per contare le pagine prima di accodare le immagini PDF. Di solito è accanto a pdftoppm.exe.", "https://poppler.freedesktop.org/"),
                        ("pdftotext", "Poppler · testo", "Necessario per PDF in TXT, Word e HTML (solo testo). Seleziona pdftotext.exe. Il riconoscimento OCR non è incluso.", "https://poppler.freedesktop.org/"),
                        ("soffice", "LibreOffice", "Necessario per Office e HTML in PDF ed Excel ↔ CSV. Seleziona soffice.exe nella cartella program di LibreOffice.", "https://www.libreoffice.org/download/download-libreoffice/"),
                        ("tesseract", "Tesseract · OCR locale", "Necessario per immagini in TXT e scansioni PDF in TXT, Word e HTML. Mantieni la cartella tessdata con le lingue accanto al programma.", "https://tesseract-ocr.github.io/tessdoc/Installation.html"),
                        ("heif-enc", "libheif · HEIC/AVIF in uscita", "Crea HEIC e AVIF. Mantieni DLL e plugin accanto agli strumenti; la build deve includere gli encoder HEVC e AV1.", "https://github.com/strukturag/libheif"),
                        ("heif-dec", "libheif · HEIC/AVIF in ingresso", "Legge HEIC, HEIF e AVIF; viene convertita la prima immagine. Mantieni DLL e plugin accanto agli strumenti.", "https://github.com/strukturag/libheif"),
                    ] {
                        theme::show_card(ui, |ui| {
                            let path = self.saved.overrides.get(key).cloned().or_else(|| get_tool(&self.tools, key));
                            let valid = path.as_ref().is_some_and(|path| executable_available(path));
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(label).family(egui::FontFamily::Name(theme::SEMIBOLD_FAMILY.into())));
                                ui.label(RichText::new(if valid { "Disponibile" } else { "Non configurato" }).small().color(if valid { theme::palette(ui.ctx()).success } else { ui.visuals().weak_text_color() }));
                            });
                            ui.add(egui::Label::new(RichText::new(description).small().weak()).wrap());
                            if let Some(path) = path {
                                egui::CollapsingHeader::new("Percorso del programma").id_salt(("tool_path",key)).default_open(false).show(ui,|ui| { ui.add(egui::Label::new(RichText::new(display_path(&path)).small()).wrap()); });
                                if !valid {
                                    ui.colored_label(ui.visuals().error_fg_color, "Il percorso salvato non è un programma .exe disponibile. Seleziona di nuovo lo strumento.");
                                }
                            }
                            ui.horizontal_wrapped(|ui| {
                                let engine = match key {
                                    "ffmpeg" => Some(RuntimeEngine::Ffmpeg),
                                    "pdftoppm" => Some(RuntimeEngine::Poppler),
                                    _ => None,
                                };
                                if let Some(engine) = engine
                                    && ui.add_enabled(self.runtime_install.is_none(), egui::Button::new("Scarica e installa…")).clicked()
                                {
                                    self.pending_runtime_install = Some(vec![engine]);
                                }
                                if ui.small_button("Seleziona programma…").clicked()
                                    && let Some(path) = rfd::FileDialog::new()
                                        .set_title(format!("Seleziona {key}.exe già installato"))
                                        .set_file_name(format!("{key}.exe"))
                                        .add_filter("Programmi Windows", &["exe"])
                                        .pick_file()
                                {
                                    if executable_available(&path) {
                                        self.saved.overrides.insert(key.into(), path);
                                        self.tools = ToolPaths::configured(&self.saved.overrides);
                                        self.settings_dirty = true;
                                    } else {
                                        self.notice = Some("Seleziona un programma esistente con estensione .exe.".into());
                                    }
                                }
                                if self.saved.overrides.contains_key(key)
                                    && ui.small_button("Usa rilevamento automatico").clicked()
                                {
                                    self.saved.overrides.remove(key);
                                    self.tools = ToolPaths::configured(&self.saved.overrides);
                                    self.settings_dirty = true;
                                }
                                if ui.link("Sito ufficiale").on_hover_text(website).clicked()
                                    && let Err(error) = platform::open_website(website)
                                {
                                    self.notice = Some(format!("{error:#}"));
                                }
                            });
                        });
                    }
                    egui::CollapsingHeader::new("Cartella personale degli strumenti").show(ui, |ui| {
                        ui.label(RichText::new("I download vengono installati qui. Puoi anche aggiungere strumenti già presenti, mantenendo le rispettive DLL e licenze. Non occorre modificare la cartella dell’app.").small().weak());
                        if let Some(path) = the_converter::tools::user_tools_dir() {
                            ui.add(egui::Label::new(RichText::new(display_path(&path)).small()).wrap());
                            if ui.small_button("Copia percorso").clicked() { ui.ctx().copy_text(display_path(&path)); }
                        }
                        ui.label(RichText::new("Sono riconosciute anche le cartelle bin, ffmpeg/bin e poppler/Library/bin. Le DLL necessarie devono restare insieme ai rispettivi programmi.").small().weak());
                    });
                    ui.add_space(8.0);
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading("Informazioni e licenze", theme::SUBTITLE));
                        ui.label(format!("Convert Here {} · © 2026 Salvatore Corsi", env!("CARGO_PKG_VERSION")));
                        ui.label("Software libero, GPL-3.0-only. Puoi usare, modificare e redistribuire il programma secondo la licenza. Il software è fornito senza garanzie, nei limiti consentiti dalla legge.");
                        ui.label("Questa licenza riguarda il programma, non i file convertiti. Dipendenze e motori mantengono le rispettive licenze.");
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Leggi la licenza").clicked() {
                                self.show_license = true;
                            }
                            if ui.link("Codice sorgente").clicked() {
                                let url = format!("https://github.com/salvatorecorsi/convert-here/releases/tag/v{}", env!("CARGO_PKG_VERSION"));
                                if let Err(error) = platform::open_website(&url) {
                                    self.notice = Some(format!("{error:#}"));
                                }
                            }
                        });
                    });
                });
                ui.separator();
                if ui.button("Rileva strumenti installati").clicked() {
                    self.tools = ToolPaths::configured(&self.saved.overrides);
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
        if let Some(task) = &self.runtime_install
            && ctx.input(|input| input.viewport().close_requested())
        {
            task.cancel.store(true, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.runtime_notice = Some((
                "Annullamento dell’installazione. Chiudi la finestra al termine.".into(),
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
        if self.pdf_merge.is_some() {
            let busy = self.pdf_merge.as_ref().is_some_and(|merge| merge.is_busy());
            let mut go_back = false;
            egui::TopBottomPanel::top("merge_return_to_files").show(ctx, |ui| {
                if self
                    .runtime_install
                    .as_ref()
                    .is_some_and(|task| task.cancel.load(Ordering::Relaxed))
                {
                    ui.label("Annullamento dell’installazione. Chiudi la finestra al termine.");
                }
                go_back = ui
                    .add_enabled(!busy, egui::Button::new("← Torna ai file da convertire"))
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
                self.show_tools = false;
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
        if self.show_tools {
            self.tools_window(ctx);
        } else {
            if self.view == MainView::Files || self.running {
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
                        if ui.small_button("Chiudi avviso").clicked() {
                            self.notice = None;
                        }
                        ui.add_space(8.0);
                    }
                    match self.view {
                        MainView::Files => self.queue_panel(ui),
                        MainView::QuickQueue => {
                            self.quick_queue.show(ui, false, &mut self.file_icons)
                        }
                        MainView::History => {
                            ui.label(theme::heading("Cronologia", theme::TITLE));
                            ui.label(
                                RichText::new(
                                    "Risultati delle conversioni rapide e dell’ultima sessione.",
                                )
                                .color(colors.secondary),
                            );
                            ui.add_space(12.0);
                            self.session_history(ui);
                            self.quick_report_panel(ui);
                            self.quick_queue.show(ui, true, &mut self.file_icons);
                        }
                    }
                });
        }
        if self.settings_dirty && !self.running {
            self.persist();
        }
        self.runtime_install_confirmation(ctx);
        self.license_dialog(ctx);
        self.profile_dialog(ctx);
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
                    "Attendi il termine della conversione"
                } else {
                    "Rilascia per aggiungere alla lista"
                },
                theme::semibold_font(theme::TITLE),
                Color32::WHITE,
            );
        }
        if self.running && ctx.input(|input| input.viewport().close_requested()) {
            self.cancel.store(true, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.notice =
                Some("Annullamento della conversione. Chiudi la finestra al termine.".into());
        }
    }
}

impl Drop for ConverterApp {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
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
                    latest = Some(Err(
                        "La lettura in background della coda è stata interrotta".into(),
                    ));
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
            Err(error) => self.error = Some(format!("Impossibile leggere la coda: {error:#}")),
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, history: bool, icons: &mut FileIcons) {
        let colors = theme::palette(ui.ctx());
        if !history {
            ui.label(theme::heading("Coda rapida", theme::TITLE));
            ui.label(
                RichText::new("I lavori proseguono anche chiudendo questa finestra.")
                    .color(colors.secondary),
            );
            ui.add_space(12.0);
        }
        for warning in &self.warnings {
            info_bar(ui, warning, InfoKind::Warning);
        }
        for error in self.error.iter().chain(self.action_error.iter()) {
            info_bar(ui, error, InfoKind::Error);
        }
        let mut cancel = None;
        let jobs: Vec<_> = if history {
            self.jobs
                .iter()
                .filter(|job| !quick_job_active(job))
                .take(50)
                .collect()
        } else {
            self.jobs
                .iter()
                .rev()
                .filter(|job| quick_job_active(job))
                .take(50)
                .collect()
        };
        if jobs.is_empty() {
            info_bar(
                ui,
                if history {
                    "Nessuna conversione rapida conclusa."
                } else {
                    "Nessuna conversione rapida in attesa o in corso."
                },
                InfoKind::Info,
            );
        }
        egui::ScrollArea::vertical()
            .id_salt(("quick_jobs", history))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for job in jobs {
                    theme::show_card(ui, |ui| {
                        ui.horizontal(|ui| {
                            file_icon(ui, icons, &job.input);
                            ui.allocate_ui_with_layout(
                                egui::vec2((ui.available_width() - 160.0).max(100.0), 42.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.add(
                                        egui::Label::new(theme::heading(
                                            job.input
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy(),
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
                                        .size(theme::CAPTION)
                                        .color(quick_state_color(&job.state, ui)),
                                    );
                                },
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if quick_job_active(job)
                                        && icon_button(
                                            ui,
                                            FluentIcon::Cancel,
                                            "Annulla conversione",
                                        )
                                        .clicked()
                                    {
                                        cancel = Some(job.id.clone());
                                    }
                                    if icon_button(
                                        ui,
                                        FluentIcon::Folder,
                                        "Apri cartella di destinazione",
                                    )
                                    .clicked()
                                    {
                                        self.action_error = open_job_folder(job)
                                            .err()
                                            .map(|error| format!("{error:#}"));
                                    }
                                },
                            );
                        });
                        if matches!(job.state, QuickJobState::Running) {
                            progress_display(ui, job.progress, &job.message);
                        } else if !job.message.is_empty() {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&job.message)
                                        .size(theme::CAPTION)
                                        .color(colors.secondary),
                                )
                                .wrap(),
                            );
                        }
                        if let Some(path) = job.outputs.first() {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(display_path(path))
                                        .size(theme::CAPTION)
                                        .color(colors.secondary),
                                )
                                .truncate(),
                            )
                            .on_hover_text(
                                job.outputs
                                    .iter()
                                    .map(|path| display_path(path))
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            );
                        }
                        if let Some(error) = &job.error {
                            info_bar(ui, error, InfoKind::Error);
                            if ui.small_button("Copia errore").clicked() {
                                ui.ctx().copy_text(error.clone());
                            }
                        }
                    });
                    ui.add_space(8.0);
                }
            });
        if let Some(id) = cancel {
            self.action_error = queue::cancel_job(&id)
                .err()
                .map(|error| format!("Impossibile annullare: {error:#}"));
            self.last_refresh = None;
        }
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
            Err(error) => self.error = Some(format!("Stato non disponibile: {error:#}")),
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
                Err(mpsc::TryRecvError::Disconnected) => Some(Err("Rimozione interrotta".into())),
            };
            if let Some(result) = result {
                self.removal_result = None;
                match result {
                    Ok(()) => {
                        self.previews_removed = true;
                        self.action_error = None;
                    }
                    Err(error) => {
                        self.action_error =
                            Some(format!("Impossibile rimuovere i segnaposto: {error}"))
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
                        RichText::new("La coda continua anche chiudendo questa finestra.")
                            .size(theme::CAPTION)
                            .color(colors.secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Chiudi").clicked() {
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
                                        "Segnaposto rimossi"
                                    } else if self.removal_result.is_some() {
                                        "Rimozione…"
                                    } else {
                                        "Rimuovi segnaposto"
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
                                            "Annullamento…"
                                        } else {
                                            "Annulla"
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
                ui.label(theme::heading("Stato della conversione",theme::SUBTITLE));
                ui.add_space(14.0);
                for error in self.error.iter().chain(self.action_error.iter()) { info_bar(ui,error,InfoKind::Error); }
                if let Some(job)=&self.job {
                    theme::show_card(ui, |ui| {
                        ui.horizontal(|ui| {
                            file_icon(ui,&mut self.file_icons,&job.input);
                            ui.vertical(|ui| {
                                ui.add(egui::Label::new(theme::heading(job.input.file_name().unwrap_or_default().to_string_lossy(),theme::BODY)).wrap());
                                ui.label(RichText::new(format!("Destinazione: {}",job.format)).size(theme::CAPTION).color(colors.secondary));
                            });
                        });
                        ui.add_space(12.0);
                        ui.label(theme::heading(quick_state_label(&job.state),theme::BODY).color(quick_state_color(&job.state,ui)));
                        if matches!(job.state,QuickJobState::Running) { progress_display(ui,job.progress,&job.message); }
                        else if !job.message.is_empty() { ui.add(egui::Label::new(RichText::new(&job.message).size(theme::CAPTION).color(colors.secondary)).wrap()); }
                        if self.cancel_sent && quick_job_active(job) { info_bar(ui,"Annullamento richiesto. Attendi la pulizia dei file temporanei.",InfoKind::Info); }
                        if let Some(error)=&job.error { info_bar(ui,error,InfoKind::Error); if ui.button("Copia errore").clicked() { ctx.copy_text(error.clone()); } }
                    });
                    ui.add_space(10.0);
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading("File di destinazione",theme::BODY));
                        for output in &job.outputs { ui.add(egui::Label::new(RichText::new(display_path(output)).size(theme::CAPTION).color(colors.secondary)).wrap()); }
                        if ui.add(egui::Button::new(theme::button_text(FluentIcon::Folder,"Apri cartella",colors.text))).clicked() { self.action_error=open_job_folder(job).err().map(|error|format!("{error:#}")); }
                    });
                    if quick_job_active(job) { ui.add_space(8.0); info_bar(ui,"Il segnaposto in Esplora file viene sostituito dal risultato al termine.",InfoKind::Info); }
                } else if self.error.is_none() { ui.spinner(); ui.label("Lettura della coda…"); }
            });
        });
        if cancel_requested {
            match queue::cancel_job(&self.id) {
                Ok(()) => {
                    self.cancel_sent = true;
                    self.action_error = None;
                }
                Err(error) => self.action_error = Some(format!("Impossibile annullare: {error:#}")),
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
                    self.action_error = Some(format!("Impossibile rimuovere i segnaposto: {error}"))
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
        QuickJobState::Queued => "In coda",
        QuickJobState::Running => "Conversione in corso",
        QuickJobState::Completed => "Completato",
        QuickJobState::Failed => "Conversione non riuscita",
        QuickJobState::Cancelled => "Annullato",
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
        .ok_or_else(|| anyhow::anyhow!("La cartella della conversione non è disponibile"))?;
    platform::open_folder(folder)
}

fn settings_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "SalvatoreCorsi", "TheConverter")
        .map(|dirs| dirs.config_dir().join("preferences.json"))
}

fn centered_column<R>(
    ui: &mut egui::Ui,
    maximum: f32,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let width = ui.available_width().min(maximum);
    let inset = ((ui.available_width() - width) * 0.5).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(inset);
        ui.allocate_ui_with_layout(
            egui::vec2(width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(width);
                contents(ui)
            },
        )
        .inner
    })
}

fn navigation_tabs(ui: &mut egui::Ui, selected: &mut MainView) -> [egui::Rect; 3] {
    let mut rectangles = [egui::Rect::NOTHING; 3];
    let colors = theme::palette(ui.ctx());
    for (index, (view, label)) in [
        (MainView::Files, "File da convertire"),
        (MainView::QuickQueue, "Coda rapida"),
        (MainView::History, "Cronologia"),
    ]
    .into_iter()
    .enumerate()
    {
        let response = ui.add(egui::Button::new(theme::heading(label, theme::BODY)).frame(false));
        rectangles[index] = response.rect;
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
    }
    rectangles
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
                    ui.label(theme::heading(title, theme::BODY));
                    ui.label(
                        RichText::new(detail)
                            .size(theme::CAPTION)
                            .color(theme::palette(ui.ctx()).secondary),
                    );
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
        MediaKind::Image => "Immagine",
        MediaKind::Pdf => "Documento PDF",
        MediaKind::Document => "Documento",
        MediaKind::Spreadsheet => "Foglio di calcolo",
        MediaKind::Html => "Pagina HTML",
        MediaKind::Unknown => "File",
    }
}

fn width_setting(ui: &mut egui::Ui, id: &str, width: &mut Option<u32>) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(
            width
                .map(|n| format!("Larghezza {n} px"))
                .unwrap_or("Dimensioni originali".into()),
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(width, None, "Originali");
            for n in [320, 640, 1280, 1920, 2560, 3840] {
                ui.selectable_value(width, Some(n), format!("{n} px di larghezza"));
            }
        });
    if let Some(width) = width {
        let minimum = if id == "video_width" { 16 } else { 1 };
        ui.horizontal(|ui| {
            ui.label("Larghezza");
            ui.add(
                egui::DragValue::new(width)
                    .range(minimum..=16384)
                    .suffix(" px"),
            );
        });
    }
    ui.label(
        RichText::new("Le proporzioni vengono mantenute.")
            .small()
            .weak(),
    );
}

fn profile_summary(profile: &ConversionProfile) -> String {
    let mut summary = profile.settings.format.label().to_owned();
    if let Some(edit) = &profile.settings.image.edit {
        if let Some(bytes) = edit.max_bytes {
            summary.push_str(&format!(" · massimo {:.0} kB", bytes as f64 / 1000.0));
        }
        if edit.allow_downsize {
            summary.push_str(" · riduzione dimensioni consentita");
        }
        if edit.crop.is_some() {
            summary.push_str(" · ritaglio");
        }
        if let Some(canvas) = &edit.canvas {
            summary.push_str(&format!(" · tela {} × {}", canvas.width, canvas.height));
        }
    }
    if let Some(page) = &profile.settings.image.page {
        summary.push_str(match page.paper {
            the_converter::page_layout::PaperSize::A4 => " · A4",
            the_converter::page_layout::PaperSize::A5 => " · A5",
        });
    }
    summary
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

fn optional_seconds(ui: &mut egui::Ui, label: &str, value: &mut Option<f64>) {
    ui.horizontal(|ui| {
        let mut enabled = value.is_some();
        if ui.checkbox(&mut enabled, label).changed() {
            *value = if enabled {
                Some(if label == "A" { 1.0 } else { 0.0 })
            } else {
                None
            };
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

fn get_tool(tools: &ToolPaths, key: &str) -> Option<PathBuf> {
    match key {
        "ffmpeg" => tools.ffmpeg.clone(),
        "ffprobe" => tools.ffprobe.clone(),
        "pdftoppm" => tools.pdftoppm.clone(),
        "pdfinfo" => tools.pdfinfo.clone(),
        "pdftotext" => tools.pdftotext.clone(),
        "soffice" => tools.soffice.clone(),
        "tesseract" => tools.tesseract.clone(),
        "heif-enc" => tools.heif_enc.clone(),
        "heif-dec" => tools.heif_dec.clone(),
        _ => None,
    }
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
        let mut rectangles = [egui::Rect::NOTHING; 3];
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
        for (index, expected) in [
            (1, MainView::QuickQueue),
            (2, MainView::History),
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
