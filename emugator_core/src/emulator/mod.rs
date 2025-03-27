pub mod cve2;
pub mod five_stage;
mod register_file;
pub mod uart;

#[cfg(test)]
mod cve2_tests;

use std::collections::BTreeSet;

use crate::assembler::AssembledProgram;
use five_stage::FiveStagePipeline;
use uart::{Uart, trigger_uart};

use cve2::CVE2Pipeline;
use register_file::RegisterFile;

#[derive(Clone, Debug)]
pub enum AnyEmulatorState {
    CVE2(EmulatorState<CVE2Pipeline>),
    FiveStage(EmulatorState<FiveStagePipeline>),
}

impl Default for AnyEmulatorState {
    fn default() -> Self {
        AnyEmulatorState::CVE2(EmulatorState {
            x: RegisterFile::default(),
            uart: Uart::default(),
            pipeline: CVE2Pipeline::default(),
        })
    }
}

impl AnyEmulatorState {
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
}

#[derive(Clone, Default, Debug)]
pub struct EmulatorState<P: Pipeline> {
    pub x: RegisterFile,
    pub uart: Uart,
    pub pipeline: P,
}

impl<P: Pipeline + Clone> EmulatorState<P> {
    pub fn clock_until_break(
        &self,
        program: &mut AssembledProgram,
        breakpoints: &BTreeSet<usize>,
        max_clocks: usize,
    ) -> Self {
        let mut state = self.clone();
        let mut num_cycles = 0;

        loop {
            state = state.clock(program);

            let hit_breakpoint = if let Some(line_num) =
                program.source_map.get_by_left(&state.pipeline.current_pc())
            {
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

    pub fn clock(&self, program: &mut AssembledProgram) -> Self {
        let mut next_state = self.clone();
        next_state.pipeline.clock(program, &mut next_state.x);
        next_state.uart = trigger_uart(&self.uart, &mut program.data_memory);
        next_state
    }
}

pub trait Pipeline: Clone {
    /// Clock all components in the pipeline by one
    fn clock(&mut self, program: &mut AssembledProgram, registers: &mut RegisterFile);

    /// Check if the pipeline is currently requesting a debug via a ebreak
    fn requesting_debug(&mut self) -> bool;

    /// Get the current program counter that should be used for triggering breakpoints
    fn current_pc(&self) -> u32;
}
