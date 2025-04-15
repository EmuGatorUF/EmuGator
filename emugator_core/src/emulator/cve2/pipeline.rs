use super::{
    controller::{CVE2Control, get_control_signals},
    datapath::CVE2Datapath,
};
use crate::{
    assembler::AssembledProgram,
    bitmask, bits,
    emulator::{
        PcPos, Pipeline, RegisterFile,
        controller_common::{DataDestSel, OpASel, OpBSel, PCSel},
        memory_module::MemoryModule,
        read_instruction,
    },
    isa::Instruction,
};

#[allow(non_snake_case)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CVE2Pipeline {
    pub IF_inst: Option<u32>, // Instruction Fetch Buffer
    pub IF_pc: u32,           // Program Counter for the IF stage
    pub ID_inst: Option<u32>, // Instruction Decode Buffer
    pub ID_pc: Option<u32>,   // Program Counter for the ID stage
    pub instr_cycle: u32,     // The number of cycles that this instruction has been in ID.
    pub datapath: CVE2Datapath,
    pub control: CVE2Control,
}

impl Pipeline for CVE2Pipeline {
    fn clock(
        &mut self,
        program: &AssembledProgram,
        registers: &mut RegisterFile,
        data_memory: &mut MemoryModule,
    ) {
        // Run the registers that had stuff to write in the last cycle
        // (this is done first to represent it taking a clock edge to write)
        self.run_pipeline_buffer_registers();
        self.run_pc_reg();
        self.run_data_memory(data_memory);
        self.run_write_register(registers);

        // Run the instruction fetch stage
        self.run_instruction_fetch(program);

        // Set control signals
        let Some(id_inst) = self.ID_inst else {
            // no id stage yet
            return;
        };
        let instr = Instruction::from_raw(id_inst);
        self.control = get_control_signals(instr, self.instr_cycle, self.datapath.cmp_result)
            .unwrap_or_default();

        // Decode the instruction
        self.run_decode(instr);

        // Read from register file
        self.run_read_registers(registers);

        // Operand muxes
        self.run_operand_muxes();

        // Run ALU
        self.run_alu();

        // Run LSU
        // (needs to go after ALU to get the address)
        self.run_lsu();

        // Write data mux
        self.run_write_data_mux();

        // Find the next PC
        self.run_cmp_reg();
        self.run_pc_mux();
    }

    fn set_if_pc(&mut self, address: u32, program: &AssembledProgram) {
        if address & 0x00000003 != 0x00 {
            panic!("PC must be on a 4-byte boundary");
        }
        self.IF_pc = address;
        self.run_pc_mux();
        self.run_instruction_fetch(program);
    }

    fn requesting_debug(&self) -> bool {
        self.control.debug_req
    }

    fn id_pc(&self) -> Option<u32> {
        self.ID_pc
    }

    fn all_pcs(&self) -> Vec<PcPos> {
        let mut pcs = Vec::new();
        pcs.push(PcPos::new(self.IF_pc, "if"));
        if let Some(id_pc) = self.ID_pc {
            pcs.push(PcPos::new(id_pc, "id"));
        }
        pcs
    }
}

impl CVE2Pipeline {
    fn run_instruction_fetch(&mut self, program: &AssembledProgram) {
        // Read the next instruction into the instruction fetch register
        self.IF_inst = read_instruction(&program.instruction_memory, self.IF_pc)
    }

    fn run_decode(&mut self, instr: Instruction) {
        self.datapath.reg_s1 = instr.rs1();
        self.datapath.reg_s2 = instr.rs2();
        self.datapath.reg_d = instr.rd();
        self.datapath.imm = instr.immediate().map(|x| x as u32);
    }

    fn run_read_registers(&mut self, register_file: &RegisterFile) {
        self.datapath.data_s1 = register_file[self.datapath.reg_s1 as usize];
        self.datapath.data_s2 = register_file[self.datapath.reg_s2 as usize];
    }

    fn run_operand_muxes(&mut self) {
        self.datapath.alu_op_a = match self.control.alu_op_a_sel {
            Some(OpASel::PC) => self.ID_pc,
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

    fn run_alu(&mut self) {
        let Some(a) = self.datapath.alu_op_a else {
            return;
        };
        let Some(b) = self.datapath.alu_op_b else {
            return;
        };

        self.datapath.alu_out = self.control.alu_op.map(|op| op.apply(a, b));
    }

    fn run_lsu(&mut self) {
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

    fn run_write_data_mux(&mut self) {
        self.datapath.reg_write_data = match self.control.data_dest_sel {
            Some(DataDestSel::ALU) => self.datapath.alu_out,
            Some(DataDestSel::LSU) => self.datapath.lsu_out,
            None => None,
        };
    }

    fn run_data_memory(&mut self, data_memory: &mut MemoryModule) {
        // Perform any requested memory read/write
        if self.datapath.data_req_o {
            if self.datapath.data_we_o {
                data_memory.write_word(
                    self.datapath.data_addr_o,
                    self.datapath.data_wdata_o,
                    self.datapath.data_be_o,
                );
                self.datapath.data_rdata_i = 0;
            } else {
                self.datapath.data_rdata_i =
                    data_memory.read_word(self.datapath.data_addr_o, self.datapath.data_be_o);
            }
            self.datapath.data_gnt_i = true;
            self.datapath.data_rvalid_i = true;
            self.datapath.data_err_i = false;
        } else {
            self.datapath.data_rdata_i = 0;
            self.datapath.data_gnt_i = false;
            self.datapath.data_rvalid_i = false;
            self.datapath.data_err_i = false;
        }
    }

    fn run_write_register(&mut self, register_file: &mut RegisterFile) {
        if self.control.reg_write {
            if let Some(data) = self.datapath.reg_write_data {
                register_file[self.datapath.reg_d as usize] = data;
            }
        }
    }

    fn run_cmp_reg(&mut self) {
        if self.control.cmp_set {
            self.datapath.cmp_result = self.datapath.alu_out.is_some_and(|x| x != 0);
        }
    }

    fn run_pc_mux(&mut self) {
        self.datapath.next_pc = match self.control.next_pc_sel {
            PCSel::PC4 => Some(self.IF_pc + 4),
            PCSel::JMP => self.datapath.alu_out,
        }
    }

    fn run_pc_reg(&mut self) {
        if self.control.pc_set {
            if let Some(next_pc) = self.datapath.next_pc {
                if next_pc & 0x00000003 != 0x00 {
                    panic!("PC must be on a 4-byte boundary");
                }
                self.IF_pc = next_pc;
            }
        }
    }

    fn run_pipeline_buffer_registers(&mut self) {
        // Move the pipeline forward
        if self.control.if_id_set {
            self.ID_pc = Some(self.IF_pc);
            self.ID_inst = self.IF_inst;
            self.instr_cycle = 0;
        } else {
            self.instr_cycle += 1;
        }
    }
}
