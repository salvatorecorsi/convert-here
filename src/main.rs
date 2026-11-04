#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
mod command;
pub mod file_icons;
pub mod pdf_merge_app;
pub mod pdf_merge_ui;
pub mod theme;
fn main() {
    command::main();
}
