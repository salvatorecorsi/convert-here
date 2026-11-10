use crate::theme::{self, FluentIcon};
use eframe::egui;
use the_converter::{
    i18n::tr,
    runtime_install::{InstallPhase, InstallProgress, RuntimeEngine},
    tools::{ToolPaths, executable_available, model_available},
};

const ENGINES: [RuntimeEngine; 5] = [
    RuntimeEngine::Ffmpeg,
    RuntimeEngine::Poppler,
    RuntimeEngine::Tesseract,
    RuntimeEngine::Libheif,
    RuntimeEngine::Whisper,
];
const LIBREOFFICE_URL: &str = "https://www.libreoffice.org/download/download-libreoffice/";

pub(crate) enum EngineSetupAction {
    Review(Vec<RuntimeEngine>),
    Close,
    Cancel,
    Website(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Choose,
    Installing,
    Finished,
}

/// Presentation and planning only. Downloads start in the app after its
/// existing source/size/license confirmation, using its sequential worker.
pub(crate) struct EngineSetup {
    selected: [bool; 5],
    stage: Stage,
    planned: Vec<RuntimeEngine>,
    completed: Vec<RuntimeEngine>,
    error: Option<String>,
    cancellation_requested: bool,
}

impl EngineSetup {
    pub(crate) fn new(tools: &ToolPaths) -> Self {
        Self {
            selected: ready_engines(tools).map(|ready| !ready),
            stage: Stage::Choose,
            planned: Vec::new(),
            completed: Vec::new(),
            error: None,
            cancellation_requested: false,
        }
    }

    /// Called only after the app confirms the exact plan with the user.
    pub(crate) fn start(&mut self, engines: Vec<RuntimeEngine>) {
        if self.stage == Stage::Installing || engines.is_empty() {
            return;
        }
        let engines = ordered(&engines);
        if self.stage == Stage::Choose {
            self.planned = engines;
            self.completed.clear();
        } else {
            // A retry keeps earlier successful engines. An explicitly repeated
            // engine (e.g. a deleted FFmpeg dependency) must complete again.
            self.completed.retain(|engine| !engines.contains(engine));
            self.planned.extend(engines);
            self.planned = ordered(&self.planned);
        }
        self.error = None;
        self.cancellation_requested = false;
        self.stage = Stage::Installing;
    }

    pub(crate) fn installed(&mut self, engine: RuntimeEngine) {
        if self.stage == Stage::Installing
            && self.planned.contains(&engine)
            && !self.completed.contains(&engine)
        {
            self.completed.push(engine);
        }
    }

    pub(crate) fn finish(&mut self, result: Result<(), String>) {
        if self.stage != Stage::Installing {
            return;
        }
        self.error = match result {
            Ok(()) if self.pending().is_empty() => None,
            Ok(()) => Some(
                tr(
                    "Alcuni motori non risultano completati. Puoi riprovare.",
                    "Some engines did not finish. You can try again.",
                )
                .into(),
            ),
            Err(error) => Some(error),
        };
        self.stage = Stage::Finished;
    }

    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        tools: &ToolPaths,
        progress: Option<&InstallProgress>,
        cancelling: bool,
    ) -> Option<EngineSetupAction> {
        if self.stage == Stage::Installing && cancelling {
            self.cancellation_requested = true;
        }
        let width = (ctx.content_rect().width() - 64.0).clamp(180.0, 620.0);
        let action = egui::Modal::new(egui::Id::new("engine_setup"))
            .show(ctx, |ui| {
                ui.set_width(width);
                let header_top = ui.cursor().top();
                self.header(ui);
                let header_height = ui.cursor().top() - header_top;
                // The actions remain visible even when the engine list or
                // expanded error details need scrolling. Measure wrapped
                // footer rows with the same fonts and padding as the buttons.
                let body_height = (ctx.content_rect().height()
                    - 64.0
                    - header_height
                    - self.footer_height(ui, cancelling)
                    - 3.0 * ui.spacing().item_spacing.y)
                    .max(40.0);
                let body_action = egui::ScrollArea::vertical()
                    .id_salt("engine_setup_scroll")
                    .max_height(body_height)
                    .show(ui, |ui| match self.stage {
                        Stage::Choose => self.choose(ui, tools),
                        Stage::Installing => {
                            self.installing(ui, progress);
                            None
                        }
                        Stage::Finished => {
                            self.finished(ui);
                            None
                        }
                    })
                    .inner;
                ui.separator();
                let footer_action = self.footer(ui, tools, cancelling);
                body_action.or(footer_action)
            })
            .inner;
        action.or_else(|| {
            (self.stage != Stage::Installing
                && ctx.input(|input| input.key_pressed(egui::Key::Escape)))
            .then_some(EngineSetupAction::Close)
        })
    }

    fn header(&self, ui: &mut egui::Ui) {
        ui.label(theme::heading(
            tr("Configura i motori", "Set up engines"),
            theme::SUBTITLE,
        ));
        let colors = theme::palette(ui.ctx());
        ui.horizontal_wrapped(|ui| {
            for (index, label) in [
                tr("1. Scegli", "1. Choose"),
                tr("2. Conferma", "2. Confirm"),
                tr("3. Installa", "3. Install"),
            ]
            .into_iter()
            .enumerate()
            {
                let active = if self.stage == Stage::Choose { 0 } else { 2 };
                ui.label(egui::RichText::new(label).color(if index == active {
                    colors.accent
                } else {
                    colors.secondary
                }));
            }
        });
        ui.add_space(8.0);
    }

    fn choose(&mut self, ui: &mut egui::Ui, tools: &ToolPaths) -> Option<EngineSetupAction> {
        let ready = ready_engines(tools);
        let needs_ffmpeg = self.selected[4] && !ready[0];
        ui.label(tr(
            "Seleziona i motori da installare.",
            "Choose the engines to install.",
        ));
        ui.add_space(6.0);
        for (index, engine) in ENGINES.into_iter().enumerate() {
            theme::show_card(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.spacing_mut().interact_size.y = 26.0;
                ui.horizontal(|ui| {
                    ui.label(theme::icon(engine_icon(engine), 22.0));
                    let required = engine == RuntimeEngine::Ffmpeg && needs_ffmpeg;
                    let mut selected = self.selected[index] || required;
                    if ui
                        .add_enabled(
                            !required,
                            egui::Checkbox::new(&mut selected, engine_name(engine)),
                        )
                        .changed()
                    {
                        self.selected[index] = selected;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!(
                            "{:.1} MB",
                            engine.manifest().download_bytes as f64 / 1_000_000.0
                        ));
                        if ready[index] {
                            ui.label(
                                egui::RichText::new(tr("Rilevato", "Detected"))
                                    .color(theme::palette(ui.ctx()).secondary),
                            );
                        }
                        if required {
                            ui.label(tr("Richiesto da Whisper", "Required by Whisper"));
                        }
                    });
                });
                ui.add(egui::Label::new(engine_description(engine)).wrap());
            });
            ui.add_space(4.0);
        }
        let plan = planned_engines(self.selected, ready);
        if self.selected[4] && !ready[0] {
            ui.label(tr(
                "Whisper richiede FFmpeg: verrà installato per primo.",
                "Whisper requires FFmpeg: it will be installed first.",
            ));
        }
        let (download_bytes, expanded_bytes) = plan_size(&plan);
        ui.label(format!(
            "{}: {:.1} MB · {}: {:.1} MB",
            tr("Download", "Download"),
            download_bytes as f64 / 1_000_000.0,
            tr("Dopo l’estrazione", "After extraction"),
            expanded_bytes as f64 / 1_000_000.0,
        ));
        ui.add_space(6.0);
        ui.separator();
        ui.label(tr(
            "LibreOffice si installa separatamente per Word, Excel e HTML.",
            "Install LibreOffice separately for Word, Excel and HTML.",
        ));
        let mut action = None;
        if button(
            ui,
            FluentIcon::ExternalLink,
            tr("Sito ufficiale LibreOffice", "LibreOffice official website"),
        )
        .clicked()
        {
            action = Some(EngineSetupAction::Website(LIBREOFFICE_URL));
        }
        action
    }

    fn installing(&self, ui: &mut egui::Ui, progress: Option<&InstallProgress>) {
        self.installation_list(ui, progress);
        ui.add_space(8.0);
        let active_fraction = progress
            .filter(|progress| {
                self.planned.contains(&progress.engine)
                    && !self.completed.contains(&progress.engine)
            })
            .map_or(0.0, |progress| progress.fraction.clamp(0.0, 1.0));
        let fraction =
            (self.completed.len() as f32 + active_fraction) / self.planned.len().max(1) as f32;
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{} / {} · {:.0}%",
                self.completed.len(),
                self.planned.len(),
                fraction * 100.0,
            ));
        });
        ui.add(egui::ProgressBar::new(fraction).desired_height(3.0));
        if let Some(progress) = progress {
            ui.add(egui::Label::new(&progress.message).wrap());
            if progress.phase == InstallPhase::Downloading {
                ui.label(format!(
                    "{:.1} / {:.1} MB",
                    progress.downloaded_bytes as f64 / 1_000_000.0,
                    progress.total_bytes as f64 / 1_000_000.0,
                ));
            }
        }
    }

    fn finished(&self, ui: &mut egui::Ui) {
        let colors = theme::palette(ui.ctx());
        if let Some(error) = &self.error {
            ui.colored_label(
                colors.error,
                if self.cancellation_requested {
                    tr("Installazione annullata", "Installation cancelled")
                } else {
                    tr("Installazione interrotta", "Installation interrupted")
                },
            );
            if !self.cancellation_requested {
                egui::CollapsingHeader::new(tr("Dettagli", "Details"))
                    .id_salt("engine_setup_error")
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(error).wrap());
                    });
            }
        } else {
            ui.colored_label(
                colors.success,
                tr("Installazione completata", "Installation complete"),
            );
        }
        ui.add_space(8.0);
        self.installation_list(ui, None);
        if self
            .planned
            .iter()
            .any(|engine| matches!(engine, RuntimeEngine::Tesseract | RuntimeEngine::Whisper))
        {
            ui.add_space(8.0);
            ui.add(
                egui::Label::new(tr(
                    "Per OCR e trascrizione, abilita TXT in Impostazioni → Formati di uscita → Raw.",
                    "For OCR and transcription, enable TXT in Settings → Output formats → Raw.",
                ))
                .wrap(),
            );
        }
    }

    fn footer_height(&self, ui: &egui::Ui, cancelling: bool) -> f32 {
        let buttons = match self.stage {
            Stage::Choose => vec![
                (FluentIcon::Cancel, tr("Salta", "Skip")),
                (FluentIcon::Check, tr("Continua", "Continue")),
            ],
            Stage::Installing => vec![(
                FluentIcon::Cancel,
                if cancelling || self.cancellation_requested {
                    tr("Annullamento…", "Cancelling…")
                } else {
                    tr("Annulla installazione", "Cancel installation")
                },
            )],
            Stage::Finished if self.error.is_some() => vec![
                (FluentIcon::Refresh, tr("Riprova", "Retry")),
                (FluentIcon::Back, tr("Modifica scelta", "Change selection")),
                (FluentIcon::Check, tr("Fine", "Done")),
            ],
            Stage::Finished => vec![(FluentIcon::Check, tr("Fine", "Done"))],
        };
        let spacing = ui.spacing();
        let mut row_width = 0.0;
        let mut rows = 1u32;
        let mut button_height = spacing.interact_size.y;
        for (icon, label) in buttons {
            let galley = theme::button_text(icon, label, ui.visuals().text_color()).into_galley(
                ui,
                Some(egui::TextWrapMode::Wrap),
                (ui.available_width() - 2.0 * spacing.button_padding.x).max(1.0),
                egui::TextStyle::Button,
            );
            let size = (galley.size() + 2.0 * spacing.button_padding).max(spacing.interact_size);
            button_height = button_height.max(size.y);
            if row_width > 0.0 && row_width + spacing.item_spacing.x + size.x > ui.available_width()
            {
                rows += 1;
                row_width = size.x;
            } else {
                row_width += size.x
                    + if row_width > 0.0 {
                        spacing.item_spacing.x
                    } else {
                        0.0
                    };
            }
        }
        rows as f32 * button_height + rows.saturating_sub(1) as f32 * spacing.item_spacing.y
    }

    fn footer(
        &mut self,
        ui: &mut egui::Ui,
        tools: &ToolPaths,
        cancelling: bool,
    ) -> Option<EngineSetupAction> {
        let mut action = None;
        ui.horizontal_wrapped(|ui| match self.stage {
            Stage::Choose => {
                if button(ui, FluentIcon::Cancel, tr("Salta", "Skip")).clicked() {
                    action = Some(EngineSetupAction::Close);
                }
                let plan = planned_engines(self.selected, ready_engines(tools));
                if ui
                    .add_enabled(
                        !plan.is_empty(),
                        egui::Button::new(theme::button_text(
                            FluentIcon::Check,
                            tr("Continua", "Continue"),
                            ui.visuals().text_color(),
                        )),
                    )
                    .clicked()
                {
                    action = Some(EngineSetupAction::Review(plan));
                }
            }
            Stage::Installing => {
                let cancelling = cancelling || self.cancellation_requested;
                if ui
                    .add_enabled(
                        !cancelling,
                        egui::Button::new(theme::button_text(
                            FluentIcon::Cancel,
                            if cancelling {
                                tr("Annullamento…", "Cancelling…")
                            } else {
                                tr("Annulla installazione", "Cancel installation")
                            },
                            ui.visuals().text_color(),
                        )),
                    )
                    .clicked()
                    && self.request_cancel()
                {
                    action = Some(EngineSetupAction::Cancel);
                }
            }
            Stage::Finished => {
                if self.error.is_some() {
                    let retry = self.retry_plan(ready_engines(tools));
                    if !retry.is_empty()
                        && button(ui, FluentIcon::Refresh, tr("Riprova", "Retry")).clicked()
                    {
                        action = Some(EngineSetupAction::Review(retry));
                    }
                    if button(
                        ui,
                        FluentIcon::Back,
                        tr("Modifica scelta", "Change selection"),
                    )
                    .clicked()
                    {
                        let pending = self.pending();
                        self.selected = ENGINES.map(|engine| pending.contains(&engine));
                        self.stage = Stage::Choose;
                        self.error = None;
                        self.cancellation_requested = false;
                    }
                }
                if button(ui, FluentIcon::Check, tr("Fine", "Done")).clicked() {
                    action = Some(EngineSetupAction::Close);
                }
            }
        });
        action
    }

    fn installation_list(&self, ui: &mut egui::Ui, progress: Option<&InstallProgress>) {
        for engine in &self.planned {
            let completed = self.completed.contains(engine);
            let active = self.stage == Stage::Installing
                && !completed
                && progress.is_some_and(|progress| progress.engine == *engine);
            ui.horizontal_wrapped(|ui| {
                ui.label(theme::icon(
                    if completed {
                        FluentIcon::Check
                    } else {
                        engine_icon(*engine)
                    },
                    18.0,
                ));
                ui.label(engine_name(*engine));
                ui.label(if completed {
                    tr("Installato", "Installed")
                } else if active {
                    tr("In corso", "In progress")
                } else if self.stage == Stage::Installing {
                    tr("In coda", "Queued")
                } else {
                    tr("Non completato", "Not completed")
                });
            });
        }
    }

    fn pending(&self) -> Vec<RuntimeEngine> {
        self.planned
            .iter()
            .copied()
            .filter(|engine| !self.completed.contains(engine))
            .collect()
    }

    fn retry_plan(&self, ready: [bool; 5]) -> Vec<RuntimeEngine> {
        let pending = self.pending();
        planned_engines(ENGINES.map(|engine| pending.contains(&engine)), ready)
    }

    fn request_cancel(&mut self) -> bool {
        if self.stage != Stage::Installing || self.cancellation_requested {
            return false;
        }
        self.cancellation_requested = true;
        true
    }
}

fn button(ui: &mut egui::Ui, icon: FluentIcon, label: &str) -> egui::Response {
    ui.button(theme::button_text(icon, label, ui.visuals().text_color()))
}

fn engine_name(engine: RuntimeEngine) -> &'static str {
    match engine {
        RuntimeEngine::Tesseract => "Tesseract OCR",
        RuntimeEngine::Whisper => "Whisper · Tiny",
        _ => engine.manifest().name,
    }
}

fn engine_icon(engine: RuntimeEngine) -> FluentIcon {
    match engine {
        RuntimeEngine::Ffmpeg => FluentIcon::Video,
        RuntimeEngine::Poppler => FluentIcon::Document,
        RuntimeEngine::Tesseract => FluentIcon::Raw,
        RuntimeEngine::Libheif => FluentIcon::Image,
        RuntimeEngine::Whisper => FluentIcon::Audio,
    }
}

fn engine_description(engine: RuntimeEngine) -> &'static str {
    match engine {
        RuntimeEngine::Ffmpeg => tr(
            "Conversione e taglio audio/video.",
            "Audio/video conversion and trimming.",
        ),
        RuntimeEngine::Poppler => tr("PDF in immagini e testo.", "PDF to images and text."),
        RuntimeEngine::Tesseract => tr(
            "Testo dalle immagini; il download include italiano e inglese.",
            "Text from images; the download includes Italian and English.",
        ),
        RuntimeEngine::Libheif => tr("Immagini HEIC, HEIF e AVIF.", "HEIC, HEIF and AVIF images."),
        RuntimeEngine::Whisper => tr(
            "Audio e video in testo, con Tiny multilingue.",
            "Audio and video to text with multilingual Tiny.",
        ),
    }
}

fn ready_engines(tools: &ToolPaths) -> [bool; 5] {
    let executable =
        |path: &Option<std::path::PathBuf>| path.as_deref().is_some_and(executable_available);
    [
        executable(&tools.ffmpeg) && executable(&tools.ffprobe),
        executable(&tools.pdftoppm) && executable(&tools.pdftotext) && executable(&tools.pdfinfo),
        executable(&tools.tesseract),
        executable(&tools.heif_dec) && executable(&tools.heif_enc),
        executable(&tools.whisper_cli)
            && tools.whisper_model.as_deref().is_some_and(model_available),
    ]
}

fn planned_engines(mut selected: [bool; 5], ready: [bool; 5]) -> Vec<RuntimeEngine> {
    if selected[4] && !ready[0] {
        selected[0] = true;
    }
    ENGINES
        .into_iter()
        .zip(selected)
        .filter_map(|(engine, selected)| selected.then_some(engine))
        .collect()
}

fn ordered(engines: &[RuntimeEngine]) -> Vec<RuntimeEngine> {
    ENGINES
        .into_iter()
        .filter(|engine| engines.contains(engine))
        .collect()
}

fn plan_size(engines: &[RuntimeEngine]) -> (u64, u64) {
    engines
        .iter()
        .fold((0u64, 0u64), |(download, expanded), engine| {
            let manifest = engine.manifest();
            (
                download.saturating_add(manifest.download_bytes),
                expanded.saturating_add(manifest.expanded_bytes),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn plans_every_selection_once_in_dependency_order_and_accounts_for_sizes() {
        for bits in 0..32 {
            let selected = std::array::from_fn(|index| bits & (1 << index) != 0);
            for ffmpeg_ready in [false, true] {
                let ready = [ffmpeg_ready, false, false, false, false];
                let plan = planned_engines(selected, ready);
                assert_eq!(plan, ordered(&plan));
                for (index, engine) in ENGINES.into_iter().enumerate() {
                    assert_eq!(
                        plan.contains(&engine),
                        selected[index] || index == 0 && selected[4] && !ffmpeg_ready,
                    );
                }
                let (download, expanded) = plan_size(&plan);
                assert_eq!(
                    download,
                    plan.iter()
                        .map(|engine| engine.manifest().download_bytes)
                        .sum::<u64>(),
                );
                assert_eq!(
                    expanded,
                    plan.iter()
                        .map(|engine| engine.manifest().expanded_bytes)
                        .sum::<u64>(),
                );
            }
        }
    }

    #[test]
    fn detected_status_requires_companions_and_keeps_ocr_language_claims_separate() {
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("tool.exe");
        let model = directory.path().join("model.bin");
        fs::write(&executable, b"fixture, never executed").unwrap();
        fs::write(&model, b"fixture model").unwrap();
        let mut tools = ToolPaths {
            ffmpeg: Some(executable.clone()),
            pdftoppm: Some(executable.clone()),
            tesseract: Some(executable.clone()),
            heif_dec: Some(executable.clone()),
            whisper_cli: Some(executable.clone()),
            ..Default::default()
        };
        assert_eq!(ready_engines(&tools), [false, false, true, false, false]);
        assert_eq!(
            EngineSetup::new(&tools).selected,
            [true, true, false, true, true]
        );
        tools.ffprobe = Some(executable.clone());
        tools.pdftotext = Some(executable.clone());
        tools.pdfinfo = Some(executable.clone());
        tools.heif_enc = Some(executable);
        tools.whisper_model = Some(model);
        assert_eq!(ready_engines(&tools), [true; 5]);
        assert_eq!(EngineSetup::new(&tools).selected, [false; 5]);
        // Detected engines may still be selected for an explicit bundle install.
        assert_eq!(planned_engines([true; 5], [true; 5]), ENGINES);
    }

    #[test]
    fn failed_sequence_retries_only_unfinished_engines_and_preserves_successes() {
        let mut setup = EngineSetup::new(&ToolPaths::default());
        setup.start(ENGINES.to_vec());
        setup.installed(RuntimeEngine::Ffmpeg);
        setup.installed(RuntimeEngine::Poppler);
        setup.installed(RuntimeEngine::Poppler);
        setup.finish(Err("connection interrupted".into()));
        assert_eq!(setup.completed.len(), 2);
        let remaining = setup.retry_plan([true, true, false, false, false]);
        assert_eq!(
            remaining,
            [
                RuntimeEngine::Tesseract,
                RuntimeEngine::Libheif,
                RuntimeEngine::Whisper
            ],
        );
        setup.start(remaining.clone());
        assert_eq!(setup.completed.len(), 2);
        assert!(setup.error.is_none());
        for engine in remaining {
            setup.installed(engine);
        }
        setup.finish(Ok(()));
        assert_eq!(setup.stage, Stage::Finished);
        assert!(setup.error.is_none());
        assert!(setup.pending().is_empty());
        assert!(setup.retry_plan([true; 5]).is_empty());
    }

    #[test]
    fn cancelling_keeps_published_engines_and_a_retry_resets_cancellation() {
        let mut setup = EngineSetup::new(&ToolPaths::default());
        assert!(!setup.request_cancel());
        setup.start(vec![RuntimeEngine::Ffmpeg, RuntimeEngine::Whisper]);
        setup.installed(RuntimeEngine::Ffmpeg);
        assert!(setup.request_cancel());
        assert!(!setup.request_cancel());
        setup.finish(Err("cancelled".into()));
        assert_eq!(setup.completed, [RuntimeEngine::Ffmpeg]);
        let remaining = setup.retry_plan([true, false, false, false, false]);
        assert_eq!(remaining, [RuntimeEngine::Whisper]);
        setup.start(remaining);
        assert!(!setup.cancellation_requested);
        setup.installed(RuntimeEngine::Whisper);
        setup.finish(Ok(()));
        assert!(setup.error.is_none());
        assert!(setup.pending().is_empty());
    }

    #[test]
    fn deleted_ffmpeg_is_added_again_before_retrying_whisper() {
        let mut setup = EngineSetup::new(&ToolPaths::default());
        setup.start(vec![RuntimeEngine::Ffmpeg, RuntimeEngine::Whisper]);
        setup.installed(RuntimeEngine::Ffmpeg);
        setup.finish(Err("model download interrupted".into()));
        let retry = setup.retry_plan([false; 5]);
        assert_eq!(retry, [RuntimeEngine::Ffmpeg, RuntimeEngine::Whisper]);
        setup.start(retry);
        assert!(setup.completed.is_empty());
        setup.installed(RuntimeEngine::Ffmpeg);
        setup.installed(RuntimeEngine::Whisper);
        setup.finish(Ok(()));
        assert!(setup.error.is_none());
    }

    #[test]
    fn incomplete_or_unrelated_worker_events_do_not_report_success() {
        let mut setup = EngineSetup::new(&ToolPaths::default());
        setup.installed(RuntimeEngine::Ffmpeg);
        setup.finish(Ok(()));
        assert_eq!(setup.stage, Stage::Choose);
        setup.start(vec![RuntimeEngine::Tesseract]);
        setup.installed(RuntimeEngine::Ffmpeg);
        setup.finish(Ok(()));
        assert!(setup.completed.is_empty());
        assert!(setup.error.is_some());
        assert_eq!(setup.pending(), [RuntimeEngine::Tesseract]);
    }
}
