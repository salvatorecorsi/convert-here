#[path = "../app.rs"]
mod app;
#[path = "../command.rs"]
mod command;
#[path = "../file_icons.rs"]
pub mod file_icons;
#[path = "../gui_history.rs"]
pub mod gui_history;
#[path = "../image_editor.rs"]
pub mod image_editor;
#[path = "../media_options_ui.rs"]
pub mod media_options_ui;
#[path = "../pdf_merge_app.rs"]
pub mod pdf_merge_app;
#[path = "../pdf_merge_ui.rs"]
pub mod pdf_merge_ui;
#[path = "../profile_manager.rs"]
pub mod profile_manager;
#[path = "../settings_ui.rs"]
pub mod settings_ui;
#[path = "../theme.rs"]
pub mod theme;
fn main() {
    command::main();
}
