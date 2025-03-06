mod code_editor;
mod interface;

use dioxus::prelude::*;
use dioxus_logger::tracing::{Level, info};
use interface::App;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    code_editor::register_riscv_language();
    launch(App);
}
