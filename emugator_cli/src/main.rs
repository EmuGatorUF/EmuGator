use std::collections::BTreeSet;

use clap::Parser;
use emugator_core::{
    assembler::assemble,
    emulator::{EmulatorState, cve2::CVE2Pipeline},
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Folder containing the programs to be simulated
    #[arg(long, default_value_t = String::from("programs"))]
    programs: String,

    /// Folder containing the tests to be run
    #[arg(long, default_value_t = String::from("tests"))]
    tests: String,

    /// Maximum number of clock cycles to simulate a program
    #[arg(short, long, default_value_t = 1_000_000)]
    timeout: usize,
}

fn main() {
    let args = Args::parse();

    // get (name, source) pairs from the programs folder
    let program_files = std::fs::read_dir(&args.programs)
        .expect("Failed to read programs directory")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_file() {
                let name = path.file_stem()?.to_str()?.to_string();
                let source = std::fs::read_to_string(path).ok()?;
                Some((name, source))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for (name, source) in &program_files {
        println!("Running: {}", name);
        // Assemble the program
        let mut program = match assemble(source) {
            Ok(p) => p,
            Err(err) => {
                println!("Failed to assemble: {:?}", err);
                continue;
            }
        };

        let starting_state = EmulatorState::<CVE2Pipeline>::new(&program);
        let ending_state =
            starting_state.clock_until_break(&mut program, &BTreeSet::new(), args.timeout);

        // TODO: run tests
    }
}
