use std::collections::BTreeMap;

use crate::{assembler::AssembledProgram, bitmask, bits, isa::Instruction};

use super::{
    controller::{ALUOp, CVE2Control, DataDestSel, OpASel, OpBSel},
    datapath::{CVE2Datapath, PCSel},
    register_file::RegisterFile,
};

#[allow(non_snake_case)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CVE2Pipeline {
    pub IF_inst: u32,     // Instruction Fetch Buffer
    pub IF_pc: u32,       // Program Counter for the IF stage
    pub ID_inst: u32,     // Instruction Decode Buffer
    pub ID_pc: u32,       // Program Counter for the ID stage
    pub instr_cycle: u32, // The number of cycles that this instruction has been in ID.
    pub datapath: CVE2Datapath,
    pub control: CVE2Control,
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
            ALUOp::EQ => (a == b) as u32,
            ALUOp::NEQ => (a != b) as u32,
            ALUOp::LT => ((a as i32) < (b as i32)) as u32,
            ALUOp::GE => ((a as i32) >= (b as i32)) as u32,
            ALUOp::LTU => (a < b) as u32,
            ALUOp::GEU => (a >= b) as u32,
            ALUOp::SELB => b,
        });
    }

    pub fn run_lsu(&mut self) {
        // pass through data memory output
        self.datapath.lsu_out = if self.datapath.data_rvalid_i {
            let data = self.datapath.data_rdata_i;
            let data_size = self
                .control
                .lsu_data_type
                .map(|d| d.size_in_bits())
                .unwrap_or(0);
            if self.control.lsu_sign_ext && data_size < 32 {
                // sign-extend the data
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

    pub fn run_cmp_reg(&mut self) {
        if self.control.cmp_set {
            self.datapath.cmp_result = self.datapath.alu_out.is_some_and(|x| x != 0);
        }
    }

    pub fn run_pc_mux(&mut self) {
        self.datapath.should_cond_jump = self.control.jump_cond && self.datapath.cmp_result;
        let should_jump = self.control.jump_uncond || self.datapath.should_cond_jump;
        self.datapath.next_pc_sel = if should_jump { PCSel::ALU } else { PCSel::PC4 };
        self.datapath.next_pc = match self.datapath.next_pc_sel {
            PCSel::PC4 => Some(self.ID_pc + 4),
            PCSel::ALU => self.datapath.alu_out,
        }
    }

    pub fn run_pc_reg(&mut self) {
        if self.control.pc_set {
            if let Some(next_pc) = self.datapath.next_pc {
                if next_pc & 0x00000003 != 0x00 {
                    panic!("PC must be on a 4-byte boundary");
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
            self.instr_cycle = 0;
        } else {
            self.instr_cycle += 1;
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
        Ok(u32::from_le_bytes(rdata_bytes))
    } else {
        Err(())
    }
}
