//! Interactive editor. A preview owns the exact bytes later saved by the worker.
use crate::theme::{self, FluentIcon};
use eframe::egui::{self, RichText};
use image::RgbaImage;
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use the_converter::{i18n::tr, trf};
use the_converter::{
    image_edit::{
        self, CanvasAnchor, CanvasFit, CanvasSettings, EncodedImage, ImageEditSettings,
        NormalizedCrop, ResizeSettings,
    },
    model::{ConversionSettings, MediaKind, OutputFormat, PdfAction},
    page_layout::{
        self, ImageFit, PageInput, PageLayoutSettings, PageOrientation, PaperSize,
        PreparedPageDocument, TextAlign,
    },
    platform::display_path,
    tools::ToolPaths,
};

const PREVIEW_DELAY: Duration = Duration::from_millis(250);
const PREVIEW_SIDE: u32 = 1600;
const MAX_TEXT_BYTES: u64 = 8 * 1024 * 1024;

pub(crate) enum EditorAction {
    Back,
    SaveProfile {
        id: Option<String>,
        name: String,
        show_in_menu: bool,
        settings: ConversionSettings,
    },
    SetDefaults(ConversionSettings),
}

enum Content {
    Image(Option<Arc<RgbaImage>>),
    Text(String),
}
enum PreviewData {
    Image(EncodedImage),
    Pdf(PreparedPageDocument, Option<u64>),
}
impl PreviewData {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Image(image) => &image.bytes,
            Self::Pdf(pdf, _) => pdf.bytes(),
        }
    }
    fn target_met(&self) -> bool {
        match self {
            Self::Image(image) => image.target_met,
            Self::Pdf(pdf, limit) => limit.is_none_or(|limit| pdf.exact_bytes() <= limit),
        }
    }
    fn pages(&self) -> usize {
        match self {
            Self::Image(_) => 1,
            Self::Pdf(pdf, _) => pdf.page_count(),
        }
    }
}
enum WorkOutput {
    LoadedImage(Arc<RgbaImage>, RgbaImage),
    LoadedText(String),
    Preview(PreviewData, RgbaImage),
    Page(usize, RgbaImage),
    Saved(PathBuf),
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkKind {
    Load,
    Preview,
    Page,
    Save,
}
struct Task {
    kind: WorkKind,
    source_epoch: u64,
    revision: u64,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Result<WorkOutput, String>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct DetailKey {
    source_epoch: u64,
    revision: u64,
    crop: bool,
    bounds: [u32; 4],
    tile_side: u32,
}
struct DetailTile {
    rect: [u32; 4],
    texture: egui::TextureHandle,
}
struct DetailCache {
    key: DetailKey,
    tiles: Vec<DetailTile>,
}

pub(crate) struct ImageEditor {
    content: Content,
    input: Option<PathBuf>,
    settings: ConversionSettings,
    tools: ToolPaths,
    formats: Vec<OutputFormat>,
    source_epoch: u64,
    revision: u64,
    preview_revision: u64,
    changed_at: Instant,
    pending_load: bool,
    dirty: bool,
    task: Option<Task>,
    preview: Option<Arc<PreviewData>>,
    preview_texture: Option<egui::TextureHandle>,
    source_texture: Option<egui::TextureHandle>,
    page: usize,
    shown_page: usize,
    crop_mode: bool,
    crop_start: Option<egui::Pos2>,
    crop_drag: Option<NormalizedCrop>,
    profile_id: Option<String>,
    profile_name: String,
    profile_menu: bool,
    notice: Option<(String, bool)>,
    zoom: f32,
    detail: bool,
    detail_cache: Option<DetailCache>,
    closing: bool,
}

pub(crate) fn editable(path: &Path) -> bool {
    MediaKind::from_path(path) == MediaKind::Image
        || path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
}

impl ImageEditor {
    pub(crate) fn new(
        input: Option<PathBuf>,
        text: bool,
        mut settings: ConversionSettings,
        tools: ToolPaths,
        formats: Vec<OutputFormat>,
    ) -> Self {
        let formats: Vec<_> = formats
            .into_iter()
            .filter(|format| {
                matches!(
                    format,
                    OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp | OutputFormat::Pdf
                )
            })
            .collect();
        if text {
            settings.format = OutputFormat::Pdf;
            settings.image.edit = None;
            settings
                .image
                .page
                .get_or_insert_with(PageLayoutSettings::default);
        } else {
            if !formats.contains(&settings.format) {
                settings.format = formats.first().copied().unwrap_or(OutputFormat::Png);
            }
            if settings.image.edit.is_none() {
                let mut edit = ImageEditSettings::default();
                if let Some(width) = settings.image.width {
                    edit.resize = Some(ResizeSettings {
                        width: Some(width),
                        ..Default::default()
                    });
                }
                settings.image.edit = Some(edit);
            }
            if settings.format == OutputFormat::Pdf {
                settings
                    .image
                    .page
                    .get_or_insert_with(PageLayoutSettings::default);
            }
        }
        settings.pdf.action = PdfAction::Compress;
        let pending_load = input.is_some();
        Self {
            content: if text {
                Content::Text(String::new())
            } else {
                Content::Image(None)
            },
            input,
            settings,
            tools,
            formats,
            source_epoch: 1,
            revision: 1,
            preview_revision: 0,
            changed_at: Instant::now() - PREVIEW_DELAY,
            pending_load,
            dirty: pending_load || text,
            task: None,
            preview: None,
            preview_texture: None,
            source_texture: None,
            page: 0,
            shown_page: 0,
            crop_mode: false,
            crop_start: None,
            crop_drag: None,
            profile_id: None,
            profile_name: String::new(),
            profile_menu: true,
            notice: None,
            zoom: 1.0,
            detail: false,
            detail_cache: None,
            closing: false,
        }
    }

    pub(crate) fn editing_profile(&mut self, id: String, name: String, show_in_menu: bool) {
        self.profile_id = Some(id);
        self.profile_name = name;
        self.profile_menu = show_in_menu;
    }
    pub(crate) fn notify(&mut self, message: String, error: bool) {
        self.notice = Some((message, error));
    }
    fn is_text(&self) -> bool {
        matches!(self.content, Content::Text(_))
    }
    fn saving(&self) -> bool {
        self.task
            .as_ref()
            .is_some_and(|task| task.kind == WorkKind::Save)
    }
    fn can_save(&self) -> bool {
        self.task.is_none()
            && self.preview_revision == self.revision
            && !self.dirty
            && self.formats.contains(&self.settings.format)
            && self
                .preview
                .as_ref()
                .is_some_and(|preview| preview.target_met())
    }
    fn source_dimensions(&self) -> Option<(u32, u32)> {
        match &self.content {
            Content::Image(Some(image)) => Some(image.dimensions()),
            _ => None,
        }
    }
    fn changed(&mut self) {
        self.closing = false;
        self.revision = self.revision.wrapping_add(1);
        self.changed_at = Instant::now();
        self.dirty = self.is_text() || self.source_dimensions().is_some() || self.pending_load;
        self.page = 0;
        if let Some(task) = &self.task
            && task.kind != WorkKind::Load
            && task.kind != WorkKind::Save
        {
            task.cancel.store(true, Ordering::Relaxed);
        }
    }
    fn set_input(&mut self, input: PathBuf) {
        if let Some(task) = &self.task {
            task.cancel.store(true, Ordering::Relaxed);
        }
        self.source_epoch = self.source_epoch.wrapping_add(1);
        self.input = Some(input);
        self.pending_load = true;
        match &mut self.content {
            Content::Image(pixels) => *pixels = None,
            Content::Text(text) => text.clear(),
        }
        self.preview = None;
        self.source_texture = None;
        self.preview_texture = None;
        self.detail_cache = None;
        self.notice = None;
        self.changed();
    }

    fn spawn(
        &mut self,
        kind: WorkKind,
        ctx: &egui::Context,
        work: impl FnOnce(Arc<AtomicBool>) -> anyhow::Result<WorkOutput> + Send + 'static,
    ) {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        self.task = Some(Task {
            kind,
            source_epoch: self.source_epoch,
            revision: self.revision,
            cancel: cancel.clone(),
            receiver,
        });
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = work(cancel).map_err(|error| format!("{error:#}"));
            let _ = sender.send(result);
            ctx.request_repaint();
        });
    }
    fn texture(ctx: &egui::Context, name: &str, image: &RgbaImage) -> egui::TextureHandle {
        ctx.load_texture(
            name,
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            ),
            egui::TextureOptions::LINEAR,
        )
    }
    fn receive(&mut self, ctx: &egui::Context) {
        let Some(task) = &self.task else {
            return;
        };
        let result = match task.receiver.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Err(tr(
                "Elaborazione interrotta. Puoi riprovare.",
                "Processing interrupted. You can try again.",
            )
            .into()),
        };
        let task = self.task.take().expect("task present");
        if task.source_epoch != self.source_epoch {
            return;
        }
        let current = task.revision == self.revision;
        match result {
            Ok(WorkOutput::LoadedImage(pixels, thumb)) => {
                self.content = Content::Image(Some(pixels));
                self.source_texture = Some(Self::texture(ctx, "editor_source", &thumb));
                self.dirty = true;
            }
            Ok(WorkOutput::LoadedText(text)) => {
                self.content = Content::Text(text);
                self.dirty = true;
            }
            Ok(WorkOutput::Preview(preview, thumb)) if current => {
                self.preview = Some(Arc::new(preview));
                self.preview_texture = Some(Self::texture(ctx, "editor_preview", &thumb));
                self.preview_revision = task.revision;
                self.shown_page = 0;
                self.notice = None;
            }
            Ok(WorkOutput::Page(page, thumb)) if current && page == self.page => {
                self.preview_texture = Some(Self::texture(ctx, "editor_preview_page", &thumb));
                self.shown_page = page;
            }
            Ok(WorkOutput::Saved(path)) => {
                self.notice = Some((trf!("Salvato: {}", "Saved: {}", display_path(&path)), false))
            }
            Err(error)
                if !task.cancel.load(Ordering::Relaxed)
                    && (current || task.kind == WorkKind::Load) =>
            {
                if task.kind == WorkKind::Load {
                    self.dirty = false;
                }
                self.notice = Some((error, true));
            }
            _ => {}
        }
    }
    fn start_pending(&mut self, ctx: &egui::Context) {
        if self.task.is_some() || self.closing {
            return;
        }
        if self.pending_load {
            self.pending_load = false;
            let Some(path) = self.input.clone() else {
                return;
            };
            let text = self.is_text();
            let tools = self.tools.clone();
            self.spawn(WorkKind::Load, ctx, move |cancel| {
                if text {
                    use std::io::Read;
                    let mut bytes = Vec::new();
                    std::fs::File::open(path)?
                        .take(MAX_TEXT_BYTES + 1)
                        .read_to_end(&mut bytes)?;
                    anyhow::ensure!(
                        bytes.len() as u64 <= MAX_TEXT_BYTES,
                        tr("Il file TXT supera 8 MiB", "The TXT file exceeds 8 MiB")
                    );
                    let text = String::from_utf8(bytes).map_err(|_| {
                        anyhow::anyhow!(tr(
                            "Apri un file TXT con codifica UTF-8.",
                            "Open a UTF-8 TXT file."
                        ))
                    })?;
                    anyhow::ensure!(
                        !cancel.load(Ordering::Relaxed),
                        tr("Lettura annullata", "Reading cancelled")
                    );
                    Ok(WorkOutput::LoadedText(
                        text.trim_start_matches('\u{feff}').to_owned(),
                    ))
                } else {
                    let loaded = image_edit::load_image(&path, &tools, &cancel, |_| {})?;
                    let thumb = thumbnail(&loaded.pixels);
                    Ok(WorkOutput::LoadedImage(loaded.pixels, thumb))
                }
            });
            return;
        }
        if self.dirty && self.changed_at.elapsed() >= PREVIEW_DELAY {
            let pixels = match &self.content {
                Content::Image(Some(pixels)) => Some(pixels.clone()),
                Content::Image(None) => return,
                Content::Text(_) => None,
            };
            let text = match &self.content {
                Content::Text(text) => Some(Arc::<str>::from(text.as_str())),
                _ => None,
            };
            let settings = self.settings.clone();
            self.dirty = false;
            self.spawn(WorkKind::Preview, ctx, move |cancel| {
                if let Some(text) = text {
                    let page = settings
                        .image
                        .page
                        .as_ref()
                        .context(tr("Scegli un formato pagina", "Choose a page format"))?;
                    let pdf = page_layout::prepare(
                        &[PageInput::Text(text)],
                        page,
                        settings.image.quality,
                        &cancel,
                        |_| {},
                    )?;
                    let thumb = pdf.preview(0, PREVIEW_SIDE, &cancel)?;
                    Ok(WorkOutput::Preview(PreviewData::Pdf(pdf, None), thumb))
                } else {
                    let recipe = settings.image.edit.as_ref().context(tr(
                        "Impostazioni immagine mancanti",
                        "Missing image settings",
                    ))?;
                    let pixels =
                        image_edit::render(&pixels.expect("image source"), recipe, &cancel)?;
                    if settings.format == OutputFormat::Pdf {
                        let page = settings.image.page.as_ref().context(tr(
                            "Scegli A4 o A5 per il PDF",
                            "Choose A4 or A5 for the PDF",
                        ))?;
                        let pdf = page_layout::prepare(
                            &[PageInput::Image(Arc::new(pixels))],
                            page,
                            settings.image.quality,
                            &cancel,
                            |_| {},
                        )?;
                        let thumb = pdf.preview(0, PREVIEW_SIDE, &cancel)?;
                        Ok(WorkOutput::Preview(
                            PreviewData::Pdf(pdf, recipe.max_bytes),
                            thumb,
                        ))
                    } else {
                        let encoded = image_edit::encode(
                            &pixels,
                            settings.format,
                            settings.image.quality,
                            recipe,
                            &cancel,
                            |_| {},
                        )?;
                        let thumb = thumbnail(&encoded.decoded_preview);
                        Ok(WorkOutput::Preview(PreviewData::Image(encoded), thumb))
                    }
                }
            });
            return;
        }
        if !self.dirty
            && self.preview_revision == self.revision
            && self.page != self.shown_page
            && let Some(preview) = self.preview.clone()
        {
            let page = self.page;
            self.spawn(WorkKind::Page, ctx, move |cancel| {
                let PreviewData::Pdf(pdf, _) = preview.as_ref() else {
                    anyhow::bail!(tr(
                        "Anteprima non paginata",
                        "The preview does not contain multiple pages"
                    ));
                };
                Ok(WorkOutput::Page(
                    page,
                    pdf.preview(page, PREVIEW_SIDE, &cancel)?,
                ))
            });
        }
    }

    pub(crate) fn show(&mut self, ctx: &egui::Context) -> Option<EditorAction> {
        self.receive(ctx);
        self.start_pending(ctx);
        theme::sync_window_theme(ctx);
        let mut action = None;
        let mut save_requested = false;
        let saving = self.saving();
        egui::TopBottomPanel::top("image_editor_navigation")
            .frame(
                egui::Frame::new()
                    .fill(theme::palette(ctx).canvas)
                    .inner_margin(egui::Margin::symmetric(20, 12)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !saving,
                            egui::Button::new(theme::button_text(
                                FluentIcon::Back,
                                tr("Indietro", "Back"),
                                theme::palette(ctx).text,
                            ))
                            .frame(false),
                        )
                        .clicked()
                    {
                        action = Some(EditorAction::Back);
                    }
                    ui.label(theme::heading(
                        if self.is_text() {
                            tr("Impagina testo", "Lay out text")
                        } else {
                            tr("Modifica immagine", "Edit image")
                        },
                        theme::SUBTITLE,
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                self.can_save(),
                                egui::Button::new(if saving {
                                    tr("Salvataggio…", "Saving…")
                                } else {
                                    tr("Salva con nome…", "Save as…")
                                }),
                            )
                            .clicked()
                        {
                            save_requested = true;
                        }
                    });
                });
            });
        egui::SidePanel::left("image_editor_options").exact_width(345.0).resizable(false).frame(
            egui::Frame::new().fill(theme::palette(ctx).canvas).inner_margin(16)
        ).show(ctx, |ui| {
            egui::ScrollArea::vertical().id_salt("image_editor_options_scroll").show(ui, |ui| {
                ui.add_enabled_ui(!saving, |ui| {
                    self.source_controls(ui);
                    ui.add_space(8.0);
                    let mut changed = false;
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading(tr("Uscita", "Output"), theme::BODY));
                        if self.is_text() {
                            ui.label(tr("PDF · testo impaginato", "PDF · laid-out text"));
                        } else {
                            let old = self.settings.format;
                            egui::ComboBox::from_id_salt("editor_output_format").selected_text(self.settings.format.label()).show_ui(ui, |ui| {
                                for format in &self.formats { ui.selectable_value(&mut self.settings.format, *format, format.label()); }
                            });
                            if old != self.settings.format {
                                changed = true;
                                if self.settings.format == OutputFormat::Pdf {
                                    self.settings.image.page.get_or_insert_with(PageLayoutSettings::default);
                                    if let Some(edit) = &mut self.settings.image.edit { edit.max_bytes = None; }
                                }
                            }
                        }
                        if !self.formats.contains(&self.settings.format) {
                            ui.colored_label(theme::palette(ctx).error, tr("Abilita questo formato nelle Impostazioni del programma per salvare.", "Enable this format in the app Settings to save."));
                        }
                        if !self.is_text() && matches!(self.settings.format, OutputFormat::Jpg | OutputFormat::Webp | OutputFormat::Pdf) {
                            let adaptive=self.settings.format!=OutputFormat::Pdf&&self.settings.image.edit.as_ref().is_some_and(|edit|edit.max_bytes.is_some());
                            changed |= ui.add(egui::Slider::new(&mut self.settings.image.quality, 1..=100).text(if adaptive{tr("Qualità massima", "Maximum quality")}else{tr("Qualità", "Quality")})).changed();
                        } else if !self.is_text() {
                            ui.label(tr("PNG conserva i pixel senza perdita.", "PNG preserves pixels losslessly."));
                        }
                        if let Some(edit) = &mut self.settings.image.edit {
                            let mut limited = edit.max_bytes.is_some();
                            if ui.checkbox(&mut limited, tr("Peso massimo", "Maximum file size")).changed() {
                                edit.max_bytes = limited.then_some(300_000); changed = true;
                            }
                            if let Some(max_bytes) = edit.max_bytes.as_mut() {
                                let mut kb = *max_bytes as f64 / 1000.0;
                                if ui.add(egui::DragValue::new(&mut kb).range(0.1..=1_000_000.0).speed(1.0).suffix(" kB")).changed() {
                                    *max_bytes = (kb * 1000.0).round() as u64; changed = true;
                                }
                                if self.settings.format==OutputFormat::Pdf {
                                    ui.label(tr("1 kB = 1.000 byte. Per rientrare nel limite PDF regola qualità, dimensioni o DPI.", "1 kB = 1,000 bytes. Adjust quality, dimensions or DPI to meet the PDF limit."));
                                } else if self.settings.format==OutputFormat::Png {
                                    ui.label(tr("PNG è senza perdita. Per ridurre il peso abilita la riduzione delle dimensioni.", "PNG is lossless. Allow smaller dimensions to reduce the file size."));
                                    changed |= ui.checkbox(&mut edit.allow_downsize, tr("Riduci anche le dimensioni se necessario", "Also reduce dimensions if necessary")).changed();
                                } else {
                                    ui.label(tr("1 kB = 1.000 byte. La qualità viene adattata al limite.", "1 kB = 1,000 bytes. Quality is adjusted to meet the limit."));
                                    changed |= ui.checkbox(&mut edit.allow_downsize, tr("Riduci anche le dimensioni se necessario", "Also reduce dimensions if necessary")).changed();
                                }
                            }
                        }
                    });
                    if !self.is_text() {
                        ui.add_space(8.0);
                        let dimensions = self.source_dimensions();
                        let edit = self.settings.image.edit.get_or_insert_with(ImageEditSettings::default);
                        changed |= edit_controls(ui, edit, dimensions, &mut self.crop_mode);
                    }
                    if self.settings.format == OutputFormat::Pdf {
                        ui.add_space(8.0);
                        let text = self.is_text();
                        let page = self.settings.image.page.get_or_insert_with(PageLayoutSettings::default);
                        changed |= page_controls(ui, page, text);
                    }
                    if changed { self.changed(); }
                    ui.add_space(8.0);
                    theme::show_card(ui, |ui| {
                        ui.label(theme::heading(tr("Salva le mie impostazioni", "Save my settings"), theme::BODY));
                        ui.add(egui::TextEdit::singleline(&mut self.profile_name).hint_text(tr("Per esempio: Salva per web", "For example: Save for web")).char_limit(40));
                        ui.checkbox(&mut self.profile_menu, tr("Mostra in Converti in", "Show in Convert to"));
                        if ui.add_enabled(!self.profile_name.trim().is_empty(), egui::Button::new(if self.profile_id.is_some() { tr("Aggiorna profilo", "Update profile") } else { tr("Salva profilo", "Save profile") })).clicked() {
                            action = Some(EditorAction::SaveProfile { id: self.profile_id.clone(), name: self.profile_name.clone(), show_in_menu: self.profile_menu, settings: self.settings.clone() });
                        }
                        if ui.add(egui::Button::new(tr("Usa come impostazioni predefinite", "Use as default settings")).wrap()).clicked() {
                            action = Some(EditorAction::SetDefaults(self.settings.clone()));
                        }
                        ui.label(RichText::new(tr("Il profilo conserva le opzioni. Immagine e testo di prova restano fuori dal profilo.", "The profile stores the options. The sample image and text are not included.")).small().weak());
                    });
                });
            });
        });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::palette(ctx).canvas)
                    .inner_margin(20),
            )
            .show(ctx, |ui| {
                if let Some((message, error)) = &self.notice {
                    ui.colored_label(
                        if *error {
                            theme::palette(ctx).error
                        } else {
                            theme::palette(ctx).success
                        },
                        message,
                    );
                }
                self.preview_header(ui);
                ui.add_space(10.0);
                ui.add_enabled_ui(!saving, |ui| self.preview_canvas(ui));
            });
        // Controls later in this frame can invalidate the preview after the
        // navigation button was clicked. Only save after processing all edits.
        if save_requested
            && action.is_none()
            && !ctx.input(|input| input.viewport().close_requested())
            && self.can_save()
        {
            self.save_dialog(ctx);
        }
        if self.task.is_some() || self.dirty {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if ctx.input(|input| input.viewport().close_requested())
            && let Some(task) = &self.task
        {
            self.closing = true;
            task.cancel.store(true, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.notice = Some((
                tr(
                    "Annullamento in corso. Chiudi la finestra al termine.",
                    "Cancelling. Close the window when finished.",
                )
                .into(),
                false,
            ));
        }
        action
    }

    fn source_controls(&mut self, ui: &mut egui::Ui) {
        theme::show_card(ui, |ui| {
            ui.label(theme::heading(
                if self.is_text() {
                    tr("Testo", "Text")
                } else {
                    tr("Immagine originale", "Original image")
                },
                theme::BODY,
            ));
            if let Some(input) = &self.input {
                ui.add(
                    egui::Label::new(input.file_name().unwrap_or_default().to_string_lossy())
                        .truncate(),
                )
                .on_hover_text(display_path(input));
            }
            if let Some((width, height)) = self.source_dimensions() {
                ui.label(format!("{width} × {height} px"));
            }
            if ui
                .button(if self.is_text() {
                    tr("Apri TXT…", "Open TXT…")
                } else {
                    tr("Apri immagine…", "Open image…")
                })
                .clicked()
            {
                let dialog = rfd::FileDialog::new();
                let dialog = if self.is_text() {
                    dialog.add_filter(tr("Testo UTF-8", "UTF-8 text"), &["txt"])
                } else {
                    dialog.add_filter(
                        tr("Immagini", "Images"),
                        &[
                            "png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff", "heic",
                            "heif", "avif", "ico", "qoi", "tga", "ff", "hdr", "exr", "pnm", "pam",
                            "pbm", "pgm", "ppm",
                        ],
                    )
                };
                if let Some(path) = dialog.pick_file() {
                    self.set_input(path);
                }
            }
            let loading = self.pending_load
                || self
                    .task
                    .as_ref()
                    .is_some_and(|task| task.kind == WorkKind::Load);
            if let Content::Text(text) = &mut self.content {
                let edited = egui::ScrollArea::vertical()
                    .id_salt("editor_source_text_scroll")
                    .max_height(220.0)
                    .show(ui, |ui| {
                        ui.add_enabled(
                            !loading,
                            egui::TextEdit::multiline(text)
                                .desired_rows(9)
                                .desired_width(f32::INFINITY)
                                .char_limit(2_000_000)
                                .hint_text(tr(
                                    "Scrivi o incolla il testo da impaginare…",
                                    "Type or paste the text to lay out…",
                                )),
                        )
                        .changed()
                    })
                    .inner;
                if edited {
                    self.changed();
                }
                ui.label(
                    RichText::new(tr(
                        "I file TXT devono essere UTF-8. L’originale viene conservato.",
                        "TXT files must be UTF-8. The original is preserved.",
                    ))
                    .small()
                    .weak(),
                );
            } else {
                ui.label(RichText::new(tr("L’editor usa la prima immagine o fotogramma. Il file originale non cambia.", "The editor uses the first image or frame. The original file is unchanged.")).small().weak());
            }
        });
    }
    fn preview_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(theme::heading(
                if self.crop_mode && !self.is_text() {
                    tr("Ritaglio sull’originale", "Crop the original")
                } else {
                    tr("Anteprima del risultato", "Result preview")
                },
                theme::SUBTITLE,
            ));
            if self.task.is_some() || self.dirty {
                ui.spinner();
                ui.label(tr("Aggiornamento…", "Updating…"));
            }
        });
        if let Some(preview) = &self.preview {
            let fresh = self.preview_revision == self.revision && !self.dirty;
            let bytes = preview.bytes().len();
            ui.horizontal_wrapped(|ui| {
                let color = if fresh && preview.target_met() {
                    theme::palette(ui.ctx()).success
                } else {
                    theme::palette(ui.ctx()).secondary
                };
                ui.colored_label(
                    color,
                    trf!(
                        "{:.1} kB · {bytes} byte{}",
                        "{:.1} kB · {bytes} bytes{}",
                        bytes as f64 / 1000.0,
                        if fresh {
                            ""
                        } else {
                            tr(" · anteprima precedente", " · previous preview")
                        }
                    ),
                );
                match preview.as_ref() {
                    PreviewData::Image(encoded) => {
                        ui.label(format!("{} × {} px", encoded.width, encoded.height));
                        if let Some(quality) = encoded.effective_quality {
                            ui.label(trf!(
                                "Qualità effettiva {quality}",
                                "Effective quality {quality}"
                            ));
                        }
                    }
                    PreviewData::Pdf(pdf, _) => {
                        ui.label(trf!("{} pagine", "{} pages", pdf.page_count()));
                    }
                }
            });
            if !preview.target_met() && fresh {
                ui.colored_label(theme::palette(ui.ctx()).error, tr("Il risultato supera il limite. Riduci dimensioni o abilita la riduzione automatica; Salva resta disattivato.", "The result exceeds the limit. Reduce dimensions or allow automatic downsizing; Save remains disabled."));
            }
            if preview.pages() > 1 {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.page > 0, egui::Button::new("←"))
                        .clicked()
                    {
                        self.page -= 1;
                    }
                    ui.label(trf!(
                        "Pagina {} di {}",
                        "Page {} of {}",
                        self.page + 1,
                        preview.pages()
                    ));
                    if ui
                        .add_enabled(self.page + 1 < preview.pages(), egui::Button::new("→"))
                        .clicked()
                    {
                        self.page += 1;
                    }
                });
            }
        }
        ui.horizontal(|ui| {
            if ui
                .selectable_label(!self.detail, tr("Adatta", "Fit"))
                .clicked()
            {
                self.detail = false;
                self.detail_cache = None;
                self.zoom = 1.0;
            }
            let can_detail = self.crop_mode && matches!(self.content, Content::Image(Some(_)))
                || self
                    .preview
                    .as_ref()
                    .is_some_and(|preview| matches!(preview.as_ref(), PreviewData::Image(_)));
            if !can_detail {
                self.detail = false;
                self.detail_cache = None;
            }
            if ui
                .add_enabled(
                    can_detail,
                    egui::Button::new(tr("Dettaglio 1:1", "Detail 1:1")).selected(self.detail),
                )
                .clicked()
            {
                self.detail = true;
            }
            if self.detail && can_detail {
                ui.label(tr(
                    "1 pixel immagine = 1 pixel schermo · usa le barre per scorrere",
                    "1 image pixel = 1 screen pixel · use the scroll bars to pan",
                ));
            } else {
                ui.label(tr("Zoom anteprima", "Preview zoom"));
                ui.add(egui::Slider::new(&mut self.zoom, 0.25..=3.0).show_value(false));
            }
        });
    }
    fn preview_canvas(&mut self, ui: &mut egui::Ui) {
        let crop = self.crop_mode && !self.is_text();
        let texture = if crop {
            self.source_texture.clone()
        } else {
            self.preview_texture.clone()
        };
        let Some(texture) = texture else {
            ui.vertical_centered(|ui| {
                ui.add_space(65.0);
                ui.label(if self.task.is_some() {
                    tr("Preparazione dell’anteprima…", "Preparing the preview…")
                } else if self.is_text() {
                    tr(
                        "Scrivi del testo per creare il foglio.",
                        "Type some text to create the page.",
                    )
                } else {
                    tr(
                        "Apri un’immagine per vedere l’anteprima.",
                        "Open an image to see the preview.",
                    )
                });
            });
            return;
        };
        if crop {
            ui.label(tr("Trascina sull’immagine per scegliere il ritaglio. Puoi anche impostare le percentuali a sinistra.", "Drag on the image to choose a crop. You can also set percentages on the left."));
        }
        // Keep the existing source/encoded snapshot alive without cloning its raster.
        let source = match &self.content {
            Content::Image(Some(source)) => Some(source.clone()),
            _ => None,
        };
        let preview = self.preview.clone();
        let detail_pixels = if crop {
            source.as_deref()
        } else {
            preview.as_deref().and_then(|preview| match preview {
                PreviewData::Image(image) => Some(&image.decoded_preview),
                PreviewData::Pdf(_, _) => None,
            })
        };
        let detail = self.detail && detail_pixels.is_some();
        let available = ui.available_size().max(egui::vec2(120.0, 120.0));
        egui::ScrollArea::both()
            .id_salt(("editor_preview_pan", detail, crop))
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, _viewport| {
                let native = detail_pixels.filter(|_| detail).map_or_else(
                    || texture.size_vec2(),
                    |pixels| egui::vec2(pixels.width() as f32, pixels.height() as f32),
                );
                let fit = ((available.x - 24.0) / native.x)
                    .min((available.y - 32.0) / native.y)
                    .clamp(0.05, 1.0);
                let size = if detail {
                    native / ui.ctx().pixels_per_point()
                } else {
                    native * fit * self.zoom
                };
                let inset = ((available.x - size.x) * 0.5).max(0.0);
                ui.horizontal_top(|ui| {
                    ui.add_space(inset);
                    let (id, allocated) = ui.allocate_space(size);
                    let ppp = ui.ctx().pixels_per_point();
                    let origin = egui::pos2(
                        (allocated.min.x * ppp).round() / ppp,
                        (allocated.min.y * ppp).round() / ppp,
                    );
                    let rect = egui::Rect::from_min_size(origin, size);
                    let response = ui.interact(
                        rect,
                        id,
                        if crop {
                            egui::Sense::click_and_drag()
                        } else {
                            egui::Sense::hover()
                        },
                    );
                    if detail {
                        self.paint_detail(ui, detail_pixels.expect("detail source"), rect, crop);
                    } else {
                        ui.painter().image(
                            texture.id(),
                            rect,
                            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    }
                    if crop {
                        let normalize = |point: egui::Pos2| {
                            egui::pos2(
                                ((point.x - response.rect.left()) / response.rect.width())
                                    .clamp(0.0, 1.0),
                                ((point.y - response.rect.top()) / response.rect.height())
                                    .clamp(0.0, 1.0),
                            )
                        };
                        let origin = ui.input(|input| input.pointer.press_origin());
                        let coalesced_press = ui.is_enabled()
                            && ui.input(|input| input.pointer.primary_pressed())
                            && origin.is_some_and(|point| {
                                response.rect.contains(point)
                                    && ui.clip_rect().contains(point)
                                    && ui.ctx().layer_id_at(point) == Some(ui.layer_id())
                            });
                        if (response.drag_started() || coalesced_press)
                            && let Some(point) = origin.or_else(|| response.interact_pointer_pos())
                        {
                            self.crop_start = Some(normalize(point));
                        }
                        if (response.dragged() || ui.input(|input| input.pointer.primary_down()))
                            && let (Some(start), Some(point)) = (
                                self.crop_start,
                                ui.input(|input| input.pointer.latest_pos()),
                            )
                        {
                            let end = normalize(point);
                            self.crop_drag = Some(NormalizedCrop {
                                left: f64::from(start.x.min(end.x)),
                                top: f64::from(start.y.min(end.y)),
                                right: f64::from(start.x.max(end.x)),
                                bottom: f64::from(start.y.max(end.y)),
                            });
                        }
                        if response.drag_stopped()
                            || self.crop_start.is_some()
                                && ui.input(|input| input.pointer.primary_released())
                        {
                            if let (Some(start), Some(point)) = (
                                self.crop_start,
                                ui.input(|input| input.pointer.latest_pos()),
                            ) {
                                let end = normalize(point);
                                self.crop_drag = Some(NormalizedCrop {
                                    left: f64::from(start.x.min(end.x)),
                                    top: f64::from(start.y.min(end.y)),
                                    right: f64::from(start.x.max(end.x)),
                                    bottom: f64::from(start.y.max(end.y)),
                                });
                            }
                            if let Some(selection) = self.crop_drag.take()
                                && let Some((width, height)) = self.source_dimensions()
                                && (selection.right * f64::from(width)).round()
                                    > (selection.left * f64::from(width)).round()
                                && (selection.bottom * f64::from(height)).round()
                                    > (selection.top * f64::from(height)).round()
                            {
                                self.settings.image.edit.as_mut().expect("image edit").crop =
                                    Some(selection);
                                self.changed();
                            }
                            self.crop_start = None;
                        }
                        if let Some(selection) = self.crop_drag.or_else(|| {
                            self.settings.image.edit.as_ref().and_then(|edit| edit.crop)
                        }) {
                            let rectangle = egui::Rect::from_min_max(
                                response.rect.min
                                    + egui::vec2(
                                        selection.left as f32 * size.x,
                                        selection.top as f32 * size.y,
                                    ),
                                response.rect.min
                                    + egui::vec2(
                                        selection.right as f32 * size.x,
                                        selection.bottom as f32 * size.y,
                                    ),
                            );
                            ui.painter().rect_stroke(
                                rectangle,
                                0.0,
                                egui::Stroke::new(2.0_f32, theme::palette(ui.ctx()).accent),
                                egui::StrokeKind::Inside,
                            );
                        }
                    }
                });
            });
    }

    fn paint_detail(
        &mut self,
        ui: &egui::Ui,
        pixels: &RgbaImage,
        full_rect: egui::Rect,
        crop: bool,
    ) {
        let visible = full_rect.intersect(ui.clip_rect());
        if !visible.is_positive() {
            return;
        }
        let ppp = ui.ctx().pixels_per_point();
        let side = (ui.ctx().input(|input| input.max_texture_side) as u32).clamp(1, 1024);
        let left =
            (((visible.left() - full_rect.left()) * ppp).floor().max(0.0) as u32 / side) * side;
        let top = (((visible.top() - full_rect.top()) * ppp).floor().max(0.0) as u32 / side) * side;
        let right = ((((visible.right() - full_rect.left()) * ppp).ceil().max(0.0) as u32)
            .div_ceil(side)
            * side)
            .min(pixels.width());
        let bottom = ((((visible.bottom() - full_rect.top()) * ppp).ceil().max(0.0) as u32)
            .div_ceil(side)
            * side)
            .min(pixels.height());
        let key = DetailKey {
            source_epoch: self.source_epoch,
            revision: if crop { 0 } else { self.preview_revision },
            crop,
            bounds: [left, top, right, bottom],
            tile_side: side,
        };
        if self
            .detail_cache
            .as_ref()
            .is_none_or(|cache| cache.key != key)
        {
            let mut previous = self
                .detail_cache
                .take()
                .filter(|cache| {
                    cache.key.source_epoch == key.source_epoch
                        && cache.key.revision == key.revision
                        && cache.key.crop == key.crop
                        && cache.key.tile_side == side
                })
                .map(|cache| cache.tiles)
                .unwrap_or_default();
            let mut tiles = Vec::new();
            for y in (top..bottom).step_by(side as usize) {
                for x in (left..right).step_by(side as usize) {
                    let rect = [
                        x,
                        y,
                        side.min(pixels.width() - x),
                        side.min(pixels.height() - y),
                    ];
                    if let Some(index) = previous.iter().position(|tile| tile.rect == rect) {
                        tiles.push(previous.swap_remove(index));
                    } else {
                        let tile =
                            image::imageops::crop_imm(pixels, x, y, rect[2], rect[3]).to_image();
                        let texture = ui.ctx().load_texture(
                            "editor_detail_tile",
                            egui::ColorImage::from_rgba_unmultiplied(
                                [rect[2] as usize, rect[3] as usize],
                                tile.as_raw(),
                            ),
                            egui::TextureOptions::NEAREST,
                        );
                        tiles.push(DetailTile { rect, texture });
                    }
                }
            }
            self.detail_cache = Some(DetailCache { key, tiles });
        }
        if let Some(cache) = &self.detail_cache {
            for tile in &cache.tiles {
                let [x, y, width, height] = tile.rect;
                let rect = egui::Rect::from_min_size(
                    full_rect.min + egui::vec2(x as f32 / ppp, y as f32 / ppp),
                    egui::vec2(width as f32 / ppp, height as f32 / ppp),
                );
                ui.painter().image(
                    tile.texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }
    }
    fn save_dialog(&mut self, ctx: &egui::Context) {
        let Some(preview) = self.preview.clone() else {
            return;
        };
        if !preview.target_met() {
            return;
        }
        let extension = self.settings.format.extension();
        let stem = self
            .input
            .as_ref()
            .and_then(|path| path.file_stem())
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| tr("Foglio", "Page").into());
        let mut dialog = rfd::FileDialog::new()
            .set_title(tr(
                "Salva il risultato senza modificare l’originale",
                "Save the result without changing the original",
            ))
            .set_file_name(trf!(
                "{stem}-modificato.{extension}",
                "{stem}-edited.{extension}"
            ))
            .add_filter(self.settings.format.label(), &[extension]);
        if let Some(directory) = self.settings.output_dir.clone().or_else(|| {
            self.input
                .as_ref()
                .and_then(|input| input.parent().map(Path::to_owned))
        }) {
            dialog = dialog.set_directory(directory);
        }
        if let Some(mut output) = dialog.save_file() {
            if output.extension().is_none() {
                output.set_extension(extension);
            }
            let valid_extension = output
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|value| {
                    value.eq_ignore_ascii_case(extension)
                        || (self.settings.format == OutputFormat::Jpg
                            && value.eq_ignore_ascii_case("jpeg"))
                });
            if !valid_extension {
                self.notice = Some((
                    trf!(
                        "Per questo risultato scegli un nome con estensione .{extension}.",
                        "Choose a filename ending in .{extension} for this result."
                    ),
                    true,
                ));
                return;
            }
            let input = self.input.clone();
            self.spawn(WorkKind::Save, ctx, move |cancel| {
                let path =
                    image_edit::save_bytes(input.as_deref(), &output, preview.bytes(), &cancel)?;
                Ok(WorkOutput::Saved(path))
            });
        }
    }
}
impl Drop for ImageEditor {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.cancel.store(true, Ordering::Relaxed);
        }
    }
}

use anyhow::Context as _;
fn thumbnail(image: &RgbaImage) -> RgbaImage {
    if image.width().max(image.height()) <= PREVIEW_SIDE {
        return image.clone();
    }
    image::DynamicImage::ImageRgba8(image.clone())
        .thumbnail(PREVIEW_SIDE, PREVIEW_SIDE)
        .into_rgba8()
}
fn edit_controls(
    ui: &mut egui::Ui,
    edit: &mut ImageEditSettings,
    dimensions: Option<(u32, u32)>,
    crop_mode: &mut bool,
) -> bool {
    let mut changed = false;
    theme::show_card(ui, |ui| {
        ui.label(theme::heading(tr("Ritaglio", "Crop"), theme::BODY));
        ui.checkbox(
            crop_mode,
            tr(
                "Seleziona area nell’anteprima",
                "Select an area in the preview",
            ),
        );
        if ui.button(tr("Immagine intera", "Entire image")).clicked() {
            edit.crop = None;
            changed = true;
        }
        if let Some(crop) = &mut edit.crop {
            for (label, edge) in [
                (tr("Sinistra", "Left"), &mut crop.left),
                (tr("Alto", "Top"), &mut crop.top),
                (tr("Destra", "Right"), &mut crop.right),
                (tr("Basso", "Bottom"), &mut crop.bottom),
            ] {
                let mut percent = *edge * 100.0;
                ui.horizontal(|ui| {
                    ui.label(label);
                    if ui
                        .add(
                            egui::DragValue::new(&mut percent)
                                .range(0.0..=100.0)
                                .speed(0.2)
                                .suffix(" %"),
                        )
                        .changed()
                    {
                        *edge = percent / 100.0;
                        changed = true;
                    }
                });
            }
            if let Some((width, height)) = dimensions {
                ui.label(trf!(
                    "Circa {} × {} px",
                    "About {} × {} px",
                    ((crop.right - crop.left) * f64::from(width))
                        .max(0.0)
                        .round(),
                    ((crop.bottom - crop.top) * f64::from(height))
                        .max(0.0)
                        .round()
                ));
            }
        }
        ui.label(RichText::new(tr("Un ritaglio salvato nel profilo si riapplica alla stessa area relativa di ogni immagine.", "A crop saved in a profile applies to the same relative area of each image.")).small().weak());
    });
    ui.add_space(8.0);
    theme::show_card(ui, |ui| {
        ui.label(theme::heading(tr("Scala", "Scale"), theme::BODY));
        let mut enabled = edit.resize.is_some();
        if ui
            .checkbox(&mut enabled, tr("Ridimensiona", "Resize"))
            .changed()
        {
            edit.resize = enabled.then(|| ResizeSettings {
                width: Some(dimensions.map_or(1920, |d| d.0.min(1920))),
                ..Default::default()
            });
            changed = true;
        }
        if let Some(resize) = &mut edit.resize {
            for (label, value, initial) in [
                (
                    tr("Larghezza", "Width"),
                    &mut resize.width,
                    dimensions.map_or(1080, |d| d.0),
                ),
                (
                    tr("Altezza", "Height"),
                    &mut resize.height,
                    dimensions.map_or(1080, |d| d.1),
                ),
            ] {
                ui.horizontal(|ui| {
                    let mut active = value.is_some();
                    if ui.checkbox(&mut active, label).changed() {
                        *value = active.then_some(initial);
                        changed = true;
                    }
                    if let Some(value) = value {
                        changed |= ui
                            .add(
                                egui::DragValue::new(value)
                                    .range(1..=image_edit::MAX_SIDE)
                                    .speed(1)
                                    .suffix(" px"),
                            )
                            .changed();
                    }
                });
            }
            changed |= ui
                .checkbox(
                    &mut resize.keep_aspect,
                    tr("Mantieni proporzioni", "Keep aspect ratio"),
                )
                .changed();
            ui.label(
                RichText::new(tr(
                    "Con entrambi i valori, l’immagine entra nelle dimensioni indicate.",
                    "With both values, the image fits within the specified dimensions.",
                ))
                .small()
                .weak(),
            );
        }
    });
    ui.add_space(8.0);
    theme::show_card(ui, |ui| {
        ui.label(theme::heading(
            tr("Tela e sfondo", "Canvas and background"),
            theme::BODY,
        ));
        let mut enabled = edit.canvas.is_some();
        if ui
            .checkbox(
                &mut enabled,
                tr(
                    "Inserisci in una risoluzione diversa",
                    "Place on a different canvas size",
                ),
            )
            .changed()
        {
            edit.canvas = enabled.then(CanvasSettings::default);
            changed = true;
        }
        if let Some(canvas) = &mut edit.canvas {
            ui.horizontal(|ui| {
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut canvas.width)
                            .range(1..=image_edit::MAX_SIDE)
                            .prefix(tr("L ", "W "))
                            .suffix(" px"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut canvas.height)
                            .range(1..=image_edit::MAX_SIDE)
                            .prefix("H ")
                            .suffix(" px"),
                    )
                    .changed();
            });
            egui::ComboBox::from_id_salt("canvas_fit")
                .selected_text(match canvas.fit {
                    CanvasFit::None => tr("Dimensioni già impostate", "Keep current image size"),
                    CanvasFit::Contain => tr("Adatta dentro la tela", "Fit inside the canvas"),
                    CanvasFit::Cover => tr("Riempi e ritaglia", "Fill and crop"),
                })
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(
                            &mut canvas.fit,
                            CanvasFit::None,
                            tr("Dimensioni già impostate", "Keep current image size"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut canvas.fit,
                            CanvasFit::Contain,
                            tr("Adatta dentro la tela", "Fit inside the canvas"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut canvas.fit,
                            CanvasFit::Cover,
                            tr("Riempi e ritaglia", "Fill and crop"),
                        )
                        .changed();
                });
            egui::ComboBox::from_id_salt("canvas_anchor")
                .selected_text(anchor_name(canvas.anchor))
                .show_ui(ui, |ui| {
                    for anchor in [
                        CanvasAnchor::TopLeft,
                        CanvasAnchor::Top,
                        CanvasAnchor::TopRight,
                        CanvasAnchor::Left,
                        CanvasAnchor::Center,
                        CanvasAnchor::Right,
                        CanvasAnchor::BottomLeft,
                        CanvasAnchor::Bottom,
                        CanvasAnchor::BottomRight,
                    ] {
                        changed |= ui
                            .selectable_value(&mut canvas.anchor, anchor, anchor_name(anchor))
                            .changed();
                    }
                });
            ui.horizontal(|ui| {
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut canvas.offset_x)
                            .range(-32768..=32768)
                            .prefix("X ")
                            .suffix(" px"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut canvas.offset_y)
                            .range(-32768..=32768)
                            .prefix("Y ")
                            .suffix(" px"),
                    )
                    .changed();
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Colore sfondo", "Background color"));
                changed |= ui
                    .color_edit_button_srgba_unmultiplied(&mut canvas.background)
                    .changed();
                if ui.small_button(tr("Bianco", "White")).clicked() {
                    canvas.background = [255, 255, 255, 255];
                    changed = true;
                }
                if ui.small_button(tr("Trasparente", "Transparent")).clicked() {
                    canvas.background = [0, 0, 0, 0];
                    changed = true;
                }
            });
        }
    });
    changed
}
fn anchor_name(anchor: CanvasAnchor) -> &'static str {
    match anchor {
        CanvasAnchor::TopLeft => tr("Alto a sinistra", "Top left"),
        CanvasAnchor::Top => tr("Alto", "Top"),
        CanvasAnchor::TopRight => tr("Alto a destra", "Top right"),
        CanvasAnchor::Left => tr("Sinistra", "Left"),
        CanvasAnchor::Center => tr("Centro", "Center"),
        CanvasAnchor::Right => tr("Destra", "Right"),
        CanvasAnchor::BottomLeft => tr("Basso a sinistra", "Bottom left"),
        CanvasAnchor::Bottom => tr("Basso", "Bottom"),
        CanvasAnchor::BottomRight => tr("Basso a destra", "Bottom right"),
    }
}
fn page_controls(ui: &mut egui::Ui, page: &mut PageLayoutSettings, text: bool) -> bool {
    let mut changed = false;
    theme::show_card(ui, |ui| {
        ui.label(theme::heading(tr("Foglio", "Page"), theme::BODY));
        ui.horizontal(|ui| {
            changed |= ui
                .selectable_value(&mut page.paper, PaperSize::A4, "A4")
                .changed();
            changed |= ui
                .selectable_value(&mut page.paper, PaperSize::A5, "A5")
                .changed();
        });
        ui.horizontal(|ui| {
            changed |= ui
                .radio_value(
                    &mut page.orientation,
                    PageOrientation::Portrait,
                    tr("Verticale", "Portrait"),
                )
                .changed();
            changed |= ui
                .radio_value(
                    &mut page.orientation,
                    PageOrientation::Landscape,
                    tr("Orizzontale", "Landscape"),
                )
                .changed();
        });
        changed |= ui
            .add(
                egui::DragValue::new(&mut page.margin_mm)
                    .range(0.0..=70.0)
                    .prefix(tr("Margini ", "Margins "))
                    .suffix(" mm")
                    .speed(0.5),
            )
            .changed();
        if text {
            changed |= ui
                .add(
                    egui::DragValue::new(&mut page.font_size_pt)
                        .range(6.0..=72.0)
                        .prefix(tr("Testo ", "Text "))
                        .suffix(" pt")
                        .speed(0.5),
                )
                .changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut page.line_height)
                        .range(1.0..=3.0)
                        .prefix(tr("Interlinea ", "Line spacing "))
                        .speed(0.05),
                )
                .changed();
            egui::ComboBox::from_id_salt("page_align")
                .selected_text(match page.align {
                    TextAlign::Left => tr("Allinea a sinistra", "Align left"),
                    TextAlign::Center => tr("Centra testo", "Center text"),
                    TextAlign::Right => tr("Allinea a destra", "Align right"),
                })
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(
                            &mut page.align,
                            TextAlign::Left,
                            tr("Allinea a sinistra", "Align left"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut page.align,
                            TextAlign::Center,
                            tr("Centra testo", "Center text"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut page.align,
                            TextAlign::Right,
                            tr("Allinea a destra", "Align right"),
                        )
                        .changed();
                });
        } else {
            ui.horizontal(|ui| {
                changed |= ui
                    .radio_value(
                        &mut page.fit,
                        ImageFit::Contain,
                        tr("Immagine intera", "Entire image"),
                    )
                    .changed();
                changed |= ui
                    .radio_value(
                        &mut page.fit,
                        ImageFit::Cover,
                        tr("Riempi foglio", "Fill the page"),
                    )
                    .changed();
            });
            changed |= ui
                .add(
                    egui::DragValue::new(&mut page.dpi)
                        .range(72..=600)
                        .prefix(tr("Risoluzione ", "Resolution "))
                        .suffix(" DPI")
                        .speed(1),
                )
                .changed();
        }
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changing_txt_source_discards_previous_content() {
        let mut editor = ImageEditor::new(
            None,
            true,
            ConversionSettings::default(),
            ToolPaths::default(),
            vec![OutputFormat::Pdf],
        );
        editor.content = Content::Text("Testo del documento precedente".into());
        editor.set_input(PathBuf::from("nuova-sorgente.txt"));
        assert!(matches!(&editor.content, Content::Text(text) if text.is_empty()));
        assert!(editor.pending_load);
        assert!(!editor.can_save());
    }

    #[test]
    fn editing_after_save_request_invalidates_encoded_preview() {
        let mut editor = ImageEditor::new(
            None,
            false,
            ConversionSettings::default(),
            ToolPaths::default(),
            vec![OutputFormat::Png],
        );
        let pixels = RgbaImage::from_pixel(2, 2, image::Rgba([20, 60, 180, 255]));
        let preview = image_edit::encode(
            &pixels,
            OutputFormat::Png,
            90,
            &ImageEditSettings::default(),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        editor.content = Content::Image(Some(Arc::new(pixels)));
        editor.preview = Some(Arc::new(PreviewData::Image(preview)));
        editor.preview_revision = editor.revision;
        editor.dirty = false;
        assert!(editor.can_save());
        editor.changed();
        assert!(!editor.can_save());
    }
}
