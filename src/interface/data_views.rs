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
    let total_quad_words = (data_memory.len() + 15) / 16;

    rsx! {
        div { class: "h-full overflow-hidden",
            div { class: "h-full overflow-auto pr-2",
                div { class: "bg-white rounded shadow-sm p-2",
                    table { class: "w-full font-mono text-gray-800 font-bold",    
                        thead {
                            tr {class: "bg-gray-300",
                                th{class: "text-left", scope: "row", colspan: "1", "Address"}
                                th{scope: "row", colspan: "4", "Hex"}
                            }
                            tr {
                                td{""}
                                td{class: "text-gray-500", "+ 0x0"}
                                td{class: "text-gray-500", "+ 0x4"}
                                td{class: "text-gray-500", "+ 0x8"}
                                td{class: "text-gray-500", "+ 0xc"}
                            }
                        }
                        tbody {
                            for i in 0..total_quad_words {
                                {
                                    let base_addr = data_start + i * 16;
                                    {
                                        let mut words: [u32; 4] = [0,0,0,0];
                                        for i in 0..4{
                                            words[i] = (data_memory.get(&((base_addr + i*4) as u32)).copied().unwrap_or(0) as u32)
                                            | ((data_memory.get(&((base_addr + i*4 + 1) as u32)).copied().unwrap_or(0) as u32)
                                                << 8)
                                            | ((data_memory.get(&((base_addr + i*4 + 2) as u32)).copied().unwrap_or(0) as u32)
                                                << 16)
                                            | ((data_memory.get(&((base_addr + i*4 + 3) as u32)).copied().unwrap_or(0) as u32)
                                                << 24);
                                        }
                                        rsx!{
                                            tr {padding: "20px",
                                                td{class: "flex-1 text-gray-500", "0x{base_addr:04x}:"}
                                                td{class: "flex-1", "0{words[0]:08x}"}
                                                td{class: "flex-1", "0{words[1]:08x}"}
                                                td{class: "flex-1", "0{words[2]:08x}"}
                                                td{class: "flex-1", "0{words[3]:08x}"}
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
