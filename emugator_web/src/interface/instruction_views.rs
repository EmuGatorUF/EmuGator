use dioxus::prelude::*;
use emugator_core::assembler::{AssembledProgram, Section};
use emugator_core::emulator::AnyEmulatorState;
use emugator_core::isa::{Instruction, InstructionDefinition, InstructionFormat};

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
                                                rsx! {
                                                    span { class: if base_addr == current_pc { "font-mono font-bold text-orange-500 text-xs" } else { "font-mono font-bold text-xs" }, "{instruction:032b}" }
                                                }
                                            }
                                            if let Some(line) = program.source_map.get_by_left(&(base_addr as u32)) {
                                                span { class: "text-xs text-gray-500", "Line {line}" }
                                            }
                                        }
                                        if base_addr == current_pc{
                                            div { class: "font-mono text-xs text-gray-500",
                                                {
                                                    let instr = Instruction::from_raw(instruction);
                                                    let instr_frmt = InstructionDefinition::from_instr(instr).unwrap().format;
                                                    rsx! {
                                                        if instr_frmt == InstructionFormat::R {
                                                            span {"Type: R, funct7: {instr.funct7():07b}, rs2: {instr.rs2():05b}, rs1: {instr.rs1():05b}, funct3: {instr.funct3():03b}, rd: {instr.rd():05b}, opcode: {instr.opcode():07b}"}
                                                        }
                                                        if instr_frmt == InstructionFormat::I {
                                                            span {"Type: I, immediate: {instr.immediate().unwrap():12b}, rs1: {instr.rs1():05b}, funct3: {instr.funct3():03b}, rd: {instr.rd():05b}, opcode: {instr.opcode():07b}"}
                                                        }
                                                        if instr_frmt == InstructionFormat::S {
                                                            span {"Type: S, immediate: {instr.immediate().unwrap():12b}, rs2: {instr.rs1():05b}, rs1: {instr.rs1():05b}, funct3: {instr.funct3():03b}, opcode: {instr.opcode():07b}"}
                                                        }
                                                        if instr_frmt == InstructionFormat::B {
                                                            span {"Type: B, immediate: {instr.immediate().unwrap():12b}, rs2: {instr.rs1():05b}, rs1: {instr.rs1():05b}, funct3: {instr.funct3():03b}, opcode: {instr.opcode():07b}"}
                                                        }
                                                        if instr_frmt == InstructionFormat::U {
                                                            span {"Type: U, immediate: {instr.immediate().unwrap():20b}, rd: {instr.rd():05b}, opcode: {instr.opcode():07b}"}
                                                        }
                                                        if instr_frmt == InstructionFormat::J {
                                                            span {"Type: J, immediate: {instr.immediate().unwrap():20b}, rd: {instr.rd():05b}, opcode: {instr.opcode():07b}"}
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
