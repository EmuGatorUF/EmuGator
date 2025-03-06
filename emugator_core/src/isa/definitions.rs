use strum::EnumString;

use super::Instruction;

#[derive(Default, Debug)]
pub struct Operands {
    pub rd: u32,
    pub rs1: u32,
    pub rs2: u32,
    pub imm: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionDefinition {
    pub _name: &'static str,
    pub opcode: u8,
    pub funct3: Option<u8>,
    pub funct7: Option<u8>,
    pub format: InstructionFormat,
}

impl InstructionDefinition {
    pub fn from_instr(instr: Instruction) -> Option<InstructionDefinition> {
        ISA::instr_to_isa(instr).map(|isa| isa.definition())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstructionFormat {
    R,
    I,
    S,
    B,
    U,
    J,
}

#[derive(EnumString, Debug)]
pub enum ISA {
    ADD,
    SUB,
    SLT,
    SLTU,
    AND,
    OR,
    XOR,
    SLL,
    SRL,
    SRA,
    ADDI,
    SLTI,
    SLTIU,
    ANDI,
    ORI,
    XORI,
    SLLI,
    SRLI,
    SRAI,
    JALR,
    CSRRW,
    CSRRS,
    CSRRC,
    CSRRWI,
    CSRRSI,
    CSRRCI,
    LW,
    LH,
    LHU,
    LB,
    LBU,
    FENCE,
    FENCE_TSO,
    PAUSE,
    ECALL,
    EBREAK,
    SW,
    SH,
    SB,
    BEQ,
    BNE,
    BLT,
    BLTU,
    BGE,
    BGEU,
    LUI,
    AUIPC,
    JAL,
}

impl ISA {
    pub fn definition(&self) -> InstructionDefinition {
        use ISA::*;
        match self {
            ADD => InstructionDefinition {
                _name: "ADD",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x0),
                funct7: Some(0x00),
            },
            SUB => InstructionDefinition {
                _name: "SUB",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x0),
                funct7: Some(0x20),
            },
            SLT => InstructionDefinition {
                _name: "SLT",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x2),
                funct7: Some(0x00),
            },
            SLTU => InstructionDefinition {
                _name: "SLTU",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x3),
                funct7: Some(0x00),
            },
            AND => InstructionDefinition {
                _name: "AND",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x7),
                funct7: Some(0x00),
            },
            OR => InstructionDefinition {
                _name: "OR",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x6),
                funct7: Some(0x00),
            },
            XOR => InstructionDefinition {
                _name: "XOR",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x4),
                funct7: Some(0x00),
            },
            SLL => InstructionDefinition {
                _name: "SLL",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x1),
                funct7: Some(0x00),
            },
            SRL => InstructionDefinition {
                _name: "SRL",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x5),
                funct7: Some(0x00),
            },
            SRA => InstructionDefinition {
                _name: "SRA",
                format: InstructionFormat::R,
                opcode: 0b0110011,
                funct3: Some(0x5),
                funct7: Some(0x20),
            },

            // I-type instructions
            ADDI => InstructionDefinition {
                _name: "ADDI",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x0),
                funct7: None,
            },
            SLTI => InstructionDefinition {
                _name: "SLTI",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x2),
                funct7: None,
            },
            SLTIU => InstructionDefinition {
                _name: "SLTIU",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x3),
                funct7: None,
            },
            ANDI => InstructionDefinition {
                _name: "ANDI",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x7),
                funct7: None,
            },
            ORI => InstructionDefinition {
                _name: "ORI",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x6),
                funct7: None,
            },
            XORI => InstructionDefinition {
                _name: "XORI",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x4),
                funct7: None,
            },
            SLLI => InstructionDefinition {
                _name: "SLLI",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x1),
                funct7: Some(0x00),
            },
            SRLI => InstructionDefinition {
                _name: "SRLI",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x5),
                funct7: Some(0x00),
            },
            SRAI => InstructionDefinition {
                _name: "SRAI",
                format: InstructionFormat::I,
                opcode: 0b0010011,
                funct3: Some(0x5),
                funct7: Some(0x20),
            },
            JALR => InstructionDefinition {
                _name: "JALR",
                format: InstructionFormat::I,
                opcode: 0b1100111,
                funct3: Some(0x0),
                funct7: None,
            },
            CSRRW => InstructionDefinition {
                _name: "CSRRW",
                format: InstructionFormat::I,
                opcode: 0b1110011,
                funct3: Some(0x1),
                funct7: None,
            },
            CSRRS => InstructionDefinition {
                _name: "CSRRS",
                format: InstructionFormat::I,
                opcode: 0b1110011,
                funct3: Some(0x2),
                funct7: None,
            },
            CSRRC => InstructionDefinition {
                _name: "CSRRC",
                format: InstructionFormat::I,
                opcode: 0b1110011,
                funct3: Some(0x3),
                funct7: None,
            },
            CSRRWI => InstructionDefinition {
                _name: "CSRRWI",
                format: InstructionFormat::I,
                opcode: 0b1110011,
                funct3: Some(0x5),
                funct7: None,
            },
            CSRRSI => InstructionDefinition {
                _name: "CSRRSI",
                format: InstructionFormat::I,
                opcode: 0b1110011,
                funct3: Some(0x6),
                funct7: None,
            },
            CSRRCI => InstructionDefinition {
                _name: "CSRRCI",
                format: InstructionFormat::I,
                opcode: 0b1110011,
                funct3: Some(0x7),
                funct7: None,
            },

            // Load instructions (I-type)
            LB => InstructionDefinition {
                _name: "LB",
                format: InstructionFormat::I,
                opcode: 0b0000011,
                funct3: Some(0x0),
                funct7: None,
            },
            LH => InstructionDefinition {
                _name: "LH",
                format: InstructionFormat::I,
                opcode: 0b0000011,
                funct3: Some(0x1),
                funct7: None,
            },
            LW => InstructionDefinition {
                _name: "LW",
                format: InstructionFormat::I,
                opcode: 0b0000011,
                funct3: Some(0x2),
                funct7: None,
            },
            LBU => InstructionDefinition {
                _name: "LBU",
                format: InstructionFormat::I,
                opcode: 0b0000011,
                funct3: Some(0x4),
                funct7: None,
            },
            LHU => InstructionDefinition {
                _name: "LHU",
                format: InstructionFormat::I,
                opcode: 0b0000011,
                funct3: Some(0x5),
                funct7: None,
            },

            // Special I-type instructions
            FENCE => InstructionDefinition {
                _name: "FENCE",
                format: InstructionFormat::I,
                opcode: 0b0001111,
                funct3: Some(0x0),
                funct7: None,
            },
            FENCE_TSO => InstructionDefinition {
                _name: "FENCE_TSO",
                format: InstructionFormat::I,
                opcode: 0b0001111,
                funct3: Some(0x0),
                funct7: None,
            },
            PAUSE => InstructionDefinition {
                _name: "PAUSE",
                format: InstructionFormat::I,
                opcode: 0b0001111,
                funct3: Some(0x0),
                funct7: None,
            },
            ECALL => InstructionDefinition {
                _name: "ECALL",
                format: InstructionFormat::I,
                opcode: 0b1110011,
                funct3: Some(0x0),
                funct7: None,
            },
            EBREAK => InstructionDefinition {
                _name: "EBREAK",
                format: InstructionFormat::I,
                opcode: 0b1110011,
                funct3: Some(0x0),
                funct7: None,
            },

            // S-type instructions
            SW => InstructionDefinition {
                _name: "SW",
                format: InstructionFormat::S,
                opcode: 0b0100011,
                funct3: Some(0x2),
                funct7: None,
            },
            SH => InstructionDefinition {
                _name: "SH",
                format: InstructionFormat::S,
                opcode: 0b0100011,
                funct3: Some(0x1),
                funct7: None,
            },
            SB => InstructionDefinition {
                _name: "SB",
                format: InstructionFormat::S,
                opcode: 0b0100011,
                funct3: Some(0x0),
                funct7: None,
            },

            // B-type instructions
            BEQ => InstructionDefinition {
                _name: "BEQ",
                format: InstructionFormat::B,
                opcode: 0b1100011,
                funct3: Some(0x0),
                funct7: None,
            },
            BNE => InstructionDefinition {
                _name: "BNE",
                format: InstructionFormat::B,
                opcode: 0b1100011,
                funct3: Some(0x1),
                funct7: None,
            },
            BLT => InstructionDefinition {
                _name: "BLT",
                format: InstructionFormat::B,
                opcode: 0b1100011,
                funct3: Some(0x4),
                funct7: None,
            },
            BLTU => InstructionDefinition {
                _name: "BLTU",
                format: InstructionFormat::B,
                opcode: 0b1100011,
                funct3: Some(0x6),
                funct7: None,
            },
            BGE => InstructionDefinition {
                _name: "BGE",
                format: InstructionFormat::B,
                opcode: 0b1100011,
                funct3: Some(0x5),
                funct7: None,
            },
            BGEU => InstructionDefinition {
                _name: "BGEU",
                format: InstructionFormat::B,
                opcode: 0b1100011,
                funct3: Some(0x7),
                funct7: None,
            },

            // U-type instructions
            LUI => InstructionDefinition {
                _name: "LUI",
                format: InstructionFormat::U,
                opcode: 0b0110111,
                funct3: None,
                funct7: None,
            },
            AUIPC => InstructionDefinition {
                _name: "AUIPC",
                format: InstructionFormat::U,
                opcode: 0b0010111,
                funct3: None,
                funct7: None,
            },

            // J-type instructions
            JAL => InstructionDefinition {
                _name: "JAL",
                format: InstructionFormat::J,
                opcode: 0b1101111,
                funct3: None,
                funct7: None,
            },
        }
    }

    #[allow(clippy::unusual_byte_groupings)]
    pub fn instr_to_isa(instr: Instruction) -> Option<ISA> {
        use ISA::*;
        match (instr.opcode(), instr.funct3(), instr.funct7()) {
            (0b0110111, _, _) => Some(LUI),
            (0b0010111, _, _) => Some(AUIPC),
            (0b1101111, _, _) => Some(JAL),
            (0b1100111, _, _) => Some(JALR),
            (0b1100011, 0b000, _) => Some(BEQ),
            (0b1100011, 0b001, _) => Some(BNE),
            (0b1100011, 0b100, _) => Some(BLT),
            (0b1100011, 0b101, _) => Some(BGE),
            (0b1100011, 0b110, _) => Some(BLTU),
            (0b1100011, 0b111, _) => Some(BGEU),
            (0b0000011, 0b000, _) => Some(LB),
            (0b0000011, 0b001, _) => Some(LH),
            (0b0000011, 0b010, _) => Some(LW),
            (0b0000011, 0b100, _) => Some(LBU),
            (0b0000011, 0b101, _) => Some(LHU),
            (0b0100011, 0b000, _) => Some(SB),
            (0b0100011, 0b001, _) => Some(SH),
            (0b0100011, 0b010, _) => Some(SW),
            (0b0010011, 0b000, _) => Some(ADDI),
            (0b0010011, 0b010, _) => Some(SLTI),
            (0b0010011, 0b011, _) => Some(SLTIU),
            (0b0010011, 0b100, _) => Some(XORI),
            (0b0010011, 0b110, _) => Some(ORI),
            (0b0010011, 0b111, _) => Some(ANDI),
            (0b0010011, 0b001, _) => Some(SLLI),
            (0b0010011, 0b101, 0b0100000) => Some(SRAI),
            (0b0010011, 0b101, _) => Some(SRLI),
            (0b0110011, 0b000, 0b0000000) => Some(ADD),
            (0b0110011, 0b000, 0b0100000) => Some(SUB),
            (0b0110011, 0b001, 0b0000000) => Some(SLL),
            (0b0110011, 0b010, 0b0000000) => Some(SLT),
            (0b0110011, 0b011, 0b0000000) => Some(SLTU),
            (0b0110011, 0b100, 0b0000000) => Some(XOR),
            (0b0110011, 0b101, 0b0000000) => Some(SRL),
            (0b0110011, 0b101, 0b0100000) => Some(SRA),
            (0b0110011, 0b110, 0b0000000) => Some(OR),
            (0b0110011, 0b111, 0b0000000) => Some(AND),
            (0b0001111, 0b000, _) => match instr.raw() {
                0b1000_0011_0011_00000_000_00000_0001111 => Some(FENCE_TSO),
                0b0000_0001_0000_00000_000_00000_0001111 => Some(PAUSE),
                _ => Some(FENCE),
            },
            (0b1110011, 0b000, 0b0000000) => match instr.raw() {
                0b0000_0000_0000_00000_000_00000_1110011 => Some(ECALL),
                0b0000_0000_0001_00000_000_00000_1110011 => Some(EBREAK),
                _ => None,
            },
            (0b1110011, 0b001, _) => Some(CSRRW),
            (0b1110011, 0b010, _) => Some(CSRRS),
            (0b1110011, 0b011, _) => Some(CSRRC),
            (0b1110011, 0b101, _) => Some(CSRRWI),
            (0b1110011, 0b110, _) => Some(CSRRSI),
            (0b1110011, 0b111, _) => Some(CSRRCI),
            _ => None,
        }
    }
}
