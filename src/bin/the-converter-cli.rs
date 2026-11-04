#[path = "../app.rs"]
mod app;
#[path = "../command.rs"]
mod command;
#[path = "../file_icons.rs"]
pub mod file_icons;
#[path = "../theme.rs"]
pub mod theme;
fn main() {
    command::main();
}
