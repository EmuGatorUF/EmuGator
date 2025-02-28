#![allow(non_snake_case)]

use super::pipeline::{ALUOp, CVE2Control, LSUDataType};
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

fn LB(_instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        state.pipeline.control = CVE2Control::load_request(LSUDataType::Byte);
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.control = CVE2Control::load_write(LSUDataType::Byte, true);
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LH(_instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        state.pipeline.control = CVE2Control::load_request(LSUDataType::HalfWord);
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.control = CVE2Control::load_write(LSUDataType::HalfWord, true);
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LW(_instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        state.pipeline.control = CVE2Control::load_request(LSUDataType::Word);
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.control = CVE2Control::load_write(LSUDataType::Word, false);
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LBU(_instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        state.pipeline.control = CVE2Control::load_request(LSUDataType::Byte);
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.control = CVE2Control::load_write(LSUDataType::Byte, false);
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn LHU(_instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        state.pipeline.control = CVE2Control::load_request(LSUDataType::HalfWord);
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.control = CVE2Control::load_write(LSUDataType::HalfWord, false);
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn SB(_instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        state.pipeline.control = CVE2Control::store_request(LSUDataType::Byte);
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.control = CVE2Control::store_completion();
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn SH(_instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        state.pipeline.control = CVE2Control::store_request(LSUDataType::HalfWord);
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.control = CVE2Control::store_completion();
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn SW(_instr: &Instruction, state: &mut EmulatorState) {
    if state.pipeline.datapath.id_multicycle == 0 {
        state.pipeline.control = CVE2Control::store_request(LSUDataType::Word);
        state.pipeline.datapath.id_multicycle = 1;
        state.pipeline.datapath.fetch_enable_i = false;
    } else {
        state.pipeline.control = CVE2Control::store_completion();
        state.pipeline.datapath.id_multicycle = 0;
        state.pipeline.datapath.fetch_enable_i = true;
    }
}

fn ADDI(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::immediate(ALUOp::ADD);
}

fn SLTI(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::immediate(ALUOp::SLT);
}

fn SLTIU(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::immediate(ALUOp::SLTU);
}

fn XORI(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::immediate(ALUOp::XOR);
}

fn ORI(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::immediate(ALUOp::OR);
}

fn ANDI(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::immediate(ALUOp::AND);
}

fn SLLI(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::immediate(ALUOp::SLL);
}

fn SRxI(instr: &Instruction, state: &mut EmulatorState) {
    let op = if bits!(instr.raw(), 30) == 0 {
        ALUOp::SRL
    } else {
        ALUOp::SRA
    };
    state.pipeline.control = CVE2Control::immediate(op);
}

fn ADD(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::ADD);
}

fn SUB(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::SUB);
}

fn SLL(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::SLL);
}

fn SLT(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::SLT);
}

fn SLTU(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::SLTU);
}

fn XOR(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::XOR);
}

fn SRL(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::SRL);
}

fn SRA(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::SRA);
}

fn OR(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::OR);
}

fn AND(_instr: &Instruction, state: &mut EmulatorState) {
    state.pipeline.control = CVE2Control::register(ALUOp::AND);
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
