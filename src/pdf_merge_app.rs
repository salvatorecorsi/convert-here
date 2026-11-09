//! A session-only PDF merge window. Preparation, output, and workspace cleanup
//! run on workers; the UI owns only progress and the current page order.
use crate::{
    pdf_merge_ui::PageOrderEditor,
    theme::{self, FluentIcon},
};
use eframe::egui::{self, RichText};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use the_converter::{
    engine::Progress,
    model::{ConversionSettings, OutputFormat, PdfAction},
    pdf_merge::{self, PreparedPdfMerge},
    platform::{self, display_path},
    tools::ToolPaths,
};
use the_converter::{i18n::tr, trf};

pub fn launch(files: Vec<PathBuf>, startup_warning: Option<String>) -> eframe::Result {
    let (settings, tools) = crate::app::quick_configuration();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Convert Here · PDF Merge")
            .with_icon(crate::app::app_icon())
            .with_inner_size([920.0, 720.0])
            .with_min_inner_size([620.0, 500.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Convert Here · PDF Merge",
        options,
        Box::new(move |context| {
            theme::configure(&context.egui_ctx);
            Ok(Box::new(PdfMergeApp::new(
                files,
                settings,
                tools,
                startup_warning,
            )))
        }),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Work {
    Preparing,
    Merging,
    Cleaning,
}

enum Event {
    Progress(Progress),
    Prepared(PreparedPdfMerge),
    Finished {
        prepared: PreparedPdfMerge,
        result: Result<Vec<PathBuf>, String>,
    },
    Failed(String),
    Cancelled,
    Cleaned,
}

pub(crate) struct PdfMergeApp {
    files: Vec<PathBuf>,
    settings: ConversionSettings,
    tools: ToolPaths,
    prepared: Option<PreparedPdfMerge>,
    editor: Option<PageOrderEditor>,
    receiver: Option<mpsc::Receiver<Event>>,
    work: Option<Work>,
    cancel: Arc<AtomicBool>,
    progress: Progress,
    startup_warning: Option<String>,
    error: Option<String>,
    output: Option<PathBuf>,
    close_when_idle: bool,
}

impl PdfMergeApp {
    pub(crate) fn new(
        files: Vec<PathBuf>,
        mut settings: ConversionSettings,
        tools: ToolPaths,
        startup_warning: Option<String>,
    ) -> Self {
        settings.format = OutputFormat::Pdf;
        settings.pdf.action = PdfAction::Merge;
        let mut app = Self {
            files,
            settings,
            tools,
            prepared: None,
            editor: None,
            receiver: None,
            work: None,
            cancel: Arc::new(AtomicBool::new(false)),
            progress: Progress {
                fraction: 0.0,
                message: tr("Preparazione delle pagine…", "Preparing pages…").into(),
            },
            startup_warning,
            error: None,
            output: None,
            close_when_idle: false,
        };
        app.prepare();
        app
    }

    pub(crate) fn is_busy(&self) -> bool {
        self.work.is_some()
    }

    fn begin_worker(
        &mut self,
        work: Work,
        task: impl FnOnce(mpsc::SyncSender<Event>, Arc<AtomicBool>) + Send + 'static,
    ) {
        let (sender, receiver) = mpsc::sync_channel(32);
        self.receiver = Some(receiver);
        self.work = Some(work);
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        if let Err(error) = std::thread::Builder::new()
            .name("pdf-merge-work".into())
            .spawn(move || task(sender, cancel))
        {
            self.receiver = None;
            self.work = None;
            self.error = Some(trf!(
                "Impossibile avviare l’operazione: {error}",
                "Unable to start the operation: {error}"
            ));
        }
    }

    fn prepare(&mut self) {
        if self.is_busy() {
            return;
        }
        self.error = None;
        self.output = None;
        self.progress = Progress {
            fraction: 0.0,
            message: tr("Preparazione delle pagine…", "Preparing pages…").into(),
        };
        let files = self.files.clone();
        let settings = self.settings.clone();
        let tools = self.tools.clone();
        self.begin_worker(Work::Preparing, move |sender, cancel| {
            let result =
                pdf_merge::prepare_pdf_merge(&files, &settings, &tools, &cancel, |progress| {
                    // Progress may be coalesced; completion must always arrive.
                    let _ = sender.try_send(Event::Progress(progress));
                });
            let event = if cancel.load(Ordering::Relaxed) {
                drop(result);
                Event::Cancelled
            } else {
                match result {
                    Ok(prepared) => Event::Prepared(prepared),
                    Err(error) => Event::Failed(format!("{error:#}")),
                }
            };
            let _ = sender.send(event);
        });
    }

    fn merge(&mut self) {
        if self.is_busy() {
            return;
        }
        let Some(prepared) = self.prepared.take() else {
            return;
        };
        let Some(editor) = &self.editor else {
            self.prepared = Some(prepared);
            return;
        };
        let pages = editor.ordered_pages().to_vec();
        let settings = self.settings.clone();
        self.error = None;
        self.output = None;
        self.progress = Progress {
            fraction: 0.0,
            message: tr("Creazione del PDF unico…", "Creating merged PDF…").into(),
        };
        self.begin_worker(Work::Merging, move |sender, cancel| {
            let result =
                pdf_merge::merge_prepared(&prepared, &pages, &settings, &cancel, |progress| {
                    let _ = sender.try_send(Event::Progress(progress));
                })
                .map_err(|error| format!("{error:#}"));
            let event = if cancel.load(Ordering::Relaxed) && result.is_err() {
                drop(prepared);
                Event::Cancelled
            } else {
                // A completed commit remains a success even if cancellation was
                // requested just after the final output was published.
                Event::Finished { prepared, result }
            };
            let _ = sender.send(event);
        });
    }

    fn cleanup_before_close(&mut self) {
        let Some(prepared) = self.prepared.take() else {
            return;
        };
        self.progress.message = tr("Chiusura…", "Closing…").into();
        self.begin_worker(Work::Cleaning, move |sender, _| {
            drop(prepared);
            let _ = sender.send(Event::Cleaned);
        });
    }

    fn receive_events(&mut self) {
        let mut events = Vec::new();
        let mut disconnected = false;
        if let Some(receiver) = &self.receiver {
            loop {
                match receiver.try_recv() {
                    Ok(event) => events.push(event),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }
        for event in events {
            match event {
                Event::Progress(progress) => self.progress = progress,
                Event::Prepared(prepared) => {
                    self.editor = Some(PageOrderEditor::new(prepared.sources(), prepared.pages()));
                    self.prepared = Some(prepared);
                    self.work = None;
                    self.receiver = None;
                    self.progress.message = tr(
                        "Pagine pronte: scegli l’ordine finale.",
                        "Pages are ready: choose the final order.",
                    )
                    .into();
                }
                Event::Finished { prepared, result } => {
                    self.prepared = Some(prepared);
                    self.work = None;
                    self.receiver = None;
                    match result {
                        Ok(mut outputs) if outputs.len() == 1 => {
                            self.output = outputs.pop();
                            self.progress = Progress {
                                fraction: 1.0,
                                message: tr("PDF unico creato", "Merged PDF created").into(),
                            };
                        }
                        Ok(_) => {
                            self.error = Some(
                                tr(
                                    "Il motore non ha restituito un PDF unico.",
                                    "The engine did not return a merged PDF.",
                                )
                                .into(),
                            )
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
                Event::Failed(error) => {
                    self.work = None;
                    self.receiver = None;
                    self.error = Some(error);
                }
                Event::Cancelled => {
                    self.work = None;
                    self.receiver = None;
                    self.editor = None;
                    self.progress.message =
                        tr("Operazione annullata.", "Operation cancelled.").into();
                }
                Event::Cleaned => {
                    self.work = None;
                    self.receiver = None;
                }
            }
        }
        if disconnected && self.receiver.is_some() {
            self.receiver = None;
            self.work = None;
            self.error = Some(
                tr(
                    "L’operazione in background è stata interrotta.",
                    "The background operation was interrupted.",
                )
                .into(),
            );
        }
    }

    fn destination(&self) -> String {
        self.settings
            .output_dir
            .as_deref()
            .or_else(|| self.files.first().and_then(|path| path.parent()))
            .map(display_path)
            .unwrap_or_else(|| tr("Accanto al primo file", "Next to the first file").into())
    }

    fn footer(&mut self, ui: &mut egui::Ui) {
        let colors = theme::palette(ui.ctx());
        ui.label(theme::heading(
            tr("Destinazione del PDF unico", "Merged PDF destination"),
            theme::BODY,
        ));
        ui.add(
            egui::Label::new(
                RichText::new(self.destination())
                    .size(theme::CAPTION)
                    .color(colors.secondary),
            )
            .wrap(),
        );
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    !self.is_busy(),
                    egui::Button::new(theme::button_text(
                        FluentIcon::Folder,
                        tr("Cambia cartella…", "Change folder…"),
                        colors.text,
                    )),
                )
                .clicked()
            {
                let mut dialog = rfd::FileDialog::new()
                    .set_title(tr("Destinazione del PDF unico", "Merged PDF destination"));
                if let Some(path) = self.settings.output_dir.as_ref() {
                    dialog = dialog.set_directory(path);
                }
                if let Some(path) = dialog.pick_folder() {
                    self.settings.output_dir = Some(path);
                }
            }
            if self.is_busy() {
                if ui
                    .add_enabled(
                        self.work != Some(Work::Cleaning) && !self.cancel.load(Ordering::Relaxed),
                        egui::Button::new(tr("Annulla", "Cancel")),
                    )
                    .clicked()
                {
                    self.cancel.store(true, Ordering::Relaxed);
                    self.progress.message = tr("Annullamento…", "Cancelling…").into();
                }
            } else if self.prepared.is_some() {
                if ui
                    .add(
                        egui::Button::new(theme::button_text(
                            FluentIcon::Convert,
                            tr("Crea PDF unico", "Create merged PDF"),
                            colors.accent_text,
                        ))
                        .fill(colors.accent),
                    )
                    .clicked()
                {
                    self.merge();
                }
            } else if ui
                .button(tr("Riprepara pagine", "Prepare pages again"))
                .clicked()
            {
                self.prepare();
            }
        });
    }
}

impl eframe::App for PdfMergeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::sync_window_theme(ctx);
        self.receive_events();
        if ctx.input(|input| input.viewport().close_requested()) {
            self.close_when_idle = true;
            if self.is_busy() {
                self.cancel.store(true, Ordering::Relaxed);
                self.progress.message =
                    tr("Annullamento e chiusura…", "Cancelling and closing…").into();
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            } else if self.prepared.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.cleanup_before_close();
            }
        }
        if self.close_when_idle && !self.is_busy() {
            if self.prepared.is_some() {
                self.cleanup_before_close();
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        let colors = theme::palette(ctx);
        egui::TopBottomPanel::bottom("pdf_merge_footer")
            .frame(
                egui::Frame::new()
                    .fill(colors.card)
                    .stroke(egui::Stroke::new(1.0_f32, colors.stroke))
                    .inner_margin(20),
            )
            .show(ctx, |ui| self.footer(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(colors.canvas).inner_margin(20))
            .show(ctx, |ui| {
                ui.label(theme::heading(
                    tr("Unisci in un PDF", "Merge into one PDF"),
                    theme::TITLE,
                ));
                ui.label(
                    RichText::new(trf!(
                        "{} file selezionati · un unico documento con l’ordine che scegli.",
                        "Selected files: {} · one document in the order you choose.",
                        self.files.len()
                    ))
                    .color(colors.secondary),
                );
                ui.add_space(12.0);
                if let Some(warning) = &self.startup_warning {
                    info_bar(ui, warning, false);
                }
                if let Some(error) = &self.error {
                    info_bar(ui, error, true);
                    if ui.small_button(tr("Copia errore", "Copy error")).clicked() {
                        ctx.copy_text(error.clone());
                    }
                    ui.add_space(6.0);
                }
                if let Some(output) = &self.output {
                    theme::show_card(ui, |ui| {
                        ui.label(
                            theme::heading(
                                tr("PDF unico creato", "Merged PDF created"),
                                theme::BODY,
                            )
                            .color(colors.success),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(display_path(output)).size(theme::CAPTION),
                            )
                            .wrap(),
                        );
                        if ui
                            .add(egui::Button::new(theme::button_text(
                                FluentIcon::Folder,
                                tr("Apri cartella", "Open folder"),
                                colors.text,
                            )))
                            .clicked()
                            && let Some(parent) = output.parent()
                            && let Err(error) = platform::open_folder(parent)
                        {
                            self.error = Some(format!("{error:#}"));
                        }
                    });
                    ui.add_space(8.0);
                }
                if self.is_busy() {
                    progress(ui, &self.progress);
                    ui.add_space(12.0);
                }
                let enabled = !self.is_busy();
                if let Some(editor) = &mut self.editor {
                    editor.show(ui, enabled);
                } else if self.work == Some(Work::Preparing) {
                    ui.label(tr(
                        "Le pagine saranno disponibili appena completata la preparazione.",
                        "Pages will be available when preparation is complete.",
                    ));
                    egui::ScrollArea::vertical()
                        .id_salt("pdf_merge_sources")
                        .show(ui, |ui| {
                            for file in &self.files {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(
                                            file.file_name().unwrap_or_default().to_string_lossy(),
                                        )
                                        .size(theme::CAPTION)
                                        .color(colors.secondary),
                                    )
                                    .wrap(),
                                );
                            }
                        });
                } else if self.error.is_none() && self.work != Some(Work::Cleaning) {
                    ui.label(&self.progress.message);
                }
            });
        if self.is_busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

impl Drop for PdfMergeApp {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(prepared) = self.prepared.take() {
            // Returning to the parent UI must not remove a workspace on its frame.
            let _ = std::thread::Builder::new()
                .name("pdf-merge-cleanup".into())
                .spawn(move || drop(prepared));
        }
    }
}

fn info_bar(ui: &mut egui::Ui, message: &str, error: bool) {
    let colors = theme::palette(ui.ctx());
    let tone = if error {
        colors.error
    } else {
        ui.visuals().warn_fg_color
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
    ui.add_space(8.0);
}

fn progress(ui: &mut egui::Ui, progress: &Progress) {
    let colors = theme::palette(ui.ctx());
    let fraction = if progress.fraction.is_finite() {
        progress.fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    ui.horizontal(|ui| {
        ui.add(
            egui::Label::new(
                RichText::new(&progress.message)
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
    let mut fill = rect;
    fill.max.x = fill.min.x + fill.width() * fraction;
    ui.painter().rect_filled(fill, 1.5, colors.accent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use the_converter::model::CsvDelimiter;

    #[test]
    fn session_initialization_preserves_import_and_destination_settings() {
        let mut settings = ConversionSettings::default();
        settings.image.quality = 83;
        settings.image.width = Some(1200);
        settings.csv.delimiter = CsvDelimiter::Semicolon;
        settings.output_dir = Some(PathBuf::from("destinazione scelta"));
        let app = PdfMergeApp::new(Vec::new(), settings, ToolPaths::default(), None);
        assert_eq!(app.settings.format, OutputFormat::Pdf);
        assert_eq!(app.settings.pdf.action, PdfAction::Merge);
        assert_eq!(app.settings.image.quality, 83);
        assert_eq!(app.settings.image.width, Some(1200));
        assert_eq!(app.settings.csv.delimiter, CsvDelimiter::Semicolon);
        assert_eq!(
            app.settings.output_dir,
            Some(PathBuf::from("destinazione scelta"))
        );
        assert!(app.is_busy());
    }

    #[test]
    fn leaving_the_session_signals_the_shared_worker_cancellation_flag() {
        let app = PdfMergeApp::new(
            Vec::new(),
            ConversionSettings::default(),
            ToolPaths::default(),
            None,
        );
        let worker_cancel = app.cancel.clone();
        assert!(!worker_cancel.load(Ordering::Relaxed));
        drop(app);
        assert!(worker_cancel.load(Ordering::Relaxed));
    }

    #[test]
    fn a_disconnected_worker_releases_the_busy_state_and_reports_failure() {
        let mut app = PdfMergeApp::new(
            Vec::new(),
            ConversionSettings::default(),
            ToolPaths::default(),
            None,
        );
        let (sender, receiver) = mpsc::sync_channel(1);
        app.receiver = Some(receiver);
        drop(sender);
        app.receive_events();
        assert!(!app.is_busy());
        assert!(
            app.error
                .as_deref()
                .is_some_and(|error| error.contains("interrotta"))
        );
    }
}
