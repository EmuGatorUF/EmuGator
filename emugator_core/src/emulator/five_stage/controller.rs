use crate::bits;
use crate::emulator::controller_common::*;
use crate::isa::Instruction;

/// Control signals for the five stage datapath.
/// Note: `Option::None` is used to represent a "don't care" value.
/// TODO: if this doesn't diverge, it can be unified with the CVE2 control signals.
#[derive(Clone, Copy, Debug)]
pub struct FiveStageControl {
    // ALU Control
    pub alu_op_a_sel: Option<OpASel>, // Mux control for selecting operand A.
    pub alu_op_b_sel: Option<OpBSel>, // Mux control for selecting operand B.
    pub alu_op: Option<ALUOp>,        // ALU operation control.

    // LSU Control
    pub lsu_data_type: Option<LSUDataType>, // Data type for load/store operations.
    pub lsu_request: bool,                  // Request signal for the LSU.
    pub lsu_write_enable: bool,             // Write enable for the LSU.
    pub lsu_sign_ext: bool,                 // Sign extension control for load operations.

    // Register Write Control
    pub wb_src: Option<DataDestSel>, // Mux control for selecting the write-back data.
    pub reg_write: bool,             // Register write control.

    // PC Control
    pub cmp_set: bool,     // Comparison result register set control.
    pub jump_uncond: bool, // Unconditional jump control.
    pub jump_cond: bool,   // Conditional jump control.
    pub pc_set: bool,      // Program counter write control.
    pub if_id_set: bool,   // ID stage registers ready

    // Debug Control
    pub debug_req: bool, // Debug request control
}

impl Default for FiveStageControl {
    fn default() -> Self {
        Self {
            alu_op_a_sel: None,
            alu_op_b_sel: None,
            alu_op: None,
            lsu_data_type: None,
            lsu_request: false,
            lsu_write_enable: false,
            lsu_sign_ext: false,
            wb_src: None,
            reg_write: false,
            cmp_set: false,
            jump_uncond: false,
            jump_cond: false,
            pc_set: true,
            if_id_set: true,
            debug_req: false,
        }
    }
}

impl FiveStageControl {
    pub fn for_instr(instr: Instruction) -> Option<FiveStageControl> {
        None
    }
}
