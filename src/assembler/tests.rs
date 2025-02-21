use std::collections::BTreeMap;

use bimap::BiBTreeMap;

use crate::assembler;
use crate::assembler::lexer::Lexer;

use super::assemble;
use crate::include_test_file;

#[test]
fn test_lexer() {
    let source = include_test_file!("syntax-check.s");

    let mut lexer = Lexer::new(source).peekable();
    let tokens: Vec<_> = lexer.map(|token_result| token_result.unwrap()).collect();

    for token in tokens {
        println!("{:?}", token);
    }
}

#[test]
fn test_assembler() {
    let source = include_test_file!("syntax-check.s");
    let assembled_program_new = assembler::assembler::assemble(source);
    let assembled_program = assemble(source);

    if let (Ok(assembled_program), Ok(assembled_program_new)) =
        (&assembled_program, &assembled_program_new)
    {
        for (label, addr) in &assembled_program.labels {
            assert_eq!(
                Some(addr),
                assembled_program_new.labels.get(label),
                "Mismatch in label \"{}\"",
                label
            );
            println!(
                "{}: 0x{:08X}, 0x{:08X}",
                label,
                addr,
                assembled_program_new.labels.get(label).unwrap()
            );
        }

        for (label, addr) in &assembled_program.data_labels {
            assert_eq!(
                Some(addr),
                assembled_program_new.data_labels.get(label),
                "Mismatch in label \"{}\"",
                label
            );
            println!(
                "{}: 0x{:08X}, 0x{:08X}",
                label,
                addr,
                assembled_program_new.data_labels.get(label).unwrap()
            );
        }

        for (addr, line) in &assembled_program.source_map {
            assert_eq!(
                Some(line),
                assembled_program_new.source_map.get_by_left(addr),
                "Mismatch in source map at address 0x{:08X}",
                addr
            );
            println!(
                "0x{:08X}: Line {}, {}",
                addr,
                line,
                assembled_program_new.source_map.get_by_left(addr).unwrap()
            );
        }

        for (addr, instr) in &assembled_program.instruction_memory {
            assert_eq!(
                Some(instr),
                assembled_program_new.instruction_memory.get(addr),
                "Mismatch in instruction memory at address 0x{:08X}",
                addr
            );
            println!(
                "0x{:08X}: 0x{:08X}, 0x{:08X}",
                addr,
                instr,
                assembled_program_new.instruction_memory.get(addr).unwrap()
            );
        }
    } else if let (Err(_), Err(_)) = (&assembled_program, &assembled_program_new) {
        // Both should fail
    } else {
        eprint!("{:?}\n{:?}", assembled_program, assembled_program_new);
        panic!("Mismatch in assembly results");
    }
}

#[ignore]
#[test]
fn print_some_output() {
    let program = include_test_file!("simple-loop.s");
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, source_map, data_mem) = assembled_program.emulator_maps();

    println!("Instruction Memory (Address -> Byte):");
    for (&addr, &byte) in inst_mem {
        println!("0x{:08X}: 0x{:02X}", addr, byte);
    }

    println!("\nSource Map (Address -> Line Number):");
    for (&addr, &line) in source_map {
        println!("0x{:08X}: Line {}", addr, line);
    }

    println!("\nData Memory (Address -> Byte):");
    for (&addr, &byte) in data_mem {
        println!("0x{:08X}: 0x{:02X}", addr, byte);
    }

    println!("\nReconstructed 32-bit Instructions:");
    for &addr in source_map.left_values() {
        let instruction = u32::from_le_bytes([
            inst_mem[&addr],
            inst_mem[&(addr + 1)],
            inst_mem[&(addr + 2)],
            inst_mem[&(addr + 3)],
        ]);
        println!("0x{:08X}: 0x{:08X}", addr, instruction);
    }
}

#[test]
fn test_large_word() {
    let program = ".data\nwords: .word 12345678";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (_, _, data_mem) = assembled_program.emulator_maps();

    // these bytes are just 12345678 in little endian so essentially what .word should store
    let expected_bytes = [0x4E, 0x61, 0xBC, 0x00];

    for i in 0..4 {
        assert_eq!(
            data_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of word value",
            i
        );
    }
}

#[test]
fn test_ADD() {
    let program = ".text\nADD X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x00, 0x31, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of ADD instruction",
            i
        );
    }
}

#[test]
fn test_SUB() {
    let program = ".text\nSUB X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x00, 0x31, 0x40];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SUB instruction",
            i
        );
    }
}

#[test]
fn test_SLT() {
    let program = ".text\nSLT X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x20, 0x31, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SLT instruction",
            i
        );
    }
}

#[test]
fn test_SLTU() {
    let program = ".text\nSLTU X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x30, 0x31, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SLTU instruction",
            i
        );
    }
}

#[test]
fn test_AND() {
    let program = ".text\nAND X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x70, 0x31, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of AND instruction",
            i
        );
    }
}

#[test]
fn test_OR() {
    let program = ".text\nOR X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x60, 0x31, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of OR instruction",
            i
        );
    }
}

#[test]
fn test_XOR() {
    let program = ".text\nXOR X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x40, 0x31, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of XOR instruction",
            i
        );
    }
}

#[test]
fn test_SLL() {
    let program = ".text\nSLL X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x10, 0x31, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SLL instruction",
            i
        );
    }
}

#[test]
fn test_SRL() {
    let program = ".text\nSRL X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x50, 0x31, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SRL instruction",
            i
        );
    }
}

#[test]
fn test_SRA() {
    let program = ".text\nSRA X1, X2, X3";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB3, 0x50, 0x31, 0x40];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SRA instruction",
            i
        );
    }
}

#[test]
fn test_ADDI() {
    let program = ".text\nADDI X1, X2, 10";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x00, 0xA1, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of ADDI instruction",
            i
        );
    }
}

#[test]
fn test_SLTI() {
    let program = ".text\nSLTI X1, X2, 10";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x20, 0xA1, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SLTI instruction",
            i
        );
    }
}

#[test]
fn test_SLTIU() {
    let program = ".text\nSLTIU X1, X2, 10";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x30, 0xA1, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SLTIU instruction",
            i
        );
    }
}

#[test]
fn test_ANDI() {
    let program = ".text\nANDI X1, X2, 0xFF";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x70, 0xF1, 0x0F];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of ANDI instruction",
            i
        );
    }
}

#[test]
fn test_ORI() {
    let program = ".text\nORI X1, X2, 0xFF";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x60, 0xF1, 0x0F];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of ORI instruction",
            i
        );
    }
}

#[test]
fn test_XORI() {
    let program = ".text\nXORI X1, X2, 0xFF";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x40, 0xF1, 0x0F];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of XORI instruction",
            i
        );
    }
}

#[test]
fn test_SLLI() {
    let program = ".text\nSLLI X1, X2, 2";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x10, 0x21, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SLLI instruction",
            i
        );
    }
}

#[test]
fn test_SRLI() {
    let program = ".text\nSRLI X1, X2, 2";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x50, 0x21, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SRLI instruction",
            i
        );
    }
}

#[test]
fn test_SRAI() {
    let program = ".text\nSRAI X1, X2, 2";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x93, 0x50, 0x21, 0x40];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of SRAI instruction",
            i
        );
    }
}

#[test]
fn test_JALR() {
    let program = ".text\nJALR X1, X2, 0x100";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xE7, 0x00, 0x01, 0x10];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of JALR instruction",
            i
        );
    }
}

#[test]
fn test_LW() {
    let program = ".text\nLW X1, 0(X2)";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x83, 0x20, 0x01, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of LW instruction",
            i
        );
    }
}

#[test]
fn test_LH() {
    let program = ".text\nLH X1, 0(X2)";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x83, 0x10, 0x01, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of LH instruction",
            i
        );
    }
}

#[test]
fn test_LHU() {
    let program = ".text\nLHU X1, 0(X2)";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x83, 0x50, 0x01, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of LHU instruction",
            i
        );
    }
}

#[test]
fn test_LB() {
    let program = ".text\nLB X1, 0(X2)";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x83, 0x00, 0x01, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of LB instruction",
            i
        );
    }
}

#[test]
fn test_LBU() {
    let program = ".text\nLBU X1, 0(X2)";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x83, 0x40, 0x01, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of LBU instruction",
            i
        );
    }
}

#[test]
fn test_FENCE() {
    let program = ".text\nFENCE";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x0F, 0x00, 0x00, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of FENCE instruction",
            i
        );
    }
}

#[test]
fn test_ECALL() {
    let program = ".text\nECALL";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x73, 0x00, 0x00, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of ECALL instruction",
            i
        );
    }
}

#[test]
fn test_EBREAK() {
    let program = ".text\nEBREAK";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x73, 0x00, 0x10, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of EBREAK instruction",
            i
        );
    }
}

#[test]
fn test_BEQ() {
    let program = ".text\nlabel:\nBEQ X1, X2, label";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x63, 0x80, 0x20, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of BEQ instruction",
            i
        );
    }
}

#[test]
fn test_BNE() {
    let program = ".text\nlabel:\nBNE X1, X2, label";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x63, 0x90, 0x20, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of BNE instruction",
            i
        );
    }
}

#[test]
fn test_BLT() {
    let program = ".text\nlabel:\nBLT X1, X2, label";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x63, 0xC0, 0x20, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of BLT instruction",
            i
        );
    }
}

#[test]
fn test_BLTU() {
    let program = ".text\nlabel:\nBLTU X1, X2, label";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x63, 0xE0, 0x20, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of BLTU instruction",
            i
        );
    }
}

#[test]
fn test_BGE() {
    let program = ".text\nlabel:\nBGE X1, X2, label";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x63, 0xD0, 0x20, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of BGE instruction",
            i
        );
    }
}

#[test]
fn test_BGEU() {
    let program = ".text\nlabel:\nBGEU X1, X2, label";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x63, 0xF0, 0x20, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of BGEU instruction",
            i
        );
    }
}

#[test]
fn test_LUI() {
    let program = ".text\nLUI X1, 0xFFF";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0xB7, 0xF0, 0xFF, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of LUI instruction",
            i
        );
    }
}

#[test]
fn test_AUIPC() {
    let program = ".text\nAUIPC X1, 0xFFF";
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, _, _) = assembled_program.emulator_maps();

    let expected_bytes = [0x97, 0xF0, 0xFF, 0x00];

    for i in 0..4 {
        assert_eq!(
            inst_mem.get(&(i as u32)),
            Some(&expected_bytes[i]),
            "Mismatch at byte {} of AUIPC instruction",
            i
        );
    }
}

#[test]
fn assembler_different_locations() {
    let program = include_test_file!("different-locations.s");
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, source_map, data_mem) = assembled_program.emulator_maps();

    // actual instruction memory
    let expected_instructions: Vec<(u32, u8)> = vec![
        (0x0100, 0x83),
        (0x0101, 0x20),
        (0x0102, 0x80),
        (0x0103, 0x3E),
        (0x0104, 0x03),
        (0x0105, 0x21),
        (0x0106, 0xC0),
        (0x0107, 0x3E),
        (0x0108, 0xB3),
        (0x0109, 0x81),
        (0x010A, 0x20),
        (0x010B, 0x00),
        (0x010C, 0x23),
        (0x010D, 0x28),
        (0x010E, 0x30),
        (0x010F, 0x3E),
    ];

    for (addr, expected_byte) in expected_instructions {
        assert_eq!(
            inst_mem.get(&addr),
            Some(&expected_byte),
            "Mismatch in instruction memory at address 0x{:08X}",
            addr
        );
    }

    // source map stuff
    let expected_source_lines: Vec<(u32, usize)> =
        vec![(0x0100, 3), (0x0104, 4), (0x0108, 5), (0x010C, 6)];

    for (addr, expected_line) in expected_source_lines {
        assert_eq!(
            source_map.get_by_left(&addr),
            Some(&expected_line),
            "Mismatch in source map at address 0x{:08X}",
            addr
        );
    }

    // data memory starting from 1000 = hex 0x03E8
    let expected_data: Vec<(u32, u8)> = vec![
        (0x03E8, 0x2A),
        (0x03E9, 0x00),
        (0x03EA, 0x00),
        (0x03EB, 0x00), // 42
        (0x03EC, 0x3A),
        (0x03ED, 0x00),
        (0x03EE, 0x00),
        (0x03EF, 0x00), // 58
        (0x03F0, 0x00),
        (0x03F1, 0x00),
        (0x03F2, 0x00),
        (0x03F3, 0x00), // 0
    ];

    for (addr, expected_byte) in expected_data {
        assert_eq!(
            data_mem.get(&addr),
            Some(&expected_byte),
            "Mismatch in data memory at address 0x{:08X}",
            addr
        );
    }

    // Test reconstructed 32-bit instructions
    let expected_32bit_instructions: Vec<(u32, u32)> = vec![
        (0x0100, 0x3E802083), // lw x1, value1
        (0x0104, 0x3EC02103), // lw x2, value2
        (0x0108, 0x002081B3), // add x3, x1, x2
        (0x010C, 0x3E302823), // sw x3, result
    ];

    for (addr, expected_instruction) in expected_32bit_instructions {
        let actual_instruction = u32::from_le_bytes([
            *inst_mem.get(&addr).unwrap(),
            *inst_mem.get(&(addr + 1)).unwrap(),
            *inst_mem.get(&(addr + 2)).unwrap(),
            *inst_mem.get(&(addr + 3)).unwrap(),
        ]);
        assert_eq!(
            actual_instruction, expected_instruction,
            "Mismatch in reconstructed instruction at address 0x{:08X}",
            addr
        );
    }
}

#[test]
fn assembler_simple_loop() {
    let program = include_test_file!("simple-loop.s");
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, source_map, data_mem) = assembled_program.emulator_maps();

    // Verify instruction memory
    let expected_inst_mem: BTreeMap<u32, u8> = [
        (0x00000000, 0x93),
        (0x00000001, 0x02),
        (0x00000002, 0x00),
        (0x00000003, 0x00),
        (0x00000004, 0x13),
        (0x00000005, 0x03),
        (0x00000006, 0x10),
        (0x00000007, 0x00),
        (0x00000008, 0x93),
        (0x00000009, 0x03),
        (0x0000000A, 0x50),
        (0x0000000B, 0x00),
        (0x0000000C, 0xB3),
        (0x0000000D, 0x82),
        (0x0000000E, 0x62),
        (0x0000000F, 0x00),
        (0x00000010, 0x13),
        (0x00000011, 0x03),
        (0x00000012, 0x13),
        (0x00000013, 0x00),
        (0x00000014, 0x33),
        (0x00000015, 0xA4),
        (0x00000016, 0x63),
        (0x00000017, 0x00),
        (0x00000018, 0xE3),
        (0x00000019, 0x0A),
        (0x0000001A, 0x04),
        (0x0000001B, 0xFE),
        (0x0000001C, 0xB7),
        (0x0000001D, 0x04),
        (0x0000001E, 0x00),
        (0x0000001F, 0x00),
        (0x00000020, 0x93),
        (0x00000021, 0x84),
        (0x00000022, 0x04),
        (0x00000023, 0x00),
        (0x00000024, 0x23),
        (0x00000025, 0xA0),
        (0x00000026, 0x54),
        (0x00000027, 0x00),
        (0x00000028, 0x73),
        (0x00000029, 0x00),
        (0x0000002A, 0x00),
        (0x0000002B, 0x00),
    ]
    .iter()
    .cloned()
    .collect();

    // Verify source map
    let expected_source_map: BiBTreeMap<u32, usize> = [
        (0x00000000, 10),
        (0x00000004, 11),
        (0x00000008, 12),
        (0x0000000C, 15),
        (0x00000010, 16),
        (0x00000014, 17),
        (0x00000018, 18),
        (0x0000001C, 21),
        (0x00000020, 22),
        (0x00000024, 23),
        (0x00000028, 26),
    ]
    .iter()
    .cloned()
    .collect();

    // Verify data memory
    let expected_data_mem: BTreeMap<u32, u8> = [
        (0x00000000, 0x00),
        (0x00000001, 0x00),
        (0x00000002, 0x00),
        (0x00000003, 0x00),
    ]
    .iter()
    .cloned()
    .collect();

    // Compare actual outputs with expected values
    assert_eq!(*inst_mem, expected_inst_mem, "Instruction memory mismatch");
    assert_eq!(*source_map, expected_source_map, "Source map mismatch");
    assert_eq!(*data_mem, expected_data_mem, "Data memory mismatch");

    // Optional: Print details if test fails
    if *inst_mem != expected_inst_mem {
        println!("Instruction Memory Differences:");
        for (&addr, &byte) in inst_mem {
            let expected = expected_inst_mem.get(&addr);
            if expected != Some(&byte) {
                println!(
                    "0x{:08X}: Got 0x{:02X}, Expected {:?}",
                    addr, byte, expected
                );
            }
        }
        for (&addr, &byte) in &expected_inst_mem {
            if !inst_mem.contains_key(&addr) {
                println!("0x{:08X}: Missing, Expected 0x{:02X}", addr, byte);
            }
        }

        // Print full 32-bit instructions for debugging
        println!("\nReconstructed 32-bit Instructions:");
        for &addr in source_map.left_values() {
            let actual = u32::from_le_bytes([
                inst_mem[&addr],
                inst_mem[&(addr + 1)],
                inst_mem[&(addr + 2)],
                inst_mem[&(addr + 3)],
            ]);
            let expected = u32::from_le_bytes([
                expected_inst_mem[&addr],
                expected_inst_mem[&(addr + 1)],
                expected_inst_mem[&(addr + 2)],
                expected_inst_mem[&(addr + 3)],
            ]);
            if actual != expected {
                println!(
                    "0x{:08X}: Got 0x{:08X}, Expected 0x{:08X}",
                    addr, actual, expected
                );
            }
        }
    }

    if *source_map != expected_source_map {
        println!("Source Map Differences:");
        for (&addr, &line) in source_map {
            let expected = expected_source_map.get_by_left(&addr);
            if expected != Some(&line) {
                println!("0x{:08X}: Got line {}, Expected {:?}", addr, line, expected);
            }
        }
        for (&addr, &line) in &expected_source_map {
            if !source_map.contains_left(&addr) {
                println!("0x{:08X}: Missing, Expected line {}", addr, line);
            }
        }
    }

    if *data_mem != expected_data_mem {
        println!("Data Memory Differences:");
        for (&addr, &byte) in data_mem {
            let expected = expected_data_mem.get(&addr);
            if expected != Some(&byte) {
                println!(
                    "0x{:08X}: Got 0x{:02X}, Expected {:?}",
                    addr, byte, expected
                );
            }
        }
        for (&addr, &byte) in &expected_data_mem {
            if !data_mem.contains_key(&addr) {
                println!("0x{:08X}: Missing, Expected 0x{:02X}", addr, byte);
            }
        }
    }
}

#[test]
fn assembler_all_instructions() {
    let program = include_test_file!("syntax-check.s");
    let assembled_program = assemble(program).unwrap_or_else(|errors| {
        for error in errors {
            panic!(
                "Assembly Error on line {}, column {}: {}",
                error.line_number, error.column, error.error_message
            );
        }
        panic!("Assembly failed");
    });
    let (inst_mem, source_map, data_mem) = assembled_program.emulator_maps();

    // actual instruction memory
    let expected_inst_mem: BTreeMap<u32, u8> = [
        (0x00000000, 0x93),
        (0x00000001, 0x02),
        (0x00000002, 0x53),
        (0x00000003, 0x00),
        (0x00000004, 0x93),
        (0x00000005, 0x22),
        (0x00000006, 0x53),
        (0x00000007, 0x00),
        (0x00000008, 0x93),
        (0x00000009, 0x32),
        (0x0000000A, 0x53),
        (0x0000000B, 0x00),
        (0x0000000C, 0x93),
        (0x0000000D, 0x72),
        (0x0000000E, 0xF3),
        (0x0000000F, 0x0F),
        (0x00000010, 0x93),
        (0x00000011, 0x62),
        (0x00000012, 0xF3),
        (0x00000013, 0x0F),
        (0x00000014, 0x93),
        (0x00000015, 0x42),
        (0x00000016, 0xF3),
        (0x00000017, 0x0F),
        (0x00000018, 0x93),
        (0x00000019, 0x12),
        (0x0000001A, 0x23),
        (0x0000001B, 0x00),
        (0x0000001C, 0x93),
        (0x0000001D, 0x52),
        (0x0000001E, 0x23),
        (0x0000001F, 0x00),
        (0x00000020, 0x93),
        (0x00000021, 0x52),
        (0x00000022, 0x23),
        (0x00000023, 0x40),
        (0x00000024, 0xB7),
        (0x00000025, 0xF2),
        (0x00000026, 0xFF),
        (0x00000027, 0x00),
        (0x00000028, 0x97),
        (0x00000029, 0xF2),
        (0x0000002A, 0xFF),
        (0x0000002B, 0x00),
        (0x0000002C, 0xB3),
        (0x0000002D, 0x02),
        (0x0000002E, 0x73),
        (0x0000002F, 0x00),
        (0x00000030, 0xB3),
        (0x00000031, 0x02),
        (0x00000032, 0x73),
        (0x00000033, 0x40),
        (0x00000034, 0xB3),
        (0x00000035, 0x22),
        (0x00000036, 0x73),
        (0x00000037, 0x00),
        (0x00000038, 0xB3),
        (0x00000039, 0x32),
        (0x0000003A, 0x73),
        (0x0000003B, 0x00),
        (0x0000003C, 0xB3),
        (0x0000003D, 0x72),
        (0x0000003E, 0x73),
        (0x0000003F, 0x00),
        (0x00000040, 0xB3),
        (0x00000041, 0x62),
        (0x00000042, 0x73),
        (0x00000043, 0x00),
        (0x00000044, 0xB3),
        (0x00000045, 0x42),
        (0x00000046, 0x73),
        (0x00000047, 0x00),
        (0x00000048, 0xB3),
        (0x00000049, 0x12),
        (0x0000004A, 0x73),
        (0x0000004B, 0x00),
        (0x0000004C, 0xB3),
        (0x0000004D, 0x52),
        (0x0000004E, 0x73),
        (0x0000004F, 0x00),
        (0x00000050, 0xB3),
        (0x00000051, 0x52),
        (0x00000052, 0x73),
        (0x00000053, 0x40),
        (0x00000054, 0xEF),
        (0x00000055, 0x02),
        (0x00000056, 0xC0),
        (0x00000057, 0x04),
        (0x00000058, 0xE7),
        (0x00000059, 0x02),
        (0x0000005A, 0x03),
        (0x0000005B, 0x10),
        (0x0000005C, 0x63),
        (0x0000005D, 0x86),
        (0x0000005E, 0x62),
        (0x0000005F, 0x04),
        (0x00000060, 0x63),
        (0x00000061, 0x96),
        (0x00000062, 0x62),
        (0x00000063, 0x04),
        (0x00000064, 0x63),
        (0x00000065, 0xC6),
        (0x00000066, 0x62),
        (0x00000067, 0x04),
        (0x00000068, 0x63),
        (0x00000069, 0xE6),
        (0x0000006A, 0x62),
        (0x0000006B, 0x04),
        (0x0000006C, 0x63),
        (0x0000006D, 0xD6),
        (0x0000006E, 0x62),
        (0x0000006F, 0x04),
        (0x00000070, 0x63),
        (0x00000071, 0xF6),
        (0x00000072, 0x62),
        (0x00000073, 0x04),
        (0x00000074, 0x83),
        (0x00000075, 0x22),
        (0x00000076, 0x03),
        (0x00000077, 0x00),
        (0x00000078, 0x83),
        (0x00000079, 0x12),
        (0x0000007A, 0x03),
        (0x0000007B, 0x00),
        (0x0000007C, 0x83),
        (0x0000007D, 0x52),
        (0x0000007E, 0x03),
        (0x0000007F, 0x00),
        (0x00000080, 0x83),
        (0x00000081, 0x02),
        (0x00000082, 0x03),
        (0x00000083, 0x00),
        (0x00000084, 0x83),
        (0x00000085, 0x42),
        (0x00000086, 0x03),
        (0x00000087, 0x00),
        (0x00000088, 0x23),
        (0x00000089, 0x20),
        (0x0000008A, 0x53),
        (0x0000008B, 0x00),
        (0x0000008C, 0x23),
        (0x0000008D, 0x10),
        (0x0000008E, 0x53),
        (0x0000008F, 0x00),
        (0x00000090, 0x23),
        (0x00000091, 0x00),
        (0x00000092, 0x53),
        (0x00000093, 0x00),
        (0x00000094, 0x0F),
        (0x00000095, 0x00),
        (0x00000096, 0x00),
        (0x00000097, 0x00),
        (0x00000098, 0x73),
        (0x00000099, 0x00),
        (0x0000009A, 0x00),
        (0x0000009B, 0x00),
        (0x0000009C, 0x73),
        (0x0000009D, 0x00),
        (0x0000009E, 0x10),
        (0x0000009F, 0x00),
        (0x000000A0, 0xEF),
        (0x000000A1, 0x02),
        (0x000000A2, 0x40),
        (0x000000A3, 0x00),
        (0x000000A4, 0xEF),
        (0x000000A5, 0x02),
        (0x000000A6, 0x80),
        (0x000000A7, 0x00),
        (0x000000A8, 0x93),
        (0x000000A9, 0x02),
        (0x000000AA, 0xA3),
        (0x000000AB, 0x00),
        (0x000000AC, 0x93),
        (0x000000AD, 0x02),
        (0x000000AE, 0xA3),
        (0x000000AF, 0x00),
        (0x000000B0, 0x93),
        (0x000000B1, 0x02),
        (0x000000B2, 0xA3),
        (0x000000B3, 0x00),
        (0x000000B4, 0x93),
        (0x000000B5, 0x02),
        (0x000000B6, 0xA3),
        (0x000000B7, 0x00),
        (0x000000B8, 0x93),
        (0x000000B9, 0x02),
        (0x000000BA, 0xA3),
        (0x000000BB, 0x00),
        (0x000000BC, 0x93),
        (0x000000BD, 0x02),
        (0x000000BE, 0xA3),
        (0x000000BF, 0x00),
    ]
    .iter()
    .cloned()
    .collect();

    // Verify instruction source map
    let expected_source_map: BiBTreeMap<u32, usize> = [
        (0x00000000, 10),
        (0x00000004, 11),
        (0x00000008, 12),
        (0x0000000C, 13),
        (0x00000010, 14),
        (0x00000014, 15),
        (0x00000018, 16),
        (0x0000001C, 17),
        (0x00000020, 18),
        (0x00000024, 21),
        (0x00000028, 22),
        (0x0000002C, 25),
        (0x00000030, 26),
        (0x00000034, 27),
        (0x00000038, 28),
        (0x0000003C, 29),
        (0x00000040, 30),
        (0x00000044, 31),
        (0x00000048, 32),
        (0x0000004C, 33),
        (0x00000050, 34),
        (0x00000054, 37),
        (0x00000058, 40),
        (0x0000005C, 43),
        (0x00000060, 44),
        (0x00000064, 45),
        (0x00000068, 46),
        (0x0000006C, 47),
        (0x00000070, 48),
        (0x00000074, 51),
        (0x00000078, 52),
        (0x0000007C, 53),
        (0x00000080, 54),
        (0x00000084, 55),
        (0x00000088, 58),
        (0x0000008C, 59),
        (0x00000090, 60),
        (0x00000094, 63),
        (0x00000098, 64),
        (0x0000009C, 65),
        (0x000000A0, 68),
        (0x000000A4, 71),
        (0x000000A8, 74),
        (0x000000AC, 77),
        (0x000000B0, 80),
        (0x000000B4, 83),
        (0x000000B8, 86),
        (0x000000BC, 89),
    ]
    .iter()
    .cloned()
    .collect();

    // verifying data memory
    let expected_data_mem: BTreeMap<u32, u8> = [
        (0x00000000, 0x74),
        (0x00000001, 0x65),
        (0x00000002, 0x73),
        (0x00000003, 0x74),
        (0x00000004, 0x0A),
        (0x00000005, 0x00),
        (0x00000006, 0x01),
        (0x00000007, 0x00),
        (0x00000008, 0x00),
        (0x00000009, 0x00),
        (0x0000000A, 0x02),
        (0x0000000B, 0x00),
        (0x0000000C, 0x00),
        (0x0000000D, 0x00),
        (0x0000000E, 0x03),
        (0x0000000F, 0x00),
        (0x00000010, 0x00),
        (0x00000011, 0x00),
        (0x00000012, 0x04),
        (0x00000013, 0x00),
        (0x00000014, 0x00),
        (0x00000015, 0x00),
        (0x00000016, 0xFF),
        (0x00000017, 0x42),
        (0x00000018, 0x33),
        (0x00000019, 0x74),
        (0x0000001A, 0x65),
        (0x0000001B, 0x73),
        (0x0000001C, 0x74),
    ]
    .iter()
    .cloned()
    .collect();

    // comparing outputs
    assert_eq!(*inst_mem, expected_inst_mem, "Instruction memory mismatch");
    assert_eq!(*source_map, expected_source_map, "Source map mismatch");
    assert_eq!(*data_mem, expected_data_mem, "Data memory mismatch");

    // printing differences
    if *inst_mem != expected_inst_mem {
        println!("Instruction Memory Differences:");
        for (addr, &byte) in &expected_inst_mem {
            if !inst_mem.contains_key(addr) || inst_mem[addr] != byte {
                println!(
                    "At 0x{:08X}: Expected 0x{:02X}, got {:?}",
                    addr,
                    byte,
                    inst_mem.get(addr)
                );
            }
        }
    }

    if *source_map != expected_source_map {
        println!("Source Map Differences:");
        for (addr, &line) in &expected_source_map {
            if !source_map.contains_left(addr) || source_map.get_by_left(addr) != Some(&line) {
                println!(
                    "At 0x{:08X}: Expected line {}, got {:?}",
                    addr,
                    line,
                    source_map.get_by_left(addr)
                );
            }
        }
    }

    if *data_mem != expected_data_mem {
        println!("Data Memory Differences:");
        for (addr, &byte) in &expected_data_mem {
            if !data_mem.contains_key(addr) || data_mem[addr] != byte {
                println!(
                    "At 0x{:08X}: Expected 0x{:02X}, got {:?}",
                    addr,
                    byte,
                    data_mem.get(addr)
                );
            }
        }
    }
}
