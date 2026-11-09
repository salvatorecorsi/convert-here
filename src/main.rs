#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
mod command;
pub mod file_icons;
pub mod gui_history;
pub mod image_editor;
pub mod media_options_ui;
pub mod pdf_merge_app;
pub mod pdf_merge_ui;
pub mod profile_manager;
pub mod settings_ui;
pub mod theme;
fn main() {
    command::main();
}
