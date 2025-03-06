use crate::bits;
use crate::isa::Instruction;

/// Control signals for the CVE2 datapath.
/// Note: `Option::None` is used to represent a "don't care" value.
#[derive(Clone, Copy, Debug)]
pub struct CVE2Control {
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
    pub data_dest_sel: Option<DataDestSel>, // Mux control for selecting the write-back data.
    pub reg_write: bool,                    // Register write control.

    // PC Control
    pub cmp_set: bool,     // Comparison result register set control.
    pub jump_uncond: bool, // Unconditional jump control.
    pub jump_cond: bool,   // Conditional jump control.
    pub pc_set: bool,      // Program counter write control.
    pub instr_req: bool,   // Instruction memory request
    pub id_in_ready: bool, // ID stage registers ready

    // Debug Control
    pub debug_req: bool, // Debug request control
}

impl Default for CVE2Control {
    fn default() -> Self {
        Self {
            alu_op_a_sel: None,
            alu_op_b_sel: None,
            alu_op: None,
            lsu_data_type: None,
            lsu_request: false,
            lsu_write_enable: false,
            lsu_sign_ext: false,
            data_dest_sel: None,
            reg_write: false,
            cmp_set: false,
            jump_uncond: false,
            jump_cond: false,
            pc_set: true,
            instr_req: true,
            id_in_ready: true,
            debug_req: false,
        }
    }
}

impl CVE2Control {
    pub fn arithmetic(op_a: OpASel, op_b: OpBSel, op: ALUOp) -> Self {
        Self {
            alu_op_a_sel: Some(op_a),
            alu_op_b_sel: Some(op_b),
            alu_op: Some(op),
            data_dest_sel: Some(DataDestSel::ALU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn register(op: ALUOp) -> Self {
        Self {
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::RF),
            alu_op: Some(op),
            data_dest_sel: Some(DataDestSel::ALU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn immediate(op: ALUOp) -> Self {
        Self {
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(op),
            data_dest_sel: Some(DataDestSel::ALU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn load_request(data_type: LSUDataType) -> Self {
        Self {
            // address calculation
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(ALUOp::ADD),

            // lsu inputs
            lsu_data_type: Some(data_type),
            lsu_request: true,
            lsu_write_enable: false,

            // don't move onto the next instruction
            pc_set: false,
            instr_req: false,
            id_in_ready: false,

            ..Default::default()
        }
    }

    pub fn load_write(data_type: LSUDataType, sign_ext: bool) -> Self {
        Self {
            // lsu inputs
            lsu_data_type: Some(data_type),
            lsu_sign_ext: sign_ext,

            // register write
            data_dest_sel: Some(DataDestSel::LSU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn store_request(data_type: LSUDataType) -> Self {
        Self {
            // address calculation
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(ALUOp::ADD),

            // lsu inputs
            lsu_data_type: Some(data_type),
            lsu_request: true,
            lsu_write_enable: true,

            // don't move onto the next instruction
            pc_set: false,
            instr_req: false,
            id_in_ready: false,

            ..Default::default()
        }
    }

    pub fn store_completion() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn jump(base_addr: OpASel) -> Self {
        CVE2Control {
            // calculate the destination address
            alu_op_a_sel: Some(base_addr),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(ALUOp::ADD),

            // set the PC to it
            jump_uncond: true,
            pc_set: true,

            // preserve the ID stage registers for link
            id_in_ready: false,

            ..Default::default()
        }
    }

    pub fn link() -> Self {
        CVE2Control {
            // add 4 to ID PC and store it in rd
            alu_op_a_sel: Some(OpASel::PC),
            alu_op_b_sel: Some(OpBSel::Four),
            alu_op: Some(ALUOp::ADD),
            data_dest_sel: Some(DataDestSel::ALU),
            reg_write: true,

            ..Default::default()
        }
    }

    pub fn branch_cmp(op: ALUOp) -> Self {
        CVE2Control {
            // compare rs1 and rs2
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::RF),
            alu_op: Some(op),
            cmp_set: true,

            // don't move onto the next instruction
            pc_set: false,
            instr_req: false,
            id_in_ready: false,

            ..Default::default()
        }
    }

    pub fn branch_jump() -> Self {
        CVE2Control {
            // calculate the destination address
            alu_op_a_sel: Some(OpASel::PC),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(ALUOp::ADD),

            // jump if the comparison was true
            jump_cond: true,
            pc_set: true,

            // wait until new PC before loading IF into ID
            id_in_ready: false,

            ..Default::default()
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ALUOp {
    ADD,
    SUB,
    XOR,
    OR,
    AND,
    SLL,
    SRL,
    SRA,
    EQ,
    NEQ,
    LT,
    GE,
    LTU,
    GEU,
    SELB,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum OpASel {
    PC,
    RF,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum OpBSel {
    RF,
    IMM,
    Four,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum DataDestSel {
    ALU,
    LSU,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum LSUDataType {
    Word,
    HalfWord,
    Byte,
}

impl LSUDataType {
    pub fn byte_enable(&self) -> [bool; 4] {
        match self {
            LSUDataType::Word => [true; 4],
            LSUDataType::HalfWord => [true, true, false, false],
            LSUDataType::Byte => [true, false, false, false],
        }
    }

    pub fn size_in_bits(&self) -> usize {
        match self {
            LSUDataType::Word => 32,
            LSUDataType::HalfWord => 16,
            LSUDataType::Byte => 8,
        }
    }
}

#[allow(clippy::unusual_byte_groupings)]
pub fn get_control_signals(instr: Instruction, instr_cycle: u32) -> Option<CVE2Control> {
    match instr.opcode() {
        0b0110111 => Some(CVE2Control::immediate(ALUOp::SELB)), // LUI
        0b0010111 => Some(CVE2Control::arithmetic(OpASel::PC, OpBSel::IMM, ALUOp::ADD)), // AUIPC
        0b1101111 => match instr_cycle {
            // JAL
            0 => Some(CVE2Control::jump(OpASel::PC)),
            1 => Some(CVE2Control::link()),
            _ => panic!("Invalid instruction cycle for JAL"),
        },
        0b1100111 => match instr_cycle {
            // JALR
            0 => Some(CVE2Control::jump(OpASel::RF)),
            1 => Some(CVE2Control::link()),
            _ => panic!("Invalid instruction cycle for JALR"),
        },
        0b1100011 => match instr_cycle {
            // Branch instructions
            0 => Some(CVE2Control::branch_cmp(match instr.funct3() {
                0b000 => ALUOp::EQ,
                0b001 => ALUOp::NEQ,
                0b100 => ALUOp::LT,
                0b101 => ALUOp::GE,
                0b110 => ALUOp::LTU,
                0b111 => ALUOp::GEU,
                _ => panic!("Invalid funct3 for branch instruction"),
            })),
            1 => Some(CVE2Control::branch_jump()),
            2 => Some(CVE2Control::default()), // NOP cycle to load next instruction into ID
            _ => panic!("Invalid instruction cycle for branch instruction"),
        },
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
            match instr_cycle {
                0 => Some(CVE2Control::load_request(data_type)),
                1 => Some(CVE2Control::load_write(data_type, sign_ext)),
                _ => panic!("Invalid instruction cycle for load instruction"),
            }
        }
        0b0100011 => {
            // Store instructions
            let data_type = match instr.funct3() {
                0b000 => LSUDataType::Byte,
                0b001 => LSUDataType::HalfWord,
                0b010 => LSUDataType::Word,
                _ => panic!("Invalid funct3 for store instruction"),
            };
            match instr_cycle {
                0 => Some(CVE2Control::store_request(data_type)),
                1 => Some(CVE2Control::store_completion()),
                _ => panic!("Invalid instruction cycle for store instruction"),
            }
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
            Some(CVE2Control::immediate(op))
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
            Some(CVE2Control::register(op))
        }
        0b0001111 => match instr.raw() {
            /*
             * Instruction for ordering device I/O and memory accesses
             * as viewed by other RISC-V harts and external devices
             * We are not emulating external devices, so this is unncessary
             * to implement and can be implemented as NOP (Chapter 2, page 13
             * of the RISC-V Instruction Set Manual)
             */
            0b1000_0011_0011_00000_000_00000_0001111 => Some(CVE2Control::default()), // FENCE_TSO
            0b0000_0001_0000_00000_000_00000_0001111 => Some(CVE2Control::default()), // PAUSE
            _ if instr.funct3() == 0b000 => Some(CVE2Control::default()),             // FENCE
            _ => None,
        },
        0b1110011 => match instr.raw() {
            0b0000_0000_0000_00000_000_00000_1110011 => Some(CVE2Control::default()), // ECALL
            0b0000_0000_0001_00000_000_00000_1110011 => Some(CVE2Control {
                debug_req: true,
                ..Default::default()
            }), // EBREAK
            _ => Some(CVE2Control::default()), // CSR (no-op),
        },
        _ => None,
    }
}
