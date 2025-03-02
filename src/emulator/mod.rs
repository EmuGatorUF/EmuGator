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
use pipeline::{ALUOp, CVE2Control, CVE2Pipeline, DataDestSel, OpASel, OpBSel, PCSel};

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

    // Set control signals
    let instr = Instruction::from_raw(next_state.pipeline.ID_inst);
    next_state.pipeline.control = CVE2Control::default();
    match get_handler(instr) {
        Err(()) => println!("Invalid Instruction {}", instr.raw()),
        Ok(handler) => handler(&instr, &mut next_state),
    };

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
    next_state.pipeline.run_pc_mux();
    next_state.pipeline.run_pc_reg();

    return next_state;
}

impl CVE2Pipeline {
    pub fn run_instruction_fetch(&mut self, program: &mut AssembledProgram) {
        // Load the fetched instruction into the instr_rdata lines
        if self.control.instr_req {
            // Read the next instruction into the instruction fetch register
            match rw_memory(
                &mut program.instruction_memory,
                self.IF_pc,
                [true; 4],
                false,
                0,
            ) {
                Ok(instr_data) => {
                    self.IF_inst = instr_data;
                    self.datapath.instr_gnt_i = true;
                    self.datapath.instr_rvalid_i = true;
                    self.datapath.instr_err_i = false;
                }
                Err(_) => {
                    self.datapath.instr_gnt_i = true;
                    self.datapath.instr_rvalid_i = false;
                    self.datapath.instr_err_i = true;
                }
            }
        }
    }

    pub fn run_decode(&mut self, instr: Instruction) {
        self.datapath.reg_s1 = instr.rs1();
        self.datapath.reg_s2 = instr.rs2();
        self.datapath.reg_d = instr.rd();
        self.datapath.imm = instr.immediate().ok().map(|x| x as u32); // FIXME: signed support
    }

    pub fn run_read_registers(&mut self, register_file: &RegisterFile) {
        self.datapath.data_s1 = register_file[self.datapath.reg_s1 as usize];
        self.datapath.data_s2 = register_file[self.datapath.reg_s2 as usize];
    }

    pub fn run_operand_muxes(&mut self) {
        self.datapath.alu_op_a = match self.control.alu_op_a_sel {
            Some(OpASel::PC) => Some(self.ID_pc),
            Some(OpASel::RF) => Some(self.datapath.data_s1),
            None => None,
        };
        self.datapath.alu_op_b = match self.control.alu_op_b_sel {
            Some(OpBSel::RF) => Some(self.datapath.data_s2),
            Some(OpBSel::IMM) => self.datapath.imm,
            Some(OpBSel::Four) => Some(4),
            None => None,
        };
    }

    pub fn run_alu(&mut self) {
        let Some(op) = self.control.alu_op else {
            return;
        };
        let Some(a) = self.datapath.alu_op_a else {
            return;
        };
        let Some(b) = self.datapath.alu_op_b else {
            return;
        };

        self.datapath.alu_out = Some(match op {
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
            ALUOp::SELB => b,
        });
    }

    pub fn run_lsu(&mut self) {
        // pass through data memory output
        self.datapath.lsu_out = if self.datapath.data_rvalid_i {
            let data = self.datapath.data_rdata_i;
            if self.control.lsu_sign_ext {
                // sign-extend the data
                let data_size = self
                    .control
                    .lsu_data_type
                    .map(|d| d.size_in_bits())
                    .unwrap_or(0);
                let sign_mask = bitmask!(31;data_size) * bits!(data, data_size - 1);
                Some(sign_mask | data)
            } else {
                Some(data)
            }
        } else {
            None
        };

        // pass through inputs to the data memory
        self.datapath.data_req_o = self.control.lsu_request;
        self.datapath.data_we_o = self.control.lsu_write_enable;
        self.datapath.data_addr_o = self.datapath.alu_out.unwrap_or_default();
        self.datapath.data_be_o = self
            .control
            .lsu_data_type
            .map(|d| d.byte_enable())
            .unwrap_or_default();
        self.datapath.data_wdata_o = self.datapath.data_s2;
    }

    pub fn run_write_data_mux(&mut self) {
        self.datapath.reg_write_data = match self.control.data_dest_sel {
            Some(DataDestSel::ALU) => self.datapath.alu_out,
            Some(DataDestSel::LSU) => self.datapath.lsu_out,
            None => None,
        };
    }

    pub fn run_data_memory(&mut self, data_memory: &mut BTreeMap<u32, u8>) {
        // Perform any requested memory read/write
        if self.datapath.data_req_o {
            match rw_memory(
                data_memory,
                self.datapath.data_addr_o,
                self.datapath.data_be_o,
                self.datapath.data_we_o,
                self.datapath.data_wdata_o,
            ) {
                Ok(rdata) => {
                    self.datapath.data_rdata_i = rdata;
                    self.datapath.data_gnt_i = true;
                    self.datapath.data_rvalid_i = true;
                    self.datapath.data_err_i = false;
                }
                Err(_) => {
                    self.datapath.data_rdata_i = 0;
                    self.datapath.data_gnt_i = true;
                    self.datapath.data_rvalid_i = false;
                    self.datapath.data_err_i = true;
                }
            }
        } else {
            self.datapath.data_rdata_i = 0;
            self.datapath.data_gnt_i = false;
            self.datapath.data_rvalid_i = false;
            self.datapath.data_err_i = false;
        }
    }

    pub fn run_write_register(&mut self, register_file: &mut RegisterFile) {
        if self.control.reg_write {
            if let Some(data) = self.datapath.reg_write_data {
                register_file[self.datapath.reg_d as usize] = data;
            }
        }
    }

    pub fn run_pc_mux(&mut self) {
        self.datapath.next_pc = match self.control.pc_sel {
            PCSel::PC4 => Some(self.ID_pc + 4),
            PCSel::ALU => self.datapath.reg_write_data,
        }
    }

    pub fn run_pc_reg(&mut self) {
        if self.control.pc_set {
            if let Some(next_pc) = self.datapath.next_pc {
                if next_pc & 0x00000003 != 0x00 {
                    panic!("JAL instruction immediate it not on a 4-byte boundary");
                }
                self.IF_pc = next_pc;
            }
        }
    }

    pub fn run_pipeline_buffer_registers(&mut self) {
        // Move the pipeline forward
        if self.control.id_in_ready {
            self.ID_pc = self.IF_pc;
            self.ID_inst = self.IF_inst;
            self.datapath.instr_first_cycle = true;
        } else {
            self.datapath.instr_first_cycle = false;
        }
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
