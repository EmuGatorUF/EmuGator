pub mod controller_common;
pub mod cve2;
pub mod data_memory;
pub mod five_stage;
mod register_file;
pub mod uart;

#[cfg(test)]
mod cve2_tests;

use std::collections::{BTreeMap, BTreeSet};

use crate::assembler::{AssembledProgram, Section};
use data_memory::DataMemory;
use five_stage::FiveStagePipeline;
use uart::{Uart, trigger_uart};

use cve2::CVE2Pipeline;
use register_file::RegisterFile;

#[derive(Clone, Debug)]
pub enum AnyEmulatorState {
    CVE2(EmulatorState<CVE2Pipeline>),
    FiveStage(EmulatorState<FiveStagePipeline>),
}

impl AnyEmulatorState {
    pub fn new_cve2(program: &AssembledProgram) -> Self {
        AnyEmulatorState::CVE2(EmulatorState::new(program))
    }

    pub fn new_five_stage(program: &AssembledProgram) -> Self {
        AnyEmulatorState::FiveStage(EmulatorState::new(program))
    }

    pub fn clock_until_break(
        &self,
        program: &mut AssembledProgram,
        breakpoints: &BTreeSet<usize>,
        max_cycles: usize,
    ) -> Self {
        match self {
            AnyEmulatorState::CVE2(state) => {
                AnyEmulatorState::CVE2(state.clock_until_break(program, breakpoints, max_cycles))
            }
            AnyEmulatorState::FiveStage(state) => AnyEmulatorState::FiveStage(
                state.clock_until_break(program, breakpoints, max_cycles),
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

    pub fn uart(&self) -> &Uart {
        match self {
            AnyEmulatorState::CVE2(state) => &state.uart,
            AnyEmulatorState::FiveStage(state) => &state.uart,
        }
    }

    pub fn data_memory(&self) -> &DataMemory {
        match self {
            AnyEmulatorState::CVE2(state) => &state.data_memory,
            AnyEmulatorState::FiveStage(state) => &state.data_memory,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmulatorState<P: Pipeline> {
    pub x: RegisterFile,
    pub data_memory: DataMemory,
    pub uart: Uart,
    pub pipeline: P,
}

impl<P: Pipeline + Clone + Default> EmulatorState<P> {
    pub fn new(program: &AssembledProgram) -> Self {
        let uart = Uart::default();
        let mut pipeline = P::default();
        let data_memory = DataMemory::new(&program.initial_data_memory, &uart);

        // set starting address to start
        let start_addr = program.get_section_start(Section::Text);
        *pipeline.if_pc() = start_addr;

        EmulatorState {
            x: RegisterFile::default(),
            data_memory,
            uart,
            pipeline,
        }
    }

    pub fn into_five_stage(self) -> EmulatorState<FiveStagePipeline> {
        EmulatorState {
            x: self.x,
            data_memory: self.data_memory,
            uart: self.uart,
            pipeline: FiveStagePipeline::default(),
        }
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

            let hit_breakpoint =
                if let Some(line_num) = program.source_map.get_by_left(state.pipeline.if_pc()) {
                    breakpoints.contains(line_num)
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
        next_state.uart = trigger_uart(&next_state.uart, &mut next_state.data_memory);
        next_state
    }
}

pub trait Pipeline: Clone {
    /// Clock all components in the pipeline by one
    fn clock(
        &mut self,
        program: &AssembledProgram,
        registers: &mut RegisterFile,
        data_memory: &mut DataMemory,
    );

    /// Check if the pipeline is currently requesting a debug via a ebreak
    fn requesting_debug(&mut self) -> bool;

    /// Mutable reference to the instruction fetch PC
    /// Allows reading to trigger breakpoints, and writing to set where to start execution
    fn if_pc(&mut self) -> &mut u32;
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
