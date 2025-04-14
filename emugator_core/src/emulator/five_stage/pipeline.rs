use crate::assembler::AssembledProgram;
use crate::emulator::controller_common::{DataDestSel, OpASel, OpBSel, PCSel};
use crate::emulator::{PcPos, read_instruction};
use crate::emulator::{Pipeline, memory_module::MemoryModule, register_file::RegisterFile};
use crate::isa::Instruction;
use crate::{bitmask, bits};

use super::controller::FiveStageControl;
use super::datapath::{
    ExLines, ExMemBuffer, IdExBuffer, IdLines, IfIdBuffer, IfLines, MemLines, MemWbBuffer, WbLines,
};
use super::hazard_detection::HazardDetector;

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

    pub id_control: FiveStageControl,
    pub ex_control: FiveStageControl,
    pub mem_control: FiveStageControl,
    pub wb_control: FiveStageControl,

    pub hazard_detector: HazardDetector,
}

impl Pipeline for FiveStagePipeline {
    fn clock(
        &mut self,
        program: &AssembledProgram,
        registers: &mut RegisterFile,
        data_memory: &mut MemoryModule,
    ) {
        // Run the registers that had stuff to write in the last cycle
        // (this is done first to represent it taking a clock edge to write)
        // Pass the control buffers along last so these all run like they were in the previous cycle
        self.run_write_register(registers);
        self.run_pipeline_buffers();
        self.run_pc_reg();
        self.run_data_memory(data_memory);
        self.run_control_buffers();

        // Run the pipeline stages
        self.run_if(program);
        self.run_id(registers);
        self.run_ex();
        self.run_mem();
        self.run_wb();

        // run hazard detection
        self.hazard_detector.detect_hazards(
            &self.if_id.id_inst,
            self.ex_lines.alu_out.is_some_and(|x| x == 0) && self.ex_control.jump_cond,
        );
    }

    fn requesting_debug(&mut self) -> bool {
        self.id_control.debug_req
    }

    fn set_if_pc(&mut self, address: u32, program: &AssembledProgram) {
        if address & 0x00000003 != 0x00 {
            panic!("PC must be on a 4-byte boundary");
        }
        self.if_pc = address;
        self.run_if(program);
    }

    fn id_pc(&self) -> Option<u32> {
        self.if_id.id_pc
    }

    fn all_pcs(&self) -> Vec<PcPos> {
        let mut pcs = Vec::new();
        pcs.push(PcPos::new(self.if_pc, "if"));

        if let Some(id_pc) = self.if_id.id_pc {
            pcs.push(PcPos::new(id_pc, "id"));
        }

        if let Some(ex_pc) = self.id_ex.ex_pc {
            pcs.push(PcPos::new(ex_pc, "ex"));
        }

        if let Some(mem_pc) = self.ex_mem.mem_pc {
            pcs.push(PcPos::new(mem_pc, "mem"));
        }

        if let Some(wb_pc) = self.mem_wb.wb_pc {
            pcs.push(PcPos::new(wb_pc, "wb"));
        }

        pcs
    }
}

impl FiveStagePipeline {
    /* ---------------------------- Instruction Fetch --------------------------- */
    fn run_if(&mut self, program: &AssembledProgram) {
        self.if_lines.instr = read_instruction(&program.instruction_memory, self.if_pc);
        self.run_pc_mux();
    }

    fn run_pc_mux(&mut self) {
        let control = self.ex_control;
        let cmp_result = self.ex_lines.alu_out.is_some_and(|x| x != 0);
        let should_cond_jump = control.jump_cond && cmp_result;
        let should_jump = control.jump_uncond || should_cond_jump;
        self.if_lines.next_pc_sel = if should_jump { PCSel::JMP } else { PCSel::PC4 };
        self.if_lines.next_pc = match self.if_lines.next_pc_sel {
            PCSel::PC4 => Some(self.if_pc + 4),
            PCSel::JMP => self.ex_lines.jmp_dst,
        }
    }

    /* --------------------------- Instruction Decode --------------------------- */

    fn run_id(&mut self, registers: &RegisterFile) {
        let Some(id_inst) = self.if_id.id_inst else {
            // no id stage yet
            return;
        };
        let instr = Instruction::from_raw(id_inst);

        // get control signals
        self.id_control = FiveStageControl::for_instr(instr).unwrap_or_default();

        // run decoder
        self.id_lines.rs1 = instr.rs1();
        self.id_lines.rs2 = instr.rs2();
        self.id_lines.rd = instr.rd();
        self.id_lines.imm = instr.immediate().map(|x| x as u32);

        // read from register file
        self.id_lines.rs1_v = registers[self.id_lines.rs1 as usize];
        self.id_lines.rs2_v = registers[self.id_lines.rs2 as usize];
    }

    /* --------------------------------- Execute -------------------------------- */

    fn run_ex(&mut self) {
        self.run_alu_mux();
        self.run_alu();
        self.run_dest_adder();
        self.run_pc_mux(); // run again in case things changed
    }

    fn run_alu_mux(&mut self) {
        self.ex_lines.op_a = match self.ex_control.alu_op_a_sel {
            Some(OpASel::PC) => self.id_ex.ex_pc,
            Some(OpASel::RF) => Some(self.id_ex.rs1_v),
            None => None,
        };
        self.ex_lines.op_b = match self.ex_control.alu_op_b_sel {
            Some(OpBSel::RF) => Some(self.id_ex.rs2_v),
            Some(OpBSel::IMM) => self.id_ex.imm,
            Some(OpBSel::Four) => Some(4),
            None => None,
        };
    }

    fn run_alu(&mut self) {
        self.ex_lines.alu_out = match (
            self.ex_control.alu_op,
            self.ex_lines.op_a,
            self.ex_lines.op_b,
        ) {
            (Some(op), Some(a), Some(b)) => Some(op.apply(a, b)),
            _ => None,
        };
    }

    fn run_dest_adder(&mut self) {
        // base address mux
        self.ex_lines.jmp_base = match self.ex_control.jmp_base {
            Some(OpASel::PC) => self.id_ex.ex_pc,
            Some(OpASel::RF) => Some(self.id_ex.rs1_v),
            None => None,
        };

        // adder
        self.ex_lines.jmp_dst = match (self.ex_lines.jmp_base, self.id_ex.imm) {
            (Some(base), Some(imm)) => Some(base.wrapping_add(imm)),
            _ => None,
        };
    }

    /* --------------------------------- Memory --------------------------------- */

    fn run_mem(&mut self) {
        // TODO: see if there is some way to share this with cve2
        // pass through data memory output
        self.mem_lines.mem_data = if self.mem_lines.data_rvalid_i {
            let data = self.mem_lines.data_rdata_i;
            let data_size = self
                .mem_control
                .lsu_data_type
                .map(|d| d.size_in_bits())
                .unwrap_or(0);
            if self.mem_control.lsu_sign_ext && data_size < 32 {
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
        if self.mem_lines.data_gnt_i {
            // if the data memory accepted the request, don't request again here
            // since this is two cycles, it will never be true in the first cycle
            self.mem_lines.data_req_o = false;
        } else {
            self.mem_lines.data_req_o = self.mem_control.lsu_request;
        }
        self.mem_lines.data_we_o = self.mem_control.lsu_write_enable;
        self.mem_lines.data_addr_o = self.ex_mem.alu_o.unwrap_or_default();
        self.mem_lines.data_be_o = self
            .mem_control
            .lsu_data_type
            .map(|d| d.byte_enable())
            .unwrap_or_default();
        self.mem_lines.data_wdata_o = self.ex_mem.rs2_v;
    }

    /* ------------------------------- Write Back ------------------------------- */

    fn run_wb(&mut self) {
        // select source
        self.wb_lines.wb_data = self.wb_control.wb_src.and_then(|s| match s {
            DataDestSel::ALU => self.mem_wb.alu,
            DataDestSel::LSU => self.mem_wb.lsu,
        });
    }

    /* ---------------------------- Clocked Registers --------------------------- */

    fn run_pipeline_buffers(&mut self) {
        self.mem_wb = MemWbBuffer {
            wb_pc: self.ex_mem.mem_pc,
            alu: self.ex_mem.alu_o,
            lsu: self.mem_lines.mem_data,
            rd: self.ex_mem.rd,
        };

        self.ex_mem = ExMemBuffer {
            mem_pc: self.id_ex.ex_pc,
            alu_o: self.ex_lines.alu_out,
            rs2_v: self.id_ex.rs2_v,
            rd: self.id_ex.rd,
        };

        if !self.hazard_detector.hazard_detected.stop_ex {
            self.id_ex = IdExBuffer {
                ex_pc: self.if_id.id_pc,
                rs1_v: self.id_lines.rs1_v,
                rs2_v: self.id_lines.rs2_v,
                imm: self.id_lines.imm,
                rd: Some(self.id_lines.rd),
            };
        } else {
            // to stall, clear the ID-EX buffer to send a no op
            self.id_ex = IdExBuffer::default();
        }

        if !self.hazard_detector.hazard_detected.stop_id {
            self.if_id = IfIdBuffer {
                id_pc: Some(self.if_pc),
                id_inst: self.if_lines.instr,
            };
        }
    }

    fn run_control_buffers(&mut self) {
        self.wb_control = self.mem_control;
        self.mem_control = self.ex_control;

        if !self.hazard_detector.hazard_detected.stop_ex {
            self.ex_control = self.id_control;
        } else {
            // to stall, clear the ex_control buffer
            self.ex_control = FiveStageControl::default();
        }
    }

    fn run_pc_reg(&mut self) {
        if !self.hazard_detector.hazard_detected.stop_if {
            if let Some(next_pc) = self.if_lines.next_pc {
                if next_pc & 0x00000003 != 0x00 {
                    panic!("PC must be on a 4-byte boundary");
                }
                self.if_pc = next_pc;
            }
        }
    }

    fn run_write_register(&self, registers: &mut RegisterFile) {
        if self.wb_control.reg_write {
            if let (Some(data), Some(rd)) = (self.wb_lines.wb_data, self.mem_wb.rd) {
                // FIXME: why is rd getting the value from the next instruction
                registers[rd as usize] = data;
            }
        }
    }

    fn run_data_memory(&mut self, data_memory: &mut MemoryModule) {
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
