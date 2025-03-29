use emugator_core::assembler::{self, AssembledProgram, AssemblerError};
use emugator_core::emulator::AnyEmulatorState;

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use std::collections::BTreeSet;
use std::ops::Deref;

#[component]
#[allow(non_snake_case)]
pub fn Navbar(
    source: Signal<String>,
    assembled_program: Signal<Option<AssembledProgram>>,
    assembler_errors: Signal<Vec<AssemblerError>>,
    emulator_state: Signal<AnyEmulatorState>,
    breakpoints: ReadOnlySignal<BTreeSet<usize>>,
    show_five_stage: Signal<bool>,
) -> Element {
    let is_assembled = assembled_program.read().is_some();
    let error_count = assembler_errors.read().len();

    rsx! {
        nav { class: "bg-gray-900 text-white w-full py-2 px-4 flex items-center justify-between shadow-md border-b-2 border-gray-950",
            div { class: "flex items-center",
                span { class: "text-xl font-semibold text-blue-400 mr-4", "Emugator" }
                div { class: "flex space-x-2",
                    button {
                        class: "bg-green-600 hover:bg-green-700 text-white font-medium py-1.5 px-3 rounded transition duration-150 ease-in-out flex items-center",
                        onclick: move |_| {
                            info!("Assembler clicked");
                            match assembler::assemble(&source.read()) {
                                Ok(assembled) => {
                                    info!("Assembly succeeded.");
                                    emulator_state.set(AnyEmulatorState::new_cve2(&assembled));
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
                        svg {
                            class: "w-4 h-4 mr-1 fill-current",
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 20 20",
                            path {
                                d: "M10 1.6a8.4 8.4 0 100 16.8 8.4 8.4 0 000-16.8zm4.3 9.6l-4 2.3a.8.8 0 01-1.2-.7V7.2a.8.8 0 011.2-.7l4 2.3a.8.8 0 010 1.4z",
                                fill_rule: "evenodd",
                                clip_rule: "evenodd",
                            }
                        }
                        "Assemble"
                    }
                    button {
                        class: "bg-yellow-600 hover:bg-yellow-700 text-white font-medium py-1.5 px-3 rounded transition duration-150 ease-in-out flex items-center",
                        onclick: move |_| {
                            let current = *show_five_stage.read();
                            show_five_stage.set(!current);
                        },
                        svg {
                            class: "w-4 h-4 mr-1 fill-current",
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 20 20",
                            path {
                                d: "M4 2h12a2 2 0 012 2v12a2 2 0 01-2 2H4a2 2 0 01-2-2V4a2 2 0 012-2zm0 2v12h12V4H4z",
                                fill_rule: "evenodd",
                                clip_rule: "evenodd",
                            }
                        }
                        if *show_five_stage.read() {
                            "Showing Five Stage"
                        } else {
                            "Showing CVE2"
                        }
                    }
                    if is_assembled {
                        button {
                            class: "bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-1.5 px-3 rounded transition duration-150 ease-in-out flex items-center",
                            onclick: move |_| {
                                if let Some(mut program) = assembled_program.as_mut() {
                                    let new_state = emulator_state.read().clock(&mut program);
                                    emulator_state.set(new_state);
                                }
                            },
                            svg {
                                class: "w-4 h-4 mr-1 fill-current",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 20 20",
                                path {
                                    d: "M13.7 10L8.3 5.5a.8.8 0 00-1.3.6v9a.8.8 0 001.3.6L13.7 10z",
                                    fill_rule: "evenodd",
                                    clip_rule: "evenodd",
                                }
                            }
                            "Step"
                        }
                        button {
                            class: "bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-1.5 px-3 rounded transition duration-150 ease-in-out flex items-center",
                            onclick: move |_| {
                                if let Some(mut program) = assembled_program.as_mut() {
                                    let new_state = emulator_state
                                        .read()
                                        .clock_until_break(&mut program, breakpoints.read().deref(), 10_000);
                                    emulator_state.set(new_state);
                                }
                            },
                            svg {
                                class: "w-4 h-4 mr-1 fill-current",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 20 20",
                                path {
                                    d: "M10 3.5l8 6.5-8 6.5V3.5zM2 4h5v12H2V4z",
                                    fill_rule: "evenodd",
                                    clip_rule: "evenodd",
                                }
                            }
                            "Run to Break"
                        }
                    } else {
                        button {
                            class: "bg-gray-600 cursor-not-allowed text-gray-300 font-medium py-1.5 px-3 rounded opacity-70 flex items-center",
                            disabled: true,
                            svg {
                                class: "w-4 h-4 mr-1 fill-current",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 20 20",
                                path {
                                    d: "M13.7 10L8.3 5.5a.8.8 0 00-1.3.6v9a.8.8 0 001.3.6L13.7 10z",
                                    fill_rule: "evenodd",
                                    clip_rule: "evenodd",
                                }
                            }
                            "Step"
                        }
                        button {
                            class: "bg-gray-600 cursor-not-allowed text-gray-300 font-medium py-1.5 px-3 rounded opacity-70 flex items-center",
                            disabled: true,
                            svg {
                                class: "w-4 h-4 mr-1 fill-current",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 20 20",
                                path {
                                    d: "M10 3.5l8 6.5-8 6.5V3.5zM2 4h5v12H2V4z",
                                    fill_rule: "evenodd",
                                    clip_rule: "evenodd",
                                }
                            }
                            "Run to Break"
                        }
                    }
                }
            }
            div { class: "flex items-center",
                if is_assembled {
                    span { class: "text-white text-sm font-medium mr-2 bg-green-600 rounded px-2 py-0.5 flex items-center",
                        svg {
                            class: "w-4 h-4 mr-1 fill-current",
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 20 20",
                            path {
                                d: "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z",
                                fill_rule: "evenodd",
                                clip_rule: "evenodd",
                            }
                        }
                        "Program Assembled"
                    }
                } else if error_count > 0 {
                    span { class: "text-white text-sm font-medium mr-2 bg-red-600 rounded px-2 py-0.5 flex items-center",
                        svg {
                            class: "w-4 h-4 mr-1 fill-current",
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 20 20",
                            path {
                                d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z",
                                fill_rule: "evenodd",
                                clip_rule: "evenodd",
                            }
                        }
                        "Errors: {error_count}"
                    }
                } else {
                    span { class: "text-white text-sm font-medium mr-2 bg-gray-700 rounded px-2 py-0.5",
                        "Ready"
                    }
                }
                button {
                    class: "ml-2 bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm font-medium py-1 px-2 rounded transition duration-150 ease-in-out",
                    onclick: move |_| {},
                    svg {
                        class: "w-4 h-4 fill-current",
                        xmlns: "http://www.w3.org/2000/svg",
                        view_box: "0 0 20 20",
                        path {
                            d: "M10 12a2 2 0 100-4 2 2 0 000 4z",
                            fill_rule: "evenodd",
                            clip_rule: "evenodd",
                        }
                        path {
                            d: "M10 3a7 7 0 100 14 7 7 0 000-14zm-9 7a9 9 0 1118 0 9 9 0 01-18 0z",
                            fill_rule: "evenodd",
                            clip_rule: "evenodd",
                        }
                    }
                }
            }
        }
    }
}
