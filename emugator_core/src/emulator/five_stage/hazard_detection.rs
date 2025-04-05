use crate::isa::{Instruction, InstructionDefinition, InstructionFormat};

#[derive(Clone, Copy, Debug, Default)]
pub struct HazardDetector {
    /// Tracks the number of cycles for each destination register that is a hazard.
    hazard_reg_track: [u8; 32],

    /// Tracks the number of cycles for branch and jump instructions that are hazards.
    /// This is 1 cycles of stopping, then 1 cycle of freeing IF, and 1 cycle of freeing ID.
    branch_jump_track: u8,
}

impl HazardDetector {
    /// Process the current instruction in the ID stage and return if there
    /// are any current hazards with running that instruction.
    pub fn detect_hazards(&mut self, instruction: &Instruction) -> Hazard {
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
            match self.branch_jump_track {
                1 => Hazard::allow_up_to_id(),
                2 => Hazard::allow_if(),
                _ => Hazard::all_stopped(),
            }
        } else if instr_frmt != InstructionFormat::U
            && instr_frmt != InstructionFormat::J
            && self.hazard_reg_track[instruction.rs1() as usize] != 0
        {
            Hazard::all_stopped()
        } else if (instr_frmt == InstructionFormat::R
            || instr_frmt == InstructionFormat::S
            || instr_frmt == InstructionFormat::B)
            && self.hazard_reg_track[instruction.rs2() as usize] != 0
        {
            Hazard::all_stopped()
        } else if instr_frmt == InstructionFormat::J
            || instr_frmt == InstructionFormat::B
            || (instr_frmt == InstructionFormat::I && instr_def.opcode == 0b1100111)
        {
            // if JAL, branch instr, or JALR
            self.branch_jump_track = 3;
            Hazard::allow_ex()
        } else {
            if instr_frmt != InstructionFormat::S && instr_frmt != InstructionFormat::B {
                // must be 4 because register track is decremented at the beginning, and it is only at the start of the fourth cycle the hazard is gone.
                self.hazard_reg_track[instruction.rd() as usize] = 4;
            }
            Hazard::all_go()
        }
    }

    /// A human readable reason for the hazard.
    /// This is used for the hover text in the pipeline visualization.
    pub fn hazard_reason(&self) -> String {
        "".to_string()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Hazard {
    pub stop_if: bool,
    pub stop_id: bool,
    pub stop_ex: bool,
}

impl Hazard {
    pub fn all_go() -> Self {
        Self {
            stop_if: false,
            stop_id: false,
            stop_ex: false,
        }
    }

    pub fn allow_if() -> Self {
        Self {
            stop_if: false,
            stop_id: true,
            stop_ex: true,
        }
    }

    pub fn allow_up_to_id() -> Self {
        Self {
            stop_if: false,
            stop_id: false,
            stop_ex: true,
        }
    }

    pub fn allow_ex() -> Self {
        Self {
            stop_if: true,
            stop_id: true,
            stop_ex: false,
        }
    }

    pub fn all_stopped() -> Self {
        Self {
            stop_if: true,
            stop_id: true,
            stop_ex: true,
        }
    }
}
