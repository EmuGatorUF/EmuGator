use clap::{Args, Parser, Subcommand};

use emugator_core::{
    assembler::assemble,
    emulator::{EmulatorState, cve2::CVE2Pipeline},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    iter::zip,
};

#[derive(Debug, Deserialize, Serialize, Default)]
struct ExpectedState {
    // STILL NEED DEFAULTS
    registers: HashMap<u8, HexValue>,
    //#[serde(default)]
    data_memory: HashMap<HexValue, HexValue>,
    //#[serde(default)]
    output_buffer: String,
}

#[derive(Serialize, Debug, Deserialize, Clone, Copy)]
#[serde(transparent)]
struct HexValue {
    #[serde(with = "hex::serde")]
    value: [u8; 4],
}

impl std::cmp::PartialEq for HexValue {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl std::cmp::Eq for HexValue {}

impl std::hash::Hash for HexValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

/// Simple program to greet a person
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

//TODO: Make it so that memory data is not u32 and is u8
const EXAMPLE_JSON: &str = r##"
{
    "registers": {
        "1": "00000076",
        "2": "00000000",
        "9": "00000042"
    },
    "data_memory": {
        "00000064": "00000000",
        "00000065": "00000001"
    },
    "output_buffer": "Alas!\nPoor\tYorick"
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
    let example_program = programs_path.join("example_program.s");
    let input_path = test_path.join("input.txt");
    let final_state = test_path.join("final_state.json");
    std::fs::write(&example_program, include_str!("example_program.s"))
        .expect("Failed to create example program");
    std::fs::write(&input_path, "input data").expect("Failed to create input file");
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

    let test_dirs = std::fs::read_dir(&args.tests)
        .expect("Failed to read tests dir")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_dir() {
                let test_name = path.file_stem()?.to_str()?.to_string();
                let mut input = None;
                let mut expected_state: Option<ExpectedState> = None;

                // read files in test directory
                for entry in std::fs::read_dir(path.as_path()).expect("Failed to read") {
                    let entry = entry.expect("Failed to read entry");
                    if entry.path().is_file() {
                        let file_path = entry.path();
                        let name = file_path.file_stem()?.to_str()?.to_string();
                        println!("Name of file: {}", name);

                        if name.contains("input") {
                            input = Some(std::fs::read_to_string(file_path).ok()?);
                        } else if name.contains("state") || name.contains("registers") {
                            let file = std::fs::File::open(file_path)
                                .expect("Failed to open expected state file.");
                            expected_state = Some(
                                serde_json::from_reader(file)
                                    .expect("Failed to read JSON, improperly formatted."),
                            );
                        }
                    }
                }
                Some((test_name, input, expected_state))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // check that output dir exists (or create it) and is valid
    let output_path = std::path::Path::new(&args.tests).join("test_output");
    if !std::fs::exists(&output_path).expect("Can't check if output directory exists") {
        std::fs::create_dir(&output_path).expect("Failed to create test output directory");
    }

    // Run programs
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

        let mut failed_tests: Vec<String> = Vec::new();
        // run program for each test
        for (test_name, input, expected_state) in &test_dirs {
            //program.data_memory = original_data_mem.clone();
            let mut starting_state = EmulatorState::<CVE2Pipeline>::new(&program);
            starting_state
                .uart
                .set_input_string(input.as_ref().map_or("", |v| v));
            let starting_state = starting_state;

            let mut ending_state =
                starting_state.clock_until_break(&mut program, &BTreeSet::new(), args.timeout);

            let mut pass: bool = true;

            let mut state_diff: ExpectedState = ExpectedState::default();

            if let Some(expected_state_ref) = expected_state {
                for (reg, data) in &expected_state_ref.registers {
                    let actual_data = ending_state.x[*reg as usize];
                    if actual_data != u32::from_be_bytes(data.value) {
                        pass = false;
                        state_diff.registers.insert(
                            *reg,
                            HexValue {
                                value: u32::to_be_bytes(actual_data),
                            },
                        );
                    }
                }
                for (addr, data) in &expected_state_ref.data_memory {
                    let actual_data = ending_state.data_memory.get(u32::from_be_bytes(addr.value));
                    if actual_data != data.value[3] {
                        pass = false;
                        state_diff.data_memory.insert(
                            *addr,
                            HexValue {
                                value: u32::to_be_bytes(actual_data as u32),
                            },
                        );
                    }
                }

                // Check UART output
                let expected_output = expected_state_ref.output_buffer.clone();
                let actual_output = ending_state.uart.get_uart_output_buffer();
                if &actual_output != &expected_state_ref.output_buffer {
                    pass = false;
                    let count = zip(expected_output.chars(), actual_output.chars())
                        .take_while(|(a, b)| a == b)
                        .count();
                    if count == 0 {
                        state_diff.output_buffer = actual_output;
                    } else {
                        // get the first n characters of the actual output
                        let mut iter = actual_output.chars();
                        iter.nth(count - 1);
                        state_diff.output_buffer = iter.collect();
                    }
                }
            }

            if !pass {
                failed_tests.push(test_name.to_string());

                let test_dir = output_path.join(test_name);
                if !std::fs::exists(&test_dir)
                    .expect("Can't check if output subdirectory for test exists")
                {
                    std::fs::create_dir(&test_dir)
                        .expect("Failed to create output subdirectory for a test");
                }

                let json_string = serde_json::to_string(&state_diff)
                    .expect("Couldn't convert state difference to string!");

                let file_name = name.to_owned() + "_finalstate.json";
                let test_result_path = test_dir.join(file_name);
                std::fs::write(&test_result_path, &json_string)
                    .expect("Failed to create test output file");
            } else {
                let test_dir = output_path.join(test_name);
                if !std::fs::exists(&test_dir)
                    .expect("Can't check if output subdirectory for test exists")
                {
                    std::fs::create_dir(&test_dir)
                        .expect("Failed to create output subdirectory for a test");
                }

                let file_name = name.to_owned() + "_finalstate.json";
                let test_result_path = test_dir.join(file_name);
                let _ = std::fs::remove_file(&test_result_path);
            }
        }
        println!("Failed {:?}\n", failed_tests);
    }
    println!(
        "The difference between ending states for failed tests can be found in: {:?}",
        output_path.to_str()
    );

    // TODO: print results to a CSV file cleanly
    // TODO: jazz up UI with ratatui
    // TODO: Print usage information
    // TODO: Clean up code, modularize
}
