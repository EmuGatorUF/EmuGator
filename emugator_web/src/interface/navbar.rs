use emugator_core::assembler::{self, AssembledProgram, AssemblerError};
use emugator_core::emulator::{AnyEmulatorState, EmulatorOption};

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use std::collections::BTreeSet;
use std::ops::Deref;

#[component]
#[allow(non_snake_case)]
pub fn Navbar(
    source: ReadOnlySignal<String>,
    assembled_program: Signal<Option<AssembledProgram>>,
    assembler_errors: Signal<Vec<AssemblerError>>,
    emulator_state: Signal<Option<AnyEmulatorState>>,
    selected_emulator: Signal<EmulatorOption>,
    breakpoints: ReadOnlySignal<BTreeSet<usize>>,
    minimize_console: Signal<bool>,
) -> Element {
    let is_started = emulator_state.read().is_some();
    let is_assembled = assembled_program.read().is_some();
    let error_count = assembler_errors.read().len();

    rsx! {
        nav { class: "bg-gray-900 text-white w-full py-2 px-4 flex items-center justify-between shadow-md border-b-2 border-gray-950",
            div { class: "flex items-center",
                span { class: "text-xl font-semibold text-blue-400 mr-4", "EmuGator" }
                div { class: "flex space-x-2",
                    button {
                        class: "bg-green-600 hover:bg-green-700 text-white font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex items-center",
                        onclick: move |_| {
                            info!("Start clicked");
                            match assembler::assemble(&source.read()) {
                                Ok(assembled) => {
                                    info!("Final assembly succeeded.");
                                    let new_state = AnyEmulatorState::new_of_type(
                                        &assembled,
                                        *selected_emulator.read(),
                                    );
                                    emulator_state.set(Some(new_state));
                                    assembled_program.set(Some(assembled));
                                    assembler_errors.set(Vec::new());
                                    minimize_console.set(false);
                                }
                                Err(errors) => {
                                    info!("Final assembly failed.");
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
                        "Start"
                    }

                    button {
                        class: format!(
                            "{} font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex items-center",
                            if is_started {
                                "bg-indigo-600 hover:bg-indigo-700 text-white"
                            } else {
                                "bg-gray-600 text-gray-300 cursor-not-allowed"
                            },
                        ),
                        disabled: !is_started,
                        onclick: move |_| {
                            if let Some(mut program) = assembled_program.as_mut() {
                                let new_state = emulator_state
                                    .read()
                                    .as_ref()
                                    .map(|e| e.clock(&mut program));
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
                        class: format!(
                            "{} text-white font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex items-center",
                            if is_started {
                                "bg-indigo-600 hover:bg-indigo-700 text-white"
                            } else {
                                "bg-gray-600 text-gray-300 cursor-not-allowed"
                            },
                        ),
                        disabled: !is_started,
                        onclick: move |_| {
                            if let Some(mut program) = assembled_program.as_mut() {
                                let new_state = emulator_state
                                    .read()
                                    .as_ref()
                                    .map(|e| {
                                        e.clock_until_break(&mut program, breakpoints.read().deref(), 10_000)
                                    });
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
                }
            }
            div { class: "flex items-stretch space-x-2",
                span {
                    class: format!(
                        "text-white text-sm font-medium {} rounded py-1 px-2 flex items-center",
                        if is_assembled {
                            "bg-green-600"
                        } else if error_count > 0 {
                            "bg-red-600"
                        } else {
                            "bg-gray-700"
                        },
                    ),
                    if is_assembled {
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
                        if is_started {
                            "Program Running"
                        } else {
                            "Program Assembled"
                        }
                    } else if error_count > 0 {
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
                    } else {
                        "Ready"
                    }
                }
                button {
                    class: "bg-yellow-600 hover:bg-yellow-700 text-white font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex items-center",
                    onclick: move |_| {
                        let new_selection = selected_emulator.read().other();
                        selected_emulator.set(new_selection);
                        emulator_state.set(None);
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
                    "{selected_emulator.read().display_string()}"
                }
                button {
                    class: "bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm font-medium py-1 px-2 rounded transition duration-150 ease-in-out",
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
