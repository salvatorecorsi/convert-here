use crate::{
    file_icons::FileIcons,
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

pub fn launch(
    files: Vec<PathBuf>,
    target: Option<OutputFormat>,
    startup_warning: Option<String>,
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
        .and_then(|path| std::fs::read(path).ok())
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

pub struct ConverterApp {
    pdf_merge: Option<crate::pdf_merge_app::PdfMergeApp>,
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
    language_dirty: bool,
    last_persist: Option<std::time::Instant>,
    quick_report: Option<SavedReport>,
    quick_queue: QuickQueueView,
}

impl ConverterApp {
    fn new(files: Vec<PathBuf>, target: Option<OutputFormat>) -> Self {
        let mut saved = load_saved();
        if let Some(format) = target {
            saved.settings.format = format;
        }
        let tools = ToolPaths::configured(&saved.overrides);
        let mut app = Self {
            pdf_merge: None,
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
            self.saved.settings.format = common[0];
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
            self.saved.settings.format = *format;
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
                    tr(
                        "Installazione completata. I motori sono pronti per le conversioni locali.",
                        "Installation complete. The engines are ready for local conversions.",
                    )
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
            ui.label(theme::heading(
                tr("Formati di uscita", "Output formats"),
                theme::SUBTITLE,
            ));
            ui.label(tr("Scegli i formati da mostrare nell’app e in «Converti in». Tutti i formati in ingresso restano supportati.", "Choose the formats to show in the app and in “Convert to”. All input formats remain supported."));
            ui.horizontal_wrapped(|ui| {
                let count = self
                    .saved
                    .enabled_output_formats
                    .filter(OutputFormat::ALL)
                    .count();
                ui.label(trf!(
                    "{count} di {} attivi",
                    "{count} of {} enabled",
                    OutputFormat::ALL.len()
                ));
                if ui.button(tr("Più comuni", "Most common")).clicked() {
                    self.saved.enabled_output_formats.reset_common();
                }
                if ui.button(tr("Attiva tutti", "Enable all")).clicked() {
                    self.saved.enabled_output_formats.enable_all();
                }
                if ui.button(tr("Disattiva tutti", "Disable all")).clicked() {
                    self.saved.enabled_output_formats.disable_all();
                }
            });
            for (index, title) in [
                "Audio",
                "Video",
                tr("Immagini", "Images"),
                tr("Documenti e PDF", "Documents and PDF"),
            ]
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
                            if ui.small_button(tr("Tutti", "All")).clicked() {
                                for format in &formats {
                                    self.saved.enabled_output_formats.set_enabled(*format, true);
                                }
                            }
                            if ui.small_button(tr("Nessuno", "None")).clicked() {
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
                    tr(
                        "Non ci sono uscite attive. Abilita almeno un formato per convertire; puoi comunque aggiungere i file.",
                        "No output formats are enabled. Enable at least one format to convert; you can still add files.",
                    ),
                    InfoKind::Info,
                );
            }
            ui.label(RichText::new(tr("Per esempio: un FLAC resta selezionabile e offre MP3 e WAV con le impostazioni iniziali.", "For example, you can still select a FLAC file and convert it to MP3 or WAV with the default settings.")).small().weak());
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
            for engine in &engines {
                let manifest = engine.manifest();
                ui.label(format!("{} {} · {:.1} MB · {}", manifest.name, manifest.version, manifest.download_bytes as f64 / 1_000_000.0, manifest.license_label()));
                ui.label(trf!("Spazio dopo l’estrazione: {:.1} MB", "Space after extraction: {:.1} MB", manifest.expanded_bytes as f64 / 1_000_000.0));
                let source = match engine {
                    RuntimeEngine::Ffmpeg => "https://www.gyan.dev/ffmpeg/builds/",
                    RuntimeEngine::Poppler => "https://github.com/oschwartz10612/poppler-windows/releases/tag/v26.09.0-0",
                };
                if ui.link(tr("Distribuzione e licenze", "Distribution and licenses")).on_hover_text(source).clicked()
                    && let Err(error) = platform::open_website(source)
                {
                    self.notice = Some(format!("{error:#}"));
                }
            }
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
                            tr("Indietro", "Back"),
                            colors.text,
                        ))
                        .frame(false),
                    )
                    .clicked()
                {
                    self.show_tools = false;
                }
                ui.label(theme::heading(
                    tr("Impostazioni", "Settings"),
                    theme::SUBTITLE,
                ));
            } else {
                navigation_tabs(ui, &mut self.view);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(theme::button_text(
                                FluentIcon::Settings,
                                tr("Impostazioni", "Settings"),
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
        ui.label(theme::heading(tr("Opzioni", "Options"), theme::SUBTITLE));
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
                                    && self.items.iter().any(|item| {
                                        MediaKind::from_path(&item.path) == MediaKind::Image
                                    }) {
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
            self.show_tools = true;
        }
        if before != self.saved.settings.format {
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
            ui.label(
                RichText::new(tr(
                    "Gli originali e i risultati già presenti restano al sicuro.",
                    "Originals and existing results are preserved.",
                ))
                .size(theme::CAPTION)
                .color(colors.secondary),
            );
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
                    ui.label(
                        RichText::new(tr(
                            "Audio, video, immagini e documenti.",
                            "Audio, video, images and documents.",
                        ))
                        .color(colors.secondary),
                    );
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
                    ui.label(
                        RichText::new(tr(
                            "Puoi anche usare il tasto destro in Esplora file.",
                            "You can also use the right-click menu in File Explorer.",
                        ))
                        .size(theme::CAPTION)
                        .color(colors.secondary),
                    );
                });
            });
            return;
        }
        let mut remove = None;
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
                                - if merging { 150.0 } else { 105.0 })
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
            trf!(
                "Questa sessione · {count} file",
                "This session · Files: {count}"
            ),
            theme::BODY,
        ))
        .id_salt("completed_session")
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
                                        RichText::new(trf!(
                                            "Completato · {} file creati",
                                            "Completed · Output files: {}",
                                            outputs.len()
                                        ))
                                        .color(theme::palette(ui.ctx()).success),
                                    );
                                }
                                JobState::Failed(error) => info_bar(ui, error, InfoKind::Error),
                                JobState::Cancelled => {
                                    ui.label(tr("Annullato", "Cancelled"));
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
                    tr("Pronto quando vuoi", "Ready when you are").into()
                } else {
                    trf!(
                        "{} file · {}",
                        "Files: {} · {}",
                        self.items.len(),
                        human_size(self.items.iter().map(|item| item.size).sum())
                    )
                };
                ui.label(theme::heading(summary, theme::BODY));
                ui.label(
                    RichText::new(tr(
                        "Conversioni locali · originali conservati",
                        "Local conversions · originals preserved",
                    ))
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
            if let Some(notice)=self.notice.clone() {
                info_bar(ui,&notice,InfoKind::Warning);
                if ui.small_button(tr("Chiudi avviso", "Dismiss notice")).clicked() { self.notice=None; }
            }
            centered_column(ui, 1000.0, |ui| {
                ui.label(RichText::new(tr("Immagini e PDF da immagini, subito disponibili.", "Images and PDFs from images are ready to use.")).family(egui::FontFamily::Name(theme::SEMIBOLD_FAMILY.into())));
                ui.label(tr("Audio, video, pagine PDF e documenti Office usano motori opzionali. Puoi installare FFmpeg e Poppler con un clic oppure selezionare programmi già presenti.", "Audio, video, PDF pages and Office documents use optional engines. You can install FFmpeg and Poppler with one click or select existing programs."));
                ui.label(tr("Convert Here è gratuito, senza abbonamenti, acquisti in-app o costi a consumo.", "Convert Here is free, with no subscriptions, in-app purchases or usage-based charges."));
                ui.label(RichText::new(tr("Le conversioni avvengono sul PC con i motori locali. I documenti HTML possono caricare risorse collegate; i siti ufficiali si aprono nel browser.", "Conversions run on your PC with local engines. HTML documents may load linked resources; official websites open in your browser.")).small().weak());
                ui.add_space(5.0);
                egui::ScrollArea::vertical().id_salt("tools_scroll").max_height((ui.available_height()-60.0).max(120.0)).show(ui, |ui| {
                    ui.add_enabled_ui(!self.running, |ui| self.output_format_settings(ui));
                    ui.add_space(8.0);
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading(tr("Lingua", "Language"), theme::SUBTITLE));
                        ui.label(tr(
                            "Segui la lingua di Windows o scegli la lingua dell’app e del menu di Esplora file.",
                            "Follow the Windows language or choose the language of the app and File Explorer menu.",
                        ));
                        let mut language = if i18n::has_language_override() {
                            i18n::current_language()
                        } else {
                            self.saved.ui_language
                        };
                        ui.horizontal(|ui| {
                            for choice in [Language::System, Language::Italian, Language::English] {
                                let response = ui.radio_value(&mut language, choice, choice.label());
                                if response.changed()
                                    || (response.clicked() && self.saved.ui_language != language)
                                {
                                    self.saved.ui_language = language;
                                    i18n::set_language(language);
                                    self.settings_dirty = true;
                                    self.language_dirty = true;
                                    ui.ctx().request_repaint();
                                }
                            }
                        });
                        ui.label(RichText::new(tr(
                            "La lingua OCR si sceglie separatamente nelle opzioni di conversione.",
                            "Choose the OCR language separately in the conversion options.",
                        )).small().weak());
                    });
                    ui.add_space(8.0);
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading(tr("Aspetto", "Appearance"),theme::SUBTITLE));
                        ui.label(RichText::new(tr("Segui Windows oppure prova un tema.", "Follow Windows or choose a theme.")).color(theme::palette(ui.ctx()).secondary));
                        let mut preference=ui.ctx().options(|options| options.theme_preference);
                        ui.horizontal(|ui| {
                            for (value,label) in [(egui::ThemePreference::System,tr("Sistema", "System")),(egui::ThemePreference::Light,tr("Chiaro", "Light")),(egui::ThemePreference::Dark,tr("Scuro", "Dark"))] {
                                if ui.radio_value(&mut preference,value,label).changed() { ui.ctx().set_theme(preference); }
                            }
                        });
                    });
                    ui.add_space(8.0);
                    ui.label(theme::heading(tr("Motori di conversione", "Conversion engines"), theme::SUBTITLE));
                    ui.label(tr("FFmpeg include FFprobe; Poppler include gli strumenti per immagini, testo e conteggio pagine PDF.", "FFmpeg includes FFprobe; Poppler includes tools for PDF images, text and page counts."));
                    if ui.add_enabled(self.runtime_install.is_none(), egui::Button::new(tr("Installa FFmpeg e Poppler…", "Install FFmpeg and Poppler…"))).clicked() {
                        self.pending_runtime_install = Some(vec![RuntimeEngine::Ffmpeg, RuntimeEngine::Poppler]);
                    }
                    self.runtime_install_progress(ui);
                    ui.add_space(8.0);
                    for (key, label, description, website) in [
                        ("ffmpeg", "FFmpeg", tr("Necessario per audio, video e GIF da video.", "Required for audio, video and GIFs from video."), "https://ffmpeg.org/download.html"),
                        ("ffprobe", "FFprobe", tr("Opzionale: durata dei media e avanzamento più preciso. Di solito è accanto a ffmpeg.exe.", "Optional: media duration and more accurate progress. Usually located next to ffmpeg.exe."), "https://ffmpeg.org/ffprobe.html"),
                        ("pdftoppm", tr("Poppler · immagini", "Poppler · images"), tr("Necessario per esportare ogni pagina PDF in PNG o JPEG. Seleziona pdftoppm.exe.", "Required to export PDF pages as PNG or JPEG. Select pdftoppm.exe."), "https://poppler.freedesktop.org/"),
                        ("pdfinfo", tr("Poppler · conteggio pagine", "Poppler · page count"), tr("Usa lo stesso motore Poppler per contare le pagine prima di accodare le immagini PDF. Di solito è accanto a pdftoppm.exe.", "Uses the same Poppler engine to count pages before queuing PDF images. Usually located next to pdftoppm.exe."), "https://poppler.freedesktop.org/"),
                        ("pdftotext", tr("Poppler · testo", "Poppler · text"), tr("Necessario per PDF in TXT, Word e HTML (solo testo). Seleziona pdftotext.exe. Il riconoscimento OCR non è incluso.", "Required for PDF to TXT, Word and HTML (text only). Select pdftotext.exe. OCR is not included."), "https://poppler.freedesktop.org/"),
                        ("soffice", "LibreOffice", tr("Necessario per Office e HTML in PDF ed Excel ↔ CSV. Seleziona soffice.exe nella cartella program di LibreOffice.", "Required for Office and HTML to PDF and Excel ↔ CSV. Select soffice.exe in the LibreOffice program folder."), "https://www.libreoffice.org/download/download-libreoffice/"),
                        ("tesseract", tr("Tesseract · OCR locale", "Tesseract · local OCR"), tr("Necessario per immagini in TXT e scansioni PDF in TXT, Word e HTML. Mantieni la cartella tessdata con le lingue accanto al programma.", "Required for images to TXT and scanned PDFs to TXT, Word and HTML. Keep the tessdata language folder next to the program."), "https://tesseract-ocr.github.io/tessdoc/Installation.html"),
                        ("heif-enc", tr("libheif · HEIC/AVIF in uscita", "libheif · HEIC/AVIF output"), tr("Crea HEIC e AVIF. Mantieni DLL e plugin accanto agli strumenti; la build deve includere gli encoder HEVC e AV1.", "Creates HEIC and AVIF. Keep DLLs and plugins next to the tools; the build must include HEVC and AV1 encoders."), "https://github.com/strukturag/libheif"),
                        ("heif-dec", tr("libheif · HEIC/AVIF in ingresso", "libheif · HEIC/AVIF input"), tr("Legge HEIC, HEIF e AVIF; viene convertita la prima immagine. Mantieni DLL e plugin accanto agli strumenti.", "Reads HEIC, HEIF and AVIF; converts the first image. Keep DLLs and plugins next to the tools."), "https://github.com/strukturag/libheif"),
                    ] {
                        theme::show_card(ui, |ui| {
                            let path = self.saved.overrides.get(key).cloned().or_else(|| get_tool(&self.tools, key));
                            let valid = path.as_ref().is_some_and(|path| executable_available(path));
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(label).family(egui::FontFamily::Name(theme::SEMIBOLD_FAMILY.into())));
                                ui.label(RichText::new(if valid { tr("Disponibile", "Available") } else { tr("Non configurato", "Not configured") }).small().color(if valid { theme::palette(ui.ctx()).success } else { ui.visuals().weak_text_color() }));
                            });
                            ui.add(egui::Label::new(RichText::new(description).small().weak()).wrap());
                            if let Some(path) = path {
                                egui::CollapsingHeader::new(tr("Percorso del programma", "Program path")).id_salt(("tool_path",key)).default_open(false).show(ui,|ui| { ui.add(egui::Label::new(RichText::new(display_path(&path)).small()).wrap()); });
                                if !valid {
                                    ui.colored_label(ui.visuals().error_fg_color, tr("Il percorso salvato non è un programma .exe disponibile. Seleziona di nuovo lo strumento.", "The saved path is not an available .exe program. Select the tool again."));
                                }
                            }
                            ui.horizontal_wrapped(|ui| {
                                let engine = match key {
                                    "ffmpeg" => Some(RuntimeEngine::Ffmpeg),
                                    "pdftoppm" => Some(RuntimeEngine::Poppler),
                                    _ => None,
                                };
                                if let Some(engine) = engine
                                    && ui.add_enabled(self.runtime_install.is_none(), egui::Button::new(tr("Scarica e installa…", "Download and install…"))).clicked()
                                {
                                    self.pending_runtime_install = Some(vec![engine]);
                                }
                                if ui.small_button(tr("Seleziona programma…", "Select program…")).clicked()
                                    && let Some(path) = rfd::FileDialog::new()
                                        .set_title(trf!("Seleziona {key}.exe già installato", "Select an installed {key}.exe"))
                                        .set_file_name(format!("{key}.exe"))
                                        .add_filter(tr("Programmi Windows", "Windows programs"), &["exe"])
                                        .pick_file()
                                {
                                    if executable_available(&path) {
                                        self.saved.overrides.insert(key.into(), path);
                                        self.tools = ToolPaths::configured(&self.saved.overrides);
                                        self.settings_dirty = true;
                                    } else {
                                        self.notice = Some(tr("Seleziona un programma esistente con estensione .exe.", "Select an existing program with an .exe extension.").into());
                                    }
                                }
                                if self.saved.overrides.contains_key(key)
                                    && ui.small_button(tr("Usa rilevamento automatico", "Use automatic detection")).clicked()
                                {
                                    self.saved.overrides.remove(key);
                                    self.tools = ToolPaths::configured(&self.saved.overrides);
                                    self.settings_dirty = true;
                                }
                                if ui.link(tr("Sito ufficiale", "Official website")).on_hover_text(website).clicked()
                                    && let Err(error) = platform::open_website(website)
                                {
                                    self.notice = Some(format!("{error:#}"));
                                }
                            });
                        });
                    }
                    egui::CollapsingHeader::new(tr("Cartella personale degli strumenti", "Personal tools folder")).id_salt("personal_tools_folder").show(ui, |ui| {
                        ui.label(RichText::new(tr("I download vengono installati qui. Puoi anche aggiungere strumenti già presenti, mantenendo le rispettive DLL e licenze. Non occorre modificare la cartella dell’app.", "Downloads are installed here. You can also add existing tools, keeping their DLLs and licenses. You do not need to modify the app folder.")).small().weak());
                        if let Some(path) = the_converter::tools::user_tools_dir() {
                            ui.add(egui::Label::new(RichText::new(display_path(&path)).small()).wrap());
                            if ui.small_button(tr("Copia percorso", "Copy path")).clicked() { ui.ctx().copy_text(display_path(&path)); }
                        }
                        ui.label(RichText::new(tr("Sono riconosciute anche le cartelle bin, ffmpeg/bin e poppler/Library/bin. Le DLL necessarie devono restare insieme ai rispettivi programmi.", "The bin, ffmpeg/bin and poppler/Library/bin folders are also recognized. Required DLLs must stay with their programs.")).small().weak());
                    });
                    ui.add_space(8.0);
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading(tr("Informazioni e licenze", "About and licenses"), theme::SUBTITLE));
                        ui.label(format!("Convert Here {} · © 2026 Salvatore Corsi", env!("CARGO_PKG_VERSION")));
                        ui.label(tr("Software libero, GPL-3.0-only. Puoi usare, modificare e redistribuire il programma secondo la licenza. Il software è fornito senza garanzie, nei limiti consentiti dalla legge.", "Free software, GPL-3.0-only. You may use, modify and redistribute the program under the license. The software is provided without warranty, to the extent permitted by law."));
                        ui.label(tr("Questa licenza riguarda il programma, non i file convertiti. Dipendenze e motori mantengono le rispettive licenze.", "This license applies to the program, not to converted files. Dependencies and engines retain their own licenses."));
                        ui.horizontal_wrapped(|ui| {
                            if ui.button(tr("Leggi la licenza", "Read the license")).clicked() {
                                self.show_license = true;
                            }
                            if ui.link(tr("Codice sorgente", "Source code")).clicked() {
                                let url = format!("https://github.com/salvatorecorsi/convert-here/releases/tag/v{}", env!("CARGO_PKG_VERSION"));
                                if let Err(error) = platform::open_website(&url) {
                                    self.notice = Some(format!("{error:#}"));
                                }
                            }
                        });
                    });
                });
                ui.separator();
                if ui.button(tr("Rileva strumenti installati", "Detect installed tools")).clicked() {
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
                tr(
                    "Annullamento dell’installazione. Chiudi la finestra al termine.",
                    "Cancelling installation. Close the window when it finishes.",
                )
                .into(),
                false,
            ));
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
                        MainView::QuickQueue => {
                            self.quick_queue.show(ui, false, &mut self.file_icons)
                        }
                        MainView::History => {
                            ui.label(theme::heading(tr("Cronologia", "History"), theme::TITLE));
                            ui.label(
                                RichText::new(tr(
                                    "Risultati delle conversioni rapide e dell’ultima sessione.",
                                    "Results from quick conversions and the last session.",
                                ))
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

    fn show(&mut self, ui: &mut egui::Ui, history: bool, icons: &mut FileIcons) {
        let colors = theme::palette(ui.ctx());
        if !history {
            ui.label(theme::heading(
                tr("Coda rapida", "Quick queue"),
                theme::TITLE,
            ));
            ui.label(
                RichText::new(tr(
                    "I lavori proseguono anche chiudendo questa finestra.",
                    "Jobs keep running when you close this window.",
                ))
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
                    tr(
                        "Nessuna conversione rapida conclusa.",
                        "No completed quick conversions.",
                    )
                } else {
                    tr(
                        "Nessuna conversione rapida in attesa o in corso.",
                        "No quick conversions are queued or running.",
                    )
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
                                            tr("Annulla conversione", "Cancel conversion"),
                                        )
                                        .clicked()
                                    {
                                        cancel = Some(job.id.clone());
                                    }
                                    if icon_button(
                                        ui,
                                        FluentIcon::Folder,
                                        tr("Apri cartella di destinazione", "Open output folder"),
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
                            if ui.small_button(tr("Copia errore", "Copy error")).clicked() {
                                ui.ctx().copy_text(error.clone());
                            }
                        }
                    });
                    ui.add_space(8.0);
                }
            });
        if let Some(id) = cancel {
            self.action_error = queue::cancel_job(&id).err().map(|error| {
                trf!(
                    "Impossibile annullare: {error:#}",
                    "Unable to cancel: {error:#}"
                )
            });
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
        (
            MainView::Files,
            tr("File da convertire", "Files to convert"),
        ),
        (MainView::QuickQueue, tr("Coda rapida", "Quick queue")),
        (MainView::History, tr("Cronologia", "History")),
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
