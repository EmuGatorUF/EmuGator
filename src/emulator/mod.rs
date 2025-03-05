mod controller;
mod datapath;
mod pipeline;
mod register_file;
pub mod uart;

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use crate::assembler::AssembledProgram;
use crate::isa::Instruction;
use uart::{Uart, trigger_uart};

use controller::get_control_signals;
use pipeline::CVE2Pipeline;
use register_file::RegisterFile;

#[derive(Clone, Default, Debug)]
pub struct EmulatorState {
    pub x: RegisterFile,
    pub pipeline: CVE2Pipeline,
}

pub fn clock_until_break(
    org_state: &EmulatorState,
    program: &mut AssembledProgram,
    breakpoints: &BTreeSet<usize>,
    uart_module: &Uart,
) -> (EmulatorState, Uart) {
    let mut state = org_state.clone();
    let mut uart_module = uart_module.clone();
    let mut num_cycles = 0;

    loop {
        (state, uart_module) = clock(&state, program, &uart_module);

        let hit_breakpoint =
            if let Some(line_num) = program.source_map.get_by_left(&state.pipeline.IF_pc) {
                breakpoints.contains(line_num)
            } else {
                false
            };
        let hit_ebreak = state.pipeline.control.debug_req;

        if hit_ebreak || hit_breakpoint {
            break;
        }

        // max 1000 cycles until we can move this to a web worker
        num_cycles += 1;
        if num_cycles > 1000 {
            break;
        }
    }
    (state, uart_module)
}

pub fn clock(
    org_state: &EmulatorState,
    program: &mut AssembledProgram,
    uart_module: &Uart,
) -> (EmulatorState, Uart) {
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

    // UART
    let next_uart = trigger_uart(uart_module, &mut program.data_memory);

    return (next_state, next_uart);
}

#[allow(dead_code)]
pub fn clock_no_uart(org_state: &EmulatorState, program: &mut AssembledProgram) -> EmulatorState {
    let temp_uart = Uart::default();
    let (next_state, _) = clock(org_state, program, &temp_uart);
    next_state
}
