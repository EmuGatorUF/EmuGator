mod data_views;
mod datapath_visualization;
mod instruction_views;
mod memory_view;
mod register_view;
mod run_buttons;

use std::collections::BTreeSet;

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, Worker, WorkerGlobalScope, WorkerOptions, WorkerType,};

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
    // let mut js = JsAssetOptions::new().with_minify(false).with_preload(true);
    const PATH_TO_MAINWORKER_JS: Asset = asset!("/assets/mainworker.js", JsAssetOptions::new().with_minify(false).with_preload(true));

    info!("Canc path: {:?}", PATH_TO_MAINWORKER_JS.resolve());
    let workerRunning: bool = false;
    let mainWorker = use_signal(|| {
        document::eval(&format!(
            r#"console.log("HI")"#
        ));


        let mut options = WorkerOptions::new();
        options.set_type(WorkerType::Module);
        Worker::new_with_options(PATH_TO_MAINWORKER_JS.resolve().to_str().unwrap(), &options).unwrap()
    });

    let source = use_signal(|| include_test_file!("prototype-demo.s").to_string());
    let assembled_program: Signal<Option<AssembledProgram>> = use_signal(|| None);
    let emulator_state: Signal<EmulatorState> = use_signal(|| EmulatorState::default());
    let breakpoints: Signal<BTreeSet<usize>> = use_signal(|| BTreeSet::new());

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
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { class: "flex h-screen w-full",
            div { class: "w-1/2 pt-4 flex flex-col h-full bg-[#1E1E1E]",
                RunButtons { source, assembled_program, emulator_state, breakpoints, workerRunning, mainWorker }
                div { class: "flex-grow",
                    CodeEditor { source, line_highlights, breakpoints }
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
