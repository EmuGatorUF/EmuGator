use dioxus::prelude::*;
use emugator_core::assembler::{AssembledProgram, Section};
use emugator_core::emulator::AnyEmulatorState;
use emugator_core::isa::{Instruction, InstructionDefinition, InstructionFormat};
use emugator_core::bits;

#[component]
#[allow(non_snake_case)]
pub fn InstructionView(
    assembled_program: Signal<Option<AssembledProgram>>,
    emulator_state: Signal<AnyEmulatorState>,
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
    let current_pc = match &*emulator_state.read() {
        AnyEmulatorState::CVE2(state) => state.pipeline.ID_pc as usize,
        AnyEmulatorState::FiveStage(_state) => todo!(),
    };

    let total_instructions = instruction_memory.len() / 4; // Since each instruction is 4 bytes

    rsx! {
        div { class: "h-full overflow-hidden",
            div { class: "h-full overflow-auto pr-2",
                div { class: "bg-white rounded shadow-sm p-2",
                    for i in 0..total_instructions {
                        {
                            let base_addr = text_start + i * 4;
                            let instruction = (instruction_memory.get(&(base_addr as u32)).copied().unwrap_or(0) as u32)
                                | ((instruction_memory.get(&((base_addr + 1) as u32)).copied().unwrap_or(0)
                                    as u32) << 8)
                                | ((instruction_memory.get(&((base_addr + 2) as u32)).copied().unwrap_or(0)
                                    as u32) << 16)
                                | ((instruction_memory.get(&((base_addr + 3) as u32)).copied().unwrap_or(0)
                                    as u32) << 24);
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
                                                if base_addr == current_pc {
                                                    if instr_frmt == InstructionFormat::R {
                                                        span { class: "text-red-700", "{instr.funct7():07b}" }
                                                    } else if instr_frmt == InstructionFormat::I {
                                                        span { class: "text-red-700", "{instr.immediate().unwrap():012b}"}
                                                    } else if instr_frmt == InstructionFormat::S {
                                                        span {class: "text-red-700", "{bits!(instr.immediate().unwrap(),11;5):07b}"}
                                                    } else if instr_frmt == InstructionFormat::B {
                                                        span { class: "text-black", "{bits!(instr.immediate().unwrap(),12):01b}" }
                                                        span { class: "text-red-700", "{bits!(instr.immediate().unwrap(),10;5):06b}" }
                                                    } else if instr_frmt == InstructionFormat::U {
                                                        span {class: "text-red-700", "{bits!(instr.immediate().unwrap(),31;12):020b}"}
                                                    } else if instr_frmt == InstructionFormat::J {
                                                        span { class: "text-red-700", "{bits!(instr.immediate().unwrap(),20):01b}" }
                                                        span { class: "text-orange-500", "{bits!(instr.immediate().unwrap(),10;1):010b}" }
                                                        span { class: "text-yellow-500", "{bits!(instr.immediate().unwrap(),11):01b}" }
                                                        span { class: "text-green-500", "{bits!(instr.immediate().unwrap(),19;12):08b}" }
                                                    }

                                                    if instr_frmt != InstructionFormat::U && instr_frmt != InstructionFormat::J{
                                                        if instr_frmt != InstructionFormat::I {
                                                            span { class: "text-orange-500", "{instr.rs2():05b}" }
                                                        }
                                                        span { class: "text-yellow-500", "{instr.rs1():05b}" }
                                                        span { class: "text-green-500", "{instr.funct3():03b}" }
                                                    }

                                                    if instr_frmt == InstructionFormat::S {
                                                        span { class: "text-blue-500", "{instr.immediate().unwrap():05b}" }
                                                    } else if instr_frmt == InstructionFormat::B {
                                                        span { class: "text-blue-500", "{bits!(instr.immediate().unwrap(),4;1):04b}" }
                                                        span { class: "text-gray-500", "{bits!(instr.immediate().unwrap(), 11):01b}" }
                                                    }else {
                                                        span { class: "text-blue-500", "{instr.rd():05b}" }
                                                    }

                                                    span { class: "text-purple-500", "{instr.opcode():07b}" }
                                                } else {
                                                    span { class: "font-mono font-bold text-gray-500 text-xs", "{instruction:032b}" }
                                                }
                                            }
                                            if let Some(line) = program.source_map.get_by_left(&(base_addr as u32)) {
                                                span { class: "text-xs text-gray-500", "Line {line}" }
                                            }
                                        }
                                        // displays information about which colors correspond to which part of the instruction
                                        if base_addr == current_pc{
                                            div{ class: "flex justify-center",
                                                div { class: "font-mono text-xs text-gray-500",
                                                    {
                                                        rsx! {
                                                            if instr_frmt == InstructionFormat::R {
                                                                span { class: "text-red-700", "funct7 " }
                                                            } else if instr_frmt == InstructionFormat::I {
                                                                span { class: "text-red-700", "imm[11:0] "}
                                                            } else if instr_frmt == InstructionFormat::S {
                                                                span {class: "text-red-700", "imm[11:5] "}
                                                            } else if instr_frmt == InstructionFormat::B {
                                                                span { class: "text-black", "imm[12] " }
                                                                span { class: "text-red-700", "imm[10:5] " }
                                                            } else if instr_frmt == InstructionFormat::U {
                                                                span {class: "text-red-700", "imm[31:12] "}
                                                            } else if instr_frmt == InstructionFormat::J {
                                                                span { class: "text-red-700", "imm[20] " }
                                                                span { class: "text-orange-500", "imm[10:1] " }
                                                                span { class: "text-yellow-500", "imm[11] " }
                                                                span { class: "text-green-500", "imm[19:12] " }
                                                            }

                                                            if instr_frmt != InstructionFormat::U && instr_frmt != InstructionFormat::J{
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
                                                            }else {
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
