use super::{InstructionDefinition, InstructionFormat, Operands};
use crate::{bitmask, bits};

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    instr: u32,
}

impl Instruction {
    pub fn new(
        format: InstructionFormat,
        opcode: u32,
        rd: u32,
        funct3: u32,
        rs1: u32,
        rs2: u32,
        funct7: u32,
        imm: i32,
    ) -> Instruction {
        assert_eq!(opcode, bits!(opcode,6;0));
        assert_eq!(rd, bits!(rd,4;0));
        assert_eq!(funct3, bits!(funct3,2;0));
        assert_eq!(rs1, bits!(rs1,4;0));
        assert_eq!(rs2, bits!(rs2,4;0));
        assert_eq!(funct7, bits!(funct7,6;0));
        let instr = match format {
            InstructionFormat::R => Self::encode_r(opcode, rd, funct3, rs1, rs2, funct7),
            InstructionFormat::I => Self::encode_i(opcode, rd, funct3, rs1, imm),
            InstructionFormat::S => Self::encode_s(opcode, funct3, rs1, rs2, imm),
            InstructionFormat::B => Self::encode_b(opcode, funct3, rs1, rs2, imm),
            InstructionFormat::U => Self::encode_u(opcode, rd, imm),
            InstructionFormat::J => Self::encode_j(opcode, rd, imm),
        };
        Self { instr }
    }

    pub fn from_def_operands(def: InstructionDefinition, operands: Operands) -> Instruction {
        Instruction::new(
            def.format,
            def.opcode as u32,
            operands.rd,
            def.funct3.unwrap_or_default() as u32,
            operands.rs1,
            operands.rs2,
            def.funct7.unwrap_or_default() as u32,
            operands.imm,
        )
    }

    pub fn from_raw(instr: u32) -> Instruction {
        Self { instr }
    }

    fn encode_r(opcode: u32, rd: u32, funct3: u32, rs1: u32, rs2: u32, funct7: u32) -> u32 {
        funct7 << 25 | rs2 << 20 | rs1 << 15 | funct3 << 12 | rd << 7 | opcode << 0
    }

    fn encode_i(opcode: u32, rd: u32, funct3: u32, rs1: u32, imm: i32) -> u32 {
        assert!((imm == bits!(imm,11;0)) || (imm & bitmask!(31;11) == bitmask!(31;11)));
        let imm: u32 = imm as u32;
        imm << 20 | rs1 << 15 | funct3 << 12 | rd << 7 | opcode << 0
    }

    fn encode_s(opcode: u32, funct3: u32, rs1: u32, rs2: u32, imm: i32) -> u32 {
        assert!((imm == bits!(imm,11;0)) || (imm & bitmask!(31;11) == bitmask!(31;11)));
        let imm: u32 = imm as u32;
        (bits!(imm,11;5) << 25
            | rs2 << 20
            | rs1 << 15
            | funct3 << 12
            | bits!(imm,4;0) << 7
            | opcode << 0)
    }

    fn encode_b(opcode: u32, funct3: u32, rs1: u32, rs2: u32, imm: i32) -> u32 {
        assert!((imm == bits!(imm,11;0)) || (imm & bitmask!(31;11) == bitmask!(31;11)));
        let imm: u32 = imm as u32;
        (bits!(imm, 12) << 31
            | bits!(imm,10;5) << 25
            | rs2 << 20
            | rs1 << 15
            | funct3 << 12
            | bits!(imm,4;1) << 8
            | bits!(imm, 11) << 7
            | opcode << 0)
    }

    fn encode_u(opcode: u32, rd: u32, imm: i32) -> u32 {
        assert_eq!(imm, bits!(imm,31;12) << 12);
        let imm: u32 = imm as u32;
        (bits!(imm,31;12) << 12 | rd << 7 | opcode << 0)
    }

    fn encode_j(opcode: u32, rd: u32, imm: i32) -> u32 {
        if imm >= 0 {
            assert_eq!(imm, bits!(imm,20;1) << 1);
        } else {
            assert_eq!(bitmask!(12), bits!(imm, 20, 12));
        }
        let imm: u32 = imm as u32;
        (bits!(imm, 20) << 31
            | bits!(imm,10;1) << 21
            | bits!(imm, 11) << 20
            | bits!(imm,19;12) << 12
            | rd << 7
            | opcode << 0)
    }

    pub fn raw(&self) -> u32 {
        self.instr
    }

    pub fn opcode(&self) -> u8 {
        bits!(self.instr,6;0) as u8
    }

    pub fn immediate(&self) -> Result<i32, ()> {
        // get format from instruction opcode, etc
        let format: InstructionFormat = InstructionDefinition::from_instr(*self).unwrap().format;
        match format {
            InstructionFormat::I => {
                Ok((bits!(self.instr, 31) * bitmask!(31; 11) | bits!(self.instr,30;20)) as i32)
            }
            InstructionFormat::S => Ok((bits!(self.instr, 31) * bitmask!(31; 11)
                | bits!(self.instr,30;25) << 5
                | bits!(self.instr,11;7 )) as i32),
            InstructionFormat::B => Ok((bits!(self.instr, 31) * bitmask!(31; 12)
                | bits!(self.instr, 7) << 11
                | bits!(self.instr,30;25) << 5
                | bits!(self.instr,11;8 ) << 1) as i32),
            InstructionFormat::U => Ok((bits!(self.instr,31;12) << 12) as i32),
            InstructionFormat::J => Ok((bits!(self.instr, 31) * bitmask!(31; 20)
                | bits!(self.instr,19;12) << 12
                | bits!(self.instr, 20) << 11
                | bits!(self.instr,30;25) << 5
                | bits!(self.instr,24;21) << 1) as i32),
            _ => Err(()),
        }
    }

    pub fn rd(&self) -> u8 {
        bits!(self.instr, 7, 5) as u8
    }

    pub fn rs1(&self) -> u8 {
        bits!(self.instr, 15, 5) as u8
    }

    pub fn rs2(&self) -> u8 {
        bits!(self.instr, 20, 5) as u8
    }

    pub fn funct3(&self) -> u8 {
        bits!(self.instr, 12, 3) as u8
    }

    pub fn funct7(&self) -> u8 {
        bits!(self.instr, 25, 7) as u8
    }

    pub fn is_valid(&self) -> bool {
        InstructionDefinition::from_instr(*self).is_some()
    }
}
