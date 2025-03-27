use std::collections::BTreeSet;

use clap::{Args, Parser, Subcommand};
use emugator_core::{
    assembler::assemble,
    emulator::{EmulatorState, cve2::CVE2Pipeline},
};

/// CLI for the Emugator emulator
#[derive(Parser, Debug)]
#[command(version, about)]
struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    New(NewArgs),
    Test(TestArgs),
}

#[derive(Args, Debug)]
#[command(about)]
/// Create scaffolding folder for a new project
struct NewArgs {
    /// Name of the new folder containing the scaffolding
    name: String,
}

///
/// Runs a set of tests on each input program, outputing the score for each program
///
#[derive(Args, Debug)]
#[command(about)]
struct TestArgs {
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

const EXAMPLE_JSON: &str = r##"
{
    "registers": {
        "1": "0x76",
        "9": "0x42"
    },
    "data_memory": {
        "0x64": "0x00",
        "0x65": "0x01"
    }
}
"##;

fn main() {
    let args = Arguments::parse();
    match args.command {
        Command::New(new_args) => {
            new_project(new_args);
        }
        Command::Test(test_args) => {
            run_tests(test_args);
        }
    }
}

fn new_project(args: NewArgs) {
    // create the new project folder relative to the current directory
    let project_path = std::path::Path::new(&args.name);
    if project_path.exists() {
        println!("Folder already exists");
        return;
    }
    std::fs::create_dir(&project_path).expect("Failed to create project directory");

    // create the programs folder
    let programs_path = project_path.join("programs");
    std::fs::create_dir(&programs_path).expect("Failed to create programs directory");

    // create the tests folder
    let tests_path = project_path.join("tests");
    std::fs::create_dir(&tests_path).expect("Failed to create tests directory");

    // create example test in the tests folder
    let test_name = "example_test";
    let test_path = tests_path.join(test_name);
    std::fs::create_dir(&test_path).expect("Failed to create example test directory");

    // populate example test
    let input_path = test_path.join("input.txt");
    let output_path = test_path.join("output.txt");
    let final_state = test_path.join("final_state.json");
    std::fs::write(&input_path, "input data").expect("Failed to create input file");
    std::fs::write(&output_path, "output data").expect("Failed to create output file");
    std::fs::write(&final_state, EXAMPLE_JSON).expect("Failed to create final state file");
}

fn run_tests(args: TestArgs) {
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
