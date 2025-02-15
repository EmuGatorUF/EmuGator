use crate::assembler::{self, AssembledProgram, Section};
use crate::emulator::{self, EmulatorState};

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use std::ops::Deref;
use std::collections::BTreeSet;

use wasm_bindgen::prelude::*;
use wasm_bindgen_spawn::ThreadCreator;
use std::cell::OnceCell;
use std::sync::Arc;
use js_sys::Promise;

use web_sys::{Worker};

//use web_sys::{console, HtmlElement, HtmlInputElement, MessageEvent, Worker};
//use crate::document::Script;
//use wasm_bindgen::prelude::wasm_bindgen;
//use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, Worker, WorkerGlobalScope, WorkerOptions, WorkerType,};

#[component]
#[allow(non_snake_case)]
pub fn RunButtons(
    source: Signal<String>,
    assembled_program: Signal<Option<AssembledProgram>>,
    emulator_state: Signal<EmulatorState>,
    breakpoints: Signal<BTreeSet<usize>>,
    workerRunning: bool,
) -> Element {
    rsx! {
        //Script { src: asset!("./assets/worker.js") }

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

            button {
                class: "bg-purple-500 hover:bg-purple-600 text-s text-white font-bold py-1 px-2 rounded",
                onclick: move |_| {
                    if workerRunning {
                        workerRunning = false;
                    } else {
                        workerRunning = true;
                        //web_worker_srt();
                        //start_dispatch_worker();
                        //spawn_worker();
                    }
                    info!("Stop button clicked! {}", workerRunning);
                },
                "Stop"
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

/*
#[wasm_bindgen(raw_module = "./worker.js")]
extern "C" {
    #[wasm_bindgen(js_name = web_worker_srt)]
    pub fn web_worker_srt();
}
*/

// thread_local! {
//     static THREAD_CREATOR: OnceCell<ThreadCreator> = OnceCell::new();
// }

thread_local! {
    static THREAD_CREATOR: OnceCell<Arc<ThreadCreator>> = OnceCell::new();
}

/*
#[wasm_bindgen]
pub fn start_dispatch_worker() {
    let worker_handle = Worker::new("../../worker.js").unwrap();
}
*/

#[wasm_bindgen]
pub async fn init_wasm_module() {
    //console_error_panic_hook::set_once();
    let thread_creator = match ThreadCreator::new("/dist/assets/dioxus/emu-gator_bg.wasm", "/dist/assets/dioxus/emu-gator.js") {
        Ok(v) => v,
        Err(e) => {
            info!("Failed to create thread creator");
            //error(&e);
            return;
        }
    };
    THREAD_CREATOR.with(|cell| {
        let _ = cell.set(Arc::new(thread_creator));
    });
}

fn thread_creator() -> Arc<ThreadCreator> {
    THREAD_CREATOR.with(|cell| Arc::clone(cell.get().unwrap()))
}

/*
#[wasm_bindgen]
pub fn create_thread_creator() -> Result<Promise, JsValue> {
    info!("Hi! I am in create_thread_creator :)");

    let thread_creator = ThreadCreator::new("/./dist/assets/dioxus/emu-gator_bg.wasm", "/./dist/assets/dioxus/emu-gator.js")?;
    let promise = thread_creator.ready_promise().clone();
    THREAD_CREATOR.with(move |tc| {
        tc.set(thread_creator);
    });
    Ok(promise)
}
*/

#[wasm_bindgen]
pub fn print_msg_worker() {
    info!("Hi! I am the webworker :)");
}

#[wasm_bindgen]
pub fn spawn_worker() {
    //let worker = web_sys::Worker::new("./worker.js");
    //init_wasm_in_worker();
    info!("In Spawn worker");

    // let handle = THREAD_CREATOR.with(|tc| {
    //     let tc = tc.get().unwrap();
        
    //     tc.spawn(|| {
    //         info!("Hi! I am the webworker :)");
    //     }).unwrap()
        
    //     // tc.spawn(print_msg_worker).unwrap()
    // });
    // handle.join().unwrap();
}
