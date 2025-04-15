pub mod controller_common;
pub mod cve2;
pub mod five_stage;
pub mod memory_module;
mod register_file;
pub mod uart;

#[cfg(test)]
mod cve2_tests;
#[cfg(test)]
mod five_stage_tests;
#[cfg(test)]
mod fuzz_test;

use std::collections::{BTreeMap, BTreeSet};

use crate::assembler::{AssembledProgram, Section};
use five_stage::FiveStagePipeline;
use memory_module::MemoryModule;

use cve2::CVE2Pipeline;
use register_file::RegisterFile;

#[derive(Clone, Copy, Debug)]
pub enum EmulatorOption {
    CVE2,
    FiveStage,
}

impl EmulatorOption {
    pub fn display_string(&self) -> &'static str {
        match self {
            EmulatorOption::CVE2 => "Two Stage Pipeline",
            EmulatorOption::FiveStage => "Five Stage Pipeline",
        }
    }

    pub fn other(&self) -> Self {
        match self {
            EmulatorOption::CVE2 => EmulatorOption::FiveStage,
            EmulatorOption::FiveStage => EmulatorOption::CVE2,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AnyEmulatorState {
    CVE2(EmulatorState<CVE2Pipeline>),
    FiveStage(EmulatorState<FiveStagePipeline>),
}

impl PartialEq for AnyEmulatorState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AnyEmulatorState::CVE2(a), AnyEmulatorState::CVE2(b)) => a == b,
            (AnyEmulatorState::FiveStage(a), AnyEmulatorState::FiveStage(b)) => a == b,
            _ => false,
        }
    }
}

impl AnyEmulatorState {
    pub fn new_cve2(program: &AssembledProgram) -> Self {
        AnyEmulatorState::CVE2(EmulatorState::new(program))
    }

    pub fn new_five_stage(program: &AssembledProgram) -> Self {
        AnyEmulatorState::FiveStage(EmulatorState::new(program))
    }

    pub fn new_of_type(program: &AssembledProgram, emulator_type: EmulatorOption) -> Self {
        match emulator_type {
            EmulatorOption::CVE2 => AnyEmulatorState::new_cve2(program),
            EmulatorOption::FiveStage => AnyEmulatorState::new_five_stage(program),
        }
    }

    pub fn clock_until_next_instruction(
        &self,
        program: &AssembledProgram,
        max_clocks: usize,
    ) -> Self {
        match self {
            AnyEmulatorState::CVE2(state) => {
                AnyEmulatorState::CVE2(state.clock_until_next_instruction(program, max_clocks))
            }
            AnyEmulatorState::FiveStage(state) => {
                AnyEmulatorState::FiveStage(state.clock_until_next_instruction(program, max_clocks))
            }
        }
    }

    pub fn clock_until_break(
        &self,
        program: &mut AssembledProgram,
        breakpoints: &BTreeSet<usize>,
        max_clocks: usize,
    ) -> Self {
        match self {
            AnyEmulatorState::CVE2(state) => {
                AnyEmulatorState::CVE2(state.clock_until_break(program, breakpoints, max_clocks))
            }
            AnyEmulatorState::FiveStage(state) => AnyEmulatorState::FiveStage(
                state.clock_until_break(program, breakpoints, max_clocks),
            ),
        }
    }

    pub fn clock(&self, program: &mut AssembledProgram) -> Self {
        match self {
            AnyEmulatorState::CVE2(state) => AnyEmulatorState::CVE2(state.clock(program)),
            AnyEmulatorState::FiveStage(state) => AnyEmulatorState::FiveStage(state.clock(program)),
        }
    }

    pub fn registers(&self) -> &RegisterFile {
        match self {
            AnyEmulatorState::CVE2(state) => &state.x,
            AnyEmulatorState::FiveStage(state) => &state.x,
        }
    }

    pub fn memory_io(&self) -> &MemoryModule {
        match self {
            AnyEmulatorState::CVE2(state) => &state.data_memory,
            AnyEmulatorState::FiveStage(state) => &state.data_memory,
        }
    }

    pub fn memory_io_mut(&mut self) -> &mut MemoryModule {
        match self {
            AnyEmulatorState::CVE2(state) => &mut state.data_memory,
            AnyEmulatorState::FiveStage(state) => &mut state.data_memory,
        }
    }

    pub fn all_pcs(&self) -> Vec<PcPos> {
        match self {
            AnyEmulatorState::CVE2(state) => state.pipeline.all_pcs(),
            AnyEmulatorState::FiveStage(state) => state.pipeline.all_pcs(),
        }
    }

    pub fn id_pc(&self) -> Option<u32> {
        match self {
            AnyEmulatorState::CVE2(state) => state.pipeline.id_pc(),
            AnyEmulatorState::FiveStage(state) => state.pipeline.id_pc(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmulatorState<P: Pipeline> {
    pub x: RegisterFile,
    pub data_memory: MemoryModule,
    pub pipeline: P,
}

impl<P: Pipeline + Clone + Default> EmulatorState<P> {
    pub fn new(program: &AssembledProgram) -> Self {
        let mut pipeline = P::default();
        let data_memory = MemoryModule::new(&program.initial_data_memory, 0xF0);

        // set starting address to start
        let start_addr = program.get_section_start(Section::Text);
        pipeline.set_if_pc(start_addr, program);

        EmulatorState {
            x: RegisterFile::default(),
            data_memory,
            pipeline,
        }
    }

    pub fn into_five_stage(self) -> EmulatorState<FiveStagePipeline> {
        EmulatorState {
            x: self.x,
            data_memory: self.data_memory,
            pipeline: FiveStagePipeline::default(),
        }
    }

    pub fn clock_until_next_instruction(
        &self,
        program: &AssembledProgram,
        max_clocks: usize,
    ) -> Self {
        // clock until ID PC changes
        let mut state = self.clone();
        let mut num_cycles = 0;
        let old_id_pc = state.pipeline.id_pc();
        while state.pipeline.id_pc() == old_id_pc {
            state = state.clock(program);

            num_cycles += 1;
            if num_cycles > max_clocks {
                break;
            }
        }
        state
    }

    pub fn clock_until_break(
        &self,
        program: &AssembledProgram,
        breakpoints: &BTreeSet<usize>,
        max_clocks: usize,
    ) -> Self {
        let mut state = self.clone();
        let mut num_cycles = 0;

        loop {
            state = state.clock(program);

            let hit_breakpoint = if let Some(id_pc) = state.pipeline.id_pc() {
                if let Some(line_num) = program.source_map.get_by_left(&id_pc) {
                    breakpoints.contains(line_num)
                } else {
                    false
                }
            } else {
                false
            };

            let hit_ebreak = state.pipeline.requesting_debug();

            if hit_ebreak || hit_breakpoint {
                break;
            }

            // max cycles until we can move this to a web worker
            num_cycles += 1;
            if num_cycles > max_clocks {
                break;
            }
        }
        state
    }

    pub fn clock(&self, program: &AssembledProgram) -> Self {
        let mut next_state = self.clone();
        next_state
            .pipeline
            .clock(program, &mut next_state.x, &mut next_state.data_memory);
        // Clock the memory module
        next_state.data_memory.clock();
        next_state
    }
}

pub trait Pipeline: Clone {
    /// Clock all components in the pipeline by one
    fn clock(
        &mut self,
        program: &AssembledProgram,
        registers: &mut RegisterFile,
        data_memory: &mut MemoryModule,
    );

    /// Set the initial address of the instruction fetch stage
    /// and resolve dependent lines
    fn set_if_pc(&mut self, address: u32, program: &AssembledProgram);

    /// Check if the pipeline is currently requesting a debug via a ebreak
    fn requesting_debug(&self) -> bool;

    /// Mutable reference to the instruction decode PC
    /// Allows reading to trigger breakpoints
    fn id_pc(&self) -> Option<u32>;

    /// Returns all current PCs in the pipeline
    /// This is used for editor line highlighting
    fn all_pcs(&self) -> Vec<PcPos>;
}

pub struct PcPos {
    pub pc: u32,
    pub name: &'static str,
}

impl PcPos {
    pub fn new(pc: u32, name: &'static str) -> Self {
        PcPos { pc, name }
    }
}

fn read_instruction(memory: &BTreeMap<u32, u8>, address: u32) -> Option<u32> {
    let mut rdata_bytes: [u8; 4] = [0; 4];
    let success = (0usize..4usize).all(|i| {
        let addr = address + i as u32;
        if let Some(byte) = memory.get(&addr).copied() {
            rdata_bytes[i] = byte;
            true
        } else {
            false
        }
    });

    if success {
        Some(u32::from_le_bytes(rdata_bytes))
    } else {
        None
    }
}
