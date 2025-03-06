#[allow(clippy::upper_case_acronyms)]
mod assembler;
mod code_editor;
#[allow(clippy::upper_case_acronyms)]
mod emulator;
mod interface;
#[allow(clippy::upper_case_acronyms)]
mod isa;
mod utils;

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
