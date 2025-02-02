use crate::assembler::{self, AssembledProgram, Section};
use crate::emulator::{self, EmulatorState};

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use std::ops::Deref;
use std::collections::BTreeSet;

#[component]
#[allow(non_snake_case)]
pub fn RunButtons(
    source: Signal<String>,
    assembled_program: Signal<Option<AssembledProgram>>,
    emulator_state: Signal<EmulatorState>,
    breakpoints: Signal<BTreeSet<usize>>,
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
                        Err(e) => {
                            info!("Error assembling program: {}", e);
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
                        }
                    },
                    "Next Clock"
                }
                button {
                    class: "bg-purple-500 hover:bg-purple-600 text-s text-white font-bold py-1 px-2 rounded",
                    onclick: move |_| {
                        if let Some(mut program) = assembled_program.as_mut() {
                            let new_state = emulator::clock(
                                emulator_state.read().deref(),
                                &mut *program,
                            );
                            *(emulator_state.write()) = new_state;

                            // TODO: Change to use function in interface/mod.rs or turn into function?
                            let mut reached_breakpoint = false;
                            if let Some(line) = (*program).source_map.get_by_left(&emulator_state.read().pipeline.ID_pc).copied(){
                                reached_breakpoint = breakpoints.read().contains(&line);
                            }

                            // while emulator hasn't hit an EBREAK, end of program, or a breakpoint
                            while !emulator_state.read().deref().pipeline.datapath.debug_req_i && !emulator_state.read().deref().pipeline.datapath.instr_err_i && !reached_breakpoint {
                                let new_state = emulator::clock(
                                    emulator_state.read().deref(),
                                    &mut *program,
                                );
                                *(emulator_state.write()) = new_state;

                                if let Some(line) = (*program).source_map.get_by_left(&emulator_state.read().pipeline.ID_pc).copied(){
                                    reached_breakpoint = breakpoints.read().contains(&line);
                                }
                            }
                        }
                    },
                    "Run to Break"
                }
            }
        }
    }
}
