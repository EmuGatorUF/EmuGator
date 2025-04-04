use dioxus::prelude::*;
use emugator_core::{
    assembler::{AssembledProgram, Section},
    emulator::AnyEmulatorState,
};

#[component]
#[allow(non_snake_case)]
pub fn DataView(
    assembled_program: ReadOnlySignal<Option<AssembledProgram>>,
    emulator_state: ReadOnlySignal<Option<AnyEmulatorState>>,
) -> Element {
    // Early return if no program is assembled
    let assembled_program = assembled_program.read();
    let emulator_state = emulator_state.read();
    let (Some(program), Some(state)) = (assembled_program.as_ref(), emulator_state.as_ref()) else {
        return rsx! {
            div { class: "flex justify-center items-center h-full",
                span { class: "text-gray-500 font-mono", "No program running" }
            }
        };
    };

    let data_memory = emulator_state.memory_io();
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
                                        for (j, b) in dw_bytes.iter_mut().enumerate() {
                                            *b = data_memory.preview((base_addr + j) as u32);
                                        }
                                        let hex_string1 = format!(
                                            "{:02x} {:02x} {:02x} {:02x}",
                                            dw_bytes[0],
                                            dw_bytes[1],
                                            dw_bytes[2],
                                            dw_bytes[3],
                                        );
                                        let hex_string2 = format!(
                                            "{:02x} {:02x} {:02x} {:02x}",
                                            dw_bytes[4],
                                            dw_bytes[5],
                                            dw_bytes[6],
                                            dw_bytes[7],
                                        );
                                        for b in &mut dw_bytes {
                                            if *b < 0x21 || *b > 0x7e {
                                                *b = b'.';
                                            }
                                        }
                                        let char_string = String::from_utf8_lossy(&dw_bytes[0..8]).to_string();
                                        rsx! {
                                            tr { padding: "20px",
                                                td { class: "flex-1 text-gray-500 text-xs", "0x{base_addr:04x}:" }
                                                td { class: "flex-1", "{hex_string1}" }
                                                td { class: "flex-1", "{hex_string2}" }
                                                td { class: "flex-1", "{char_string}" }
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
