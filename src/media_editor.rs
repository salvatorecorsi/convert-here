//! Single-track trim/gain editor. Source overviews and conversions run on workers.
use crate::theme::{self, FluentIcon};
use eframe::egui::{self, Color32, RichText};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};
use the_converter::{
    engine::Progress,
    i18n::tr,
    media_preview::{self, MediaPreview},
    model::{ConversionSettings, MediaKind, OutputFormat, supported_formats},
    platform::{self, display_path},
    tools::ToolPaths,
    trf,
};

pub(crate) enum MediaEditorAction {
    Back,
    SetDefaults(ConversionSettings),
    SaveProfile {
        id: Option<String>,
        name: String,
        show_in_menu: bool,
        settings: ConversionSettings,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Handle {
    Start,
    End,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Selection {
    start: f64,
    end: f64,
    duration: f64,
}
impl Selection {
    fn from_settings(duration: f64, settings: &ConversionSettings) -> Self {
        let gap = duration.min(0.01);
        let start = settings
            .video
            .trim_start
            .unwrap_or(0.0)
            .clamp(0.0, (duration - gap).max(0.0));
        let end = settings
            .video
            .trim_end
            .unwrap_or(duration)
            .clamp(start + gap, duration);
        Self {
            start,
            end,
            duration,
        }
    }
    fn set(&mut self, handle: Handle, time: f64) {
        let gap = self.duration.min(0.01);
        match handle {
            Handle::Start => self.start = time.clamp(0.0, (self.end - gap).max(0.0)),
            Handle::End => {
                self.end = time.clamp((self.start + gap).min(self.duration), self.duration)
            }
        }
    }
}

#[derive(Default)]
struct Timeline {
    active: Option<Handle>,
    selected_end: bool,
}
impl Timeline {
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        selection: &mut Selection,
        wave: Option<&egui::TextureHandle>,
        frames: &[(f64, egui::TextureHandle)],
    ) -> bool {
        let height = 166.0 + if frames.is_empty() { 0.0 } else { 104.0 };
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width().max(180.0), height),
            egui::Sense::click_and_drag(),
        );
        let content = rect.shrink2(egui::vec2(12.0, 18.0));
        let colors = theme::palette(ui.ctx());
        let x = |time: f64| content.left() + (time / selection.duration) as f32 * content.width();
        let left = x(selection.start);
        let right = x(selection.end);
        ui.painter().rect_filled(rect, 8.0, colors.card_hover);
        let wave_rect = egui::Rect::from_min_size(content.min, egui::vec2(content.width(), 118.0));
        if let Some(wave) = wave {
            ui.painter().image(
                wave.id(),
                wave_rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter().line_segment(
                [
                    egui::pos2(content.left(), wave_rect.center().y),
                    egui::pos2(content.right(), wave_rect.center().y),
                ],
                egui::Stroke::new(1.0_f32, colors.stroke),
            );
            ui.painter().text(
                wave_rect.center(),
                egui::Align2::CENTER_CENTER,
                tr("Intervallo della sorgente", "Source interval"),
                theme::semibold_font(theme::CAPTION),
                colors.secondary,
            );
        }
        if !frames.is_empty() {
            let width = content.width() / frames.len() as f32;
            for (index, (time, texture)) in frames.iter().enumerate() {
                let frame = egui::Rect::from_min_size(
                    egui::pos2(content.left() + index as f32 * width, content.top() + 124.0),
                    egui::vec2((width - 4.0).max(1.0), 72.0),
                );
                let source = texture.size_vec2();
                let source_aspect = source.x / source.y;
                let frame_aspect = frame.width() / frame.height();
                let (u, v) = if frame_aspect > source_aspect {
                    (0.0, (1.0 - source_aspect / frame_aspect) * 0.5)
                } else {
                    ((1.0 - frame_aspect / source_aspect) * 0.5, 0.0)
                };
                ui.painter().image(
                    texture.id(),
                    frame,
                    egui::Rect::from_min_max(egui::pos2(u, v), egui::pos2(1.0 - u, 1.0 - v)),
                    Color32::WHITE,
                );
                ui.painter().text(
                    egui::pos2(frame.center().x, frame.bottom() + 3.0),
                    egui::Align2::CENTER_TOP,
                    time_label(*time),
                    theme::semibold_font(theme::CAPTION),
                    colors.secondary,
                );
            }
        }
        for outside in [
            egui::Rect::from_min_max(content.min, egui::pos2(left, content.bottom())),
            egui::Rect::from_min_max(egui::pos2(right, content.top()), content.max),
        ] {
            ui.painter()
                .rect_filled(outside, 0.0, Color32::from_black_alpha(125));
        }
        let selected = egui::Rect::from_min_max(
            egui::pos2(left, content.top()),
            egui::pos2(right, content.bottom()),
        );
        ui.painter().rect_stroke(
            selected,
            0.0,
            egui::Stroke::new(2.0_f32, colors.accent),
            egui::StrokeKind::Inside,
        );
        for (handle, position, label) in [
            (Handle::Start, left, tr("Inizio", "Start")),
            (Handle::End, right, tr("Fine", "End")),
        ] {
            let focus = response.has_focus() && self.selected_end == (handle == Handle::End);
            let color = if focus { colors.text } else { colors.accent };
            ui.painter().line_segment(
                [
                    egui::pos2(position, content.top()),
                    egui::pos2(position, content.bottom()),
                ],
                egui::Stroke::new(3.0_f32, color),
            );
            ui.painter().rect_filled(
                egui::Rect::from_center_size(
                    egui::pos2(position, content.top()),
                    egui::vec2(18.0, 28.0),
                ),
                4.0,
                color,
            );
            ui.painter().text(
                egui::pos2(position, rect.bottom() - 2.0),
                egui::Align2::CENTER_BOTTOM,
                label,
                theme::semibold_font(theme::CAPTION),
                colors.text,
            );
        }
        let before = *selection;
        let pressed = ui.input(|input| input.pointer.primary_pressed());
        let origin = ui.input(|input| input.pointer.press_origin());
        // Native events can coalesce press+move into one frame; use press origin.
        if ui.is_enabled()
            && pressed
            && let Some(origin) = origin
            && rect.contains(origin)
            && ui.ctx().layer_id_at(origin) == Some(ui.layer_id())
        {
            let start_distance = (origin.x - left).abs();
            let end_distance = (origin.x - right).abs();
            let handle = if start_distance < end_distance
                || start_distance == end_distance && !self.selected_end
            {
                Handle::Start
            } else {
                Handle::End
            };
            self.selected_end = handle == Handle::End;
            self.active = Some(handle);
            response.request_focus();
        }
        if let Some(handle) = self.active {
            if let Some(pointer) = ui.input(|input| {
                input
                    .pointer
                    .interact_pos()
                    .or_else(|| input.pointer.latest_pos())
            }) {
                selection.set(
                    handle,
                    f64::from((pointer.x - content.left()) / content.width()) * selection.duration,
                );
            }
            if !ui.input(|input| input.pointer.primary_down()) {
                self.active = None;
            }
        }
        if ui.is_enabled() && response.has_focus() {
            let selected = if self.selected_end {
                Handle::End
            } else {
                Handle::Start
            };
            ui.input(|input| {
                let step = if input.modifiers.shift {
                    1.0
                } else if input.modifiers.alt {
                    0.01
                } else {
                    0.1
                };
                let value = if self.selected_end {
                    selection.end
                } else {
                    selection.start
                };
                if input.key_pressed(egui::Key::ArrowLeft) {
                    selection.set(selected, value - step);
                }
                if input.key_pressed(egui::Key::ArrowRight) {
                    selection.set(selected, value + step);
                }
                if input.key_pressed(egui::Key::Home) {
                    selection.set(selected, 0.0);
                }
                if input.key_pressed(egui::Key::End) {
                    selection.set(selected, selection.duration);
                }
                if input.key_pressed(egui::Key::Space) {
                    self.selected_end = !self.selected_end;
                }
            });
        }
        response.on_hover_text(tr("Trascina le maniglie. Tastiera: Spazio cambia maniglia; ←/→ 0,1 s, Maiusc 1 s, Alt 0,01 s.", "Drag the handles. Keyboard: Space switches handle; ←/→ 0.1 s, Shift 1 s, Alt 0.01 s."));
        *selection != before
    }
}

enum Event {
    Progress(Progress),
    Prepared(Result<MediaPreview, String>),
    Saved {
        result: Result<PathBuf, String>,
        cancelled: bool,
        history_warning: Option<String>,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Work {
    Prepare,
    Save,
}
struct Task {
    kind: Work,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Event>,
}

pub(crate) struct MediaEditor {
    input: PathBuf,
    settings: ConversionSettings,
    tools: ToolPaths,
    formats: Vec<OutputFormat>,
    preview: Option<MediaPreview>,
    waveform: Option<egui::TextureHandle>,
    frames: Vec<(f64, egui::TextureHandle)>,
    selection: Option<Selection>,
    timeline: Timeline,
    range_dirty: bool,
    gain_db: f64,
    muted: bool,
    gain_dirty: bool,
    task: Option<Task>,
    pending_prepare: bool,
    progress: Progress,
    notice: Option<(String, bool)>,
    output: Option<PathBuf>,
    profile_id: Option<String>,
    profile_name: String,
    profile_menu: bool,
    closing: bool,
    back_when_idle: bool,
}

impl MediaEditor {
    pub(crate) fn new(
        input: PathBuf,
        mut settings: ConversionSettings,
        tools: ToolPaths,
        formats: Vec<OutputFormat>,
    ) -> Self {
        let supported = supported_formats(MediaKind::from_path(&input));
        let formats: Vec<_> = formats
            .into_iter()
            .filter(|format| {
                supported.contains(format)
                    && (matches!(format.kind(), MediaKind::Audio | MediaKind::Video)
                        || matches!(format, OutputFormat::Gif | OutputFormat::Txt))
            })
            .collect();
        let preferred = if MediaKind::from_path(&input) == MediaKind::Video {
            OutputFormat::Mp4
        } else {
            OutputFormat::Mp3
        };
        if !formats.contains(&settings.format)
            && let Some(format) = formats
                .iter()
                .find(|format| **format == preferred)
                .or_else(|| formats.first())
        {
            settings.select_output_format(*format);
            settings.media_options.clear();
        }
        let volume = settings
            .media_options
            .get("audio_volume")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(1.0);
        let muted = volume == 0.0;
        let gain_db = if volume > 0.0 {
            (20.0 * volume.log10()).clamp(-60.0, 20.0)
        } else {
            0.0
        };
        Self {
            input,
            settings,
            tools,
            formats,
            preview: None,
            waveform: None,
            frames: vec![],
            selection: None,
            timeline: Timeline::default(),
            range_dirty: false,
            gain_db,
            muted,
            gain_dirty: false,
            task: None,
            pending_prepare: true,
            progress: Progress {
                fraction: 0.0,
                message: tr("Preparazione…", "Preparing…").into(),
            },
            notice: None,
            output: None,
            profile_id: None,
            profile_name: String::new(),
            profile_menu: true,
            closing: false,
            back_when_idle: false,
        }
    }
    pub(crate) fn is_busy(&self) -> bool {
        self.task.is_some()
    }
    pub(crate) fn notify(&mut self, message: String, error: bool) {
        self.notice = Some((message, error));
    }
    pub(crate) fn editing_profile(&mut self, id: String, name: String, show: bool) {
        self.profile_id = Some(id);
        self.profile_name = name;
        self.profile_menu = show;
    }

    fn begin(
        &mut self,
        kind: Work,
        task: impl FnOnce(mpsc::SyncSender<Event>, Arc<AtomicBool>) + Send + 'static,
    ) {
        let (sender, receiver) = mpsc::sync_channel(32);
        let cancel = Arc::new(AtomicBool::new(false));
        self.task = Some(Task {
            kind,
            cancel: cancel.clone(),
            receiver,
        });
        if let Err(error) = std::thread::Builder::new()
            .name("media-editor".into())
            .spawn(move || task(sender, cancel))
        {
            self.task = None;
            self.notify(
                format!(
                    "{}: {error}",
                    tr(
                        "Impossibile avviare l’operazione",
                        "Could not start the operation"
                    )
                ),
                true,
            );
        }
    }
    fn prepare(&mut self, ctx: &egui::Context) {
        self.pending_prepare = false;
        self.notice = None;
        let input = self.input.clone();
        let tools = self.tools.clone();
        let ctx = ctx.clone();
        self.begin(Work::Prepare, move |sender, cancel| {
            let result = media_preview::prepare(&input, &tools, &cancel, |progress| {
                let _ = sender.try_send(Event::Progress(progress));
                ctx.request_repaint();
            })
            .map_err(|error| format!("{error:#}"));
            let _ = sender.send(Event::Prepared(result));
            ctx.request_repaint();
        });
    }
    fn receive(&mut self, ctx: &egui::Context) {
        while let Some(task) = &self.task {
            let event = match task.receiver.try_recv() {
                Ok(event) => event,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.task = None;
                    self.notify(
                        tr(
                            "Elaborazione interrotta. Puoi riprovare.",
                            "Processing interrupted. You can try again.",
                        )
                        .into(),
                        true,
                    );
                    break;
                }
            };
            match event {
                Event::Progress(progress) => self.progress = progress,
                Event::Prepared(result) => {
                    self.task = None;
                    match result {
                        Ok(preview) => {
                            self.selection =
                                Some(Selection::from_settings(preview.duration, &self.settings));
                            if let Some(wave) = &preview.waveform {
                                self.waveform = Some(texture(ctx, "media-waveform", wave));
                            }
                            self.frames = preview
                                .frames
                                .iter()
                                .enumerate()
                                .map(|(index, frame)| {
                                    (
                                        frame.time,
                                        texture(
                                            ctx,
                                            &format!("media-frame-{index}"),
                                            &frame.pixels,
                                        ),
                                    )
                                })
                                .collect();
                            self.preview = Some(preview);
                        }
                        Err(error) => {
                            if !self.closing && !self.back_when_idle {
                                self.notify(error, true);
                            }
                        }
                    }
                }
                Event::Saved {
                    result,
                    cancelled,
                    history_warning,
                } => {
                    self.task = None;
                    match result {
                        Ok(path) => {
                            self.output = Some(path.clone());
                            self.notify(
                                trf!("Salvato: {}", "Saved: {}", display_path(&path)),
                                false,
                            );
                        }
                        Err(error) => {
                            self.notify(
                                if cancelled {
                                    tr("Operazione annullata", "Operation cancelled").into()
                                } else {
                                    error
                                },
                                !cancelled,
                            );
                        }
                    }
                    if let Some(warning) = history_warning {
                        self.notify(
                            trf!(
                                "Cronologia non salvata: {warning}",
                                "History was not saved: {warning}"
                            ),
                            true,
                        );
                    }
                }
            }
        }
    }
    fn current_settings(&self) -> anyhow::Result<ConversionSettings> {
        let selection = self.selection.context(tr(
            "Attendi la lettura della durata",
            "Wait for the duration to load",
        ))?;
        if !self.range_dirty {
            anyhow::ensure!(
                self.settings.video.trim_start.unwrap_or(0.0) < selection.duration,
                tr(
                    "L’inizio del profilo supera la durata del file: modifica l’intervallo.",
                    "The profile start is beyond this file’s duration: adjust the interval."
                )
            );
        }
        let text = self.settings.format == OutputFormat::Txt;
        let mut settings = media_preview::settings_for_interval(
            &self.settings,
            selection.start,
            selection.end,
            if text { 0.0 } else { self.gain_db },
            !text && self.muted,
        )?;
        if !self.range_dirty {
            settings.video.trim_start = self.settings.video.trim_start;
            settings.video.trim_end = self.settings.video.trim_end;
        }
        if text {
            settings.media_options.clear();
        } else if !self.gain_dirty {
            match self.settings.media_options.get("audio_volume") {
                Some(value) => {
                    settings
                        .media_options
                        .insert("audio_volume".into(), value.clone());
                }
                None => {
                    settings.media_options.remove("audio_volume");
                }
            }
        }
        Ok(settings)
    }
    fn save_dialog(&mut self, ctx: &egui::Context) {
        let settings = match self.current_settings() {
            Ok(settings) => settings,
            Err(error) => {
                self.notify(format!("{error:#}"), true);
                return;
            }
        };
        let name = format!(
            "{}-edited.{}",
            self.input.file_stem().unwrap_or_default().to_string_lossy(),
            settings.format.extension()
        );
        let mut dialog = rfd::FileDialog::new()
            .set_title(tr("Salva il file modificato", "Save the edited file"))
            .set_file_name(name)
            .add_filter(settings.format.label(), &[settings.format.extension()]);
        if let Some(folder) = settings
            .output_dir
            .as_deref()
            .or_else(|| self.input.parent())
        {
            dialog = dialog.set_directory(folder);
        }
        let Some(mut destination) = dialog.save_file() else {
            return;
        };
        if destination.extension().is_none() {
            destination.set_extension(settings.format.extension());
        }
        let Some(stamp) = self.preview.as_ref().map(|preview| preview.stamp.clone()) else {
            return;
        };
        let input = self.input.clone();
        let tools = self.tools.clone();
        let history = crate::gui_history::path();
        let ctx = ctx.clone();
        self.output = None;
        self.progress = Progress {
            fraction: 0.0,
            message: tr("Salvataggio del file…", "Saving the file…").into(),
        };
        self.begin(Work::Save, move |sender, cancel| {
            let result = media_preview::save(
                &input,
                &destination,
                &settings,
                &tools,
                &stamp,
                &cancel,
                |progress| {
                    let _ = sender.try_send(Event::Progress(progress));
                    ctx.request_repaint();
                },
            )
            .map_err(|error| format!("{error:#}"));
            let cancelled = cancel.load(Ordering::Relaxed);
            let history_warning =
                record_history(history, input, settings.format, &result, cancelled)
                    .err()
                    .map(|error| format!("{error:#}"));
            let _ = sender.send(Event::Saved {
                result,
                cancelled,
                history_warning,
            });
            ctx.request_repaint();
        });
    }

    pub(crate) fn show(&mut self, ctx: &egui::Context) -> Option<MediaEditorAction> {
        self.receive(ctx);
        if self.pending_prepare && !self.is_busy() && !self.closing && !self.back_when_idle {
            self.prepare(ctx);
        }
        if ctx.input(|input| input.viewport().close_requested())
            && let Some(task) = &self.task
        {
            task.cancel.store(true, Ordering::Relaxed);
            self.closing = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.closing && !self.is_busy() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.back_when_idle && !self.is_busy() {
            return Some(MediaEditorAction::Back);
        }
        theme::sync_window_theme(ctx);
        let mut back = false;
        let mut save = false;
        let mut defaults = false;
        let mut profile = false;
        egui::TopBottomPanel::top("media_editor_header")
            .frame(
                egui::Frame::new()
                    .fill(theme::palette(ctx).canvas)
                    .inner_margin(16),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    back = ui
                        .button(theme::button_text(
                            FluentIcon::Back,
                            tr("Indietro", "Back"),
                            theme::palette(ctx).text,
                        ))
                        .clicked();
                    ui.label(theme::heading(
                        tr("Modifica audio / video", "Edit audio / video"),
                        theme::SUBTITLE,
                    ));
                    ui.label(self.input.file_name().unwrap_or_default().to_string_lossy())
                        .on_hover_text(display_path(&self.input));
                });
            });
        egui::SidePanel::right("media_editor_options").exact_width(300.0).resizable(false).show(ctx,|ui|{
            egui::ScrollArea::vertical().show(ui,|ui|{
                ui.add_enabled_ui(!self.is_busy(),|ui|{
                    ui.label(theme::heading(tr("Uscita","Output"),theme::BODY));
                    let before=self.settings.format;
                    egui::ComboBox::from_id_salt("media_output").height(240.0).selected_text(before.label()).show_ui(ui,|ui|{for format in &self.formats {ui.selectable_value(&mut self.settings.format,*format,format.label());}});
                    if before!=self.settings.format {let volume=self.settings.media_options.get("audio_volume").cloned();self.settings.media_options.clear();if let Some(volume)=volume{self.settings.media_options.insert("audio_volume".into(),volume);}self.settings.select_output_format(self.settings.format);self.settings.audio.sample_rate=0;}
                    if !self.formats.contains(&self.settings.format){ui.colored_label(theme::palette(ctx).error,tr("Abilita questo formato nelle Impostazioni o scegli un’altra uscita.","Enable this format in Settings or choose another output."));}
                    ui.add_space(12.0);ui.label(theme::heading(tr("Volume","Volume"),theme::BODY));
                    let has_audio=self.preview.as_ref().is_some_and(|preview|preview.has_audio);
                    let transcribing=self.settings.format==OutputFormat::Txt;
                    ui.add_enabled_ui(has_audio && !transcribing,|ui|{
                        self.gain_dirty|=ui.checkbox(&mut self.muted,tr("Muto","Mute")).changed();
                        self.gain_dirty|=ui.add_enabled(!self.muted,egui::Slider::new(&mut self.gain_db,-60.0..=20.0).text("dB")).changed();
                        if ui.button(tr("Ripristina 0 dB","Reset to 0 dB")).clicked(){self.gain_db=0.0;self.muted=false;self.gain_dirty=true;}
                    });
                    if self.preview.is_some() && !has_audio {ui.label(tr("Il file non contiene una traccia audio.","This file has no audio stream."));}
                    if transcribing {
                        ui.label(tr("TXT usa soltanto l’intervallo selezionato: gain, muto e opzioni encoder non vengono applicati.","TXT uses only the selected interval: gain, mute and encoder options do not apply."));
                        crate::settings_ui::speech_controls(ui,&mut self.settings.speech);
                    } else if self.settings.audio.normalize {ui.label(tr("Il gain viene applicato dopo la normalizzazione attiva.","Gain is applied after the enabled normalization."));}
                    if let Some(limit)=self.settings.limits.max_duration {ui.label(trf!("Limite durata del profilo: {}","Profile duration limit: {}",time_label(limit)));}
                    if let Some(limit)=self.settings.limits.max_bytes {ui.label(trf!("Peso massimo: {:.0} kB","Maximum size: {:.0} kB",limit as f64/1000.0));}
                    ui.add_space(12.0);
                    let ready=self.preview.is_some() && self.formats.contains(&self.settings.format);
                    save=ui.add_enabled(ready,egui::Button::new(tr("Salva con nome…","Save as…"))).clicked();
                    defaults=ui.add_enabled(ready,egui::Button::new(tr("Usa queste impostazioni","Use these settings")).wrap()).clicked();
                    ui.add_space(16.0);ui.label(theme::heading(tr("Profilo riutilizzabile","Reusable profile"),theme::BODY));
                    ui.add(egui::TextEdit::singleline(&mut self.profile_name).hint_text(tr("Nome profilo","Profile name")).char_limit(40));
                    ui.checkbox(&mut self.profile_menu,tr("Mostra in Converti in","Show in Convert to"));
                    profile=ui.add_enabled(ready && !self.profile_name.trim().is_empty(),egui::Button::new(if self.profile_id.is_some(){tr("Aggiorna profilo","Update profile")}else{tr("Salva profilo","Save profile")})).clicked();
                });
            });
        });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::palette(ctx).canvas).inner_margin(20)).show(ctx,|ui|{
            egui::ScrollArea::vertical().show(ui,|ui|{
                if let Some((message,error))=&self.notice {ui.colored_label(if *error{theme::palette(ctx).error}else{theme::palette(ctx).success},message);}
                if let Some(preview)=&self.preview {
                    ui.label(trf!("Durata sorgente: {}","Source duration: {}",time_label(preview.duration)));
                    ui.label(RichText::new(tr("Forma d’onda e fotogrammi della sorgente, prima delle modifiche.","Source waveform and frames, before edits.")).color(theme::palette(ctx).secondary));
                    for warning in &preview.warnings {ui.colored_label(theme::palette(ctx).secondary,warning);}
                }
                if let Some(selection)=&mut self.selection {
                    ui.add_space(12.0);
                    ui.add_enabled_ui(self.task.as_ref().is_none_or(|task|task.kind!=Work::Save),|ui|{
                        self.range_dirty|=self.timeline.show(ui,selection,self.waveform.as_ref(),&self.frames);
                        ui.add_space(10.0);
                        ui.horizontal_wrapped(|ui|{
                            ui.label(tr("Inizio","Start"));
                            let old=selection.start;
                            if ui.add(egui::DragValue::new(&mut selection.start).range(0.0..=selection.duration).speed(0.1).suffix(" s")).changed(){let value=selection.start;selection.start=old;selection.set(Handle::Start,value);self.range_dirty=true;}
                            ui.label(tr("Fine","End"));
                            let old=selection.end;
                            if ui.add(egui::DragValue::new(&mut selection.end).range(0.0..=selection.duration).speed(0.1).suffix(" s")).changed(){let value=selection.end;selection.end=old;selection.set(Handle::End,value);self.range_dirty=true;}
                            if ui.button(tr("Intero file","Entire file")).clicked(){selection.start=0.0;selection.end=selection.duration;self.range_dirty=true;}
                        });
                        ui.label(trf!("Intervallo: {} → {} · selezionati {}","Interval: {} → {} · selected {}",time_label(selection.start),time_label(selection.end),time_label(selection.end-selection.start)));
                        ui.label(RichText::new(tr("Tastiera sulla timeline: Spazio cambia maniglia, frecce spostano il bordo.","Keyboard on the timeline: Space switches handle, arrows move the edge.")).color(theme::palette(ctx).secondary));
                    });
                }
                if self.is_busy(){
                    ui.add_space(14.0);ui.label(&self.progress.message);
                    ui.horizontal(|ui|{ui.add(egui::ProgressBar::new(self.progress.fraction.clamp(0.0,1.0)).desired_width((ui.available_width()-60.0).max(80.0)).desired_height(3.0));ui.label(format!("{:.0}%",self.progress.fraction*100.0));});
                    if ui.button(tr("Annulla","Cancel")).clicked() && let Some(task)=&self.task {task.cancel.store(true,Ordering::Relaxed);}
                } else if self.preview.is_none() && ui.button(tr("Riprova anteprima","Retry preview")).clicked(){self.pending_prepare=true;}
                if let Some(output)=&self.output && ui.button(theme::button_text(FluentIcon::Folder,tr("Apri cartella risultato","Open output folder"),theme::palette(ctx).text)).clicked() && let Some(parent)=output.parent() && let Err(error)=platform::open_folder(parent){self.notify(format!("{error:#}"),true);}
            });
        });
        if let Err(error) = self.current_settings()
            && self.preview.is_some()
        {
            self.notice = Some((format!("{error:#}"), true));
            save = false;
            defaults = false;
            profile = false;
        }
        if back {
            if let Some(task) = &self.task {
                task.cancel.store(true, Ordering::Relaxed);
                self.back_when_idle = true;
            } else {
                return Some(MediaEditorAction::Back);
            }
        }
        if save && !self.is_busy() {
            self.save_dialog(ctx);
        }
        let action = if (defaults || profile) && !self.is_busy() {
            self.current_settings().ok().map(|settings| {
                if defaults {
                    MediaEditorAction::SetDefaults(settings)
                } else {
                    MediaEditorAction::SaveProfile {
                        id: self.profile_id.clone(),
                        name: self.profile_name.trim().into(),
                        show_in_menu: self.profile_menu,
                        settings,
                    }
                }
            })
        } else {
            None
        };
        if self.is_busy() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        if let Some((message, error)) =
            crate::gui_history::guard_close(ctx, self.is_busy(), self.closing)
        {
            self.notify(message, error);
        }
        action
    }
}
impl Drop for MediaEditor {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.cancel.store(true, Ordering::Relaxed);
        }
    }
}

fn texture(ctx: &egui::Context, name: &str, pixels: &image::RgbaImage) -> egui::TextureHandle {
    ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(
            [pixels.width() as usize, pixels.height() as usize],
            pixels.as_raw(),
        ),
        egui::TextureOptions::LINEAR,
    )
}
fn time_label(value: f64) -> String {
    let millis = (value.max(0.0) * 1000.0).round() as u64;
    let seconds = millis / 1000;
    if seconds >= 3600 {
        format!(
            "{}:{:02}:{:02}.{:03}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60,
            millis % 1000
        )
    } else {
        format!("{}:{:02}.{:03}", seconds / 60, seconds % 60, millis % 1000)
    }
}
fn record_history(
    path: Option<PathBuf>,
    input: PathBuf,
    format: OutputFormat,
    result: &Result<PathBuf, String>,
    cancelled: bool,
) -> anyhow::Result<()> {
    use the_converter::queue::{JobState, QueueJob};
    let Some(path) = path else { return Ok(()) };
    let (state, outputs, error) = match result {
        Ok(output) => (JobState::Completed, vec![output.clone()], None),
        Err(_) if cancelled => (JobState::Cancelled, vec![], None),
        Err(error) => (JobState::Failed, vec![], Some(error.clone())),
    };
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    crate::gui_history::record_async(
        path,
        QueueJob {
            id: format!("media-editor-{}-{stamp}", std::process::id()),
            input,
            format: format.label().into(),
            outputs,
            state,
            progress: if state == JobState::Completed {
                1.0
            } else {
                0.0
            },
            message: String::new(),
            error,
        },
    )
}

use anyhow::Context;

#[cfg(test)]
mod tests {
    use super::*;
    fn timeline_frame(
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        timeline: &mut Timeline,
        selection: &mut Selection,
    ) -> egui::Rect {
        let mut rect = egui::Rect::NOTHING;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 500.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    rect = egui::Rect::from_min_size(
                        ui.available_rect_before_wrap().min,
                        egui::vec2(ui.available_width(), 166.0),
                    );
                    timeline.show(ui, selection, None, &[]);
                });
            },
        );
        rect
    }
    #[test]
    fn timeline_handles_coalesced_native_drag_and_keyboard_without_crossing_edges() {
        let ctx = egui::Context::default();
        theme::configure(&ctx);
        let mut timeline = Timeline::default();
        let mut selection = Selection {
            start: 1.0,
            end: 3.0,
            duration: 5.0,
        };
        let rect = timeline_frame(&ctx, vec![], &mut timeline, &mut selection)
            .shrink2(egui::vec2(12.0, 18.0));
        let start = egui::pos2(rect.left() + rect.width() / 5.0, rect.top() + 24.0);
        let target = egui::pos2(rect.left() + rect.width() / 2.0, start.y);
        timeline_frame(
            &ctx,
            vec![egui::Event::PointerMoved(start)],
            &mut timeline,
            &mut selection,
        );
        timeline_frame(
            &ctx,
            vec![
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
                egui::Event::PointerMoved(target),
            ],
            &mut timeline,
            &mut selection,
        );
        assert!((selection.start - 2.5).abs() < 0.001, "{}", selection.start);
        timeline_frame(
            &ctx,
            vec![egui::Event::PointerButton {
                pos: target,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            &mut timeline,
            &mut selection,
        );
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        };
        timeline_frame(
            &ctx,
            vec![key(egui::Key::ArrowLeft)],
            &mut timeline,
            &mut selection,
        );
        assert!((selection.start - 2.4).abs() < 0.001);
        timeline_frame(
            &ctx,
            vec![key(egui::Key::Space)],
            &mut timeline,
            &mut selection,
        );
        timeline_frame(
            &ctx,
            vec![key(egui::Key::ArrowRight)],
            &mut timeline,
            &mut selection,
        );
        assert!((selection.end - 3.1).abs() < 0.001);
        assert!(
            selection.start >= 0.0
                && selection.end <= selection.duration
                && selection.start < selection.end
        );
    }
    #[test]
    fn timeline_bounds_keep_positive_interval_and_support_subsecond_precision() {
        let mut selection = Selection {
            start: 1.0,
            end: 3.0,
            duration: 5.0,
        };
        selection.set(Handle::Start, 20.0);
        assert!((selection.start - 2.99).abs() < 1e-9);
        selection.set(Handle::End, -1.0);
        assert!(selection.end > selection.start);
        selection.set(Handle::Start, 0.25);
        selection.set(Handle::End, 0.75);
        assert_eq!((selection.start, selection.end), (0.25, 0.75));
    }
    #[test]
    fn profile_format_gain_and_other_options_survive_opening_editor() {
        let mut settings = ConversionSettings::default();
        settings.select_output_format(OutputFormat::Flac);
        settings
            .media_options
            .insert("audio_volume".into(), "0.123456789".into());
        settings
            .media_options
            .insert("title".into(), "Original".into());
        let mut editor = MediaEditor::new(
            PathBuf::from("input.wav"),
            settings.clone(),
            ToolPaths::default(),
            vec![OutputFormat::Mp3, OutputFormat::Flac],
        );
        editor.selection = Some(Selection::from_settings(20.0, &settings));
        let snapshot = editor.current_settings().unwrap();
        assert_eq!(snapshot.format, OutputFormat::Flac);
        assert_eq!(snapshot.media_options, settings.media_options);
        assert_eq!(snapshot.video.trim_end, settings.video.trim_end);
    }
    #[test]
    fn transcription_target_falls_back_to_an_enabled_media_encoder() {
        for (input, expected) in [
            ("audio.wav", OutputFormat::Mp3),
            ("video.mp4", OutputFormat::Mp4),
        ] {
            let mut settings = ConversionSettings::default();
            settings.select_output_format(OutputFormat::Txt);
            let editor = MediaEditor::new(
                PathBuf::from(input),
                settings,
                ToolPaths::default(),
                vec![OutputFormat::Mp3, OutputFormat::Mp4],
            );
            assert_eq!(editor.settings.format, expected);
            assert!(!editor.formats.contains(&OutputFormat::Txt));
        }
    }
    #[test]
    fn transcription_profile_keeps_text_output_and_trim_without_encoder_or_gain_options() {
        let mut settings = ConversionSettings::default();
        settings.select_output_format(OutputFormat::Txt);
        settings.video.trim_start = Some(1.0);
        settings.video.trim_end = Some(3.0);
        settings.speech.language = "it".into();
        settings
            .media_options
            .insert("audio_volume".into(), "2".into());
        settings
            .media_options
            .insert("audio_encoder".into(), "aac".into());
        let mut editor = MediaEditor::new(
            PathBuf::from("input.wav"),
            settings.clone(),
            ToolPaths::default(),
            vec![OutputFormat::Txt, OutputFormat::Mp3],
        );
        editor.selection = Some(Selection::from_settings(10.0, &settings));
        editor.gain_db = 10.0;
        editor.muted = true;
        editor.gain_dirty = true;
        let snapshot = editor.current_settings().unwrap();
        assert_eq!(snapshot.format, OutputFormat::Txt);
        assert_eq!(snapshot.video.trim_start, Some(1.0));
        assert_eq!(snapshot.video.trim_end, Some(3.0));
        assert_eq!(snapshot.speech.language, "it");
        assert!(snapshot.media_options.is_empty());
        assert_eq!(editor.settings.media_options["audio_volume"], "2");
    }
}
