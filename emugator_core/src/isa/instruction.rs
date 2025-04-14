use super::{InstructionDefinition, InstructionFormat, Operands};
use crate::{bitmask, bits};

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    instr: u32,
}

#[derive(Debug)]
pub struct InstructionBuildError {
    pub error_message: String,
    pub error_type: InstructionBuildErrorType,
}

#[derive(Debug)]
pub enum InstructionBuildErrorType {
    InvalidOpcode,
    InvalidRd,
    InvalidFunct3,
    InvalidRs1,
    InvalidRs2,
    InvalidFunct7,
    InvalidImm,
}

impl Instruction {
    #[allow(dead_code, clippy::too_many_arguments)]
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
        Self::try_build(format, opcode, rd, funct3, rs1, rs2, funct7, imm)
            .expect("Invalid instruction")
    }

    #[allow(clippy::too_many_arguments)]
    pub fn try_build(
        format: InstructionFormat,
        opcode: u32,
        rd: u32,
        funct3: u32,
        rs1: u32,
        rs2: u32,
        funct7: u32,
        imm: i32,
    ) -> Result<Instruction, InstructionBuildError> {
        if opcode != bits!(opcode,6;0) {
            Err(InstructionBuildError {
                error_message: format!("Opcode {opcode:#05x} is out of range."),
                error_type: InstructionBuildErrorType::InvalidOpcode,
            })
        } else if rd != bits!(rd,4;0) {
            Err(InstructionBuildError {
                error_message: format!("Rd {rd:#05x} is out of range."),
                error_type: InstructionBuildErrorType::InvalidRd,
            })
        } else if funct3 != bits!(funct3,2;0) {
            Err(InstructionBuildError {
                error_message: format!("Funct3 {funct3:#05x} is out of range."),
                error_type: InstructionBuildErrorType::InvalidFunct3,
            })
        } else if rs1 != bits!(rs1,4;0) {
            Err(InstructionBuildError {
                error_message: format!("Rs1 {rs1:#05x} is out of range."),
                error_type: InstructionBuildErrorType::InvalidRs1,
            })
        } else if rs2 != bits!(rs2,4;0) {
            Err(InstructionBuildError {
                error_message: format!("Rs2 {rs2:#05x} is out of range."),
                error_type: InstructionBuildErrorType::InvalidRs2,
            })
        } else if funct7 != bits!(funct7,6;0) {
            Err(InstructionBuildError {
                error_message: format!("Funct7 {funct7:#05x} is out of range."),
                error_type: InstructionBuildErrorType::InvalidFunct7,
            })
        } else {
            let instr = match format {
                InstructionFormat::R => {
                    if imm != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand immediate for R type instruction."
                                .into(),
                            error_type: InstructionBuildErrorType::InvalidImm,
                        })
                    } else {
                        Self::encode_r(opcode, rd, funct3, rs1, rs2, funct7)
                    }
                }
                InstructionFormat::I => {
                    if rs2 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand rs2 for I type instruction.".into(),
                            error_type: InstructionBuildErrorType::InvalidRs2,
                        })
                    } else if opcode == 0b0010011 && (funct3 == 0b001 || funct3 == 0b101) {
                        // Special case for the SLLI SRLI SRAI instructions
                        if bits!(imm,11;5) != 0 {
                            Err(InstructionBuildError {
                                error_message: format!(
                                    "Immediate {imm:#05x} is out of range for shift instruction."
                                ),
                                error_type: InstructionBuildErrorType::InvalidImm,
                            })
                        } else {
                            Self::encode_i(opcode, rd, funct3, rs1, imm | ((funct7 << 5) as i32))
                        }
                    } else if funct7 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand funct7 for I type instruction."
                                .into(),
                            error_type: InstructionBuildErrorType::InvalidFunct7,
                        })
                    } else {
                        Self::encode_i(opcode, rd, funct3, rs1, imm)
                    }
                }
                InstructionFormat::S => {
                    if rd != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand rd for S type instruction.".into(),
                            error_type: InstructionBuildErrorType::InvalidRd,
                        })
                    } else if funct7 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand funct7 for S type instruction."
                                .into(),
                            error_type: InstructionBuildErrorType::InvalidFunct7,
                        })
                    } else {
                        Self::encode_s(opcode, funct3, rs1, rs2, imm)
                    }
                }
                InstructionFormat::B => {
                    if rd != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand rd for B type instruction.".into(),
                            error_type: InstructionBuildErrorType::InvalidRd,
                        })
                    } else if funct7 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand funct7 for B type instruction."
                                .into(),
                            error_type: InstructionBuildErrorType::InvalidFunct7,
                        })
                    } else {
                        Self::encode_b(opcode, funct3, rs1, rs2, imm)
                    }
                }
                InstructionFormat::U => {
                    if funct3 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand funct3 for U type instruction."
                                .into(),
                            error_type: InstructionBuildErrorType::InvalidFunct3,
                        })
                    } else if rs1 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand rs1 for U type instruction.".into(),
                            error_type: InstructionBuildErrorType::InvalidRs1,
                        })
                    } else if rs2 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand rs2 for U type instruction.".into(),
                            error_type: InstructionBuildErrorType::InvalidRs2,
                        })
                    } else if funct7 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand funct7 for U type instruction."
                                .into(),
                            error_type: InstructionBuildErrorType::InvalidFunct7,
                        })
                    } else {
                        Self::encode_u(opcode, rd, imm)
                    }
                }
                InstructionFormat::J => {
                    if funct3 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand funct3 for J type instruction."
                                .into(),
                            error_type: InstructionBuildErrorType::InvalidFunct3,
                        })
                    } else if rs1 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand rs1 for J type instruction.".into(),
                            error_type: InstructionBuildErrorType::InvalidRs1,
                        })
                    } else if rs2 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand rs2 for J type instruction.".into(),
                            error_type: InstructionBuildErrorType::InvalidRs2,
                        })
                    } else if funct7 != 0 {
                        Err(InstructionBuildError {
                            error_message: "Unexpected operand funct7 for J type instruction."
                                .into(),
                            error_type: InstructionBuildErrorType::InvalidFunct7,
                        })
                    } else {
                        Self::encode_j(opcode, rd, imm)
                    }
                }
            }?;
            Ok(Self { instr })
        }
    }

    #[allow(dead_code)]
    pub fn from_def_operands(def: InstructionDefinition, operands: Operands) -> Instruction {
        Self::try_from_def_operands(def, operands).expect("Invalid instruction")
    }

    pub fn try_from_def_operands(
        def: InstructionDefinition,
        operands: Operands,
    ) -> Result<Instruction, InstructionBuildError> {
        Instruction::try_build(
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

    fn encode_r(
        opcode: u32,
        rd: u32,
        funct3: u32,
        rs1: u32,
        rs2: u32,
        funct7: u32,
    ) -> Result<u32, InstructionBuildError> {
        Ok((funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)
    }

    fn encode_i(
        opcode: u32,
        rd: u32,
        funct3: u32,
        rs1: u32,
        imm: i32,
    ) -> Result<u32, InstructionBuildError> {
        if !((imm == bits!(imm,11;0)) || (imm & bitmask!(31;11) == bitmask!(31;11))) {
            Err(InstructionBuildError {
                error_message: format!(
                    "Immediate {imm:#05x} is out of range for I type instruction."
                ),
                error_type: InstructionBuildErrorType::InvalidImm,
            })
        } else {
            let imm: u32 = imm as u32;
            Ok((imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)
        }
    }

    fn encode_s(
        opcode: u32,
        funct3: u32,
        rs1: u32,
        rs2: u32,
        imm: i32,
    ) -> Result<u32, InstructionBuildError> {
        if !((imm == bits!(imm,11;0)) || (imm & bitmask!(31;11) == bitmask!(31;11))) {
            Err(InstructionBuildError {
                error_message: format!(
                    "Immediate {imm:#05x} is out of range for S type instruction."
                ),
                error_type: InstructionBuildErrorType::InvalidImm,
            })
        } else {
            let imm: u32 = imm as u32;
            Ok((bits!(imm,11;5) << 25)
                | (rs2 << 20)
                | (rs1 << 15)
                | (funct3 << 12)
                | (bits!(imm,4;0) << 7)
                | opcode)
        }
    }

    fn encode_b(
        opcode: u32,
        funct3: u32,
        rs1: u32,
        rs2: u32,
        imm: i32,
    ) -> Result<u32, InstructionBuildError> {
        if !((imm == bits!(imm,11;0)) || (imm & bitmask!(31;11) == bitmask!(31;11))) {
            Err(InstructionBuildError {
                error_message: format!(
                    "Immediate {imm:#05x} is out of range for B type instruction."
                ),
                error_type: InstructionBuildErrorType::InvalidImm,
            })
        } else {
            let imm: u32 = imm as u32;
            Ok((bits!(imm, 12) << 31)
                | (bits!(imm,10;5) << 25)
                | (rs2 << 20)
                | (rs1 << 15)
                | (funct3 << 12)
                | (bits!(imm,4;1) << 8)
                | (bits!(imm, 11) << 7)
                | opcode)
        }
    }

    fn encode_u(opcode: u32, rd: u32, imm: i32) -> Result<u32, InstructionBuildError> {
        if imm != bits!(imm,31;12) << 12 {
            Err(InstructionBuildError {
                error_message: format!(
                    "Immediate {imm:#05x} is out of range for U type instruction. Lower 12 bits must be 0."
                ),
                error_type: InstructionBuildErrorType::InvalidImm,
            })
        } else {
            let imm: u32 = imm as u32;
            Ok((bits!(imm,31;12) << 12) | (rd << 7) | opcode)
        }
    }

    fn encode_j(opcode: u32, rd: u32, imm: i32) -> Result<u32, InstructionBuildError> {
        let _shifted = bits!(imm, 20;1) << 1;
        if (imm >= 0 && imm != bits!(imm,20;1) << 1)
            || (imm < 0 && bits!(imm, 20, 12) != bitmask!(12))
        {
            Err(InstructionBuildError {
                error_message: format!(
                    "Immediate {imm:#05x} is out of range for J type instruction."
                ),
                error_type: InstructionBuildErrorType::InvalidImm,
            })
        } else {
            let imm: u32 = imm as u32;
            Ok((bits!(imm, 20) << 31)
                | (bits!(imm,10;1) << 21)
                | (bits!(imm, 11) << 20)
                | (bits!(imm,19;12) << 12)
                | (rd << 7)
                | opcode)
        }
    }

    pub fn raw(&self) -> u32 {
        self.instr
    }

    pub fn opcode(&self) -> u8 {
        bits!(self.instr,6;0) as u8
    }

    pub fn immediate(&self) -> Option<i32> {
        // get format from instruction opcode, etc
        let format: InstructionFormat = InstructionDefinition::from_instr(*self)?.format;
        match format {
            InstructionFormat::I => {
                Some(((bits!(self.instr, 31) * bitmask!(31; 11)) | bits!(self.instr,30;20)) as i32)
            }
            InstructionFormat::S => Some(
                ((bits!(self.instr, 31) * bitmask!(31; 11))
                    | (bits!(self.instr,30;25) << 5)
                    | bits!(self.instr,11;7 )) as i32,
            ),
            InstructionFormat::B => Some(
                ((bits!(self.instr, 31) * bitmask!(31; 12))
                    | (bits!(self.instr, 7) << 11)
                    | (bits!(self.instr,30;25) << 5)
                    | (bits!(self.instr,11;8 ) << 1)) as i32,
            ),
            InstructionFormat::U => Some((bits!(self.instr,31;12) << 12) as i32),
            InstructionFormat::J => Some(
                ((bits!(self.instr, 31) * bitmask!(31; 20))
                    | (bits!(self.instr,19;12) << 12)
                    | (bits!(self.instr, 20) << 11)
                    | (bits!(self.instr,30;25) << 5)
                    | (bits!(self.instr,24;21) << 1)) as i32,
            ),
            _ => None,
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

    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        InstructionDefinition::from_instr(*self).is_some()
    }
}
