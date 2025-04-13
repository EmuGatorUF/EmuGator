use emugator_core::assembler::{self, AssembledProgram, AssemblerError};
use emugator_core::emulator::{AnyEmulatorState, EmulatorOption};

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use std::collections::BTreeSet;
use std::ops::Deref;

use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdCircleArrowRight, LdCircleCheck, LdCircleX, LdInfo, LdPlay,
};
use dioxus_free_icons::icons::ld_icons::{LdClock3, LdClock6, LdClock9, LdClock12};

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
    help_panel_displayed: Signal<bool>,
) -> Element {
    let is_started = emulator_state.read().is_some();
    let is_assembled = assembled_program.read().is_some();
    let error_count = assembler_errors.read().len();

    let mut tick = use_signal(|| 1);

    rsx! {
        nav { class: "bg-gray-900 text-white w-full py-2 px-4 flex items-center justify-between shadow-md border-b-2 border-gray-950",
            div { class: "flex items-center",
                span { class: "text-xl font-semibold text-blue-400 mr-4", "EmuGator" }
                div { class: "flex space-x-2",
                    button {
                        class: "bg-green-600 gap-x-1 hover:bg-green-700 text-white font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex items-center cursor-pointer",
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
                        Icon { width: 15, icon: LdPlay }
                        "Start"
                    }

                    button {
                        class: format!(
                            "{} font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex gap-x-1 items-center",
                            if is_started {
                                "bg-indigo-600 hover:bg-indigo-700 text-white cursor-pointer"
                            } else {
                                "bg-gray-600 text-gray-300 cursor-not-allowed"
                            },
                        ),
                        disabled: !is_started,
                        onclick: move |_| {
                            let new_tick = *tick.read() + 1;
                            tick.set(new_tick);
                            if let Some(mut program) = assembled_program.as_mut() {
                                let new_state = emulator_state
                                    .read()
                                    .as_ref()
                                    .map(|e| e.clock(&mut program));
                                emulator_state.set(new_state);
                            }
                        },
                        match *tick.read() % 4 {
                            0 => rsx! {
                                Icon { width: 18, icon: LdClock12 }
                            },
                            1 => rsx! {
                                Icon { width: 18, icon: LdClock3 }
                            },
                            2 => rsx! {
                                Icon { width: 18, icon: LdClock6 }
                            },
                            _ => rsx! {
                                Icon { width: 18, icon: LdClock9 }
                            },
                        }
                        "Tick Clock"
                    }
                    button {
                        class: format!(
                            "{} text-white font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex items-center gap-x-1",
                            if is_started {
                                "bg-indigo-600 hover:bg-indigo-700 text-white cursor-pointer"
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
                                    .map(|e| { e.clock_until_next_instruction(&mut program, 1000) });
                                emulator_state.set(new_state);
                            }
                        },
                        Icon { width: 17, icon: LdCircleArrowRight }
                        "Next Instruction"
                    }
                    button {
                        class: format!(
                            "{} text-white font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex items-center gap-x-1",
                            if is_started {
                                "bg-indigo-600 hover:bg-indigo-700 text-white cursor-pointer"
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
                        Icon { width: 17, icon: LdCircleArrowRight }
                        "Until Break"
                    }
                }
            }
            div { class: "flex items-stretch space-x-2",
                span {
                    class: format!(
                        "flex items-center gap-2 text-sm font-medium mr-4 {}",
                        if is_assembled {
                            "text-green-300"
                        } else if error_count > 0 {
                            "text-red-400"
                        } else {
                            "text-green-100"
                        },
                    ),
                    if is_assembled {
                        Icon { width: 17, icon: LdCircleCheck }
                        if is_started {
                            "Program Running"
                        } else {
                            "Program Assembled"
                        }
                    } else if error_count > 0 {
                        Icon { width: 17, icon: LdCircleX }
                        "Errors: {error_count}"
                    } else {
                        "Ready"
                    }
                }
                button {
                    class: "bg-yellow-600 hover:bg-yellow-700 text-white font-medium py-1 px-2 rounded transition duration-150 ease-in-out flex gap-x-1 items-center cursor-pointer",
                    onclick: move |_| {
                        let new_selection = selected_emulator.read().other();
                        selected_emulator.set(new_selection);
                        emulator_state.set(None);
                    },
                    img { width: 20, src: asset!("assets/pipeline.svg") }
                    "{selected_emulator.read().display_string()}"
                }
                button {
                    class: "bg-gray-700 hover:bg-gray-600 text-white text-sm font-medium py-1 px-2 rounded transition duration-150 ease-in-out cursor-pointer",
                    onclick: move |_| {
                        let help_panel_toggle = !*help_panel_displayed.read();
                        help_panel_displayed.set(help_panel_toggle);
                        info!("Help panel toggled: {:?}", help_panel_toggle);
                    },
                    match *help_panel_displayed.read() {
                        true => rsx! {
                            Icon { width: 18, icon: LdCircleX }
                        },
                        _ => rsx! {
                            Icon { width: 18, icon: LdInfo }
                        },
                    }
                }
            }
        }
    }
}
