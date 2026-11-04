//! Page ordering only: no file reads, conversion work, or persistent preferences.
use crate::theme::{self, FluentIcon};
use eframe::egui::{self, Rect, Sense};
use std::sync::atomic::{AtomicU64, Ordering};
use the_converter::pdf_merge::{MergePageRef, MergeSource};

const ROW_HEIGHT: f32 = 64.0;

#[derive(Clone, Copy)]
struct DraggedPage {
    editor: u64,
    page: MergePageRef,
}

struct SourceLabel {
    name: String,
    full_path: String,
    page_count: u32,
}

pub(crate) struct PageOrderEditor {
    id: u64,
    sources: Vec<SourceLabel>,
    pages: Vec<MergePageRef>,
    original: Vec<MergePageRef>,
    selected: Option<MergePageRef>,
    row_rects: Vec<(usize, Rect)>,
}

impl PageOrderEditor {
    pub(crate) fn new(sources: &[MergeSource], pages: Vec<MergePageRef>) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            sources: sources
                .iter()
                .map(|source| SourceLabel {
                    name: source
                        .input
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    full_path: the_converter::platform::display_path(&source.input),
                    page_count: source.page_count,
                })
                .collect(),
            selected: pages.first().copied(),
            original: pages.clone(),
            pages,
            row_rects: Vec::new(),
        }
    }

    pub(crate) fn ordered_pages(&self) -> &[MergePageRef] {
        &self.pages
    }

    /// Move to a gap in the pre-move list, preserving every page exactly once.
    fn move_to_gap(&mut self, from: usize, gap: usize) -> bool {
        if from >= self.pages.len() || gap > self.pages.len() {
            return false;
        }
        let destination = gap.saturating_sub(usize::from(gap > from));
        if destination == from {
            return false;
        }
        let page = self.pages.remove(from);
        self.pages.insert(destination, page);
        self.selected = Some(page);
        true
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, enabled: bool) -> bool {
        let colors = theme::palette(ui.ctx());
        // Native event loops can deliver the press and first movement in one
        // frame. egui's hit test then uses the final pointer position, so retain
        // the actual primary-press position for choosing the source handle.
        let pressed_origin = ui.input(|input| {
            if input.pointer.primary_pressed()
                && input.pointer.button_down(egui::PointerButton::Primary)
            {
                input.raw.events.iter().find_map(|event| match event {
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        ..
                    } => Some(*pos),
                    _ => None,
                })
            } else {
                None
            }
        });
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            ui.label(theme::heading(
                format!("Ordine delle pagine · {}", self.pages.len()),
                theme::SUBTITLE,
            ));
            if ui
                .add_enabled(
                    enabled && self.pages != self.original,
                    egui::Button::new("Ripristina ordine"),
                )
                .clicked()
            {
                self.pages.clone_from(&self.original);
                changed = true;
            }
        });
        ui.label(
            egui::RichText::new(
                "Trascina la maniglia per spostare una pagina. Puoi usare anche le frecce o Alt+↑ / Alt+↓ sulla pagina selezionata.",
            )
            .size(theme::CAPTION)
            .color(colors.secondary),
        );
        ui.add_space(8.0);
        let mut movement = None;
        if enabled && let Some(selected) = self.selected {
            let index = self.pages.iter().position(|page| *page == selected);
            if let Some(index) = index {
                if ui.input_mut(|input| input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowUp))
                    && index > 0
                {
                    movement = Some((index, index - 1));
                }
                if ui.input_mut(|input| {
                    input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowDown)
                }) && index + 1 < self.pages.len()
                {
                    movement = Some((index, index + 2));
                }
            }
        }
        self.row_rects.clear();
        let editor_id = self.id;
        let total_pages = self.pages.len();
        let mut released = None;
        let scroll = egui::ScrollArea::vertical()
            .id_salt(("pdf_merge_pages", editor_id))
            .auto_shrink([false, false])
            .show_rows(ui, ROW_HEIGHT, total_pages, |ui, range| {
                if enabled
                    && egui::DragAndDrop::payload::<DraggedPage>(ui.ctx())
                        .is_some_and(|page| page.editor == editor_id)
                    && let Some(pointer) = ui.ctx().pointer_interact_pos()
                {
                    let clip = ui.clip_rect();
                    if clip.contains(pointer) {
                        let edge = 36.0;
                        let speed = if pointer.y < clip.top() + edge {
                            (clip.top() + edge - pointer.y) * 12.0
                        } else if pointer.y > clip.bottom() - edge {
                            -(pointer.y - (clip.bottom() - edge)) * 12.0
                        } else {
                            0.0
                        };
                        if speed != 0.0 {
                            let elapsed = ui.input(|input| input.stable_dt.min(0.05));
                            ui.scroll_with_delta(egui::vec2(0.0, speed * elapsed));
                            ui.ctx()
                                .request_repaint_after(std::time::Duration::from_millis(16));
                        }
                    }
                }
                for index in range {
                    let page = self.pages[index];
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), ROW_HEIGHT),
                        Sense::click(),
                    );
                    self.row_rects.push((index, rect));
                    let selected = self.selected == Some(page);
                    ui.painter().rect(
                        rect,
                        theme::CARD_RADIUS,
                        colors.card,
                        egui::Stroke::new(
                            1.0_f32,
                            if selected {
                                colors.accent
                            } else {
                                colors.stroke
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    let mut row = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt(("pdf_merge_page", editor_id, page.source, page.page))
                            .max_rect(rect.shrink2(egui::vec2(10.0, 8.0)))
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    row.add_enabled_ui(enabled, |row| {
                        let (handle, drag) =
                            row.allocate_exact_size(egui::vec2(24.0, 48.0), Sense::drag());
                        let origin_matches = pressed_origin.is_none_or(|origin| {
                            row.is_enabled()
                                && handle.intersect(row.clip_rect()).contains(origin)
                                && row.ctx().layer_id_at(origin) == Some(row.layer_id())
                        });
                        if drag.drag_started() && origin_matches {
                            self.selected = Some(page);
                        }
                        for x in [-3.0, 3.0] {
                            for y in [-6.0, 0.0, 6.0] {
                                row.painter().circle_filled(
                                    handle.center() + egui::vec2(x, y),
                                    1.3,
                                    colors.secondary,
                                );
                            }
                        }
                        let drag = drag.on_hover_text("Trascina per spostare questa pagina");
                        if origin_matches {
                            let payload = DraggedPage {
                                editor: editor_id,
                                page,
                            };
                            drag.dnd_set_drag_payload(payload);
                            if pressed_origin.is_some()
                                && !egui::DragAndDrop::has_any_payload(row.ctx())
                            {
                                egui::DragAndDrop::set_payload(row.ctx(), payload);
                                self.selected = Some(page);
                            }
                        }
                        row.allocate_ui_with_layout(
                            egui::vec2(68.0, 40.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.set_width(68.0);
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.spacing_mut().button_padding = egui::vec2(4.0, 4.0);
                                for (icon, possible, gap, tooltip) in [
                                    (
                                        FluentIcon::ChevronUp,
                                        index > 0,
                                        index.saturating_sub(1),
                                        "Sposta prima",
                                    ),
                                    (
                                        FluentIcon::ChevronDown,
                                        index + 1 < total_pages,
                                        index + 2,
                                        "Sposta dopo",
                                    ),
                                ] {
                                    if ui
                                        .add_enabled(
                                            possible,
                                            egui::Button::new(theme::icon(icon, 16.0))
                                                .frame(false)
                                                .min_size(egui::vec2(28.0, 32.0)),
                                        )
                                        .on_hover_text(tooltip)
                                        .clicked()
                                    {
                                        movement = Some((index, gap));
                                    }
                                }
                            },
                        );
                        let text_width = row.available_width().max(40.0);
                        row.allocate_ui_with_layout(
                            egui::vec2(text_width, 40.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(text_width);
                                if let Some(source) = self.sources.get(page.source) {
                                    ui.add(
                                        egui::Label::new(theme::heading(&source.name, theme::BODY))
                                            .truncate(),
                                    )
                                    .on_hover_text(&source.full_path);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Pagina {} di {} nell’originale",
                                            page.page, source.page_count
                                        ))
                                        .size(theme::CAPTION)
                                        .color(colors.secondary),
                                    );
                                }
                            },
                        );
                    });
                    if response.clicked() && enabled {
                        self.selected = Some(page);
                    }
                    if enabled
                        && let Some(payload) = response.dnd_hover_payload::<DraggedPage>()
                        && payload.editor == editor_id
                    {
                        let after = ui
                            .ctx()
                            .pointer_interact_pos()
                            .is_some_and(|pointer| pointer.y >= rect.center().y);
                        let y = if after { rect.bottom() } else { rect.top() };
                        ui.painter().line_segment(
                            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                            egui::Stroke::new(2.0_f32, colors.accent),
                        );
                        if let Some(payload) = response.dnd_release_payload::<DraggedPage>() {
                            released = Some((payload.page, index + usize::from(after)));
                        }
                    }
                }
            });
        if enabled
            && let Some(pointer) = ui.ctx().pointer_interact_pos()
            && scroll.inner_rect.contains(pointer)
            && self
                .row_rects
                .last()
                .is_some_and(|(index, rect)| *index + 1 == total_pages && pointer.y > rect.bottom())
            && let Some(payload) = egui::DragAndDrop::payload::<DraggedPage>(ui.ctx())
            && payload.editor == editor_id
            && ui.input(|input| input.pointer.any_released())
        {
            egui::DragAndDrop::clear_payload(ui.ctx());
            released = Some((payload.page, total_pages));
        }
        if let Some((page, gap)) = released
            && let Some(index) = self.pages.iter().position(|candidate| *candidate == page)
        {
            movement = Some((index, gap));
        }
        if let Some((index, gap)) = movement {
            changed |= self.move_to_gap(index, gap);
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn editor(count: u32) -> PageOrderEditor {
        PageOrderEditor::new(
            &[MergeSource {
                input: PathBuf::from("documento.pdf"),
                page_count: count,
            }],
            (1..=count)
                .map(|page| MergePageRef { source: 0, page })
                .collect(),
        )
    }

    fn frame(ctx: &egui::Context, editor: &mut PageOrderEditor, events: Vec<egui::Event>) {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600.0, 450.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| editor.show(ui, true));
            },
        );
    }

    #[test]
    fn moves_are_insertions_and_preserve_the_exact_page_permutation() {
        let mut editor = editor(4);
        assert!(editor.move_to_gap(0, 4));
        assert_eq!(
            editor
                .pages
                .iter()
                .map(|page| page.page)
                .collect::<Vec<_>>(),
            [2, 3, 4, 1]
        );
        assert!(editor.move_to_gap(3, 0));
        assert_eq!(editor.pages, editor.original);
        assert!(!editor.move_to_gap(1, 1));
        assert!(!editor.move_to_gap(1, 2));
        assert!(!editor.move_to_gap(9, 0));
        assert!(!editor.move_to_gap(0, 9));
        assert!(editor.move_to_gap(3, 1));
        let mut pages = editor.pages.clone();
        pages.sort_by_key(|page| (page.source, page.page));
        assert_eq!(pages, editor.original);
    }

    #[test]
    fn identical_page_numbers_from_different_sources_remain_distinct() {
        let sources = [
            MergeSource {
                input: PathBuf::from("primo.pdf"),
                page_count: 2,
            },
            MergeSource {
                input: PathBuf::from("secondo.docx"),
                page_count: 2,
            },
        ];
        let pages = vec![
            MergePageRef { source: 0, page: 1 },
            MergePageRef { source: 0, page: 2 },
            MergePageRef { source: 1, page: 1 },
            MergePageRef { source: 1, page: 2 },
        ];
        let mut editor = PageOrderEditor::new(&sources, pages.clone());
        assert!(editor.move_to_gap(2, 0));
        assert_eq!(
            editor.ordered_pages(),
            &[pages[2], pages[0], pages[1], pages[3]]
        );
    }

    #[test]
    fn real_headless_pointer_drag_reorders_a_page_at_the_drop_gap() {
        let ctx = egui::Context::default();
        theme::configure(&ctx);
        let mut editor = editor(3);
        frame(&ctx, &mut editor, Vec::new());
        let from = editor.row_rects[0].1.min + egui::vec2(22.0, 32.0);
        let to = editor.row_rects[2].1.left_bottom() + egui::vec2(120.0, -6.0);
        frame(
            &ctx,
            &mut editor,
            vec![
                egui::Event::PointerMoved(from),
                egui::Event::PointerButton {
                    pos: from,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame(&ctx, &mut editor, vec![egui::Event::PointerMoved(to)]);
        frame(&ctx, &mut editor, vec![egui::Event::PointerMoved(to)]);
        frame(
            &ctx,
            &mut editor,
            vec![egui::Event::PointerButton {
                pos: to,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(
            editor
                .pages
                .iter()
                .map(|page| page.page)
                .collect::<Vec<_>>(),
            [2, 3, 1]
        );
    }

    #[test]
    fn coalesced_native_press_and_first_movement_use_the_original_handle() {
        for (from_index, to_index, target_x, after, expected) in [
            (0, 2, 520.0, true, [2, 3, 1, 4, 5]),
            (4, 1, 120.0, false, [1, 5, 2, 3, 4]),
            // The current pointer can already be over another drag handle.
            (0, 2, 42.0, true, [2, 3, 1, 4, 5]),
        ] {
            let ctx = egui::Context::default();
            theme::configure(&ctx);
            let mut editor = editor(5);
            frame(&ctx, &mut editor, Vec::new());
            let from = editor.row_rects[from_index].1.min + egui::vec2(22.0, 32.0);
            let target = editor.row_rects[to_index].1;
            let to = egui::pos2(
                target_x,
                target.center().y + if after { 10.0 } else { -10.0 },
            );
            frame(&ctx, &mut editor, vec![egui::Event::PointerMoved(from)]);
            frame(
                &ctx,
                &mut editor,
                vec![
                    egui::Event::PointerButton {
                        pos: from,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                    egui::Event::PointerMoved(to),
                ],
            );
            frame(&ctx, &mut editor, Vec::new());
            frame(
                &ctx,
                &mut editor,
                vec![egui::Event::PointerButton {
                    pos: to,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            assert_eq!(
                editor
                    .pages
                    .iter()
                    .map(|page| page.page)
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }

    #[test]
    fn a_coalesced_press_outside_the_handle_does_not_drag_the_target_page() {
        let ctx = egui::Context::default();
        theme::configure(&ctx);
        let mut editor = editor(3);
        frame(&ctx, &mut editor, Vec::new());
        let from = editor.row_rects[0].1.center();
        let to = editor.row_rects[2].1.min + egui::vec2(22.0, 32.0);
        frame(&ctx, &mut editor, vec![egui::Event::PointerMoved(from)]);
        frame(
            &ctx,
            &mut editor,
            vec![
                egui::Event::PointerButton {
                    pos: from,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerMoved(to),
            ],
        );
        frame(&ctx, &mut editor, Vec::new());
        frame(
            &ctx,
            &mut editor,
            vec![egui::Event::PointerButton {
                pos: to,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(editor.pages, editor.original);
        assert!(!egui::DragAndDrop::has_any_payload(&ctx));
    }

    #[test]
    fn large_documents_only_layout_visible_pages() {
        let ctx = egui::Context::default();
        theme::configure(&ctx);
        let mut editor = editor(10_000);
        frame(&ctx, &mut editor, Vec::new());
        assert!(!editor.row_rects.is_empty());
        assert!(editor.row_rects.len() < 12);
        assert_eq!(editor.ordered_pages().len(), 10_000);
    }

    #[test]
    fn alt_arrow_moves_the_selected_page_without_losing_its_identity() {
        let ctx = egui::Context::default();
        theme::configure(&ctx);
        let mut editor = editor(3);
        frame(&ctx, &mut editor, Vec::new());
        frame(
            &ctx,
            &mut editor,
            vec![egui::Event::Key {
                key: egui::Key::ArrowDown,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::ALT,
            }],
        );
        assert_eq!(
            editor
                .pages
                .iter()
                .map(|page| page.page)
                .collect::<Vec<_>>(),
            [2, 1, 3]
        );
        assert_eq!(editor.selected, Some(MergePageRef { source: 0, page: 1 }));
    }
}
