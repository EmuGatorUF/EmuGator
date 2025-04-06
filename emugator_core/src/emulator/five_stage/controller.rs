use crate::bits;
use crate::emulator::controller_common::*;
use crate::isa::Instruction;

/// Control signals for the five stage datapath.
/// Note: `Option::None` is used to represent a "don't care" value.
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
    pub jmp_base: Option<OpASel>, // Mux control for selecting the jump base address.
    pub jump_uncond: bool,        // Unconditional jump control.
    pub jump_cond: bool,          // Conditional jump control.

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
            jmp_base: None,
            jump_uncond: false,
            jump_cond: false,
            debug_req: false,
        }
    }
}

impl FiveStageControl {
    pub fn arithmetic(op_a: OpASel, op_b: OpBSel, op: ALUOp) -> Self {
        Self {
            alu_op_a_sel: Some(op_a),
            alu_op_b_sel: Some(op_b),
            alu_op: Some(op),
            wb_src: Some(DataDestSel::ALU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn register(op: ALUOp) -> Self {
        Self {
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::RF),
            alu_op: Some(op),
            wb_src: Some(DataDestSel::ALU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn immediate(op: ALUOp) -> Self {
        Self {
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(op),
            wb_src: Some(DataDestSel::ALU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn load(data_type: LSUDataType, sign_ext: bool) -> Self {
        Self {
            // address calculation
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(ALUOp::ADD),

            // lsu inputs
            lsu_data_type: Some(data_type),
            lsu_request: true,
            lsu_write_enable: false,
            lsu_sign_ext: sign_ext,

            // register write
            wb_src: Some(DataDestSel::LSU),
            reg_write: true,

            ..Default::default()
        }
    }

    pub fn store(data_type: LSUDataType) -> Self {
        Self {
            // address calculation
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(ALUOp::ADD),

            // lsu inputs
            lsu_data_type: Some(data_type),
            lsu_request: true,
            lsu_write_enable: true,

            ..Default::default()
        }
    }

    pub fn jump(base_addr: OpASel) -> Self {
        Self {
            // calculate the destination address
            jmp_base: Some(base_addr),

            // set the PC to it
            jump_uncond: true,

            // calculate the link address
            alu_op_a_sel: Some(OpASel::PC),
            alu_op_b_sel: Some(OpBSel::Four),
            alu_op: Some(ALUOp::ADD),

            // write link address
            wb_src: Some(DataDestSel::ALU),
            reg_write: true,

            ..Default::default()
        }
    }

    pub fn branch(op: ALUOp) -> Self {
        Self {
            // calculate the destination address
            jmp_base: Some(OpASel::PC),

            // compare rs1 and rs2
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::RF),
            alu_op: Some(op),

            // conditional jump
            jump_cond: true,

            ..Default::default()
        }
    }
}

impl FiveStageControl {
    pub fn for_instr(instr: Instruction) -> Option<FiveStageControl> {
        match instr.opcode() {
            0b0110111 => Some(FiveStageControl::immediate(ALUOp::SELB)), // LUI
            0b0010111 => Some(FiveStageControl::arithmetic(
                OpASel::PC,
                OpBSel::IMM,
                ALUOp::ADD,
            )), // AUIPC
            0b1101111 => Some(FiveStageControl::jump(OpASel::PC)),
            0b1100111 => Some(FiveStageControl::jump(OpASel::RF)),
            0b1100011 => Some(FiveStageControl::branch(match instr.funct3() {
                0b000 => ALUOp::EQ,
                0b001 => ALUOp::NEQ,
                0b100 => ALUOp::LT,
                0b101 => ALUOp::GE,
                0b110 => ALUOp::LTU,
                0b111 => ALUOp::GEU,
                _ => panic!("Invalid funct3 for branch instruction"),
            })),
            0b0000011 => {
                // Load instructions
                let funct3 = instr.funct3();
                let data_type = match bits!(funct3, 1;0) {
                    0b00 => LSUDataType::Byte,
                    0b01 => LSUDataType::HalfWord,
                    0b10 => LSUDataType::Word,
                    _ => panic!("Invalid funct3 for load instruction"),
                };
                let sign_ext = bits!(funct3, 2) == 0;
                Some(FiveStageControl::load(data_type, sign_ext))
            }
            0b0100011 => {
                // Store instructions
                let data_type = match instr.funct3() {
                    0b000 => LSUDataType::Byte,
                    0b001 => LSUDataType::HalfWord,
                    0b010 => LSUDataType::Word,
                    _ => panic!("Invalid funct3 for store instruction"),
                };
                Some(FiveStageControl::store(data_type))
            }
            0b0010011 => {
                // Immediate arithmetic instructions
                let op = match instr.funct3() {
                    0b000 => ALUOp::ADD,
                    0b001 => ALUOp::SLL,
                    0b010 => ALUOp::LT,
                    0b011 => ALUOp::LTU,
                    0b100 => ALUOp::XOR,
                    0b101 => {
                        if bits!(instr.raw(), 30) == 0 {
                            ALUOp::SRL
                        } else {
                            ALUOp::SRA
                        }
                    }
                    0b110 => ALUOp::OR,
                    0b111 => ALUOp::AND,
                    _ => panic!("Invalid funct3 for immediate arithmetic instruction"),
                };
                Some(FiveStageControl::immediate(op))
            }
            0b0110011 => {
                // Register arithmetic instructions
                let op = match (instr.funct3(), instr.funct7()) {
                    (0b000, 0b0000000) => ALUOp::ADD,
                    (0b000, 0b0100000) => ALUOp::SUB,
                    (0b001, 0b0000000) => ALUOp::SLL,
                    (0b010, 0b0000000) => ALUOp::LT,
                    (0b011, 0b0000000) => ALUOp::LTU,
                    (0b100, 0b0000000) => ALUOp::XOR,
                    (0b101, 0b0000000) => ALUOp::SRL,
                    (0b101, 0b0100000) => ALUOp::SRA,
                    (0b110, 0b0000000) => ALUOp::OR,
                    (0b111, 0b0000000) => ALUOp::AND,
                    _ => panic!("Invalid funct3/funct7 for register arithmetic instruction"),
                };
                Some(FiveStageControl::register(op))
            }
            0b1110011 => match instr.raw() {
                0b0000_0000_0000_00000_000_00000_1110011 => Some(FiveStageControl::default()), // ECALL
                0b0000_0000_0001_00000_000_00000_1110011 => Some(FiveStageControl {
                    debug_req: true,
                    ..Default::default()
                }), // EBREAK
                _ => Some(FiveStageControl::default()), // CSR (no-op),
            },
            _ => None,
        }
    }
}
