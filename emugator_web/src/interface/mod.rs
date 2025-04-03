mod data_views;
mod instruction_views;
mod memory_view;
mod navbar;
mod pipeline_visualization;
mod register_view;
mod uart_view;

use std::{collections::BTreeSet, time::Duration};

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use dioxus_sdk::utils::timing::use_debounce;

use self::{
    memory_view::MemoryView, navbar::Navbar, pipeline_visualization::PipelineVisualization,
    register_view::RegisterView, uart_view::UartView,
};
use crate::code_editor::{CodeEditor, LineHighlight};
use emugator_core::{
    assembler::{self, AssembledProgram, AssemblerError},
    emulator::AnyEmulatorState,
    include_test_file,
};

#[component]
#[allow(non_snake_case)]
pub fn App() -> Element {
    let source = use_signal(|| include_test_file!("beta-demo.s").to_string());
    let mut assembled_program: Signal<Option<AssembledProgram>> = use_signal(|| None);
    let mut assembler_errors: Signal<Vec<AssemblerError>> = use_signal(Vec::new);
    let emulator_state: Signal<AnyEmulatorState> =
        use_signal(|| AnyEmulatorState::new_cve2(&AssembledProgram::empty()));
    let breakpoints: Signal<BTreeSet<usize>> = use_signal(BTreeSet::new);

    let minimize_console: Signal<bool> = use_signal(|| true);

    let mut assemble_debounce = use_debounce(Duration::from_secs(1), move |_| {
        info!("Assembling...");
        match assembler::assemble(&source.peek()) {
            Ok(assembled) => {
                info!("Assembly succeeded.");
                assembled_program.set(Some(assembled));
                assembler_errors.set(Vec::new());
            }
            Err(errors) => {
                info!("Assembly failed.");
                assembled_program.set(None);
                assembler_errors.set(errors);
            }
        }
    });

    use_effect(move || {
        info!("Source changed");
        let _ = source.read();
        assemble_debounce.action(());
    });

    let mut line_highlights = use_signal(Vec::<LineHighlight>::new);
    use_effect(move || {
        line_highlights.write().clear();

        fn get_pc_line(
            pc: u32,
            assembled_program: &Signal<Option<AssembledProgram>>,
        ) -> Option<usize> {
            assembled_program
                .read()
                .as_ref()
                .and_then(|p| p.source_map.get_by_left(&pc).copied())
        }

        line_highlights.set(
            emulator_state
                .read()
                .all_pcs()
                .iter()
                .filter_map(|pc_pos| {
                    if let Some(line) = get_pc_line(pc_pos.pc, &assembled_program) {
                        Some(LineHighlight {
                            line,
                            css_class: format!("{}-pc-decoration", pc_pos.name),
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        );
    });

    rsx! {
        document::Stylesheet { href: asset!("tailwind.css") }
        style { "html, body {{ margin: 0; padding: 0; }} #main {{ margin: 0; }}" }

        div { class: "flex flex-col h-screen w-full bg-gray-800 m-0 p-0",
            Navbar {
                source,
                assembled_program,
                assembler_errors,
                emulator_state,
                breakpoints,
                minimize_console,
            }
            div { class: "flex flex-1 overflow-hidden",
                div { class: "w-1/2 flex flex-col h-full bg-[#1E1E1E] overflow-hidden border-r-2 border-gray-900",
                    div { class: "flex-1 relative overflow-hidden",
                        CodeEditor {
                            source,
                            line_highlights,
                            breakpoints,
                            assembler_errors,
                        }
                    }
                    div {
                        class: format!(
                            "transition-all duration-300 ease-in-out bg-[#2D2D2D] border-t-2 border-gray-900 {}",
                            if *minimize_console.read() { "h-min" } else { "h-4/10" },
                        ),
                        UartView { emulator_state, minimize_console }
                    }
                }
                div { class: "w-1/2 flex flex-col bg-gray-700 text-white",
                    div { class: "h-1/3 bg-gray-700 p-2 border-b-2 border-gray-900",
                        div { class: "bg-gray-800 rounded h-full p-2",
                            div { class: "flex items-center mb-2",
                                div { class: "h-4 w-1 bg-blue-500 mr-2" }
                                span { class: "text-sm font-medium text-gray-300",
                                    "Pipeline Visualization"
                                }
                            }
                            div { class: "h-[calc(100%-2rem)] overflow-auto",
                                PipelineVisualization { emulator_state }
                            }
                        }
                    }
                    div { class: "h-1/3 bg-gray-700 p-2 border-b-2 border-gray-900",
                        div { class: "bg-gray-800 rounded h-full p-2",
                            div { class: "flex items-center mb-2",
                                div { class: "h-4 w-1 bg-green-500 mr-2" }
                                span { class: "text-sm font-medium text-gray-300", "Register View" }
                            }
                            div { class: "h-[calc(100%-2rem)] overflow-auto",
                                RegisterView { emulator_state }
                            }
                        }
                    }
                    div { class: "h-1/3 bg-gray-700 p-2",
                        div { class: "bg-gray-800 rounded h-full p-2",
                            div { class: "flex items-center mb-2",
                                div { class: "h-4 w-1 bg-purple-500 mr-2" }
                                span { class: "text-sm font-medium text-gray-300", "Memory View" }
                            }
                            div { class: "h-[calc(100%-2rem)] overflow-auto",
                                MemoryView { assembled_program, emulator_state }
                            }
                        }
                    }
                }
            }
        }
    }
}
