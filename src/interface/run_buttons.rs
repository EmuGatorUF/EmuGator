use crate::assembler::{self, AssembledProgram, AssemblerError, Section};
use crate::emulator::{
    self, EmulatorState,
    uart::{LineStatusRegisterBitMask, Uart},
};

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
    emulator_state: Signal<EmulatorState>,
    uart_module: Signal<Uart>,
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
                        Ok(assembled) => {
                            info!("Assembly succeeded.");
                            let mut new_state = EmulatorState::default();
                            let start_addr = assembled.get_section_start(Section::Text);
                            new_state.pipeline.IF_pc = start_addr;
                            emulator_state.set(new_state);
                            *uart_module.write() = Uart::default();
                            let mut assembled = assembled;
                            assembled.data_memory.insert(uart_module.read().rx_buffer_address, 0);
                            assembled.data_memory.insert(uart_module.read().tx_buffer_address, 0);
                            assembled
                                .data_memory
                                .insert(
                                    uart_module.read().lsr_address,
                                    LineStatusRegisterBitMask::TransmitReady as u8
                                        | LineStatusRegisterBitMask::ReceiveReady as u8,
                                );
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
                            let (new_state, new_uart) = emulator::clock(
                                emulator_state.read().deref(),
                                &mut *program,
                                uart_module.read().deref(),
                            );
                            *(emulator_state.write()) = new_state;
                            *(uart_module.write()) = new_uart;
                        }
                    },
                    "Next Clock"
                }
                button {
                    class: "bg-purple-500 hover:bg-purple-600 text-s text-white font-bold py-1 px-2 rounded",
                    onclick: move |_| {
                        if let Some(mut program) = assembled_program.as_mut() {
                            let (new_state, new_uart) = emulator::clock_until_break(
                                emulator_state.read().deref(),
                                &mut *program,
                                breakpoints.read().deref(),
                                uart_module.read().deref(),
                            );
                            *(emulator_state.write()) = new_state;
                            *(uart_module.write()) = new_uart;
                        }
                    },
                    "Run to Break"
                }
            }
        }
    }
}
