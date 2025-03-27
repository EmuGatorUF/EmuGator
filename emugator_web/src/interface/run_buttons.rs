use emugator_core::assembler::{self, AssembledProgram, AssemblerError, Section};
use emugator_core::emulator::AnyEmulatorState;
use emugator_core::emulator::EmulatorState;
use emugator_core::emulator::cve2::CVE2Pipeline;

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use std::collections::BTreeSet;
use std::ops::Deref;

#[component]
#[allow(non_snake_case)]
pub fn RunButtons(
    source: Signal<String>,
    assembled_program: Signal<Option<AssembledProgram>>,
    assembler_errors: Signal<Vec<AssemblerError>>,
    emulator_state: Signal<AnyEmulatorState>,
    breakpoints: ReadOnlySignal<BTreeSet<usize>>,
) -> Element {
    rsx! {
        // bottom margin
        div { class: "flex content-center gap-2 justify-center mb-2",
            button {
                class: "bg-green-500 hover:bg-green-600 text-s text-white font-bold py-1 px-2 rounded",
                onclick: move |_| {
                    info!("Assembler clicked");
                    match assembler::assemble(&source.read()) {
                        Ok(mut assembled) => {
                            info!("Assembly succeeded.");
                            let mut new_state = EmulatorState::<CVE2Pipeline>::default();
                            let start_addr = assembled.get_section_start(Section::Text);
                            new_state.pipeline.IF_pc = start_addr;
                            assembled.init_uart_data_memory(&new_state.uart);
                            emulator_state.set(AnyEmulatorState::CVE2(new_state));
                            assembled_program.set(Some(assembled));
                            assembler_errors.set(Vec::new());
                        }
                        Err(errors) => {
                            info!("Assembly failed.");
                            assembled_program.set(None);
                            assembler_errors.set(errors);
                        }
                    }
                },
                "Assemble"
            }
            if assembled_program.read().is_some() {
                button {
                    class: "bg-purple-500 hover:bg-purple-600 text-s text-white font-bold py-1 px-2 rounded",
                    onclick: move |_| {
                        if let Some(mut program) = assembled_program.as_mut() {
                            let new_state = emulator_state.read().clock(&mut program);
                            emulator_state.set(new_state);
                        }
                    },
                    "Next Clock"
                }
                button {
                    class: "bg-purple-500 hover:bg-purple-600 text-s text-white font-bold py-1 px-2 rounded",
                    onclick: move |_| {
                        if let Some(mut program) = assembled_program.as_mut() {
                            let new_state = emulator_state
                                .read()
                                .clock_until_break(&mut program, breakpoints.read().deref(), 10_000);
                            emulator_state.set(new_state);
                        }
                    },
                    "Run to Break"
                }
            }
        }
    }
}
