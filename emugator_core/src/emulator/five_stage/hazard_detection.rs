use std::alloc::handle_alloc_error;

use crate::isa::{Instruction, InstructionDefinition, InstructionFormat};

#[derive(Clone, Copy, Debug, Default)]
pub struct HazardDetector {
    hazard_reg_track: [u8; 32],
    branch_jump_track: u8,
}

impl HazardDetector {
    /// Process the current instruction in the ID stage and return if there
    /// are any current hazards with running that instruction.
    pub fn detect_hazards(&mut self, instruction: &Instruction) -> bool {        
        // decrement cycles left for each register that is a hazard.
        for i in 0..31 {
            if self.hazard_reg_track[i] != 0 {
                self.hazard_reg_track[i] -= 1;
            }
        }
        if self.branch_jump_track != 0 {
            self.branch_jump_track -= 1;
        }
        
        // check that neither register being read is a hazard.
        let instr_def = InstructionDefinition::from_instr(instruction.clone()).unwrap();
        let instr_frmt = instr_def.format;
        if self.branch_jump_track != 0 {
            true
        } else if instr_frmt != InstructionFormat::U && instr_frmt != InstructionFormat::J && self.hazard_reg_track[instruction.rs1() as usize] != 0 {
            true
        } else if (instr_frmt == InstructionFormat::R || instr_frmt == InstructionFormat::S || instr_frmt == InstructionFormat::B) && self.hazard_reg_track[instruction.rs2() as usize] != 0 {
            true
        } else {
            // if JAL, branch instr, or JALR
            if instr_frmt == InstructionFormat::J || instr_frmt == InstructionFormat::B || (instr_frmt == InstructionFormat::I && instr_def.opcode == 0b1100111) {
                self.branch_jump_track = 3;
            }
            if instr_frmt != InstructionFormat::S && instr_frmt != InstructionFormat::B{
                // must be 4 because register track is decremented at the beginning, and it is only at the start of the fourth cycle the hazard is gone.
                self.hazard_reg_track[instruction.rd() as usize] = 4;
            }
            false
        }
    }

    /// A human readable reason for the hazard.
    /// This is used for the hover text in the pipeline visualization.
    pub fn hazard_reason(&self) -> String {
        "".to_string()
    }
}
