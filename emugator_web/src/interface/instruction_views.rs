use dioxus::prelude::*;
use emugator_core::assembler::{AssembledProgram, Section};
use emugator_core::bits;
use emugator_core::emulator::AnyEmulatorState;
use emugator_core::isa::{Instruction, InstructionDefinition, InstructionFormat};

#[component]
#[allow(non_snake_case)]
pub fn InstructionView(
    assembled_program: ReadOnlySignal<Option<AssembledProgram>>,
    emulator_state: ReadOnlySignal<Option<AnyEmulatorState>>,
) -> Element {
    // Early return if no program is assembled
    let Some(program) = assembled_program.as_ref() else {
        return rsx! {
            div { class: "flex justify-center items-center h-full",
                span { class: "text-gray-500 font-mono", "No program loaded" }
            }
        };
    };

    let instruction_memory = &program.instruction_memory;
    let text_start = program.get_section_start(Section::Text);
    let emulator_state = emulator_state.read();
    let current_pc = emulator_state.as_ref().and_then(|e| e.id_pc());

    let total_instructions = (instruction_memory.len() / 4) as u32; // Since each instruction is 4 bytes

    rsx! {
        div { class: "h-full overflow-hidden",
            div { class: "h-full overflow-auto pr-2",
                div { class: "bg-white rounded shadow-sm p-2",
                    for i in 0..total_instructions {
                        {
                            let base_addr = text_start + i * 4;
                            let instruction = (instruction_memory.get(&(base_addr)).copied().unwrap_or(0)
                                as u32)
                                | ((instruction_memory.get(&(base_addr + 1)).copied().unwrap_or(0) as u32)
                                    << 8)
                                | ((instruction_memory.get(&(base_addr + 2)).copied().unwrap_or(0) as u32)
                                    << 16)
                                | ((instruction_memory.get(&(base_addr + 3)).copied().unwrap_or(0) as u32)
                                    << 24);
                            let instr = Instruction::from_raw(instruction);
                            let instr_frmt = InstructionDefinition::from_instr(instr).unwrap().format;
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
                                            div { class: "font-mono font-bold text-xs",
                                                // if instruction is current instruction, color each piece
                                                if Some(base_addr) == current_pc {
                                                    div {
                                                        class: "invisible",
                                                        onmounted: move |ctx| async move {
                                                            let scroll = ctx.data();
                                                            scroll.scroll_to(ScrollBehavior::Instant).await.unwrap();
                                                        },
                                                    }
                                                    if instr_frmt == InstructionFormat::R {
                                                        span { class: "text-red-700", "{instr.funct7():07b}" }
                                                    } else if instr_frmt == InstructionFormat::I {
                                                        span { class: "text-red-700", "{bits!(instr.immediate().unwrap(), 11;0):012b}" }
                                                    } else if instr_frmt == InstructionFormat::S {
                                                        span { class: "text-red-700", "{bits!(instr.immediate().unwrap(), 11;5):07b}" }
                                                    } else if instr_frmt == InstructionFormat::B {
                                                        span { class: "text-black", "{bits!(instr.immediate().unwrap(),12):01b}" }
                                                        span { class: "text-red-700", "{bits!(instr.immediate().unwrap(),10;5):06b}" }
                                                    } else if instr_frmt == InstructionFormat::U {
                                                        span { class: "text-red-700", "{bits!(instr.immediate().unwrap(), 31;12):020b}" }
                                                    } else if instr_frmt == InstructionFormat::J {
                                                        span { class: "text-red-700", "{bits!(instr.immediate().unwrap(),20):01b}" }
                                                        span { class: "text-orange-500", "{bits!(instr.immediate().unwrap(),10;1):010b}" }
                                                        span { class: "text-yellow-500", "{bits!(instr.immediate().unwrap(),11):01b}" }
                                                        span { class: "text-green-500", "{bits!(instr.immediate().unwrap(),19;12):08b}" }
                                                    }

                                                    if instr_frmt != InstructionFormat::U && instr_frmt != InstructionFormat::J {
                                                        if instr_frmt != InstructionFormat::I {
                                                            span { class: "text-orange-500", "{instr.rs2():05b}" }
                                                        }
                                                        span { class: "text-yellow-500", "{instr.rs1():05b}" }
                                                        span { class: "text-green-500", "{instr.funct3():03b}" }
                                                    }

                                                    if instr_frmt == InstructionFormat::S {
                                                        span { class: "text-red-700", "{bits!(instr.immediate().unwrap(), 4;0):05b}" }
                                                    } else if instr_frmt == InstructionFormat::B {
                                                        span { class: "text-blue-500", "{bits!(instr.immediate().unwrap(), 4;1):04b}" }
                                                        span { class: "text-gray-500", "{bits!(instr.immediate().unwrap(), 11):01b}" }
                                                    } else {
                                                        span { class: "text-blue-500", "{instr.rd():05b}" }
                                                    }

                                                    span { class: "text-purple-500", "{instr.opcode():07b}" }
                                                } else {
                                                    span { class: "font-mono font-bold text-gray-500 text-xs", "{instruction:032b}" }
                                                }
                                            }
                                            if let Some(line) = program.source_map.get_by_left(&base_addr) {
                                                span { class: "text-xs text-gray-500", "Line {line}" }
                                            }
                                        }
                                        // displays information about which colors correspond to which part of the instruction
                                        if Some(base_addr) == current_pc {
                                            div { class: "flex justify-center",
                                                div { class: "font-mono font-bold text-xs text-gray-500",
                                                    {
                                                        rsx! {
                                                            if instr_frmt == InstructionFormat::R {
                                                                span { class: "text-red-700", "funct7 " }
                                                            } else if instr_frmt == InstructionFormat::I {
                                                                span { class: "text-red-700", "imm[11:0] " }
                                                            } else if instr_frmt == InstructionFormat::S {
                                                                span { class: "text-red-700", "imm[11:5] " }
                                                            } else if instr_frmt == InstructionFormat::B {
                                                                span { class: "text-black", "imm[12] " }
                                                                span { class: "text-red-700", "imm[10:5] " }
                                                            } else if instr_frmt == InstructionFormat::U {
                                                                span { class: "text-red-700", "imm[31:12] " }
                                                            } else if instr_frmt == InstructionFormat::J {
                                                                span { class: "text-red-700", "imm[20] " }
                                                                span { class: "text-orange-500", "imm[10:1] " }
                                                                span { class: "text-yellow-500", "imm[11] " }
                                                                span { class: "text-green-500", "imm[19:12] " }
                                                            }

                                                            if instr_frmt != InstructionFormat::U && instr_frmt != InstructionFormat::J {
                                                                if instr_frmt != InstructionFormat::I {
                                                                    span { class: "text-orange-500", "rs2 " }
                                                                }
                                                                span { class: "text-yellow-500", "rs1 " }
                                                                span { class: "text-green-500", "funct3 " }
                                                            }

                                                            if instr_frmt == InstructionFormat::S {
                                                                span { class: "text-blue-500", "imm[4:0] " }
                                                            } else if instr_frmt == InstructionFormat::B {
                                                                span { class: "text-blue-500", "imm[4:1] " }
                                                                span { class: "text-gray-500", "imm[11] " }
                                                            } else {
                                                                span { class: "text-blue-500", "rd " }
                                                            }

                                                            span { class: "text-purple-500", "opcode" }
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
    }
}
