use crate::assembler::{self, AssembledProgram, Section};
use crate::emulator::{self, EmulatorState};
use crate::uart::{Uart, trigger_uart};

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use std::ops::Deref;

#[component]
#[allow(non_snake_case)]
pub fn RunButtons(
    source: Signal<String>,
    assembled_program: Signal<Option<AssembledProgram>>,
    emulator_state: Signal<EmulatorState>,
    uart_module: Signal<Uart>,
) -> Element {
    rsx! {
        // bottom margin
        div { class: "flex content-center gap-2 justify-center mb-2",
            button {
                class: "bg-green-500 hover:bg-green-600 text-s text-white font-bold py-1 px-2 rounded",
                onclick: move |_| {
                    match assembler::assemble(&source.read()) {
                        Ok(assembled) => {
                            let mut new_state = EmulatorState::default();
                            let start_addr = assembled.get_section_start(Section::Text);
                            new_state.pipeline.datapath.instr_addr_o = start_addr;
                            emulator_state.set(new_state);
                            assembled_program.set(Some(assembled));
                        }
                        Err(errors) => {
                            for error in errors {
                                info!(
                                    "Assembly Error on line {}, column {}: {}", error.line_number,
                                    error.column, error.error_message
                                );
                            }
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
                            let new_state = emulator::clock(
                                emulator_state.read().deref(),
                                &mut *program,
                            );
                            *(emulator_state.write()) = new_state;

                            let new_uart = trigger_uart(
                                uart_module.read().deref().clone(),
                                &mut program.data_memory,
                            );
                            *(uart_module.write()) = new_uart;
                        }
                    },
                    "Next Clock"
                }
            }
        }
    }
}
