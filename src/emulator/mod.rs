mod controller;
mod datapath;
mod pipeline;
mod register_file;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::assembler::AssembledProgram;
use crate::isa::Instruction;

use controller::get_control_signals;
use pipeline::CVE2Pipeline;
use register_file::RegisterFile;

#[derive(Clone, Default, Debug)]
pub struct EmulatorState {
    pub x: RegisterFile,
    pub csr: BTreeMap<u32, u32>,
    pub pipeline: CVE2Pipeline,
}

pub fn clock(org_state: &EmulatorState, program: &mut AssembledProgram) -> EmulatorState {
    let mut next_state = org_state.clone();

    // Set control signals
    let instr = Instruction::from_raw(next_state.pipeline.ID_inst);
    next_state.pipeline.control =
        get_control_signals(instr, next_state.pipeline.instr_cycle).unwrap_or_default();

    // Run data memory
    // (Do this with last cycle's LSU signals to represent it taking a clock cycle to access)
    next_state
        .pipeline
        .run_data_memory(&mut program.data_memory);

    // Run the instruction fetch stage
    next_state.pipeline.run_instruction_fetch(program);

    // Decode the instruction
    next_state.pipeline.run_decode(instr);

    // Read from register file
    next_state.pipeline.run_read_registers(&next_state.x);

    // Operand muxes
    next_state.pipeline.run_operand_muxes();

    // Run ALU
    next_state.pipeline.run_alu();

    // Run LSU
    // (needs to go after ALU to get the address)
    next_state.pipeline.run_lsu();

    // Write data mux
    next_state.pipeline.run_write_data_mux();

    // Write to register file
    next_state.pipeline.run_write_register(&mut next_state.x);

    // Pipeline buffer
    next_state.pipeline.run_pipeline_buffer_registers();

    // Find the next PC
    next_state.pipeline.run_cmp_reg();
    next_state.pipeline.run_pc_mux();
    next_state.pipeline.run_pc_reg();

    return next_state;
}
