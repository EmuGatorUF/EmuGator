#![allow(non_snake_case)]

use bimap::BiBTreeMap;
use std::collections::{BTreeMap, HashMap};

use super::*;
use crate::isa::{ISA, Instruction, Operands};

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
        initial_data_memory: BTreeMap::new(),
        source_map: BiBTreeMap::new(),
        symbol_table: HashMap::new(),
    }
}

#[test]
fn test_LUI() {
    // LUI ( x1 := 0x12345000)
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // After LUI, x1 should be loaded with the upper 20 bits of the immediate
    state = state.clock(&program);
    assert_eq!(state.x[1], 0x12345000);
    state = state.clock(&program);
    assert_eq!(state.x[0], 0x0);
}

#[test]
fn test_AUIPC() {
    // AUIPC ( x1 := PC + 0x12345000)
    let program = populate(&[ISA::AUIPC.build(Operands {
        rd: 1,
        imm: 0x12345000,
        ..Default::default()
    })]);

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // After AUIPC, x1 should hold the value (PC + 0x12345000)
    state = state.clock(&program);
    assert_eq!(state.x[1], pc + 0x12345000);
}

#[test]
fn test_JAL() {
    // JAL ( x1 := PC + 4, jump to PC + 0x100)
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    assert_eq!(state.pipeline.if_pc, pc + 0x8);
    assert_eq!(state.pipeline.if_pc, 12);

    // Padding Instruction
    state = state.clock(&program); //WB

    // After JAL, x1 should contain PC + 4, and the PC should jump to PC + 0x8
    state = state.clock(&program);
    assert_eq!(state.x[1], pc + 4);

    // ADDI that it jumps to
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    assert_eq!(state.x[5], 0);
    state = state.clock(&program); //WB
    assert_eq!(state.x[5], 2);
}

#[test]
fn test_JAL_bigger_jump() {
    // JAL ( x1 := PC + 4, jump to PC + 0x100)
    let program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 9,
            rs1: 0,
            imm: 0,
            ..Default::default()
        }),
        ISA::JAL.build(Operands {
            rd: 1,
            imm: 16,
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
        ISA::ADDI.build(Operands {
            rd: 6,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 7,
            rs1: 0,
            imm: 10,
            ..Default::default()
        }),
    ]);

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    assert_eq!(state.pipeline.if_pc, pc + 16);
    assert_eq!(state.pipeline.if_pc, 20);

    // Padding Instruction
    state = state.clock(&program); //WB

    // After JAL, x1 should contain PC + 4, and the PC should jump to PC + 16
    state = state.clock(&program);
    assert_eq!(state.x[1], pc + 4);

    // ADDI that it jumps to
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    assert_eq!(state.x[5], 0);
    assert_eq!(state.x[6], 0);
    assert_eq!(state.x[7], 0);
    state = state.clock(&program); //WB
    assert_eq!(state.x[5], 0);
    assert_eq!(state.x[6], 0);
    assert_eq!(state.x[7], 10);
}

#[test]
fn test_JAL_neg_offset() {
    // JAL ( x1 := PC + 4, jump to PC - 4)
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program); //WB
    assert_eq!(state.x[5], 1); // ADDI ( x5 := x5 + 1)
    state = state.clock(&program); //extra clock cycles because of hazard
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program);
    state = state.clock(&program);
    // After JAL, x1 should contain PC + 4, and the PC should jump to PC + 0x04
    assert_eq!(state.pipeline.if_pc, pc - 0x04);
    assert_eq!(state.pipeline.if_pc, 4);
    assert_eq!(state.x[5], 1);
    state = state.clock(&program);
    assert_eq!(state.x[5], 2);
    state = state.clock(&program);
    assert_eq!(state.x[1], pc + 4);

    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    assert_eq!(state.x[5], 2);
    state = state.clock(&program); //WB
    assert_eq!(state.x[5], 3); // ADDI ( x5 := x5 + 1)
}

#[test]
#[should_panic(expected = "PC must be on a 4-byte boundary")]
fn test_JAL_panic() {
    // JAL ( x1 := PC + 4, jump to PC + 0x122)
    let program = populate(&[ISA::JAL.build(Operands {
        rd: 1,
        imm: 0x122,
        ..Default::default()
    })]);

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Should panic because the immediate is not on a 4-byte boundary
    state.clock(&program);
}

#[test]
fn test_JALR() {
    // JALR ( x1 := PC + 4, jump to (x2 + 0x4) & ~1)
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // After ADDI, x2 should be loaded with 0b100
    state = state.clock(&program); //WB
    let pc = state.pipeline.if_id.id_pc.unwrap();
    assert_eq!(state.x[2], 0x4);

    // After JALR, x1 should contain PC + 4, and the PC should jump to (x2 + 0x8) & ~1
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, (state.x[2] + 0x8) & !1);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1], pc + 4);
    assert_eq!(state.x[1], 8);

    // After ADDI
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[4], 2);

    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[3], 0);
    assert_eq!(state.x[5], 7);
}

#[test]
fn test_JALR_neg_offset() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    // ADDI ( x5 := x0 + 1)
    state = state.clock(&program); //WB
    // ADDI ( x5 := x0 + 1)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);

    // After JALR, x1 should contain PC + 4, and the PC should jump to x2 (12) - 4
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, (state.x[2] as i32 - 4) as u32 & !1);
    assert_eq!(8, state.pipeline.if_pc);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1], pc + 4);
    assert_eq!(state.x[1], 12);
}

#[test]
fn test_BEQ() {
    let program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x1 := x0 + 1)
        ISA::BEQ.build(Operands {
            rs1: 1,
            rs2: 2,
            imm: 20,
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    // ADDI ( x1 := x0 + 1)
    state = state.clock(&program); //WB
    assert_eq!(state.x[1], 1);

    // BEQ (branch if x1 == x2) - should not branch because x1 != x2
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc + 0x4);
    state = state.clock(&program);

    // BEQ (branch if x0 == x2) - should branch because x0 == x2
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc + 0x8);
    assert_eq!(state.pipeline.if_pc, 16);

    // ADDI ( x5 := x0 + 2)
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[5], 0);
    state = state.clock(&program);
    assert_eq!(state.x[5], 2);
}

#[test]
fn test_BNE() {
    let program = populate(&[
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
            imm: 12,
            ..Default::default()
        }), // BNE (branch if x1 != x2)
        ISA::ADDI.build(Operands {
            rd: 7,
            rs1: 0,
            imm: 7,
            ..Default::default()
        }),
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //EX
    // EX for BNE should finish and set if_pc (branch if x0 != x2) - should not branch because x0 == x2
    assert_eq!(state.pipeline.if_pc, pc + 0x4);
    state = state.clock(&program); //MEM

    // ADDI ( x1 := x0 + 1)
    state = state.clock(&program);
    let pc = state.pipeline.if_id.id_pc.unwrap();
    assert_eq!(state.x[1], 1);
    
    // BNE (branch if x1 != x2) - should branch because x1 != x2
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, 20);
    assert_eq!(20, pc + 12);

    // ADDI ( x5 := x0 + 2)
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[5], 0);
    state = state.clock(&program);
    assert_eq!(state.x[5], 2);
}

#[test]
fn test_BNE_single_branch() {
    let program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x1 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 3,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x3 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 4,
            rs1: 0,
            imm: 1,
            ..Default::default()
        }), // ADDI ( x4 := x0 + 1)
        ISA::ADDI.build(Operands {
            rd: 6,
            rs1: 0,
            imm: 2,
            ..Default::default()
        }), // ADDI ( x4 := x0 + 1)
        ISA::BNE.build(Operands {
            rs1: 1,
            rs2: 2,
            imm: 12,
            ..Default::default()
        }), // BNE (branch if x1 != x2)
        ISA::ADDI.build(Operands {
            rd: 7,
            rs1: 0,
            imm: 7,
            ..Default::default()
        }),
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program); //WB
    let pc = state.pipeline.if_id.id_pc.unwrap();
    assert_eq!(state.x[1], 1);
    state = state.clock(&program);
    assert_eq!(state.x[3], 1);
    state = state.clock(&program);
    assert_eq!(state.x[4], 1);
    assert_eq!(state.pipeline.if_pc, 28);
    assert_eq!(28, pc + 12);

    // IF for ADDI ( x5 := x0 + 2)
    state = state.clock(&program);
    assert_eq!(state.x[6], 2);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[5], 0);
    state = state.clock(&program);
    assert_eq!(state.x[5], 2);
}

#[test]
fn test_BLT() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program); //WB
    assert_eq!(state.x[1], u32::MAX); // ADDI ( x1 := x0 - 1)
    state = state.clock(&program); 
    assert_eq!(state.pipeline.if_pc, pc + 0x4); // BLT (branch if x0 < x1) - should not branch because x0 > x1

    state = state.clock(&program); 
    let pc = state.pipeline.if_id.id_pc.unwrap();

    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc + 0x8); // BLT (branch if x1 < x0) - should branch because x1 < x0
    assert_eq!(16, pc + 0x8);

    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    // ADDI ( x5 := x0 + 2)
    assert_eq!(state.x[5], 0);
    state = state.clock(&program);
    assert_eq!(state.x[5], 2);
}

#[test]
fn test_BGE() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program); //WB, ADDI ( x1 := x0 - 1)
    assert_eq!(state.x[1], u32::MAX);
    state = state.clock(&program); // BGE (branch if x1 >= x0) - should not branch because x0 > x1
    assert_eq!(state.pipeline.if_pc, pc + 0x4);

    // BLT (branch if x0 >= x1) - should branch because x1 < x0
    state = state.clock(&program);
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc + 0x8);
    assert_eq!(16, pc + 0x8);

    state = state.clock(&program);
    state = state.clock(&program);
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc - 0x8); // BGE (branch if x0 >= x2) - should branch because x0 == x2
    assert_eq!(12, pc - 0x8);
    // ADDI ( x5 := x0 + 2)
    assert_eq!(state.x[5], 0);
    state = state.clock(&program);
    assert_eq!(state.x[5], 2);

    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    // ADDI ( x5 := x0 + 1)
    assert_eq!(state.x[5], 2);
    state = state.clock(&program);
    assert_eq!(state.x[5], 1);
}

#[test]
fn test_BLTU() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program); // ADDI ( x1 := x0 - 1)
    assert_eq!(state.x[1], u32::MAX);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc + 0x4); // BLTU (branch if x1 < x0) - should not branch because x1 > x0

    state = state.clock(&program); 
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc + 0x8); // BLTU (branch if x0 < x1) - should branch because x0 < x1
    assert_eq!(state.pipeline.if_pc, 16);

    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    // ADDI ( x5 := x0 + 2)
    assert_eq!(state.x[5], 0);
    state = state.clock(&program);
    assert_eq!(state.x[5], 2);
}

#[test]
fn test_BGEU() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program); //WB
    assert_eq!(state.x[1], u32::MAX); // ADDI ( x1 := x0 - 1)
    state = state.clock(&program); // BGEU (branch if x0 >= x1) - should not branch because x0 < x1
    assert_eq!(state.pipeline.if_pc, pc + 0x4);

    // BLT (branch if x1 >= x0) - should branch because x1 > x0
    state = state.clock(&program);
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc + 0x8);
    assert_eq!(16, pc + 0x8);


    state = state.clock(&program);
    state = state.clock(&program);
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.pipeline.if_pc, pc - 0x8); // BGEU (branch if x0 >= x2) - should branch because x0 == x2
    assert_eq!(12, pc - 0x8);

    // ADDI ( x5 := x0 + 2)
    assert_eq!(state.x[5], 0);
    state = state.clock(&program);
    assert_eq!(state.x[5], 2);

    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    // ADDI ( x5 := x0 + 1)
    assert_eq!(state.x[5], 2);
    state = state.clock(&program);
    assert_eq!(state.x[5], 1);
}

#[test]
fn test_LB() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state.data_memory.set(0x10, 0xFB);
    state.data_memory.set(0x11, 0xFC);
    state.data_memory.set(0x12, 0x7D);
    state.data_memory.set(0x13, 0x7E);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // ADDI ( x1 := x0 + 0x8)
    state = state.clock(&program);
    assert_eq!(state.x[1], 8);

    // LB ( x5 := MEM[x1 + 0x8])
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[5], 0xFFFFFFFB);

    // LB ( x5 := MEM[x1 + 0xA])
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[5], 0x0000007D);
}

#[test]
fn test_LB_with_JAL() {
    let program = populate(&[
        ISA::LB.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 0x0,
            ..Default::default()
        }),
        ISA::JAL.build(Operands {
            rd: 10,
            imm: -4,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 0x8,
            ..Default::default()
        }),
    ]);

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state.data_memory.set(0x00, 0xDE);
    state.data_memory.set(0x01, 0xAD);
    state.data_memory.set(0x02, 0xBE);
    state.data_memory.set(0x03, 0xEF);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM1
    let pc = state.pipeline.if_id.id_pc.unwrap();
    state = state.clock(&program); //MEM2

    // LB ( x2 := MEM[x0 + 0x0])
    assert_eq!(state.x[2], 0);
    state = state.clock(&program);
    assert_eq!(state.x[2], 0xFFFFFFDE);
    assert_eq!(state.pipeline.if_pc, 0);
    assert_eq!(pc - 4, 0);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
}

#[test]
fn test_LB_with_ADDI() {
    let program = populate(&[
        ISA::LB.build(Operands {
            rd: 2,
            rs1: 0,
            imm: 0x0,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 0,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 3,
            rs1: 0,
            imm: 0x8,
            ..Default::default()
        }),
    ]);

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state.data_memory.set(0x00, 0xDE);
    state.data_memory.set(0x01, 0xAD);
    state.data_memory.set(0x02, 0xBE);
    state.data_memory.set(0x03, 0xEF);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM1
    state = state.clock(&program); //MEM2

    // LB ( x2 := MEM[x0 + 0x0])
    assert_eq!(state.x[2], 0);
    state = state.clock(&program);
    assert_eq!(state.x[2], 0xFFFFFFDE);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
}

#[test]
fn test_LB_and_SB() {
    let program = populate(&[
        ISA::ADDI.build(Operands {
            rd: 10,
            rs1: 0,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::LB.build(Operands {
            rd: 2,
            rs1: 10,
            imm: 0x8,
            ..Default::default()
        }),
        ISA::SB.build(Operands {
            rs1: 1,
            rs2: 2,
            imm: 0x0,
            ..Default::default()
        }),
        ISA::ADDI.build(Operands {
            rd: 1,
            rs1: 1,
            imm: 0x1,
            ..Default::default()
        }),
    ]);

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state.data_memory.set(0x10, 0xFB);
    state.data_memory.set(0x11, 0xFC);
    state.data_memory.set(0x12, 0x7D);
    state.data_memory.set(0x13, 0x7E);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // ADDI ( x10 := x0 + 0x8)
    state = state.clock(&program);
    assert_eq!(state.x[10], 8);

    // LB ( x2 := MEM[x10 + 0x8] )
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[2], 0xFFFFFFFB);

    // SB ( MEM[x1 + 0x0] := x2 )
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.data_memory.get(0x0), 0xFB);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1], 0);
    state = state.clock(&program);
    assert_eq!(state.x[1], 1);
}

#[test]
fn test_LH() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state.data_memory.set(0x10, 0xFB);
    state.data_memory.set(0x11, 0xFC);
    state.data_memory.set(0x12, 0x7D);
    state.data_memory.set(0x13, 0x7E);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // ADDI ( x1 := x0 + 0x8)
    state = state.clock(&program);
    assert_eq!(state.x[1], 8);

    // LB ( x5 := MEM[x1 + 0x8])
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[5], 0xFFFFFCFB);

    // LB ( x5 := MEM[x1 + 0xA])
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[5], 0x00007E7D);
}

#[test]
fn test_LW() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state.data_memory.set(0x10, 0xFB);
    state.data_memory.set(0x11, 0xFC);
    state.data_memory.set(0x12, 0x7D);
    state.data_memory.set(0x13, 0x7E);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // ADDI ( x1 := x0 + 0x8)
    state = state.clock(&program);
    assert_eq!(state.x[1], 8);

    // LB ( x5 := MEM[x1 + 0x8])
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[5], 0x7E7DFCFB);
}

#[test]
fn test_SB() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 0xFEFDFCFB
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);

    // Set x2 := 100
    state = state.clock(&program);

    // SH -> Write first byte of x1 to address x2 (100) + 5
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.data_memory.get(105), 0xFB);
    assert_eq!(state.data_memory.get(106), 0x00);
    assert_eq!(state.data_memory.get(107), 0x00);
    assert_eq!(state.data_memory.get(108), 0x00);
}

#[test]
fn test_SH() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 0xFEFDFCFB (lowest byte )
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);

    // Set x2 := 100
    state = state.clock(&program);

    // SH -> Write x1 to address x2 (100) + 5
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.data_memory.get(105), 0xFB);
    assert_eq!(state.data_memory.get(106), 0xFC);
    assert_eq!(state.data_memory.get(107), 0x00);
    assert_eq!(state.data_memory.get(108), 0x00);
}

#[test]
fn test_SW() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 0xFEFDFCFB
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);

    // Set x2 := 100
    state = state.clock(&program);

    // SW -> Write x1 to address x2 (100) + 5
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.data_memory.get(105), 0xFB);
    assert_eq!(state.data_memory.get(106), 0xFC);
    assert_eq!(state.data_memory.get(107), 0xFD);
    assert_eq!(state.data_memory.get(108), 0xFE);
}

#[test]
fn test_ADDI() {
    // ADDI ( x1 := x0 + 1)
    // ADDI ( x1 := x1 + (-1))
    // ADDI ( x0 := x0 + 1 )
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // ADDI ( x1 := x0 + 1)
    state = state.clock(&program);
    assert_eq!(state.x[1], 1);
    // ADDI ( x1 := x1 + 1)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1], 0);
    // ADDI ( x0 := x0 + 1) <= special case should be a noop
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_SLTI() {
    // SLTI ( x1 := x0 < 1)
    // SLTI ( x1 := x1 < (-1))
    // SLTI ( x0 := x0 < 1 )
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // SLTI ( x1 := x0 < 1)
    state = state.clock(&program);
    assert_eq!(state.x[1], 1);
    // SLTI ( x1 := x1 < (-1))
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1], 0);
    // SLTI ( x0 := x0 < 1 ) <= Should not change x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_SLTIU() {
    // SLTIU ( x1 := x0 < 1)
    // SLTIU ( x1 := x1 < (-1))
    // SLTIU ( x0 := x0 < 1 )
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // SLTI ( x1 := x0 < 1)
    state = state.clock(&program);
    assert_eq!(state.x[1], 1);
    // SLTI ( x1 := x1 < (-1))
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1], 1);
    // SLTI ( x0 := x0 < 1 ) <= Should not change x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_XORI() {
    // XORI ( x1 := x0 ^ 4)
    // XORI ( x1 := x1 ^ (-1))
    // XORI ( x0 := x0 ^ 100 )
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // XORI ( x1 := x0 ^ 4)
    state = state.clock(&program);
    assert_eq!(state.x[1], 4);
    // XORI ( x1 := x1 ^ (-1))
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1] as i32, -5);
    // XORI ( x0 := x0 ^ 100 ) <= Should not change x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_ORI() {
    // ORI ( x1 := x0 | 12)
    // ORI ( x1 := x1 | (-1))
    // ORI ( x0 := x0 | 100 )
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // ORI ( x1 := x0 | 12)
    state = state.clock(&program);
    assert_eq!(state.x[1], 12);
    // ORI ( x1 := x1 ^ (-10))
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1] as i32, -2);
    // ORI ( x0 := x0 ^ 100 ) <= Should not change x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_ANDI() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 37
    state = state.clock(&program);
    assert_eq!(state.x[1], 37);

    // ANDI ( x1 := x1 & 5)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1], 5);

    // ANDI ( x1 := x1 & (-10))
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[1], 4);

    // ANDI ( x0 := x0 & 100 ) <= Should not change x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_SLLI() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 10
    state = state.clock(&program);
    assert_eq!(state.x[1], 10);

    // SLLI ( x2 := x1 << 4)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[2], 160);

    // SLLI ( x3 := x1 << 0b1000001) Should only shift 1 time since we only look at last 5 bits
    state = state.clock(&program);
    assert_eq!(state.x[3], 20);

    // SLLI ( x0 := x1 << 3 ) <= Should not change x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_SRLI() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 10
    state = state.clock(&program);
    assert_eq!(state.x[1], 10);

    // SRLI ( x2 := x1 >> 1)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[2], 5);

    // SRLI ( x3 := x1 >> 0b1000010) Should only shift 1 time since we only look at last 5 bits
    state = state.clock(&program);
    assert_eq!(state.x[3], 2);

    // SRLI ( x0 := x1 << 3 ) <= Should not change x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_SRAI() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := -10
    state = state.clock(&program);
    assert_eq!(state.x[1] as i32, -10);

    // SRAI ( x2 := x1 >> -1)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[2] as i32, -1);

    // SRAI ( x3 := x1 >> 0b1000001) Should only shift 1 time since we only look at last 5 bits
    state = state.clock(&program);
    assert_eq!(state.x[3] as i32, -5);

    // SRAI ( x0 := x1 << 3 ) <= Should not change x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_ADD() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 15
    state = state.clock(&program);
    assert_eq!(state.x[1] as i32, 15);

    // Set x2 := -10
    state = state.clock(&program);
    assert_eq!(state.x[2] as i32, -10);

    // ADD (x3 := x1 + x2)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[3] as i32, 5);

    // ADD (x4 := x1 + x1)
    state = state.clock(&program);
    assert_eq!(state.x[4] as i32, 30);

    // ADD (x0 := x1 + x2) - No change to x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_SUB() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 20
    state = state.clock(&program);
    assert_eq!(state.x[1] as i32, 20);

    // Set x2 := 5
    state = state.clock(&program);
    assert_eq!(state.x[2] as i32, 5);

    // SUB (x3 := x1 - x2)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[3] as i32, 15);

    // SUB (x4 := x2 - x1)
    state = state.clock(&program);
    assert_eq!(state.x[4] as i32, -15);

    // SUB (x5 := x1 - x1)
    state = state.clock(&program);
    assert_eq!(state.x[5] as i32, 0);

    // SUB (x0 := x1 - x2) - No change to x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_SLL() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    // Set x1 := 1
    state = state.clock(&program);
    assert_eq!(state.x[1] as i32, 1);

    // Set x2 := 2
    state = state.clock(&program);
    assert_eq!(state.x[2] as i32, 2);

    // SLL (x3 := x1 << x2)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[3] as i32, 4);

    // Set x2 := 0b100000 (masked to 0)
    state = state.clock(&program);

    // SLL (x4 := x1 << x2, with x2 effectively 0)
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[4] as i32, 1);

    // SLL (x5 := x2 << x2)
    state = state.clock(&program);
    assert_eq!(state.x[5] as i32, 32);

    // SLL (x0 := x1 << x2) - Ensure no change to x0
    state = state.clock(&program);
    assert_eq!(state.x[0], 0);
}

#[test]
fn test_SLT() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    // Execute each instruction and validate
    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM

    state = state.clock(&program); // Set x1 = 5
    state = state.clock(&program); // Set x2 = 10
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[3], 1); // x3 = 1 (5 < 10)

    state = state.clock(&program);
    assert_eq!(state.x[4], 0); // x4 = 0 (10 < 5 false)

    state = state.clock(&program);
    assert_eq!(state.x[5], 0); // x5 = 0 (5 < 5 false)
}

#[test]
fn test_SLTU() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program); // Set x1 = -1 (0xFFFFFFFF unsigned)
    state = state.clock(&program); // Set x2 = 1
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    assert_eq!(state.x[3], 1); // x3 = 1 (1 < 0xFFFFFFFF true)

    state = state.clock(&program);
    assert_eq!(state.x[4], 0); // x4 = 0 (0xFFFFFFFF < 1 false)
}

#[test]
fn test_XOR() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);

    assert_eq!(state.x[3], 0b0110); // x3 = 6 (0b1100 ^ 0b1010)
}

#[test]
fn test_SRL() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);

    assert_eq!(state.x[3], 4); // x3 = 4 (16 >> 2)
}

#[test]
fn test_SRA() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);

    assert_eq!(state.x[3] as i32, -4); // x3 = -4 (-16 >> 2, arithmetic)
}

#[test]
fn test_OR() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);

    assert_eq!(state.x[3], 0b1110); // x3 = 14 (0b1100 | 0b1010)
}

#[test]
fn test_AND() {
    let program = populate(&[
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

    let mut state = EmulatorState::<FiveStagePipeline>::new(&program);

    state = state.clock(&program); //IF
    state = state.clock(&program); //ID
    state = state.clock(&program); //EX
    state = state.clock(&program); //MEM
    state = state.clock(&program); //WB
    state = state.clock(&program);
    state = state.clock(&program);
    state = state.clock(&program); //extra clock cycles because of hazard
    state = state.clock(&program);
    state = state.clock(&program);

    assert_eq!(state.x[3], 0b1000); // x3 = 8 (0b1100 & 0b1010)
}
