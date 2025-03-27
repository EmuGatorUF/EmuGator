use crate::assembler::AssembledProgram;

use super::{Pipeline, data_memory::DataMemory, register_file::RegisterFile};

#[derive(Clone, Default, Debug)]
pub struct FiveStagePipeline {}

impl Pipeline for FiveStagePipeline {
    fn clock(
        &mut self,
        _program: &AssembledProgram,
        _registers: &mut RegisterFile,
        _data_memory: &mut DataMemory,
    ) {
        todo!()
    }

    fn requesting_debug(&mut self) -> bool {
        todo!()
    }

    fn if_pc(&mut self) -> &mut u32 {
        todo!()
    }
}
