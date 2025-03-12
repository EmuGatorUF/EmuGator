use crate::assembler::AssembledProgram;

use super::{Pipeline, register_file::RegisterFile};

#[derive(Clone, Default, Debug)]
pub struct FiveStagePipeline {}

impl Pipeline for FiveStagePipeline {
    fn clock(&mut self, _program: &mut AssembledProgram, _registers: &mut RegisterFile) {
        todo!()
    }

    fn requesting_debug(&mut self) -> bool {
        todo!()
    }

    fn current_pc(&self) -> u32 {
        todo!()
    }
}
