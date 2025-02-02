mod datapath;
mod handlers;

#[cfg(test)]
mod tests;

use crate::assembler::AssembledProgram;
use crate::isa::Instruction;
use std::{
    collections::BTreeMap,
    ops::{Index, IndexMut},
};

use datapath::CVE2Pipeline;
use handlers::get_handler;

pub type InstructionHandler = fn(&Instruction, &mut EmulatorState);

#[derive(Copy, Clone, Default, Debug)]
pub struct RegisterFile {
    pub x: [u32; 32],
}

impl Index<usize> for RegisterFile {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            return &0;
        } else {
            &self.x[index]
        }
    }
}

impl IndexMut<usize> for RegisterFile {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.x[0] = 0;
        &mut self.x[index]
    }
}

#[derive(Clone, Default, Debug)]
pub struct EmulatorState {
    pub x: RegisterFile,
    pub csr: BTreeMap<u32, u32>,
    pub pipeline: CVE2Pipeline,
}

fn rw_memory(memory: &mut BTreeMap<u32, u8>, address: u32, byte_enable: [bool; 4], wenable: bool, wdata: u32) -> Result<u32, ()> {
    let mut rdata_bytes: [u8; 4] = [0; 4];
    let wdata_bytes = wdata.to_le_bytes();
    let success = (0usize..4usize).all(|i| {
        if byte_enable[i] {
            let addr = address + i as u32;
            rdata_bytes[i] = if wenable {
                memory.insert(addr, wdata_bytes[i]).unwrap_or_default()
            } else {
                memory.get(&addr).copied().unwrap_or_default()
            };
            true 
        } else {
            true
        }
    });

    if success {
        return Ok(u32::from_le_bytes(rdata_bytes));
    } else {
        return Err(());
    }
}

pub fn clock(org_state: &EmulatorState, program: &mut AssembledProgram) -> EmulatorState {
    let mut next_state = org_state.clone();
    next_state.pipeline.datapath.debug_req_i = false;

    // Load the fetched instruction into the instr_rdata lines
    if next_state.pipeline.datapath.instr_req_o {
        // Read the next instruction into the instruction fetch register
        match rw_memory(
                &mut program.instruction_memory,
                next_state.pipeline.datapath.instr_addr_o,
                [true; 4],
                false,
                0,
            ) {
            Ok(instr) => {
                // if instruction fetched is default, i.e invalid
                if instr == 0 {
                    next_state.pipeline.datapath.instr_gnt_i = true;
                    next_state.pipeline.datapath.instr_rvalid_i = false;
                    next_state.pipeline.datapath.instr_err_i = true;
                } else {
                    next_state.pipeline.datapath.instr_rdata_i = instr;
                    next_state.pipeline.datapath.instr_gnt_i = true;
                    next_state.pipeline.datapath.instr_rvalid_i = true;
                    next_state.pipeline.datapath.instr_err_i = false;
                }

                next_state.pipeline.IF = next_state.pipeline.datapath.instr_rdata_i;
                next_state.pipeline.IF_pc = next_state.pipeline.datapath.instr_addr_o;
            }
            Err(_) => {
                next_state.pipeline.datapath.instr_gnt_i = true;
                next_state.pipeline.datapath.instr_rvalid_i = false;
                next_state.pipeline.datapath.instr_err_i = true;
            }
        }
    }

    // Decode the instruction in the instruction decode register
    let instr = Instruction::from_raw(next_state.pipeline.ID);

    match get_handler(instr) {
        Err(()) => println!("Invalid Instruction {}", instr.raw()),
        Ok(handler) => handler(&instr, &mut next_state),
    };

    // Perform any requested memory read/write
    if next_state.pipeline.datapath.data_req_o {
        match rw_memory(
            &mut program.data_memory,
            next_state.pipeline.datapath.data_addr_o,
            next_state.pipeline.datapath.data_be_o,
            next_state.pipeline.datapath.data_we_o,
            next_state.pipeline.datapath.data_wdata_o,
        ) {
            Ok(rdata) => {
                next_state.pipeline.datapath.data_rdata_i = rdata;
                next_state.pipeline.datapath.data_gnt_i = true;
                next_state.pipeline.datapath.data_rvalid_i = true;
                next_state.pipeline.datapath.data_err_i = false;
            }
            Err(_) => {
                next_state.pipeline.datapath.data_gnt_i = true;
                next_state.pipeline.datapath.data_rvalid_i = false;
                next_state.pipeline.datapath.data_err_i = true;
            }
        }
    }

    // Only load the next instruction if the fetch is enabled
    if next_state.pipeline.datapath.fetch_enable_i {
        next_state.pipeline.ID = next_state.pipeline.IF;
        next_state.pipeline.ID_pc = next_state.pipeline.IF_pc;
        next_state.pipeline.datapath.instr_addr_o += 4;
    } else {
        // if instruction is acutally a branch instruction, then the next fetch isn't gonna happen, so it isn't invalid.
        next_state.pipeline.datapath.instr_gnt_i = true;
        next_state.pipeline.datapath.instr_rvalid_i = true;
        next_state.pipeline.datapath.instr_err_i = false;
    }
    return next_state;
}
