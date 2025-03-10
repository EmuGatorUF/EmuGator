use dioxus::prelude::*;
use emugator_core::assembler::{AssembledProgram, Section};
use emugator_core::emulator::EmulatorState;

#[component]
#[allow(non_snake_case)]
pub fn InstructionView(
    assembled_program: Signal<Option<AssembledProgram>>,
    emulator_state: Signal<EmulatorState>,
) -> Element {
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
    let instruction_memory = &program.instruction_memory;
    let text_start = program.get_section_start(Section::Text) as usize;
    let current_pc = emulator_state.read().pipeline.ID_pc as usize;

    let total_instructions = instruction_memory.len() / 4; // Since each instruction is 4 bytes

    rsx! {
        div { class: "h-full overflow-hidden",
            div { class: "h-full overflow-auto pr-2",
                div { class: "bg-white rounded shadow-sm p-2",
                    for i in 0..total_instructions {
                        {
                            let base_addr = text_start + i * 4;
                            rsx! {
                                div {
                                    class: {
                                        if i < total_instructions - 1 {
                                            "flex justify-between items-center border-b border-gray-100 py-1"
                                        } else {
                                            "flex justify-between items-center py-1"
                                        }
                                    },
                                    div { class: "flex-1",
                                        div { class: "flex justify-between",
                                            div { class: "font-mono text-gray-500 text-xs", "0x{base_addr:04x}:" }
                                            {
                                                let instruction = (instruction_memory
                                                    .get(&(base_addr as u32))
                                                    .copied()
                                                    .unwrap_or(0) as u32)
                                                    | ((instruction_memory.get(&((base_addr + 1) as u32)).copied().unwrap_or(0)
                                                        as u32) << 8)
                                                    | ((instruction_memory.get(&((base_addr + 2) as u32)).copied().unwrap_or(0)
                                                        as u32) << 16)
                                                    | ((instruction_memory.get(&((base_addr + 3) as u32)).copied().unwrap_or(0)
                                                        as u32) << 24);
                                                rsx! {
                                                    span { class: if base_addr == current_pc { "font-mono font-bold text-orange-500" } else { "font-mono font-bold" }, "{instruction:032b}" }
                                                }
                                            }
                                            if let Some(line) = program.source_map.get_by_left(&(base_addr as u32)) {
                                                span { class: "text-xs text-gray-500", "Line {line}" }
                                            }
                                        }
                                        div { class: "font-mono font-bold",
                                            {
                                                let instruction = (instruction_memory
                                                    .get(&(base_addr as u32))
                                                    .copied()
                                                    .unwrap_or(0) as u32)
                                                    | ((instruction_memory.get(&((base_addr + 1) as u32)).copied().unwrap_or(0)
                                                        as u32) << 8)
                                                    | ((instruction_memory.get(&((base_addr + 2) as u32)).copied().unwrap_or(0)
                                                        as u32) << 16)
                                                    | ((instruction_memory.get(&((base_addr + 3) as u32)).copied().unwrap_or(0)
                                                        as u32) << 24);
                                                rsx! {
                                                    span { class: if base_addr == current_pc { "text-orange-500" } else { "" }, "{instruction:032b}" }
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
