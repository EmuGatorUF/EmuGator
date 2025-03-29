use crate::assembler::AssembledProgram;
use crate::emulator::controller_common::{DataDestSel, OpASel, OpBSel, PCSel};
use crate::emulator::read_instruction;
use crate::emulator::{Pipeline, data_memory::DataMemory, register_file::RegisterFile};
use crate::isa::Instruction;
use crate::{bitmask, bits};

use super::controller::FiveStageControl;
use super::datapath::{
    ExLines, ExMemBuffer, IdExBuffer, IdLines, IfIdBuffer, IfLines, MemLines, MemWbBuffer, WbLines,
};

#[derive(Clone, Default, Debug)]
pub struct FiveStagePipeline {
    pub if_pc: u32,
    pub if_id: IfIdBuffer,
    pub id_ex: IdExBuffer,
    pub ex_mem: ExMemBuffer,
    pub mem_wb: MemWbBuffer,

    pub if_lines: IfLines,
    pub id_lines: IdLines,
    pub ex_lines: ExLines,
    pub mem_lines: MemLines,
    pub wb_lines: WbLines,
}

impl Pipeline for FiveStagePipeline {
    fn clock(
        &mut self,
        program: &AssembledProgram,
        registers: &mut RegisterFile,
        data_memory: &mut DataMemory,
    ) {
        self.run_data_memory(data_memory); // run seperate from LSU so takes mulitple cycles
        self.run_pipeline_buffers(); // FIXME: this is probably incorrectly placed
        self.run_if(program);
        self.run_id(registers);
        self.run_ex();
        self.run_mem();
        self.run_wb(registers);
    }

    fn requesting_debug(&mut self) -> bool {
        todo!()
    }

    fn if_pc(&mut self) -> &mut u32 {
        &mut self.if_pc
    }
}

impl FiveStagePipeline {
    fn run_pipeline_buffers(&mut self) {
        self.mem_wb = MemWbBuffer {
            wb_control: self.ex_mem.mem_control,
            wb_pc: self.ex_mem.mem_pc,
            alu: self.ex_mem.alu_o,
            lsu: self.mem_lines.rd_data,
        };

        self.ex_mem = ExMemBuffer {
            mem_control: self.id_ex.ex_control,
            mem_pc: self.id_ex.ex_pc,
            alu_o: self.ex_lines.alu_out,
            rs2_v: self.id_ex.rs2_v,
        };

        self.id_ex = IdExBuffer {
            ex_control: self.id_lines.id_control,
            ex_pc: self.if_id.id_pc,
            br_dst: self.id_lines.br_dst,
            rs1_v: self.id_lines.rs1_v,
            rs2_v: self.id_lines.rs2_v,
            imm: self.id_lines.imm,
        };

        self.if_id = IfIdBuffer {
            id_pc: self.if_pc,
            id_inst: self.if_lines.instr,
        };

        // FIXME: conditional based on the control signal
        if let Some(next_pc) = self.ex_lines.next_pc {
            // FIXME: ensure alignment
            self.if_pc = next_pc;
        }
    }

    fn run_if(&mut self, program: &AssembledProgram) {
        match read_instruction(&program.instruction_memory, self.if_pc) {
            Some(instr_data) => {
                self.if_lines.instr = instr_data;
                self.if_lines.instr_read_err = false;
            }
            None => {
                self.if_lines.instr_read_err = true;
            }
        }
    }

    fn run_id(&mut self, registers: &RegisterFile) {
        let instr = Instruction::from_raw(self.if_id.id_inst);

        // get control signals
        self.id_lines.id_control = FiveStageControl::for_instr(instr).unwrap_or_default();

        // run decoder
        self.id_lines.rs1 = instr.rs1();
        self.id_lines.rs2 = instr.rs2();
        self.id_lines.rd = instr.rd();
        self.id_lines.imm = instr.immediate().ok().map(|x| x as u32);

        // TODO: handle branch destination?

        // read from register file
        self.id_lines.rs1_v = registers[self.id_lines.rs1 as usize];
        self.id_lines.rs2_v = registers[self.id_lines.rs2 as usize];
    }

    fn run_ex(&mut self) {
        // find operands
        self.ex_lines.op_b = match self.id_ex.ex_control.alu_op_a_sel {
            Some(OpASel::PC) => Some(self.id_ex.ex_pc),
            Some(OpASel::RF) => Some(self.id_ex.rs1_v),
            None => None,
        };
        self.ex_lines.op_b = match self.id_ex.ex_control.alu_op_b_sel {
            Some(OpBSel::RF) => Some(self.id_ex.rs1_v),
            Some(OpBSel::IMM) => self.id_ex.imm,
            Some(OpBSel::Four) => Some(4),
            None => None,
        };

        // run ALU
        let Some(a) = self.ex_lines.op_a else {
            return;
        };
        let Some(b) = self.ex_lines.op_b else {
            return;
        };
        self.ex_lines.alu_out = self.id_ex.ex_control.alu_op.map(|op| op.apply(a, b));

        // TODO: branch unit

        // find the next pc
        self.ex_lines.next_pc = match self.ex_lines.next_pc_sel {
            PCSel::PC4 => Some(self.if_pc + 4),
            PCSel::JMP => Some(self.ex_lines.jmp_dst.unwrap_or(0)),
        };
    }

    fn run_mem(&mut self) {
        // TODO: see if there is some way to share this with cve2
        // pass through data memory output
        self.mem_lines.rd_data = if self.mem_lines.data_rvalid_i {
            let data = self.mem_lines.data_rdata_i;
            let data_size = self
                .ex_mem
                .mem_control
                .lsu_data_type
                .map(|d| d.size_in_bits())
                .unwrap_or(0);
            if self.ex_mem.mem_control.lsu_sign_ext && data_size < 32 {
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
        self.mem_lines.data_req_o = self.ex_mem.mem_control.lsu_request;
        self.mem_lines.data_we_o = self.ex_mem.mem_control.lsu_write_enable;
        self.mem_lines.data_addr_o = self.ex_mem.alu_o.unwrap_or_default();
        self.mem_lines.data_be_o = self
            .ex_mem
            .mem_control
            .lsu_data_type
            .map(|d| d.byte_enable())
            .unwrap_or_default();
        self.mem_lines.data_wdata_o = self.ex_mem.rs2_v;
    }

    fn run_wb(&mut self, registers: &mut RegisterFile) {
        // select source
        self.wb_lines.wb_data = self
            .mem_wb
            .wb_control
            .wb_src
            .map(|s| match s {
                DataDestSel::ALU => self.mem_wb.alu,
                DataDestSel::LSU => self.mem_wb.lsu,
            })
            .flatten();

        // write to register file
        if self.mem_wb.wb_control.reg_write {
            if let Some(data) = self.wb_lines.wb_data {
                registers[self.id_lines.rd as usize] = data;
            }
        }
    }

    fn run_data_memory(&mut self, data_memory: &mut DataMemory) {
        if self.mem_lines.data_req_o {
            if self.mem_lines.data_we_o {
                data_memory.write_word(
                    self.mem_lines.data_addr_o,
                    self.mem_lines.data_wdata_o,
                    self.mem_lines.data_be_o,
                );
                self.mem_lines.data_rdata_i = 0;
            } else {
                self.mem_lines.data_rdata_i =
                    data_memory.read_word(self.mem_lines.data_addr_o, self.mem_lines.data_be_o);
            }
            self.mem_lines.data_gnt_i = true;
            self.mem_lines.data_rvalid_i = true;
            self.mem_lines.data_err_i = false;
        } else {
            self.mem_lines.data_rdata_i = 0;
            self.mem_lines.data_gnt_i = false;
            self.mem_lines.data_rvalid_i = false;
            self.mem_lines.data_err_i = false;
        }
    }
}
