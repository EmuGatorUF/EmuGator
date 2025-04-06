use crate::isa::{Instruction, InstructionDefinition, InstructionFormat};

#[derive(Clone, Copy, Debug, Default)]
pub struct HazardDetector {
    // Which stages are blocked if a hazard is detected
    pub hazard_detected: Hazard,

    /// Tracks the number of cycles for each destination register that is a hazard.
    hazard_reg_track: [u8; 32],

    /// Tracks the number of cycles for branch and jump instructions that are hazards.
    /// This is 1 cycles of stopping, then 1 cycle of freeing IF, and 1 cycle of freeing ID.
    branch_jump_track: u8,

    mem_access_track: u8,
}

impl HazardDetector {
    /// Process the current instruction in the ID stage and return if there
    /// are any current hazards with running that instruction.
    pub fn detect_hazards(&mut self, id_inst: &Option<u32>, jump_not_taken: bool) {
        let Some(id_inst) = id_inst else {
            // no id stage yet
            return;
        };
        let instruction = Instruction::from_raw(*id_inst);

        // decrement cycles left for each register that is a hazard.
        for i in 0..31 {
            if self.hazard_reg_track[i] != 0 {
                self.hazard_reg_track[i] -= 1;
            }
        }
        if self.branch_jump_track != 0 && self.hazard_detected != Hazard::all_stopped() {
            self.branch_jump_track -= 1;
        }
        // if a jump was not taken, reduce the hazard so the old if_pc goes through and isn't skipped.
        if jump_not_taken && self.branch_jump_track != 0 && self.hazard_detected != Hazard::all_stopped() {
            self.branch_jump_track -= 1;
        }

        // check that neither register being read is a hazard.
        let instr_def = InstructionDefinition::from_instr(instruction.clone()).unwrap();
        let instr_frmt = instr_def.format;
        if instr_frmt != InstructionFormat::U
            && instr_frmt != InstructionFormat::J
            && self.hazard_reg_track[instruction.rs1() as usize] != 0
        {
            self.hazard_detected = Hazard::stop_up_to_ex();
        } else if (instr_frmt == InstructionFormat::R
            || instr_frmt == InstructionFormat::S
            || instr_frmt == InstructionFormat::B)
            && self.hazard_reg_track[instruction.rs2() as usize] != 0
        {
            self.hazard_detected = Hazard::stop_up_to_ex();
        } else if self.branch_jump_track != 0 {
            match self.branch_jump_track {
                1 => self.hazard_detected = Hazard::allow_up_to_id(),
                2 => self.hazard_detected = Hazard::allow_if(),
                3 => self.hazard_detected = Hazard::allow_ex(),
                _ => self.hazard_detected = Hazard::stop_up_to_ex(),
            }
        } else if self.mem_access_track != 0 {
            match self.mem_access_track {
                1 => self.hazard_detected = Hazard::allow_up_to_id(),
                2 => self.hazard_detected = Hazard::allow_ex(),
                _ => self.hazard_detected = Hazard::all_go(),
            }
            self.mem_access_track -= 1;
        } else if instr_def.opcode == 0b0000011 {
            self.hazard_detected = Hazard::allow_ex();
            self.mem_access_track = 2;
        } else if instr_frmt == InstructionFormat::J
            || instr_frmt == InstructionFormat::B
            || (instr_frmt == InstructionFormat::I && instr_def.opcode == 0b1100111)
        {
            // if JAL, branch instr, or JALR
            self.branch_jump_track = 3;
            self.hazard_detected = Hazard::allow_ex();

            // Mark JAL destination as hazard
            if instr_frmt != InstructionFormat::B {
                // must be 4 because register track is decremented at the beginning, and it is only at the start of the fourth cycle the hazard is gone.
                self.hazard_reg_track[instruction.rd() as usize] = 4;
            }
        } else {
            if instr_frmt != InstructionFormat::S {
                // must be 4 because register track is decremented at the beginning, and it is only at the start of the fourth cycle the hazard is gone.
                self.hazard_reg_track[instruction.rd() as usize] = 4;
            }
            self.hazard_detected = Hazard::all_go();
        }
    }

    /// A human readable reason for the hazard.
    /// This is used for the hover text in the pipeline visualization.
    pub fn hazard_reason(&self) -> String {
        "".to_string()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq)]
pub struct Hazard {
    pub stop_if: bool,
    pub stop_id: bool,
    pub stop_ex: bool,
    pub stop_mem: bool,
}

impl PartialEq for Hazard {
    fn eq(&self, other: &Self) -> bool {
        self.stop_if == other.stop_if 
            && self.stop_id == other.stop_id 
            && self.stop_ex == other.stop_ex 
            && self.stop_mem == other.stop_mem
    }
}

impl Hazard {
    pub fn all_go() -> Self {
        Self {
            stop_if: false,
            stop_id: false,
            stop_ex: false,
            stop_mem: false,
        }
    }

    pub fn allow_if() -> Self {
        Self {
            stop_if: false,
            stop_id: true,
            stop_ex: true,
            stop_mem: false,
        }
    }

    pub fn allow_up_to_id() -> Self {
        Self {
            stop_if: false,
            stop_id: false,
            stop_ex: true,
            stop_mem: false,
        }
    }

    pub fn allow_ex() -> Self {
        Self {
            stop_if: true,
            stop_id: true,
            stop_ex: false,
            stop_mem: false,
        }
    }

    pub fn stop_up_to_ex() -> Self {
        Self {
            stop_if: true,
            stop_id: true,
            stop_ex: true,
            stop_mem: false,
        }
    }

    pub fn all_stopped() -> Self {
        Self {
            stop_if: true,
            stop_id: true,
            stop_ex: true,
            stop_mem: true,
        }
    }
}
