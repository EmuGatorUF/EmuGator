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
    let total_double_words = (data_memory.len() + 15) / 8;

    rsx! {
        div { class: "h-full overflow-hidden",
            div { class: "h-full overflow-auto pr-2",
                div { class: "bg-white rounded shadow-sm p-2",
                    table { class: "w-full font-mono text-gray-800 font-bold",    
                        tbody {
                            for i in 0..total_double_words {
                                {
                                    let base_addr = data_start + i * 8;
                                    {
                                        let mut dw_bytes: [u8; 8] = [0; 8];
                                        for j in 0..8{
                                            dw_bytes[j] = (data_memory.get(&((base_addr + j) as u32)).copied().unwrap_or(0) as u8);
                                        }
                                        
                                        let hex_string1 = format!("{:02x} {:02x} {:02x} {:02x}", dw_bytes[0], dw_bytes[1], dw_bytes[2], dw_bytes[3]);
                                        let hex_string2 = format!("{:02x} {:02x} {:02x} {:02x}", dw_bytes[4], dw_bytes[5], dw_bytes[6], dw_bytes[7]);    
                                        // replace invalid characters with '.'
                                        for j in 0..8 {
                                            if dw_bytes[j] < 0x21 || dw_bytes[j] > 0x7e {
                                                dw_bytes[j] = b'.';
                                            }
                                        }
                                        let char_string = String::from_utf8_lossy(&dw_bytes[0..8]).to_string();
                                        rsx!{
                                            tr {padding: "20px",
                                                td{class: "flex-1 text-gray-500 text-xs", "0x{base_addr:04x}:"}
                                                td{class: "flex-1", "{hex_string1}"}
                                                td{class: "flex-1", "{hex_string2}"}
                                                td{class: "flex-1", "{char_string}"}
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
