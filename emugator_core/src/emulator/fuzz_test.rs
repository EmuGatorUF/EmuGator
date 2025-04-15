use std::collections::BTreeSet;

use crate::{
    assembler::assemble,
    emulator::{EmulatorState, Pipeline, cve2::CVE2Pipeline, five_stage::FiveStagePipeline},
};

// Random instructions :)
fn gen_random_instruction<R: rand::Rng>(rng: &mut R) -> String {
    match rng.random_range(0..6) {
        0 => {
            let instrs = [
                "ADD", "SUB", "SLL", "SLT", "SLTU", "XOR", "SRL", "SRA", "OR", "AND",
            ];
            let instr = instrs[rng.random_range(0..instrs.len())];
            let rd = format!("x{}", rng.random_range(0..32));
            let rs1 = format!("x{}", rng.random_range(0..32));
            let rs2 = format!("x{}", rng.random_range(0..32));
            format!("{} {}, {}, {}", instr, rd, rs1, rs2)
        }

        1 => {
            let instrs = ["ADDI", "SLTI", "SLTIU", "XORI", "ORI", "ANDI"];
            let instr = instrs[rng.random_range(0..instrs.len())];
            let rd = format!("x{}", rng.random_range(0..32));
            let rs1 = format!("x{}", rng.random_range(0..32));
            let imm = rng.random_range(-2048..2048);
            format!("{} {}, {}, {}", instr, rd, rs1, imm)
        }

        2 => {
            let instrs = ["SLLI", "SRLI", "SRAI"];
            let instr = instrs[rng.random_range(0..instrs.len())];
            let rd = format!("x{}", rng.random_range(0..32));
            let rs1 = format!("x{}", rng.random_range(0..32));
            let shamt = rng.random_range(0..32);
            format!("{} {}, {}, {}", instr, rd, rs1, shamt)
        }

        3 => {
            let instrs = ["LUI", "AUIPC"];
            let instr = instrs[rng.random_range(0..instrs.len())];
            let rd = format!("x{}", rng.random_range(0..32));
            let imm = rng.random_range(0..0x100000);
            format!("{} {}, {}", instr, rd, imm)
        }

        4 => {
            let instrs = ["LB", "LH", "LW", "LBU", "LHU"];
            let instr = instrs[rng.random_range(0..instrs.len())];
            let rd = format!("x{}", rng.random_range(0..32));
            let rs1 = format!("x{}", rng.random_range(0..32));
            let offset = rng.random_range(-2048..2048);
            format!("{} {}, {}({})", instr, rd, offset, rs1)
        }

        5 => {
            let instrs = ["SB", "SH", "SW"];
            let instr = instrs[rng.random_range(0..instrs.len())];
            let rs2 = format!("x{}", rng.random_range(0..32));
            let rs1 = format!("x{}", rng.random_range(0..32));
            let offset = rng.random_range(-2048..2048);
            format!("{} {}, {}({})", instr, rs2, offset, rs1)
        }

        // 6 => {
        //     let rd = format!("x{}", rng.random_range(0..32));
        //     let rs1 = format!("x{}", rng.random_range(0..32));
        //     let offset = rng.random_range(-2048..2048);
        //     format!("JALR {}, {}, {}", rd, rs1, offset)
        // }
        _ => unreachable!(),
    }
}

#[ignore]
#[test]
fn test_fuzz() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use std::fmt::Write;

    let seed = [40u8; 32];
    let mut rng = StdRng::from_seed(seed);

    let mut errors_panic = "".to_string();

    for i in 0..100 {
        let mut source = String::from(".text\n");
        let num_instructions = rng.random_range(5..20);

        for _ in 0..num_instructions {
            let instr = gen_random_instruction(&mut rng);
            writeln!(source, "{}", instr).unwrap();
        }
        writeln!(source, "EBREAK").unwrap();

        let Ok(program) = assemble(&source) else {
            continue;
        };

        let cve2_state = EmulatorState::<CVE2Pipeline>::new(&program);
        let five_stage_state = EmulatorState::<FiveStagePipeline>::new(&program);

        let breakpoints = BTreeSet::new();
        let cve2_state = cve2_state.clock_until_break(&program, &breakpoints, 1_000_000);
        let five_stage_state =
            five_stage_state.clock_until_break(&program, &breakpoints, 1_000_000);

        // If neither made it to the ebreak then skip this case its probably a bad source
        if !cve2_state.pipeline.requesting_debug() && !five_stage_state.pipeline.requesting_debug()
        {
            continue;
        }

        // Check if the states are equal
        let mut current_error_summary = String::new();
        let mut error_detected = false;
        writeln!(
            current_error_summary,
            "Iteration {}:\n\nProgram:\n{}\n",
            i, source,
        )
        .unwrap();

        for reg in 1..32 {
            if cve2_state.x[reg] != five_stage_state.x[reg] {
                writeln!(
                    current_error_summary,
                    "Register x{} mismatch: CVE2: {} Five Stage: {}\n",
                    reg, cve2_state.x[reg], five_stage_state.x[reg]
                )
                .unwrap();
                error_detected = true;
            }
        }

        let merge_keys = BTreeSet::from_iter(
            cve2_state
                .data_memory
                .ram()
                .keys()
                .chain(five_stage_state.data_memory.ram().keys()),
        );

        for key in merge_keys {
            let cve2_value = cve2_state.data_memory.ram().get(key);
            let five_stage_value = five_stage_state.data_memory.ram().get(key);

            if cve2_value != five_stage_value {
                writeln!(
                    current_error_summary,
                    "Data memory mismatch at 0x{:08}: CVE2: {:?} Five Stage: {:?}\n",
                    key, cve2_value, five_stage_value
                )
                .unwrap();
                error_detected = true;
            }
        }

        if cve2_state.data_memory.uart().get_output()
            != five_stage_state.data_memory.uart().get_output()
        {
            writeln!(
                current_error_summary,
                "UART output mismatch: CVE2: {:?} Five Stage: {:?}\n",
                cve2_state.data_memory.uart().get_output(),
                five_stage_state.data_memory.uart().get_output()
            )
            .unwrap();
            error_detected = true;
        }

        if error_detected {
            errors_panic += &current_error_summary;
        }
    }

    if !errors_panic.is_empty() {
        panic!("Fuzz test failed:\n{}", errors_panic);
    }
}
