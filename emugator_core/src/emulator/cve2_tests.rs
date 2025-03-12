#![allow(non_snake_case)]

use bimap::BiBTreeMap;
use std::collections::{BTreeMap, HashMap};

use super::*;
use crate::isa::{ISA, Instruction, Operands};

impl ISA {
    pub fn build(&self, operands: Operands) -> Instruction {
        Instruction::from_def_operands(self.definition(), operands)
    }
}

// normally used to write to memory map for data during testing
fn write(map: &mut BTreeMap<u32, u8>, address: u32, bytes: &[u8]) {
    for (i, &byte) in bytes.iter().enumerate() {
        map.insert(address + i as u32, byte);
    }
}

// used to create assembled programs for testing
fn populate(instructions: &[Instruction]) -> AssembledProgram {
    populate_with_offset(instructions, 0)
}

fn populate_with_offset(instructions: &[Instruction], offset: u32) -> AssembledProgram {
    let mut instruction_memory = BTreeMap::new();
    for (i, &instruction) in instructions.iter().enumerate() {
        write(
            &mut instruction_memory,
            offset + (4 * i) as u32,
            &instruction.raw().to_le_bytes(),
        );
    }

    AssembledProgram {
        instruction_memory,
        data_memory: BTreeMap::new(),
        source_map: BiBTreeMap::new(),
        symbol_table: HashMap::new(),
    }
}

#[test]
fn test_LUI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // LUI ( x1 := 0x12345000)
    let mut program = populate(&[
        ISA::LUI.build(Operands {
            rd: 1,
            imm: 0x12345000,
            ..Default::default()
        }),
        ISA::LUI.build(Operands {
            rd: 0,
            imm: 0x12345000,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // After LUI, x1 should be loaded with the upper 20 bits of the immediate
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 0x12345000);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0x0);
}

#[test]
fn test_AUIPC() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // AUIPC ( x1 := PC + 0x12345000)
    let mut program = populate(&[ISA::AUIPC.build(Operands {
        rd: 1,
        imm: 0x12345000,
        ..Default::default()
    })]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // After AUIPC, x1 should hold the value (PC + 0x12345000)
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], pc + 0x12345000);
}

#[test]
fn test_JAL() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // JAL ( x1 := PC + 4, jump to PC + 0x100)
    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 0,
            ..Default::default()
        }),
        ISA::JAL.build(Operands {
            rd: 1,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Padding Instruction
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // After JAL, x1 should contain PC + 4, and the PC should jump to PC + 0x8
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.IF_pc, pc + 0x8);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], pc + 4);

    // ADDI that it jumps to
    assert_eq!(emulator_state.x[5], 0);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 2);
}

#[test]
fn test_JAL_neg_offset() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // JAL ( x1 := PC + 4, jump to PC - 4)
    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 5,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::JAL.build(Operands {
            rd: 1,
            imm: -4,
            ..Default::default()
        }), // JAL (pc = pc - 4)
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);
    // ADDI ( x5 := x0 + 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    // ADDI ( x5 := x5 + 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // After JAL, x1 should contain PC + 4, and the PC should jump to PC + 0x04
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.IF_pc, pc - 0x04);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], pc + 4);

    // ADDI ( x5 := x5 + 1)
    assert_eq!(emulator_state.x[5], 2);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 3);
}

#[test]
#[should_panic(expected = "PC must be on a 4-byte boundary")]
fn test_JAL_panic() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // JAL ( x1 := PC + 4, jump to PC + 0x122)
    let mut program = populate(&[ISA::JAL.build(Operands {
        rd: 1,
        imm: 0x122,
        ..Default::default()
    })]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Should panic because the immediate is not on a 4-byte boundary
    emulator_state.clock_no_uart(&mut program);
}

#[test]
fn test_JALR() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // JALR ( x1 := PC + 4, jump to (x2 + 0x4) & ~1)
    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 0x4,
            ..Default::default()
        }), // ADDI ( x2 := x0 + 0b100)
        ISA::JALR.build(Operands {
            rd: 1,
            rs1: 2,
            imm: 0x8,
            ..Default::default()
        }), // JALR ( x1 := PC + 8, jump to (x2 + 0x8) & ~1)
        ISA::ADDI.build(Operands {
            rd: 3,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x3 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 4,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 7,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // After ADDI, x2 should be loaded with 0b100
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[2], 0x4);

    // After JALR, x1 should contain PC + 4, and the PC should jump to (x4 + 0x2) & ~1
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(
        emulator_state.pipeline.IF_pc,
        (emulator_state.x[2] + 0x8) & !1
    );
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], pc + 4);

    // After ADDI
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[4], 2);

    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 7);
}

#[test]
fn test_JALR_neg_offset() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 6,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 2,
            imm: 6,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::JALR.build(Operands {
            rd: 1,
            rs1: 2,
            imm: -4,
            ..Default::default()
        }), // JALR ( x1 := PC + 4, jump to (x2 - 4) & ~1)
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);
    // ADDI ( x5 := x0 + 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    // ADDI ( x5 := x0 + 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // After JALR, x1 should contain PC + 4, and the PC should jump to x2 (12) - 4
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(
        emulator_state.pipeline.IF_pc,
        (emulator_state.x[2] as i32 - 4) as u32 & !1
    );
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], pc + 4);
}

#[test]
fn test_BEQ() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x1 := x0 + 1)
        ISA::BEQ.build(Operands {
            rs1: 1,
            rs2: 2,
            imm: 0x8,
            ..Default::default()
        }), // BEQ (branch if x1 == x2)
        ISA::BEQ.build(Operands {
            rs1: 0,
            rs2: 2,
            imm: 0x8,
            ..Default::default()
        }), // BEQ (branch if x0 == x2)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 + 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 1);

    // BEQ (branch if x1 == x2) - should not branch because x1 != x2
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x4);

    // BEQ (branch if x0 == x2) - should branch because x0 == x2
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x8);

    // ADDI ( x5 := x0 + 2)
    assert_eq!(emulator_state.x[5], 0);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 2);
}

#[test]
fn test_BNE() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x1 := x0 + 1)
        ISA::BNE.build(Operands {
            rs1: 0,
            rs2: 2,
            imm: 0x8,
            ..Default::default()
        }), // BNE (branch if x0 != x2)
        ISA::BNE.build(Operands {
            rs1: 1,
            rs2: 2,
            imm: 0x8,
            ..Default::default()
        }), // BNE (branch if x1 != x2)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 + 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // BNE (branch if x0 != x2) - should not branch because x0 == x2
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x4);

    // BNE (branch if x1 != x2) - should branch because x1 != x2
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x8);

    // ADDI ( x5 := x0 + 2)
    assert_eq!(emulator_state.x[5], 0);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 2);
}

#[test]
fn test_BLT() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: -1,
            ..Default::default()
        }), // ADDI ( x1 := x0 - 1)
        ISA::BLT.build(Operands {
            rs1: 0,
            rs2: 1,
            imm: 0x8,
            ..Default::default()
        }), // BLT (branch if x0 < x1)
        ISA::BLT.build(Operands {
            rs1: 1,
            rs2: 0,
            imm: 0x8,
            ..Default::default()
        }), // BLT (branch if x1 < x0)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 2)
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 - 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], u32::MAX);

    // BLT (branch if x0 < x1) - should not branch because x0 > x1
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x4);

    // BLT (branch if x1 < x0) - should branch because x1 < x0
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x8);

    // ADDI ( x5 := x0 + 2)
    assert_eq!(emulator_state.x[5], 0);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 2);
}

#[test]
fn test_BGE() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: -1,
            ..Default::default()
        }), // ADDI ( x1 := x0 - 1)
        ISA::BGE.build(Operands {
            rs1: 1,
            rs2: 0,
            imm: 0x8,
            ..Default::default()
        }), // BGE (branch if x1 >= x0)
        ISA::BGE.build(Operands {
            rs1: 0,
            rs2: 1,
            imm: 0x8,
            ..Default::default()
        }), // BGE (branch if x0 >= x1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 2)
        ISA::BGE.build(Operands {
            rs1: 0,
            rs2: 2,
            imm: -0x8,
            ..Default::default()
        }), // BGE (branch if x0 >= x2)
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 - 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], u32::MAX);

    // BGE (branch if x1 >= x0) - should not branch because x0 > x1
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x4);

    // BLT (branch if x0 >= x1) - should branch because x1 < x0
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x8);

    // ADDI ( x5 := x0 + 2)
    assert_eq!(emulator_state.x[5], 0);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 2);

    // BGE (branch if x0 >= x2) - should branch because x0 == x2
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc - 0x8);

    // ADDI ( x5 := x0 + 1)
    assert_eq!(emulator_state.x[5], 2);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 1);
}

#[test]
fn test_BLTU() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: u32::MAX as i32,
            ..Default::default()
        }), // ADDI ( x1 := x0 - 1)
        ISA::BLTU.build(Operands {
            rs1: 1,
            rs2: 0,
            imm: 0x8,
            ..Default::default()
        }), // BLTU (branch if x1 < x0)
        ISA::BLTU.build(Operands {
            rs1: 0,
            rs2: 1,
            imm: 0x8,
            ..Default::default()
        }), // BLTU (branch if x0 < x1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 2)
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 - 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], u32::MAX);

    // BLTU (branch if x1 < x0) - should not branch because x1 > x0
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x4);

    // BLTU (branch if x0 < x1) - should branch because x0 < x1
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x8);

    // ADDI ( x5 := x0 + 2)
    assert_eq!(emulator_state.x[5], 0);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 2);
}

#[test]
fn test_BGEU() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: u32::MAX as i32,
            ..Default::default()
        }), // ADDI ( x1 := x0 - 1)
        ISA::BGEU.build(Operands {
            rs1: 0,
            rs2: 1,
            imm: 0x8,
            ..Default::default()
        }), // BGEU (branch if x0 >= x1)
        ISA::BGEU.build(Operands {
            rs1: 1,
            rs2: 0,
            imm: 0x8,
            ..Default::default()
        }), // BGEU (branch if x1 >= x0)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 5,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }), // ADDI ( x5 := x0 + 2)
        ISA::BGEU.build(Operands {
            rs1: 0,
            rs2: 2,
            imm: -0x8,
            ..Default::default()
        }), // BGEU (branch if x0 >= x2)
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 - 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], u32::MAX);

    // BGEU (branch if x0 >= x1) - should not branch because x0 < x1
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x4);

    // BLT (branch if x1 >= x0) - should branch because x1 > x0
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc + 0x8);

    // ADDI ( x5 := x0 + 2)
    assert_eq!(emulator_state.x[5], 0);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 2);

    // BGEU (branch if x0 >= x2) - should branch because x0 == x2
    let pc = emulator_state.pipeline.ID_pc;
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.pipeline.ID_pc, pc - 0x8);

    // ADDI ( x5 := x0 + 1)
    assert_eq!(emulator_state.x[5], 2);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 1);
}

#[test]
fn test_LB() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::LB.build(Operands {
            rd: 5,
            rs1: 1,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::LB.build(Operands {
            rd: 5,
            rs1: 1,
            imm: 0xA,
            ..Default::default()
        }),
    ]);

    program.data_memory.insert(0x10, 0xFB);
    program.data_memory.insert(0x11, 0xFC);
    program.data_memory.insert(0x12, 0x7D);
    program.data_memory.insert(0x13, 0x7E);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 + 0x8)
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // LB ( x5 := MEM[x1 + 0x8])
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 0xFFFFFFFB);

    // LB ( x5 := MEM[x1 + 0xA])
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 0x0000007D);
}

#[test]
fn test_LH() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::LH.build(Operands {
            rd: 5,
            rs1: 1,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::LH.build(Operands {
            rd: 5,
            rs1: 1,
            imm: 0xA,
            ..Default::default()
        }),
    ]);

    program.data_memory.insert(0x10, 0xFB);
    program.data_memory.insert(0x11, 0xFC);
    program.data_memory.insert(0x12, 0x7D);
    program.data_memory.insert(0x13, 0x7E);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 + 0x8)
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // LB ( x5 := MEM[x1 + 0x8])
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 0xFFFFFCFB);

    // LB ( x5 := MEM[x1 + 0xA])
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 0x00007E7D);
}

#[test]
fn test_LW() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::LW.build(Operands {
            rd: 5,
            rs1: 1,
            imm: 0x8,
            ..Default::default()
        }),
    ]);

    program.data_memory.insert(0x10, 0xFB);
    program.data_memory.insert(0x11, 0xFC);
    program.data_memory.insert(0x12, 0x7D);
    program.data_memory.insert(0x13, 0x7E);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 + 0x8)
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // LB ( x5 := MEM[x1 + 0x8])
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 0x7E7DFCFB);
}

#[test]
fn test_SB() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // Set x1 := 0xFEFDFCFB (Data to write)
        ISA::LUI.build(Operands {
            rd: 1,
            imm: 0xFEFDF000u32.wrapping_sub(0xFFFFF000u32) as i32,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: 0xCFB,
            ..Default::default()
        }),
        // Set x2 := 100 (Base Address to write to)
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 100,
            ..Default::default()
        }),
        // SB x1, 100(x2) -> Write x1 to address x2 (100) + 5
        ISA::SB.build(Operands {
            rd: 0,
            rs1: 2,
            rs2: 1,
            imm: 5,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 0xFEFDFCFB
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x2 := 100
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // SH -> Write first byte of x1 to address x2 (100) + 5
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state.clock_no_uart(&mut program);
    assert_eq!(program.data_memory.get(&105), Some(&0xFB));
    assert_eq!(program.data_memory.get(&106), None);
    assert_eq!(program.data_memory.get(&107), None);
    assert_eq!(program.data_memory.get(&108), None);
}

#[test]
fn test_SH() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // Set x1 := 0xFEFDFCFB (Data to write)
        ISA::LUI.build(Operands {
            rd: 1,
            imm: 0xFEFDF000u32.wrapping_sub(0xFFFFF000u32) as i32,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: 0xCFB,
            ..Default::default()
        }),
        // Set x2 := 100 (Base Address to write to)
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 100,
            ..Default::default()
        }),
        // SH x1, 100(x2) -> Write x1 to address x2 (100) + 5
        ISA::SH.build(Operands {
            rd: 0,
            rs1: 2,
            rs2: 1,
            imm: 5,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 0xFEFDFCFB (lowest byte )
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x2 := 100
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // SH -> Write x1 to address x2 (100) + 5
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state.clock_no_uart(&mut program);
    assert_eq!(program.data_memory.get(&105), Some(&0xFB));
    assert_eq!(program.data_memory.get(&106), Some(&0xFC));
    assert_eq!(program.data_memory.get(&107), None);
    assert_eq!(program.data_memory.get(&108), None);
}

#[test]
fn test_SW() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // Set x1 := 0xFEFDFCFB (Data to write)
        ISA::LUI.build(Operands {
            rd: 1,
            imm: 0xFEFDF000u32.wrapping_sub(0xFFFFF000u32) as i32,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: 0xCFB,
            ..Default::default()
        }),
        // Set x2 := 100 (Base Address to write to)
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 100,
            ..Default::default()
        }),
        // SW x1, 100(x2) -> Write x1 to address x2 (100) + 5
        ISA::SW.build(Operands {
            rd: 0,
            rs1: 2,
            rs2: 1,
            imm: 5,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 0xFEFDFCFB
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x2 := 100
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // SW -> Write x1 to address x2 (100) + 5
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state.clock_no_uart(&mut program);
    assert_eq!(program.data_memory.get(&105), Some(&0xFB));
    assert_eq!(program.data_memory.get(&106), Some(&0xFC));
    assert_eq!(program.data_memory.get(&107), Some(&0xFD));
    assert_eq!(program.data_memory.get(&108), Some(&0xFE));
}

#[test]
fn test_ADDI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // ADDI ( x1 := x0 + 1)
    // ADDI ( x1 := x1 + (-1))
    // ADDI ( x0 := x0 + 1 )
    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: -1,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ADDI ( x1 := x0 + 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 1);
    // ADDI ( x1 := x1 + 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 0);
    // ADDI ( x0 := x0 + 1) <= special case should be a noop
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_SLTI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // SLTI ( x1 := x0 < 1)
    // SLTI ( x1 := x1 < (-1))
    // SLTI ( x0 := x0 < 1 )

    let mut program = populate(&[
        ISA::SLTI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
        ISA::SLTI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: -1,
            ..Default::default()
        }),
        ISA::SLTI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // SLTI ( x1 := x0 < 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 1);
    // SLTI ( x1 := x1 < (-1))
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 0);
    // SLTI ( x0 := x0 < 1 ) <= Should not change x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_SLTIU() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // SLTIU ( x1 := x0 < 1)
    // SLTIU ( x1 := x1 < (-1))
    // SLTIU ( x0 := x0 < 1 )

    let mut program = populate(&[
        ISA::SLTIU.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
        ISA::SLTIU.build(Operands {
            rd: 1,
            rs1: 1,
            imm: -1, // Should be treated as an unsigned number (pretty large)
            ..Default::default()
        }),
        ISA::SLTIU.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // SLTI ( x1 := x0 < 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 1);
    // SLTI ( x1 := x1 < (-1))
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 1);
    // SLTI ( x0 := x0 < 1 ) <= Should not change x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_XORI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // XORI ( x1 := x0 ^ 4)
    // XORI ( x1 := x1 ^ (-1))
    // XORI ( x0 := x0 ^ 100 )

    let mut program = populate(&[
        ISA::XORI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 4,
            ..Default::default()
        }),
        ISA::XORI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: -1,
            ..Default::default()
        }),
        ISA::XORI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 100,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // XORI ( x1 := x0 ^ 4)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 4);
    // XORI ( x1 := x1 ^ (-1))
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1] as i32, -5);
    // XORI ( x0 := x0 ^ 100 ) <= Should not change x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_ORI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    // ORI ( x1 := x0 | 12)
    // ORI ( x1 := x1 | (-1))
    // ORI ( x0 := x0 | 100 )

    let mut program = populate(&[
        ISA::ORI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 12,
            ..Default::default()
        }),
        ISA::ORI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: -10,
            ..Default::default()
        }),
        ISA::ORI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 100,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // ORI ( x1 := x0 | 12)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 12);
    // ORI ( x1 := x1 ^ (-10))
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1] as i32, -2);
    // ORI ( x0 := x0 ^ 100 ) <= Should not change x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_ANDI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 37,
            ..Default::default()
        }),
        ISA::ANDI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: 5,
            ..Default::default()
        }),
        ISA::ANDI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: -10,
            ..Default::default()
        }),
        ISA::ANDI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 100,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 37
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 37);

    // ANDI ( x1 := x1 & 5)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 5);

    // ANDI ( x1 := x1 & (-10))
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 4);

    // ANDI ( x0 := x0 & 100 ) <= Should not change x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_SLLI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 10,
            ..Default::default()
        }),
        ISA::SLLI.build(Operands {
            rd: 2,
            rs1: 1,
            imm: 4,
            ..Default::default()
        }),
        ISA::SLLI.build(Operands {
            rd: 3,
            rs1: 1,
            imm: 1,
            ..Default::default()
        }),
        ISA::SLLI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 3,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 10
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 10);

    // SLLI ( x2 := x1 << 4)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[2], 160);

    // SLLI ( x3 := x1 << 0b1000001) Should only shift 1 time since we only look at last 5 bits
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[3], 20);

    // SLLI ( x0 := x1 << 3 ) <= Should not change x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_SRLI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 10,
            ..Default::default()
        }),
        ISA::SRLI.build(Operands {
            rd: 2,
            rs1: 1,
            imm: 1,
            ..Default::default()
        }),
        ISA::SRLI.build(Operands {
            rd: 3,
            rs1: 1,
            imm: 0b00010,
            ..Default::default()
        }),
        ISA::SRLI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 3,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 10
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1], 10);

    // SRLI ( x2 := x1 >> 1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[2], 5);

    // SRLI ( x3 := x1 >> 0b1000010) Should only shift 1 time since we only look at last 5 bits
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[3], 2);

    // SRLI ( x0 := x1 << 3 ) <= Should not change x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_SRAI() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: -10,
            ..Default::default()
        }),
        ISA::SRAI.build(Operands {
            rd: 2,
            rs1: 1,
            imm: 0b11111,
            ..Default::default()
        }),
        ISA::SRAI.build(Operands {
            rd: 3,
            rs1: 1,
            imm: 0b01 & 0x1F,
            ..Default::default()
        }),
        ISA::SRAI.build(Operands {
            rd: 0,
            rs1: 0,
            imm: 3,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := -10
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1] as i32, -10);

    // SRAI ( x2 := x1 >> -1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[2] as i32, -1);

    // SRAI ( x3 := x1 >> 0b1000001) Should only shift 1 time since we only look at last 5 bits
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[3] as i32, -5);

    // SRAI ( x0 := x1 << 3 ) <= Should not change x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_ADD() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, 15 -> Set x1 := 15
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 15,
            ..Default::default()
        }),
        // ADDI x2, x0, -10 -> Set x2 := -10
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: -10,
            ..Default::default()
        }),
        // ADD x3, x1, x2 -> Set x3 := x1 + x2 (15 + (-10) = 5)
        ISA::ADD.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
        // ADD x4, x1, x1 -> Set x4 := x1 + x1 (15 + 15 = 30)
        ISA::ADD.build(Operands {
            rd: 4,
            rs1: 1,
            rs2: 1,
            ..Default::default()
        }),
        // ADD x0, x1, x2 -> Should not modify x0 (x0 always 0)
        ISA::ADD.build(Operands {
            rd: 0,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 15
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1] as i32, 15);

    // Set x2 := -10
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[2] as i32, -10);

    // ADD (x3 := x1 + x2)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[3] as i32, 5);

    // ADD (x4 := x1 + x1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[4] as i32, 30);

    // ADD (x0 := x1 + x2) - No change to x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_SUB() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, 20 -> Set x1 := 20
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 20,
            ..Default::default()
        }),
        // ADDI x2, x0, 5 -> Set x2 := 5
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 5,
            ..Default::default()
        }),
        // SUB x3, x1, x2 -> Set x3 := x1 - x2 (20 - 5 = 15)
        ISA::SUB.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
        // SUB x4, x2, x1 -> Set x4 := x2 - x1 (5 - 20 = -15)
        ISA::SUB.build(Operands {
            rd: 4,
            rs1: 2,
            rs2: 1,
            ..Default::default()
        }),
        // SUB x5, x1, x1 -> Set x5 := x1 - x1 (20 - 20 = 0)
        ISA::SUB.build(Operands {
            rd: 5,
            rs1: 1,
            rs2: 1,
            ..Default::default()
        }),
        // SUB x0, x1, x2 -> Should not modify x0 (x0 always 0)
        ISA::SUB.build(Operands {
            rd: 0,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 20
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1] as i32, 20);

    // Set x2 := 5
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[2] as i32, 5);

    // SUB (x3 := x1 - x2)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[3] as i32, 15);

    // SUB (x4 := x2 - x1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[4] as i32, -15);

    // SUB (x5 := x1 - x1)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5] as i32, 0);

    // SUB (x0 := x1 - x2) - No change to x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_SLL() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, 1 -> Set x1 := 1
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
        // ADDI x2, x0, 2 -> Set x2 := 2
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }),
        // SLL x3, x1, x2 -> Set x3 := x1 << x2 (1 << 2 = 4)
        ISA::SLL.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
        // SLL x4, x1, x2 -> Test ignoring upper bits of shift amount
        // Set x2 := 0b100000 -> (1 << 0 = 1, because shift amount is masked to 5 bits)
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 0b100000,
            ..Default::default()
        }),
        ISA::SLL.build(Operands {
            rd: 4,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
        // SLL x5, x2, x2 -> Shift a zero by any value (0 << n = 0)
        ISA::SLL.build(Operands {
            rd: 5,
            rs1: 2,
            rs2: 2,
            ..Default::default()
        }),
        // SLL x0, x1, x2 -> Ensure x0 remains unchanged
        ISA::SLL.build(Operands {
            rd: 0,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    // Instruction fetch
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // Set x1 := 1
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[1] as i32, 1);

    // Set x2 := 2
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[2] as i32, 2);

    // SLL (x3 := x1 << x2)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[3] as i32, 4);

    // Set x2 := 0b100000 (masked to 0)
    emulator_state = emulator_state.clock_no_uart(&mut program);

    // SLL (x4 := x1 << x2, with x2 effectively 0)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[4] as i32, 1);

    // SLL (x5 := x2 << x2)
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5] as i32, 32);

    // SLL (x0 := x1 << x2) - Ensure no change to x0
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[0], 0);
}

#[test]
fn test_SLT() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, 5 -> Set x1 := 5
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 5,
            ..Default::default()
        }),
        // ADDI x2, x0, 10 -> Set x2 := 10
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 10,
            ..Default::default()
        }),
        // SLT x3, x1, x2 -> x3 := (x1 < x2) ? 1 : 0
        ISA::SLT.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
        // SLT x4, x2, x1 -> x4 := (x2 < x1) ? 1 : 0
        ISA::SLT.build(Operands {
            rd: 4,
            rs1: 2,
            rs2: 1,
            ..Default::default()
        }),
        // SLT x5, x1, x1 -> x5 := (x1 < x1) ? 1 : 0
        ISA::SLT.build(Operands {
            rd: 5,
            rs1: 1,
            rs2: 1,
            ..Default::default()
        }),
    ]);

    // Execute each instruction and validate
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program); // Set x1 = 5
    emulator_state = emulator_state.clock_no_uart(&mut program); // Set x2 = 10
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[3], 1); // x3 = 1 (5 < 10)

    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[4], 0); // x4 = 0 (10 < 5 false)

    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[5], 0); // x5 = 0 (5 < 5 false)
}

#[test]
fn test_SLTU() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, -1 -> Set x1 := -1 (interpreted as 0xFFFFFFFF unsigned)
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: -1,
            ..Default::default()
        }),
        // ADDI x2, x0, 1 -> Set x2 := 1
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }),
        // SLTU x3, x2, x1 -> x3 := (x2 < x1) ? 1 : 0
        ISA::SLTU.build(Operands {
            rd: 3,
            rs1: 2,
            rs2: 1,
            ..Default::default()
        }),
        // SLTU x4, x1, x2 -> x4 := (x1 < x2) ? 1 : 0
        ISA::SLTU.build(Operands {
            rd: 4,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program); // Set x1 = -1 (0xFFFFFFFF unsigned)
    emulator_state = emulator_state.clock_no_uart(&mut program); // Set x2 = 1
    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[3], 1); // x3 = 1 (1 < 0xFFFFFFFF true)

    emulator_state = emulator_state.clock_no_uart(&mut program);
    assert_eq!(emulator_state.x[4], 0); // x4 = 0 (0xFFFFFFFF < 1 false)
}

#[test]
fn test_XOR() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, 0b1100 -> Set x1 := 12
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 0b1100,
            ..Default::default()
        }),
        // ADDI x2, x0, 0b1010 -> Set x2 := 10
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 0b1010,
            ..Default::default()
        }),
        // XOR x3, x1, x2 -> x3 := x1 ^ x2
        ISA::XOR.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);

    assert_eq!(emulator_state.x[3], 0b0110); // x3 = 6 (0b1100 ^ 0b1010)
}

#[test]
fn test_SRL() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, 16 -> Set x1 := 16 (0b10000)
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 16,
            ..Default::default()
        }),
        // ADDI x2, x0, 2 -> Set x2 := 2
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }),
        // SRL x3, x1, x2 -> x3 := x1 >> x2
        ISA::SRL.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);

    assert_eq!(emulator_state.x[3], 4); // x3 = 4 (16 >> 2)
}

#[test]
fn test_SRA() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, -16 -> Set x1 := -16
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: -16,
            ..Default::default()
        }),
        // ADDI x2, x0, 2 -> Set x2 := 2
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }),
        // SRA x3, x1, x2 -> x3 := x1 >> x2 (arithmetic)
        ISA::SRA.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);

    assert_eq!(emulator_state.x[3] as i32, -4); // x3 = -4 (-16 >> 2, arithmetic)
}

#[test]
fn test_OR() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, 0b1100 -> Set x1 := 12
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 0b1100,
            ..Default::default()
        }),
        // ADDI x2, x0, 0b1010 -> Set x2 := 10
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 0b1010,
            ..Default::default()
        }),
        // OR x3, x1, x2 -> x3 := x1 | x2
        ISA::OR.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);

    assert_eq!(emulator_state.x[3], 0b1110); // x3 = 14 (0b1100 | 0b1010)
}

#[test]
fn test_AND() {
    let mut emulator_state = EmulatorState::<CVE2Pipeline>::default();

    let mut program = populate(&[
        // ADDI x1, x0, 0b1100 -> Set x1 := 12
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 0b1100,
            ..Default::default()
        }),
        // ADDI x2, x0, 0b1010 -> Set x2 := 10
        ISA::ADDI.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 0b1010,
            ..Default::default()
        }),
        // AND x3, x1, x2 -> x3 := x1 & x2
        ISA::AND.build(Operands {
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        }),
    ]);

    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);
    emulator_state = emulator_state.clock_no_uart(&mut program);

    assert_eq!(emulator_state.x[3], 0b1000); // x3 = 8 (0b1100 & 0b1010)
}
