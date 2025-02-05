use crate::assembler::{AssembledProgram, Section};
use dioxus::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn DataView(assembled_program: Signal<Option<AssembledProgram>>) -> Element {
    let program = assembled_program.read();

    // Early return if no program is assembled
    if program.is_none() {
        return rsx! {
            div { class: "flex justify-center items-center h-full",
                span { class: "text-gray-500 font-mono", "No program loaded" }
            }
        };
    }

    let program = program.as_ref().unwrap();
    let data_memory = &program.data_memory;
    let data_start = program.get_section_start(Section::Data) as usize;

    // changed this to fix a bug where partial words did not show in data view
    let total_words = (data_memory.len() + 3) / 4;

    rsx! {
        div { class: "h-full overflow-hidden",
            div { class: "h-full overflow-auto pr-2",
                div { class: "bg-white rounded shadow-sm p-2",
                    for i in 0..total_words {
                        {
                            let base_addr = data_start + i * 4;
                            rsx! {
                                div {
                                    class: {
                                        if i < total_words - 1 {
                                            "flex justify-between items-center border-b border-gray-100 py-1"
                                        } else {
                                            "flex justify-between items-center py-1"
                                        }
                                    },
                                    div { class: "flex-1",
                                        div { class: "flex justify-between",
                                            div { class: "font-mono text-gray-500 text-xs", "0x{base_addr:04x}:" }
                                        }
                                        div { class: "font-mono font-bold",
                                            {
                                                let word = (data_memory.get(&(base_addr as u32)).copied().unwrap_or(0) as u32)
                                                    | ((data_memory.get(&((base_addr + 1) as u32)).copied().unwrap_or(0) as u32)
                                                        << 8)
                                                    | ((data_memory.get(&((base_addr + 2) as u32)).copied().unwrap_or(0) as u32)
                                                        << 16)
                                                    | ((data_memory.get(&((base_addr + 3) as u32)).copied().unwrap_or(0) as u32)
                                                        << 24);
                                                rsx! {
                                                "0x{word:08x}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
