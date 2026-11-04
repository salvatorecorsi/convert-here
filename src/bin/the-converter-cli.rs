#[path = "../app.rs"]
mod app;
#[path = "../command.rs"]
mod command;
#[path = "../file_icons.rs"]
pub mod file_icons;
#[path = "../pdf_merge_app.rs"]
pub mod pdf_merge_app;
#[path = "../pdf_merge_ui.rs"]
pub mod pdf_merge_ui;
#[path = "../theme.rs"]
pub mod theme;
fn main() {
    command::main();
}
