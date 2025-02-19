#![allow(non_snake_case)]

use super::pipeline::{ALUOp, CVE2Control, DataDestSel, OpASel, OpBSel};
use super::{EmulatorState, InstructionHandler};
use crate::isa::Instruction;
use crate::{bitmask, bits};

pub fn get_handler(instr: Instruction) -> Result<InstructionHandler, ()> {
    match (instr.opcode(), instr.funct3(), instr.funct7()) {
        (0b0110111, _, _) => Ok(LUI),
        (0b0010111, _, _) => Ok(AUIPC),
        (0b1101111, _, _) => Ok(JAL),
        (0b1100111, _, _) => Ok(JALR),
        (0b1100011, 0b000, _) => Ok(BEQ),
        (0b1100011, 0b001, _) => Ok(BNE),
        (0b1100011, 0b100, _) => Ok(BLT),
        (0b1100011, 0b101, _) => Ok(BGE),
        (0b1100011, 0b110, _) => Ok(BLTU),
        (0b1100011, 0b111, _) => Ok(BGEU),
        (0b0000011, 0b000, _) => Ok(LB),
        (0b0000011, 0b001, _) => Ok(LH),
        (0b0000011, 0b010, _) => Ok(LW),
        (0b0000011, 0b100, _) => Ok(LBU),
        (0b0000011, 0b101, _) => Ok(LHU),
        (0b0100011, 0b000, _) => Ok(SB),
        (0b0100011, 0b001, _) => Ok(SH),
        (0b0100011, 0b010, _) => Ok(SW),
        (0b0010011, 0b000, _) => Ok(ADDI),
        (0b0010011, 0b010, _) => Ok(SLTI),
        (0b0010011, 0b011, _) => Ok(SLTIU),
        (0b0010011, 0b100, _) => Ok(XORI),
        (0b0010011, 0b110, _) => Ok(ORI),
        (0b0010011, 0b111, _) => Ok(ANDI),
        (0b0010011, 0b001, _) => Ok(SLLI),
        (0b0010011, 0b101, _) => Ok(SRxI),
        (0b0110011, 0b000, 0b0000000) => Ok(ADD),
        (0b0110011, 0b000, 0b0100000) => Ok(SUB),
        (0b0110011, 0b001, 0b0000000) => Ok(SLL),
        (0b0110011, 0b010, 0b0000000) => Ok(SLT),
        (0b0110011, 0b011, 0b0000000) => Ok(SLTU),
        (0b0110011, 0b100, 0b0000000) => Ok(XOR),
        (0b0110011, 0b101, 0b0000000) => Ok(SRL),
        (0b0110011, 0b101, 0b0100000) => Ok(SRA),
        (0b0110011, 0b110, 0b0000000) => Ok(OR),
        (0b0110011, 0b111, 0b0000000) => Ok(AND),
        (0b0001111, 0b000, _) => match instr.raw() {
            0b1000_0011_0011_00000_000_00000_0001111 => Ok(FENCE_TSO),
            0b0000_0001_0000_00000_000_00000_0001111 => Ok(PAUSE),
            _ => Ok(FENCE),
        },
        (0b1110011, 0b000, 0b0000000) => match instr.raw() {
            0b0000_0000_0000_00000_000_00000_1110011 => Ok(ECALL),
            0b0000_0000_0001_00000_000_00000_1110011 => Ok(EBREAK),
            _ => Err(()),
        },
        (0b1110011, 0b001, _) => Ok(CSRRW),
        (0b1110011, 0b010, _) => Ok(CSRRS),
        (0b1110011, 0b011, _) => Ok(CSRRC),
        (0b1110011, 0b101, _) => Ok(CSRRWI),
        (0b1110011, 0b110, _) => Ok(CSRRSI),
        (0b1110011, 0b111, _) => Ok(CSRRCI),
        _ => Err(()),
    }
}

fn LUI(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    // immediate should already be shifted appropriate amount by encoder
    let immediate = instr.immediate().unwrap() as i32;

    state.x[rd] = immediate as u32;
}

fn AUIPC(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;

    // immediate should already be shifted by the appropriate amount by encoder
    let immediate = instr.immediate().unwrap() as i32;
    let result = state.pipeline.ID_pc as i32 + immediate;

    state.x[rd] = result as u32;
}

fn JAL(instr: &Instruction, state: &mut EmulatorState) {
    // TODO: Push onto Return Address stack when rd = x1 or x5
    if state.pipeline.datapath.id_multicycle == 0 {
        let immed = (instr.immediate()).unwrap();
        let new_pc = state.pipeline.ID_pc.checked_add_signed(immed).unwrap();

        // if unaligned on 4-byte boundary
        if new_pc & 0x00000003 != 0x00 {
            panic!("JAL instruction immediate it not on a 4-byte boundary");
        }
        // stores pc+4 into rd
        let rd = instr.rd() as usize;
        state.x[rd] = state.pipeline.ID_pc + 4;

        // update PC
        state.pipeline.datapath.instr_addr_o = new_pc;
        state.pipeline.datapath.fetch_enable_i = false;
        state.pipeline.datapath.id_multicycle = 1;
    } else {
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn JALR(instr: &Instruction, state: &mut EmulatorState) {
    // TODO: Push onto RAS
    if state.pipeline.datapath.id_multicycle == 0 {
        let immed = (instr.immediate()).unwrap();
        let new_pc = (state.x[instr.rs1() as usize] as i32 + immed) as u32 & bitmask!(31;1);

        // if unaligned on 4-byte boundary
        if new_pc & 0x003 != 0x00 {
            panic!("JALR target addess is not on a 4-byte boundary");
        }

        // stores pc+4 into rd
        let rd = instr.rd() as usize;
        state.x[rd] = state.pipeline.ID_pc + 4;

        // update PC
        state.pipeline.datapath.instr_addr_o = new_pc;
        state.pipeline.datapath.fetch_enable_i = false;
        state.pipeline.datapath.id_multicycle = 1;
    } else {
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn BEQ(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let immed = (instr.immediate()).unwrap();
        let new_pc = state.pipeline.ID_pc.checked_add_signed(immed).unwrap();

        // if unaligned on 4-byte boundary
        if new_pc & 0x003 != 0x00 {
            panic!("BEQ instruction immediate it not on a 4-byte boundary");
        }

        if state.x[instr.rs1() as usize] == state.x[instr.rs2() as usize] {
            // update PC
            state.pipeline.datapath.instr_addr_o = new_pc;
            state.pipeline.datapath.fetch_enable_i = false;
            state.pipeline.datapath.id_multicycle = 1;
        }
    } else {
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn BNE(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let immed = (instr.immediate()).unwrap();
        let new_pc = state.pipeline.ID_pc.checked_add_signed(immed).unwrap();

        // if unaligned on 4-byte boundary
        if new_pc & 0x003 != 0x00 {
            panic!("BNE instruction immediate it not on a 4-byte boundary");
        }

        if state.x[instr.rs1() as usize] != state.x[instr.rs2() as usize] {
            // update PC
            state.pipeline.datapath.instr_addr_o = new_pc;
            state.pipeline.datapath.fetch_enable_i = false;
            state.pipeline.datapath.id_multicycle = 1;
        }
    } else {
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn BLT(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let immed = (instr.immediate()).unwrap();
        let new_pc = state.pipeline.ID_pc.checked_add_signed(immed).unwrap();

        // if unaligned on 4-byte boundary
        if new_pc & 0x003 != 0x00 {
            panic!("BLT instruction immediate it not on a 4-byte boundary");
        }

        if (state.x[instr.rs1() as usize] as i32) < state.x[instr.rs2() as usize] as i32 {
            // update PC
            state.pipeline.datapath.instr_addr_o = new_pc;
            state.pipeline.datapath.fetch_enable_i = false;
            state.pipeline.datapath.id_multicycle = 1;
        }
    } else {
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn BGE(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let immed = (instr.immediate()).unwrap();
        let new_pc = state.pipeline.ID_pc.checked_add_signed(immed).unwrap();

        // if unaligned on 4-byte boundary
        if new_pc & 0x003 != 0x00 {
            panic!("BGE instruction immediate it not on a 4-byte boundary");
        }

        if (state.x[instr.rs1() as usize] as i32) >= state.x[instr.rs2() as usize] as i32 {
            // update PC
            state.pipeline.datapath.instr_addr_o = new_pc;
            state.pipeline.datapath.fetch_enable_i = false;
            state.pipeline.datapath.id_multicycle = 1;
        }
    } else {
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn BLTU(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let immed = (instr.immediate()).unwrap();
        let new_pc = state.pipeline.ID_pc.checked_add_signed(immed).unwrap();

        // if unaligned on 4-byte boundary
        if new_pc & 0x003 != 0x00 {
            panic!("BLTU instruction immediate it not on a 4-byte boundary");
        }

        if state.x[instr.rs1() as usize] < state.x[instr.rs2() as usize] {
            // stores pc+4 into rd
            let rd = instr.rd() as usize;
            state.x[rd] = state.pipeline.datapath.instr_addr_o + 4;

            // update PC
            state.pipeline.datapath.instr_addr_o = new_pc;
            state.pipeline.datapath.fetch_enable_i = false;
            state.pipeline.datapath.id_multicycle = 1;
        }
    } else {
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn BGEU(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let immed = (instr.immediate()).unwrap();
        let new_pc = state.pipeline.ID_pc.checked_add_signed(immed).unwrap();

        // if unaligned on 4-byte boundary
        if new_pc & 0x003 != 0x00 {
            panic!("BGEU instruction immediate it not on a 4-byte boundary");
        }

        if state.x[instr.rs1() as usize] >= state.x[instr.rs2() as usize] {
            // stores pc+4 into rd
            let rd = instr.rd() as usize;
            state.x[rd] = state.pipeline.datapath.instr_addr_o + 4;

            // update PC
            state.pipeline.datapath.instr_addr_o = new_pc;
            state.pipeline.datapath.fetch_enable_i = false;
            state.pipeline.datapath.id_multicycle = 1;
        }
    } else {
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LB(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let address: u32 =
            (state.x[instr.rs1() as usize] as i32 + instr.immediate().unwrap()) as u32;

        // set data on pipline
        let byte_enable = [true, false, false, false];
        state.pipeline.datapath.data_req_o = true;
        state.pipeline.datapath.data_addr_o = address;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = byte_enable;
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        let read_value = state.pipeline.datapath.data_rdata_i;
        state.x[instr.rd() as usize] = bitmask!(31;8) * bits!(read_value, 7) | read_value;

        state.pipeline.datapath.data_req_o = false;
        state.pipeline.datapath.data_addr_o = 0;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = [false; 4];
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LH(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let address: u32 =
            (state.x[instr.rs1() as usize] as i32 + instr.immediate().unwrap()) as u32;

        // set data on pipline
        let byte_enable = [true, true, false, false];
        state.pipeline.datapath.data_req_o = true;
        state.pipeline.datapath.data_addr_o = address;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = byte_enable;
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        let read_value = state.pipeline.datapath.data_rdata_i;
        state.x[instr.rd() as usize] = bitmask!(31;16) * bits!(read_value, 15) | read_value;

        state.pipeline.datapath.data_req_o = false;
        state.pipeline.datapath.data_addr_o = 0;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = [false; 4];
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LW(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let address: u32 =
            (state.x[instr.rs1() as usize] as i32 + instr.immediate().unwrap()) as u32;

        // set data on pipline
        let byte_enable = [true, true, true, true];
        state.pipeline.datapath.data_req_o = true;
        state.pipeline.datapath.data_addr_o = address;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = byte_enable;
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        let read_value = state.pipeline.datapath.data_rdata_i;

        state.pipeline.datapath.data_req_o = false;
        state.pipeline.datapath.data_addr_o = 0;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = [false; 4];
        state.x[instr.rd() as usize] = read_value;
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LBU(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let address: u32 =
            (state.x[instr.rs1() as usize] as i32 + instr.immediate().unwrap()) as u32;

        // set data on pipline
        let byte_enable = [true, false, false, false];
        state.pipeline.datapath.data_req_o = true;
        state.pipeline.datapath.data_addr_o = address;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = byte_enable;
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        let read_value = state.pipeline.datapath.data_rdata_i;

        state.pipeline.datapath.data_req_o = false;
        state.pipeline.datapath.data_addr_o = 0;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = [false; 4];
        state.x[instr.rd() as usize] = read_value;
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LHU(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let address: u32 =
            (state.x[instr.rs1() as usize] as i32 + instr.immediate().unwrap()) as u32;

        // set data on pipline
        let byte_enable = [true, true, false, false];
        state.pipeline.datapath.data_req_o = true;
        state.pipeline.datapath.data_addr_o = address;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = byte_enable;
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        let read_value = state.pipeline.datapath.data_rdata_i;

        state.pipeline.datapath.data_req_o = false;
        state.pipeline.datapath.data_addr_o = 0;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = [false; 4];
        state.x[instr.rd() as usize] = read_value;
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn SB(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let data = state.x[instr.rs2() as usize];
        let address: u32 =
            (state.x[instr.rs1() as usize] as i32 + instr.immediate().unwrap()) as u32;

        // set data on pipline
        state.pipeline.datapath.data_req_o = true;
        state.pipeline.datapath.data_addr_o = address;
        state.pipeline.datapath.data_we_o = true;
        state.pipeline.datapath.data_be_o = [true, false, false, false]; // access LSB only
        state.pipeline.datapath.data_wdata_o = data;
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.datapath.data_req_o = false;
        state.pipeline.datapath.data_addr_o = 0;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = [false; 4];
        state.pipeline.datapath.data_wdata_o = 0;
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn SH(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let data = state.x[instr.rs2() as usize];
        let address: u32 =
            (state.x[instr.rs1() as usize] as i32 + instr.immediate().unwrap()) as u32;

        // set data on pipline
        state.pipeline.datapath.data_req_o = true;
        state.pipeline.datapath.data_addr_o = address;
        state.pipeline.datapath.data_we_o = true;
        state.pipeline.datapath.data_be_o = [true, true, false, false];
        state.pipeline.datapath.data_wdata_o = data;
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.datapath.data_req_o = false;
        state.pipeline.datapath.data_addr_o = 0;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = [false; 4];
        state.pipeline.datapath.data_wdata_o = 0;
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn SW(instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        let data = state.x[instr.rs2() as usize];
        let address: u32 =
            (state.x[instr.rs1() as usize] as i32 + instr.immediate().unwrap()) as u32;

        // set data on pipline
        state.pipeline.datapath.data_req_o = true;
        state.pipeline.datapath.data_addr_o = address;
        state.pipeline.datapath.data_we_o = true;
        state.pipeline.datapath.data_be_o = [true, true, true, true];
        state.pipeline.datapath.data_wdata_o = data;
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.datapath.data_req_o = false;
        state.pipeline.datapath.data_addr_o = 0;
        state.pipeline.datapath.data_we_o = false;
        state.pipeline.datapath.data_be_o = [false; 4];
        state.pipeline.datapath.data_wdata_o = 0;
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn ADDI(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control {
        op_a_sel: Some(OpASel::RF),
        op_b_sel: Some(OpBSel::IMM),
        alu_op: Some(ALUOp::ADD),
        data_dest_sel: Some(DataDestSel::ALU),
        reg_write: true,
    };
}

fn SLTI(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let immediate = instr.immediate().unwrap() as i32;

    // must treat as signed
    let rs1 = state.x[rs1] as i32;

    state.x[rd] = (rs1 < immediate) as u32;
}

fn SLTIU(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let immediate = instr.immediate().unwrap() as u32;

    state.x[rd] = (state.x[rs1] < immediate) as u32;
}

fn XORI(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs = instr.rs1() as usize;
    let immediate = instr.immediate().unwrap() as u32;

    state.x[rd] = state.x[rs] ^ immediate;
}

fn ORI(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs = instr.rs1() as usize;
    let immediate = instr.immediate().unwrap() as u32;

    state.x[rd] = state.x[rs] | immediate;
}

fn ANDI(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs = instr.rs1() as usize;
    let immediate = instr.immediate().unwrap() as u32;

    state.x[rd] = state.x[rs] & immediate;
}

fn SLLI(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs = instr.rs1() as usize;
    let immediate = instr.immediate().unwrap() as u32;

    state.x[rd] = state.x[rs] << (immediate & 0x1F);
}

fn SRxI(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs = instr.rs1() as usize;
    let immediate = instr.immediate().unwrap() as u32;

    let shamt = immediate & 0x1F;
    state.x[rd] = state.x[rs] >> (immediate & 0x1F)
        | (bitmask!(31;32-shamt) * bits!(state.x[rs], 31) * bits!(instr.raw(), 30));
}

fn ADD(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control {
        op_a_sel: Some(OpASel::RF),
        op_b_sel: Some(OpBSel::RF),
        alu_op: Some(ALUOp::ADD),
        data_dest_sel: Some(DataDestSel::ALU),
        reg_write: true,
    };
}

fn SUB(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    state.x[rd] = (state.x[rs1] as i32 - state.x[rs2] as i32) as u32;
}

fn SLL(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    state.x[rd] = state.x[rs1] << (state.x[rs2] & 0x1F);
}

fn SLT(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    // must treat both as signed
    let rs1 = state.x[rs1] as i32;
    let rs2 = state.x[rs2] as i32;

    state.x[rd] = (rs1 < rs2) as u32;
}

fn SLTU(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    state.x[rd] = (state.x[rs1] < state.x[rs2]) as u32;
}

fn XOR(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    state.x[rd] = state.x[rs1] ^ state.x[rs2];
}

fn SRL(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    state.x[rd] = state.x[rs1] >> (state.x[rs2] & 0x1F);
}

fn SRA(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    state.x[rd] = (state.x[rs1] as i32 >> (state.x[rs2] & 0x1F)) as u32;
}

fn OR(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    state.x[rd] = state.x[rs1] | state.x[rs2];
}

fn AND(instr: &Instruction, state: &mut EmulatorState) {
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let rs2 = instr.rs2() as usize;

    state.x[rd] = state.x[rs1] & state.x[rs2];
}

#[allow(unused_variables)]
fn FENCE(instr: &Instruction, state: &mut EmulatorState) {
    /*
     * Instruction for ordering device I/O and memory accesses
     * as viewed by other RISC-V harts and external devices
     * We are not emulating external devices, so this is unncessary
     * to implement and can be implemented as NOP (Chapter 2, page 13
     * of the RISC-V Instruction Set Manual)
     */
}

#[allow(unused_variables)]
fn FENCE_TSO(instr: &Instruction, state: &mut EmulatorState) {
    /*
     * Instruction for ordering device I/O and memory accesses
     * as viewed by other RISC-V harts and external devices
     * We are not emulating external devices, so this is unncessary
     * to implement and can be implemented as NOP (Chapter 2, page 13
     * of the RISC-V Instruction Set Manual)
     */
}

#[allow(unused_variables)]
fn PAUSE(instr: &Instruction, state: &mut EmulatorState) {
    todo!()
}

#[allow(unused_variables)]
fn ECALL(instr: &Instruction, state: &mut EmulatorState) {
    todo!()
    /* System call */
}

#[allow(unused_variables)]
fn EBREAK(instr: &Instruction, state: &mut EmulatorState) {
    /* Call to debugger, likely going to be used to implement break points */
    state.pipeline.datapath.debug_req_i = true;
}

fn CSRRW(instr: &Instruction, state: &mut EmulatorState) {
    let csr = instr.immediate().unwrap() as u32;
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;

    // if rd = x0, CSR shall do nothing
    if rd == 0 {
        return;
    }

    let tmp = if state.csr.contains_key(&csr) {
        state.csr[&csr]
    } else {
        0
    };

    state.csr.insert(csr, state.x[rs1]);
    state.x[rd] = tmp;
}

fn CSRRS(instr: &Instruction, state: &mut EmulatorState) {
    let csr = instr.immediate().unwrap() as u32;
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;

    let tmp = if state.csr.contains_key(&csr) {
        state.csr[&csr]
    } else {
        0
    };

    state.csr.insert(csr, state.x[rs1] | tmp);
    state.x[rd] = tmp;
}

fn CSRRC(instr: &Instruction, state: &mut EmulatorState) {
    let csr = instr.immediate().unwrap() as u32;
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;

    let tmp = if state.csr.contains_key(&csr) {
        state.csr[&csr]
    } else {
        0
    };

    state.csr.insert(csr, tmp & !state.x[rs1]);
    state.x[rd] = tmp;
}

fn CSRRWI(instr: &Instruction, state: &mut EmulatorState) {
    let csr = instr.immediate().unwrap() as u32;
    let rd = instr.rd() as usize;
    let zimm = instr.rs1() as u32;

    let tmp = if state.csr.contains_key(&csr) {
        state.csr[&csr]
    } else {
        0
    };

    state.csr.insert(csr, zimm);
    state.x[rd] = tmp;
}

fn CSRRSI(instr: &Instruction, state: &mut EmulatorState) {
    let csr = instr.immediate().unwrap() as u32;
    let rd = instr.rd() as usize;
    let zimm = instr.rs1() as u32;

    let tmp = if state.csr.contains_key(&csr) {
        state.csr[&csr]
    } else {
        0
    };

    state.csr.insert(csr, tmp | zimm);
    state.x[rd] = tmp;
}

fn CSRRCI(instr: &Instruction, state: &mut EmulatorState) {
    let csr = instr.immediate().unwrap() as u32;
    let rd = instr.rd() as usize;
    let zimm = instr.rs1() as u32;

    let tmp = if state.csr.contains_key(&csr) {
        state.csr[&csr]
    } else {
        0
    };

    state.csr.insert(csr, tmp & !zimm);
    state.x[rd] = tmp;
}
