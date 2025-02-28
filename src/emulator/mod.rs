mod handlers;
mod pipeline;

#[cfg(test)]
mod tests;

use crate::bitmask;
use crate::isa::Instruction;
use crate::{assembler::AssembledProgram, bits};
use std::{
    collections::BTreeMap,
    ops::{Index, IndexMut},
};

use handlers::get_handler;
use pipeline::{
    ALUOp, CVE2Control, CVE2Datapath, CVE2Pipeline, DataDestSel, LSUDataType, OpASel, OpBSel,
};

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

pub fn clock(org_state: &EmulatorState, program: &mut AssembledProgram) -> EmulatorState {
    let mut next_state = org_state.clone();

    // Reset control signals
    next_state.pipeline.control = CVE2Control::default();

    // Run the instruction fetch stage
    next_state.pipeline.datapath.run_instruction_fetch(
        program,
        &mut next_state.pipeline.IF,
        &mut next_state.pipeline.IF_pc,
    );

    // Run data memory
    // (Do this with last cycle's LSU signals to represent it taking a clock cycle to access)
    next_state
        .pipeline
        .datapath
        .run_data_memory(&mut program.data_memory);

    // Decode the instruction in the instruction decode register
    let instr = Instruction::from_raw(next_state.pipeline.ID);
    if instr.is_valid() {
        // Decode the instruction
        next_state.pipeline.datapath.run_decode(instr);

        // Run handler to populate control signals
        match get_handler(instr) {
            Err(()) => println!("Invalid Instruction {}", instr.raw()),
            Ok(handler) => handler(&instr, &mut next_state),
        };

        // Read from register file
        next_state
            .pipeline
            .datapath
            .run_read_registers(&next_state.x);

        // Operand muxes
        next_state
            .pipeline
            .datapath
            .run_operand_muxes(next_state.pipeline.ID_pc, &next_state.pipeline.control);

        // Run ALU
        next_state
            .pipeline
            .datapath
            .run_alu(next_state.pipeline.control.alu_op);

        // Run LSU
        // (needs to go after ALU to get the address)
        next_state
            .pipeline
            .datapath
            .run_lsu(&next_state.pipeline.control);

        // Write data mux
        next_state
            .pipeline
            .datapath
            .run_write_data_mux(next_state.pipeline.control.data_dest_sel);

        // Write to register file
        next_state
            .pipeline
            .datapath
            .run_write_register(&mut next_state.x, next_state.pipeline.control.reg_write);

        // Find the next PC
        next_state.pipeline.datapath.run_pc_mux();
    }

    // Only load the next instruction if the fetch is enabled
    if next_state.pipeline.datapath.fetch_enable_i {
        next_state.pipeline.ID = next_state.pipeline.IF;
        next_state.pipeline.ID_pc = next_state.pipeline.IF_pc;
        next_state.pipeline.datapath.instr_addr_o += 4;
    }
    return next_state;
}

impl CVE2Datapath {
    pub fn run_instruction_fetch(
        &mut self,
        program: &mut AssembledProgram,
        instr: &mut u32,
        pc: &mut u32,
    ) {
        // Load the fetched instruction into the instr_rdata lines
        if self.instr_req_o {
            // Read the next instruction into the instruction fetch register
            match rw_memory(
                &mut program.instruction_memory,
                self.instr_addr_o,
                [true; 4],
                false,
                0,
            ) {
                Ok(instr_data) => {
                    self.instr_rdata_i = instr_data;
                    self.instr_gnt_i = true;
                    self.instr_rvalid_i = true;
                    self.instr_err_i = false;

                    *instr = self.instr_rdata_i;
                    *pc = self.instr_addr_o;
                }
                Err(_) => {
                    self.instr_gnt_i = true;
                    self.instr_rvalid_i = false;
                    self.instr_err_i = true;
                }
            }
        }
    }

    pub fn run_decode(&mut self, instr: Instruction) {
        self.reg_s1 = instr.rs1();
        self.reg_s2 = instr.rs2();
        self.reg_d = instr.rd();
        self.imm = instr.immediate().ok().map(|x| x as u32); // FIXME: signed support
    }

    pub fn run_read_registers(&mut self, register_file: &RegisterFile) {
        self.data_s1 = register_file[self.reg_s1 as usize];
        self.data_s2 = register_file[self.reg_s2 as usize];
    }

    pub fn run_operand_muxes(&mut self, pc: u32, control: &CVE2Control) {
        self.alu_op_a = match control.alu_op_a_sel {
            Some(OpASel::PC) => Some(pc),
            Some(OpASel::RF) => Some(self.data_s1),
            None => None,
        };
        self.alu_op_b = match control.alu_op_b_sel {
            Some(OpBSel::RF) => Some(self.data_s2),
            Some(OpBSel::IMM) => self.imm,
            None => None,
        };
    }

    pub fn run_alu(&mut self, op: Option<ALUOp>) {
        let Some(op) = op else { return };
        let Some(a) = self.alu_op_a else { return };
        let Some(b) = self.alu_op_b else { return };

        self.alu_out = Some(match op {
            ALUOp::ADD => ((a as i32) + (b as i32)) as u32,
            ALUOp::SUB => ((a as i32) - (b as i32)) as u32,
            ALUOp::XOR => a ^ b,
            ALUOp::OR => a | b,
            ALUOp::AND => a & b,
            ALUOp::SLL => a << (b & 0x1F),
            ALUOp::SRL => a >> (b & 0x1F),
            ALUOp::SRA => ((a as i32) >> (b & 0x1F)) as u32,
            ALUOp::SLT => ((a as i32) < (b as i32)) as u32,
            ALUOp::SLTU => (a < b) as u32,
        });
    }

    pub fn run_lsu(&mut self, control: &CVE2Control) {
        // pass through data memory output
        self.lsu_out = if self.data_rvalid_i {
            let data = self.data_rdata_i;
            if control.lsu_sign_ext {
                // sign-extend the data
                let data_size = control.lsu_data_type.map(|d| d.size_in_bits()).unwrap_or(0);
                let sign_mask = bitmask!(31;data_size) * bits!(data, data_size - 1);
                Some(sign_mask | data)
            } else {
                Some(data)
            }
        } else {
            None
        };

        // pass through inputs to the data memory
        self.data_req_o = control.lsu_request;
        self.data_we_o = control.lsu_write_enable;
        self.data_addr_o = self.alu_out.unwrap_or_default();
        self.data_be_o = control
            .lsu_data_type
            .map(|d| d.byte_enable())
            .unwrap_or_default();
        self.data_wdata_o = self.data_s2;
    }

    pub fn run_write_data_mux(&mut self, sel: Option<DataDestSel>) {
        self.reg_write_data = match sel {
            Some(DataDestSel::ALU) => self.alu_out,
            Some(DataDestSel::LSU) => self.lsu_out,
            None => None,
        };
    }

    pub fn run_data_memory(&mut self, data_memory: &mut BTreeMap<u32, u8>) {
        // Perform any requested memory read/write
        if self.data_req_o {
            match rw_memory(
                data_memory,
                self.data_addr_o,
                self.data_be_o,
                self.data_we_o,
                self.data_wdata_o,
            ) {
                Ok(rdata) => {
                    self.data_rdata_i = rdata;
                    self.data_gnt_i = true;
                    self.data_rvalid_i = true;
                    self.data_err_i = false;
                }
                Err(_) => {
                    self.data_rdata_i = 0;
                    self.data_gnt_i = true;
                    self.data_rvalid_i = false;
                    self.data_err_i = true;
                }
            }
        } else {
            self.data_rdata_i = 0;
            self.data_gnt_i = false;
            self.data_rvalid_i = false;
            self.data_err_i = false;
        }
    }

    pub fn run_write_register(&mut self, register_file: &mut RegisterFile, write_enable: bool) {
        if write_enable {
            if let Some(data) = self.reg_write_data {
                register_file[self.reg_d as usize] = data;
            }
        }
    }

    pub fn run_pc_mux(&mut self) {
        // TODO
    }
}

fn rw_memory(
    memory: &mut BTreeMap<u32, u8>,
    address: u32,
    byte_enable: [bool; 4],
    wenable: bool,
    wdata: u32,
) -> Result<u32, ()> {
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
