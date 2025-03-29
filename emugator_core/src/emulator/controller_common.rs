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

impl ALUOp {
    pub fn apply(self, a: u32, b: u32) -> u32 {
        match self {
            ALUOp::ADD => ((a as i32) + (b as i32)) as u32,
            ALUOp::SUB => ((a as i32) - (b as i32)) as u32,
            ALUOp::XOR => a ^ b,
            ALUOp::OR => a | b,
            ALUOp::AND => a & b,
            ALUOp::SLL => a << (b & 0x1F),
            ALUOp::SRL => a >> (b & 0x1F),
            ALUOp::SRA => ((a as i32) >> (b & 0x1F)) as u32,
            ALUOp::EQ => (a == b) as u32,
            ALUOp::NEQ => (a != b) as u32,
            ALUOp::LT => ((a as i32) < (b as i32)) as u32,
            ALUOp::GE => ((a as i32) >= (b as i32)) as u32,
            ALUOp::LTU => (a < b) as u32,
            ALUOp::GEU => (a >= b) as u32,
            ALUOp::SELB => b,
        }
    }
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

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub enum PCSel {
    #[default]
    PC4,
    JMP,
}
