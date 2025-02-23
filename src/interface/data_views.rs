use crate::assembler::{AssembledProgram, Section};
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Copy)]
pub enum DataViewType {
    Hex,
    Chars,
}

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
    let mut view_type = use_signal(|| DataViewType::Hex);

    rsx! {
        div { class: "h-full overflow-hidden",
            div { class: "h-full overflow-auto pr-2",

                // Data type view selector buttons
                div { class: "flex gap-4 mb-2 flex-shrink-0",
                    button {
                        class: "text-lg font-mono font-bold text-gray-900 hover:text-gray-700 transition-colors",
                        style: if *view_type.read() == DataViewType::Hex { "text-decoration: underline" } else { "" },
                        onclick: move |_| view_type.set(DataViewType::Hex),
                        "Hex"
                    }
                    span { class: "text-lg font-mono font-bold text-gray-900", "/" }
                    button {
                        class: "text-lg font-mono font-bold text-gray-900 hover:text-gray-700 transition-colors",
                        style: if *view_type.read() == DataViewType::Chars { "text-decoration: underline" } else { "" },
                        onclick: move |_| view_type.set(DataViewType::Chars),
                        "Char"
                    }
                }

                div { class: "bg-white rounded shadow-sm p-2",
                    table { class: "w-full font-mono text-gray-800 font-bold",    
                        tbody {
                            for i in 0..total_quad_words {
                                {
                                    let base_addr = data_start + i * 16;
                                    {
                                        let mut dw_bytes: [u8; 16] = [0; 16];
                                        let mut dw_string1 = String::new();
                                        let mut dw_string2 = String::new();
                                        for j in 0..16{
                                            dw_bytes[j] = (data_memory.get(&((base_addr + j) as u32)).copied().unwrap_or(0) as u8);
                                        }
                                        match *view_type.read(){
                                            DataViewType::Hex => {
                                                dw_string1 = format!("{:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", dw_bytes[0], dw_bytes[1], 
                                                    dw_bytes[2], dw_bytes[3], dw_bytes[4], dw_bytes[5], dw_bytes[6], dw_bytes[7]);
                                                dw_string2 = format!("{:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", dw_bytes[8], dw_bytes[9], 
                                                    dw_bytes[10], dw_bytes[11], dw_bytes[12], dw_bytes[13], dw_bytes[14], dw_bytes[15]);
                                            },
                                            DataViewType::Chars => {
                                                // replace invalid characters with '.'
                                                for j in 0..16 {
                                                    if dw_bytes[j] < 0x21 || dw_bytes[j] > 0x7e {
                                                        dw_bytes[j] = b'.';
                                                    }
                                                }
                                                dw_string1 = String::from_utf8_lossy(&dw_bytes[0..8]).to_string();
                                                dw_string2 = String::from_utf8_lossy(&dw_bytes[8..16]).to_string();
                                            }
                                        }
                                            rsx!{
                                            tr {padding: "20px",
                                                td{class: "flex-1 text-gray-500 text-xs", "0x{base_addr:04x}:"}
                                                td{class: "flex-1", "{dw_string1}"}
                                                td{class: "flex-1", "{dw_string2}"}
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
