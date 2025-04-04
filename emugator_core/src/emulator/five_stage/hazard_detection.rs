use crate::isa::Instruction;

#[derive(Clone, Copy, Debug, Default)]
pub struct HazardDetector {}

impl HazardDetector {
    /// Process the current instruction in the ID stage and return if there
    /// are any current hazards with running that instruction.
    pub fn detect_hazards(&self, instruction: &Instruction) -> bool {
        // TODO: implement
        false
    }

    /// A human readable reason for the hazard.
    /// This is used for the hover text in the pipeline visualization.
    pub fn hazard_reason(&self) -> String {
        "".to_string()
    }
}
