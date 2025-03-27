mod data_views;
mod instruction_views;
mod memory_view;
mod pipeline_visualization;
mod register_view;
mod run_buttons;
mod uart_view;

use std::collections::BTreeSet;

use dioxus::prelude::*;
use dioxus_logger::tracing::info;

use self::{
    memory_view::MemoryView, pipeline_visualization::PipelineVisualization,
    register_view::RegisterView, run_buttons::RunButtons, uart_view::UartView,
};
use crate::code_editor::{CodeEditor, LineHighlight};
use emugator_core::{
    assembler::{AssembledProgram, AssemblerError},
    emulator::AnyEmulatorState,
    include_test_file,
};

#[component]
#[allow(non_snake_case)]
pub fn App() -> Element {
    let source = use_signal(|| include_test_file!("beta-demo.s").to_string());
    let assembled_program: Signal<Option<AssembledProgram>> = use_signal(|| None);
    let assembler_errors: Signal<Vec<AssemblerError>> = use_signal(Vec::new);
    let emulator_state: Signal<AnyEmulatorState> =
        use_signal(|| AnyEmulatorState::new_cve2(&AssembledProgram::empty()));
    let breakpoints: Signal<BTreeSet<usize>> = use_signal(BTreeSet::new);

    let minimize_console: Signal<bool> = use_signal(|| false);

    use_effect(move || {
        info!("source changed");
        // TODO: Get diagnostics
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

        match &*emulator_state.read() {
            AnyEmulatorState::CVE2(state) => {
                if let Some(line) = get_pc_line(state.pipeline.ID_pc, &assembled_program) {
                    line_highlights.write().push(LineHighlight {
                        line,
                        css_class: "id-pc-decoration",
                    });
                }

                if let Some(line) = get_pc_line(state.pipeline.IF_pc, &assembled_program) {
                    line_highlights.write().push(LineHighlight {
                        line,
                        css_class: "if-pc-decoration",
                    });
                }
            }
            AnyEmulatorState::FiveStage(_) => todo!(),
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("tailwind.css") }

        div { class: "flex h-screen w-full",
            div { class: "w-1/2 pt-4 flex flex-col h-full bg-[#1E1E1E] overflow-hidden",
                RunButtons {
                    source,
                    assembled_program,
                    assembler_errors,
                    emulator_state,
                    breakpoints,
                }
                if assembled_program.read().is_some() {
                    div { class: "flex-1 relative overflow-hidden",
                        CodeEditor {
                            source,
                            line_highlights,
                            breakpoints,
                            assembler_errors,
                        }
                    }
                    div {
                        class: "transition-all duration-300 ease-in-out ".to_owned()
                            + { if *minimize_console.read() { "h-min" } else { "h-4/10" } },
                        UartView {
                            uart_module: emulator_state.map(|s| s.uart()),
                            minimize_console,
                        }
                    }
                } else {
                    div { class: "flex-col h-screen",
                        CodeEditor {
                            source,
                            line_highlights,
                            breakpoints,
                            assembler_errors,
                        }
                    }
                }
            }
            div { class: "w-1/2 flex flex-col",
                div { class: "h-1/3 bg-gray-200 p-4",
                    PipelineVisualization { emulator_state }
                }
                div { class: "h-1/3 bg-gray-300 p-4",
                    RegisterView { emulator_state }
                }
                div { class: "h-1/3 bg-gray-400 p-4",
                    MemoryView { assembled_program, emulator_state }
                }
            }
        }
    }
}
