mod data_views;
mod datapath_visualization;
mod instruction_views;
mod memory_view;
mod register_view;
mod run_buttons;

use dioxus::prelude::*;
use dioxus_logger::tracing::info;

use self::{
    datapath_visualization::DatapathVisualization, memory_view::MemoryView,
    register_view::RegisterView, run_buttons::RunButtons,
};
use crate::{
    assembler::AssembledProgram,
    code_editor::{CodeEditor, LineHighlight},
    emulator::EmulatorState,
    include_test_file,
};

#[component]
#[allow(non_snake_case)]
pub fn App() -> Element {
    let source = use_signal(|| include_test_file!("prototype-demo.s").to_string());
    let assembled_program: Signal<Option<AssembledProgram>> = use_signal(|| None);
    let emulator_state: Signal<EmulatorState> = use_signal(|| EmulatorState::default());

    use_effect(move || {
        info!("source changed");
        // TODO: Get diagnostics
    });

    let mut line_highlights = use_signal(|| Vec::<LineHighlight>::new());
    use_effect(move || {
        line_highlights.write().clear();

        fn get_pc_line(
            pc: u32,
            assembled_program: &Signal<Option<AssembledProgram>>,
        ) -> Option<usize> {
            assembled_program
                .read()
                .as_ref()
                .map(|p| p.source_map.get_by_left(&pc).copied())
                .flatten()
        }

        if let Some(line) = get_pc_line(emulator_state.read().pipeline.ID_pc, &assembled_program) {
            line_highlights.write().push(LineHighlight {
                line,
                css_class: "id-pc-decoration",
            });
        }

        if let Some(line) = get_pc_line(emulator_state.read().pipeline.IF_pc, &assembled_program) {
            line_highlights.write().push(LineHighlight {
                line,
                css_class: "if-pc-decoration",
            });
        }
    });

    rsx! {
        div { class: "flex h-screen w-full",
            div { class: "w-1/2 p-4 flex flex-col h-full bg-[#1E1E1E]",
                RunButtons { source, assembled_program, emulator_state }
                div { class: "flex-grow",
                    CodeEditor { source, line_highlights }
                }
            }
            div { class: "w-1/2 flex flex-col",
                div { class: "h-1/3 bg-gray-200 p-4",
                    DatapathVisualization { emulator_state }
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
