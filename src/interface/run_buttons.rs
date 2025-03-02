use crate::assembler::{self, AssembledProgram, AssemblerError, Section};
use crate::emulator::{self, EmulatorState};
use crate::uart::{trigger_uart, Uart};

use dioxus::prelude::*;
use std::ops::Deref;

#[component]
#[allow(non_snake_case)]
pub fn RunButtons(
    source: Signal<String>,
    assembled_program: Signal<Option<AssembledProgram>>,
    assembler_errors: Signal<Vec<AssemblerError>>,
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
                            new_state.pipeline.IF_pc = start_addr;
                            emulator_state.set(new_state);

                            // Setup UART with data memory addresses
                            // TOD: Probably a better way to do this
                            let new_uart = Uart::default();
                            let mut assembled = assembled;
                            assembled.data_memory.insert(new_uart.rx_buffer_address, 0);
                            assembled.data_memory.insert(new_uart.tx_buffer_address, 0);
                            assembled.data_memory.insert(new_uart.lsr_address, 0);

                            assembled_program.set(Some(assembled));
                            assembler_errors.set(Vec::new());
                        }
                        Err(errors) => {
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
