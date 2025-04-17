use clap::{Args, Parser, Subcommand};

use emugator_core::{
    assembler::{AssembledProgram, assemble},
    emulator::{EmulatorState, cve2::CVE2Pipeline},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    fs::OpenOptions,
    io::prelude::*,
    iter::zip,
};

#[derive(Debug, Default)]
struct Test {
    name: String,
    input: String,
    expected_state: OutputState,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
struct OutputState {
    registers: HashMap<u8, HexValue>,
    data_memory: HashMap<HexValue, HexValue>,
    output_buffer: String,
}

impl OutputState {
    /// Checks that every value in self is found in state
    /// Returns any values that are not there
    pub fn validate(&self, state: &EmulatorState<CVE2Pipeline>) -> Option<OutputState> {
        let mut pass = true;
        let mut diff = OutputState::default();

        for (reg, data) in &self.registers {
            let actual_data = state.x[*reg as usize];
            if actual_data != u32::from_be_bytes(data.value) {
                pass = false;
                diff.registers.insert(
                    *reg,
                    HexValue {
                        value: u32::to_be_bytes(actual_data),
                    },
                );
            }
        }
        for (addr, data) in &self.data_memory {
            let actual_data = state.data_memory.preview(u32::from_be_bytes(addr.value));
            if actual_data != data.value[3] {
                pass = false;
                diff.data_memory.insert(
                    *addr,
                    HexValue {
                        value: u32::to_be_bytes(actual_data as u32),
                    },
                );
            }
        }

        // Check UART output
        let expected_output = self.output_buffer.as_bytes();
        let actual_output = state.data_memory.get_serial_output();
        if actual_output != expected_output {
            pass = false;
            let count = zip(expected_output, actual_output)
                .take_while(|(a, b)| a == b)
                .count();

            diff.output_buffer = String::from_utf8_lossy(&actual_output[count..]).to_string();
        }

        if pass { None } else { Some(diff) }
    }
}

#[derive(Serialize, Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(transparent)]
struct HexValue {
    #[serde(with = "hex::serde")]
    value: [u8; 4],
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    New(NewArgs),
    Test(TestArgs),
}

#[derive(Args, Debug)]
#[command(about)]
/// Create scaffolding folder for a new project
pub struct NewArgs {
    /// Name of the new folder containing the scaffolding
    name: String,
}

///
/// Runs a set of tests on each input program, outputing the score for each program
///
#[derive(Args, Debug)]
#[command(about)]
pub struct TestArgs {
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

pub fn new_project(args: NewArgs) {
    // create the new project folder relative to the current directory
    let project_path = std::path::Path::new(&args.name);
    if project_path.exists() {
        println!("Folder already exists");
        return;
    }
    std::fs::create_dir(project_path).expect("Failed to create project directory");

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

#[derive(Debug, Default)]
pub struct TestInfo {
    programs: Vec<(String, Option<AssembledProgram>)>,
    tests: Vec<Test>,
    pub curr_prog: usize,
    pub curr_test: usize,
    output_path: std::path::PathBuf,
    test_results: Vec<Vec<bool>>,
    timeout: usize,
}

impl TestInfo {
    pub fn prepare_to_test(&mut self, args: TestArgs) {
        self.timeout = args.timeout;
        self.curr_prog = 0;
        self.curr_test = 0;

        // get (name, source) pairs from the programs folder
        self.programs = std::fs::read_dir(&args.programs)
            .expect("Failed to read programs directory")
            .filter_map(|entry| {
                let entry = entry.expect("Failed to read entry");
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_stem()?.to_str()?.to_string();
                    let source = std::fs::read_to_string(path).ok()?;
                    match assemble(&source) {
                        Ok(program) => Some((name, Some(program))),
                        Err(err) => {
                            println!("Failed to assemble {}: {:?}", name, err);
                            Some((name, None))
                        }
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.tests = std::fs::read_dir(&args.tests)
            .expect("Failed to read tests dir")
            .filter_map(|entry| {
                let entry = entry.expect("Failed to read entry");
                let path = entry.path();
                if path.is_dir() {
                    let test_name = path.file_stem()?.to_str()?.to_string();
                    let mut input = None;
                    let mut expected_state: Option<OutputState> = None;

                    // read files in test directory
                    for entry in std::fs::read_dir(path.as_path()).expect("Failed to read") {
                        let entry = entry.expect("Failed to read entry");
                        if entry.path().is_file() {
                            let file_path = entry.path();
                            let name = file_path.file_stem()?.to_str()?.to_string();

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
                    Some(Test {
                        name: test_name,
                        input: input.unwrap_or_default(),
                        expected_state: expected_state.unwrap_or_default(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // fill test results
        self.test_results = vec![vec![false; self.tests.len()]; self.programs.len()];

        // check that output dir exists (or create it) and is valid
        self.output_path = std::path::Path::new(&args.tests)
            .parent()
            .expect("Cannot get parent of test dir")
            .join("test_output");
        if !std::fs::exists(&self.output_path).expect("Can't check if output directory exists") {
            std::fs::create_dir(&self.output_path).expect("Failed to create test output directory");
        }

        let mut tests_str: String = String::new();
        for test in &self.tests {
            tests_str.push_str(&format!(",{}", test.name));
        }

        // write header to .csv file
        let output_csv_path = self.output_path.join("testresults.csv");
        std::fs::write(&output_csv_path, format!("program name{}\n", tests_str))
            .expect("Failed to create test output file");
    }

    // tests a program against all tests and appends results to .csv file.
    // returns true if there are more tests to run
    pub fn run_curr_test(&mut self) -> bool {
        if self.curr_prog >= self.programs.len() {
            return false;
        }

        // run the current test on the current program
        let (name, program) = &self.programs[self.curr_prog];
        if let Some(program) = program {
            let test = &self.tests[self.curr_test];

            let mut starting_state = EmulatorState::<CVE2Pipeline>::new(&program);
            starting_state
                .data_memory
                .set_serial_input(test.input.as_bytes());
            let starting_state = starting_state;

            let ending_state =
                starting_state.clock_until_break(&program, &BTreeSet::new(), self.timeout);

            let state_diff = test.expected_state.validate(&ending_state);
            let pass = state_diff.is_none();

            self.test_results[self.curr_prog][self.curr_test] = pass;

            if !pass {
                let test_dir = self.output_path.join(&test.name);
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
                let test_dir = self.output_path.join(&test.name);
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

        // move to the next test
        self.curr_test += 1;

        // if done with all the tests for this program, move to the next program
        if self.curr_test >= self.tests.len() {
            self.curr_test = 0;
            self.curr_prog += 1;
        }

        // return true if there are more tests to run
        if self.curr_prog >= self.programs.len() {
            self.export_results();
            false
        } else {
            true
        }
    }

    fn export_results(&self) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(self.output_path.join("testresults.csv"))
            .expect("Could not open test results file");

        for (prog, test_results) in self.programs.iter().zip(self.test_results.iter()) {
            let test_count = test_results.len();
            let passed_count = test_results.iter().filter(|&&val| val).count();

            let str = test_results
                .iter()
                .map(|val| format!(",{}", if *val { "PASSED" } else { "FAILED" }))
                .collect::<Vec<String>>()
                .join("");

            writeln!(file, "{}{} ({}/{})", prog.0, str, passed_count, test_count)
                .expect("Failed to write test results");
        }
    }

    pub fn finish_up(&self) -> String {
        format!(
            "Done! The difference between ending states for failed tests can be found in: {:?}",
            self.output_path.to_str()
        )
    }

    pub fn num_programs(&self) -> usize {
        self.programs.len()
    }

    pub fn num_tests(&self) -> usize {
        self.tests.len()
    }

    pub fn current_program(&self) -> &str {
        self.programs
            .get(self.curr_prog)
            .map_or("", |p| p.0.as_str())
    }

    pub fn current_test(&self) -> &str {
        self.tests
            .get(self.curr_test)
            .map_or("", |p| p.name.as_str())
    }
}
