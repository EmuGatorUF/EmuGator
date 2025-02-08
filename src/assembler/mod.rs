#[cfg(test)]
mod tests;

mod assembled_program;

pub use assembled_program::{AssembledProgram, Section};

use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use crate::isa::{Instruction, InstructionDefinition, InstructionFormat, Operands, ISA};

#[derive(Debug)]
struct DataItem {
    size: usize, // in bytes
    values: Vec<u8>,
}

#[derive(Debug)]
pub struct AssemblerError {
    pub error_message: String,
    pub line_number: usize,
    pub column: usize,
    pub width: usize,
}

impl AssemblerError {
    pub fn new(error_message: String, line_number: usize, column: usize, width: usize) -> Self {
        Self {
            error_message,
            line_number,
            column,
            width,
        }
    }
}

pub fn assemble(program: &str) -> Result<AssembledProgram, Vec<AssemblerError>> {
    let mut errors = Vec::new();
    let mut assembled = AssembledProgram::new();
    let mut current_section = Section::Text;
    let mut text_address = 0;
    let mut data_address = 0;

    // First pass: collect labels and process data
    for (line_num, line) in program.lines().enumerate() {
        let line = clean_line(line);
        if line.is_empty() {
            continue;
        }

        let (label_opt, content) = split_label_and_content(&line);

        // Handle section directives with optional address
        if let Some((section, address)) = parse_section_directive(&content) {
            current_section = section;
            match section {
                Section::Text => text_address = address,
                Section::Data => data_address = address,
            }
            continue;
        }

        // Handle label if present
        if let Some(label) = label_opt {
            match current_section {
                Section::Text => {
                    if let Err(e) = assembled.add_label(label.clone(), text_address, false) {
                        let column = line.find(&label).unwrap_or(0);
                        errors.push(AssemblerError::new(e, line_num + 1, column, label.len()));
                    }
                }
                Section::Data => {
                    if let Err(e) = assembled.add_label(label.clone(), data_address, true) {
                        let column = line.find(&label).unwrap_or(0);
                        errors.push(AssemblerError::new(e, line_num + 1, column, label.len()));
                    }
                }
            }
        }

        // Handle data directives
        if content.starts_with('.') {
            if current_section == Section::Data {
                match parse_data_line(&content) {
                    Ok(Some((_, data))) => {
                        assembled.add_data(data_address, &data.values);
                        data_address += data.size as u32;
                    }
                    Err(e) => {
                        errors.push(AssemblerError::new(
                            e,
                            line_num + 1,
                            content.find('.').unwrap_or(0),
                            content.len(),
                        ));
                    }
                    _ => {}
                }
            } else {
                errors.push(AssemblerError::new(
                    format!("Data directive '{}' outside of .data section", content),
                    line_num + 1,
                    0,
                    content.len(),
                ));
            }
            continue;
        }

        // Count instruction size for text section
        if current_section == Section::Text && !content.is_empty() {
            text_address += 4;
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // Second pass: assemble instructions
    current_section = Section::Text;
    text_address = assembled.get_section_start(Section::Text);

    for (line_num, line) in program.lines().enumerate() {
        let line = clean_line(line);
        if line.is_empty() {
            continue;
        }

        let (_, content) = split_label_and_content(&line);
        if content.is_empty() {
            continue;
        }

        // Handle section directives
        if let Some((section, address)) = parse_section_directive(&content) {
            current_section = section;
            match section {
                Section::Text => text_address = address,
                Section::Data => (),
            }
            continue;
        }

        if current_section == Section::Text && !content.starts_with('.') {
            match parse_instruction(
                &content,
                &assembled.labels,
                &assembled.data_labels,
                text_address,
            ) {
                Ok(instruction) => {
                    let encoded = instruction.raw();
                    assembled.add_instruction(text_address, encoded, line_num + 1);
                    text_address += 4;
                }
                Err(e) => {
                    // For instruction errors, try to be more specific about the error location
                    let parts: Vec<&str> = content
                        .split(|c: char| c.is_whitespace() || c == ',')
                        .filter(|s| !s.is_empty())
                        .collect();

                    let (column, width) = if parts.is_empty() {
                        (0, content.len())
                    } else {
                        // Try to identify which part of the instruction caused the error
                        let error_part = if e.contains("register") {
                            parts.iter().find(|&&p| p.starts_with('x'))
                        } else if e.contains("immediate") || e.contains("offset") {
                            parts.last()
                        } else {
                            Some(&parts[0]) // Instruction name
                        };

                        if let Some(part) = error_part {
                            (content.find(part).unwrap_or(0), part.len())
                        } else {
                            (0, content.len())
                        }
                    };

                    errors.push(AssemblerError::new(e, line_num + 1, column, width));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(assembled)
    } else {
        Err(errors)
    }
}

fn parse_section_directive(line: &str) -> Option<(Section, u32)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() || !parts[0].starts_with('.') {
        return None;
    }

    let section = match parts[0] {
        ".data" => Some(Section::Data),
        ".text" => Some(Section::Text),
        _ => None,
    }?;

    let address = if parts.len() > 1 {
        // Parse hex or decimal address
        if parts[1].starts_with("0x") {
            u32::from_str_radix(&parts[1][2..], 16).ok()?
        } else {
            parts[1].parse().ok()?
        }
    } else {
        0 // Default address
    };

    Some((section, address))
}

fn clean_line(line: &str) -> String {
    match line.split('#').next() {
        Some(l) => l.trim().to_string(),
        None => String::new(),
    }
}

fn split_label_and_content(line: &str) -> (Option<String>, String) {
    if let Some(colon_pos) = line.find(':') {
        let (label, rest) = line.split_at(colon_pos);
        let content = rest[1..].trim().to_string();
        (Some(label.trim().to_string()), content)
    } else {
        (None, line.to_string())
    }
}

fn parse_data_line(line: &str) -> Result<Option<(String, DataItem)>, String> {
    if line.ends_with(':') {
        return Ok(None);
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(format!(
            "Data directive requires a directive type and at least one value, got: '{}'",
            line
        ));
    }

    let directive = parts[0];
    let joined = parts[1..].join(" ");
    let values: Vec<&str> = joined.split(',').map(|s| s.trim()).collect();

    match directive {
        ".byte" => {
            let bytes = values
                .iter()
                .map(|v| parse_number(v).map(|n| n as u8))
                .collect::<Result<Vec<u8>, _>>()
                .map_err(|e| format!("Invalid byte value in .byte directive: {}", e))?;
            Ok(Some((
                String::new(),
                DataItem {
                    size: bytes.len(),
                    values: bytes,
                },
            )))
        }
        ".word" => {
            let mut bytes = Vec::new();
            for value in &values {
                let word = parse_number(value)
                    .map_err(|e| format!("Invalid word value in .word directive: {}", e))?;
                bytes.extend_from_slice(&word.to_le_bytes());
            }
            Ok(Some((
                String::new(),
                DataItem {
                    size: bytes.len(),
                    values: bytes,
                },
            )))
        }
        ".ascii" | ".string" => {
            let text = values
                .join(",")
                .trim_matches('"')
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\r", "\r");
            let mut bytes = text.as_bytes().to_vec();
            if directive == ".string" {
                bytes.push(0);
            }
            Ok(Some((
                String::new(),
                DataItem {
                    size: bytes.len(),
                    values: bytes,
                },
            )))
        }
        _ => Err(format!("Unknown data directive: '{}'", directive)),
    }
}

fn parse_number(value: &str) -> Result<u32, String> {
    let value = value.trim();
    if value.starts_with("0x") {
        u32::from_str_radix(&value[2..], 16)
    } else {
        value.parse::<u32>()
    }
    .map_err(|_| format!("Invalid numeric value: {}", value))
}

fn parse_instruction(
    line: &str,
    text_labels: &HashMap<String, u32>,
    data_labels: &HashMap<String, u32>,
    current_address: u32,
) -> Result<Instruction, String> {
    let parts: Vec<&str> = line
        .split(|c| c == ' ' || c == ',')
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return Err("Empty instruction".to_string());
    }

    let name = parts[0].to_uppercase();
    let def = ISA::from_str(&name)
        .map_err(|_| format!("Unknown instruction: {}", name))?
        .definition();

    if def.format == InstructionFormat::I && def.opcode == 0b0000011
        || def.format == InstructionFormat::S && def.opcode == 0b0100011
    {
        if parts.len() == 3 && data_labels.contains_key(parts[2]) {
            let offset = data_labels[parts[2]];
            let modified_addr = format!("{}(x0)", offset);
            let mut modified_parts = parts.to_vec();
            modified_parts[2] = &modified_addr;
            return match def.format {
                InstructionFormat::I => parse_i_type(&modified_parts, def.clone()),
                InstructionFormat::S => parse_s_type(&modified_parts, def.clone()),
                _ => unreachable!(),
            };
        }
    }

    match def.format {
        InstructionFormat::R => parse_r_type(&parts, def),
        InstructionFormat::I => parse_i_type(&parts, def),
        InstructionFormat::S => parse_s_type(&parts, def),
        InstructionFormat::B => parse_b_type(&parts, def, text_labels, current_address),
        InstructionFormat::U => parse_u_type(&parts, def),
        InstructionFormat::J => parse_j_type(&parts, def, text_labels, current_address),
    }
}

fn parse_r_type(parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
    if parts.len() != 4 {
        return Err(format!(
            "R-type instruction '{}' requires 3 registers (rd, rs1, rs2), got {} operands",
            parts[0],
            parts.len() - 1
        ));
    }

    let operands = Operands {
        rd: parse_register(parts[1]).map_err(|e| format!("Invalid destination register: {}", e))?,
        rs1: parse_register(parts[2])
            .map_err(|e| format!("Invalid first source register: {}", e))?,
        rs2: parse_register(parts[3])
            .map_err(|e| format!("Invalid second source register: {}", e))?,
        imm: 0,
    };
    Ok(Instruction::from_def_operands(def, operands))
}

fn parse_i_type(parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
    match def.opcode {
        0b0000011 => parse_load_type(&parts, def),
        0b1110011 => {
            // Special handling for ECALL/EBREAK
            if parts.len() != 1 {
                return Err(format!(
                    "{} instruction takes no operands, got {} operands",
                    parts[0],
                    parts.len() - 1
                ));
            }

            let operands = Operands {
                rd: 0,
                rs1: 0,
                imm: match parts[0] {
                    "ECALL" => 0,
                    "EBREAK" => 1,
                    _ => unreachable!(),
                },
                ..Default::default()
            };
            Ok(Instruction::from_def_operands(def, operands))
        }
        0b0001111 => {
            if parts.len() != 1 {
                return Err(format!(
                    "FENCE instruction takes no operands, got {} operands",
                    parts.len() - 1
                ));
            }

            let operands = Operands {
                rd: 0,
                rs1: 0,
                imm: 0,
                ..Default::default()
            };
            Ok(Instruction::from_def_operands(def, operands))
        }
        _ => {
            if parts.len() != 4 {
                return Err(format!(
                    "I-type instruction '{}' requires a destination register, source register, and immediate value, got {} operands",
                    parts[0],
                    parts.len() - 1
                ));
            }

            let mut imm =
                parse_immediate(parts[3]).map_err(|e| format!("Invalid immediate value: {}", e))?;

            if imm > 2047 || imm < -2048 {
                return Err(format!(
                    "Immediate value {} is out of range (-2048 to 2047)",
                    imm
                ));
            }

            if let Some(funct7) = def.funct7 {
                // Shift instructions (immediate split into funct7 and shamt)
                if def.opcode == 0b0010011 && (def.funct3 == Some(0x1) || def.funct3 == Some(0x5)) {
                    // SLLI, SRLI, SRAI
                    let shamt = imm & 0x1F; // Bottom 5 bits only
                    imm = ((funct7 as i32) << 5) | shamt; // Combine funct7 and shamt
                }
            }

            let operands = Operands {
                rd: parse_register(parts[1])
                    .map_err(|e| format!("Invalid destination register: {}", e))?,
                rs1: parse_register(parts[2])
                    .map_err(|e| format!("Invalid source register: {}", e))?,
                imm,
                ..Default::default()
            };
            Ok(Instruction::from_def_operands(def, operands))
        }
    }
}

fn parse_load_type(parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
    if parts.len() != 3 {
        return Err(format!(
            "Load instruction '{}' requires a destination register and memory address, got {} operands",
            parts[0],
            parts.len() - 1
        ));
    }

    let (offset, base) =
        parse_mem_address(parts[2]).map_err(|e| format!("Invalid memory address: {}", e))?;

    let operands = Operands {
        rd: parse_register(parts[1]).map_err(|e| format!("Invalid destination register: {}", e))?,
        rs1: base,
        imm: offset,
        ..Default::default()
    };
    Ok(Instruction::from_def_operands(def, operands))
}

fn parse_s_type(parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
    if parts.len() != 3 {
        return Err(format!(
            "Store instruction '{}' requires a source register and memory address, got {} operands",
            parts[0],
            parts.len() - 1
        ));
    }

    let (offset, base) =
        parse_mem_address(parts[2]).map_err(|e| format!("Invalid memory address: {}", e))?;

    let operands = Operands {
        rs1: base,
        rs2: parse_register(parts[1]).map_err(|e| format!("Invalid source register: {}", e))?,
        imm: offset,
        ..Default::default()
    };
    Ok(Instruction::from_def_operands(def, operands))
}

fn parse_b_type(
    parts: &[&str],
    def: InstructionDefinition,
    labels: &HashMap<String, u32>,
    current_address: u32,
) -> Result<Instruction, String> {
    if parts.len() != 4 {
        return Err(format!(
            "Branch instruction '{}' requires two source registers and a label, got {} operands",
            parts[0],
            parts.len() - 1
        ));
    }

    let target = labels.get(parts[3]).ok_or(format!(
        "Branch target '{}' is not a defined label",
        parts[3]
    ))?;

    let offset = (*target as i32) - (current_address as i32);
    if offset & 1 != 0 {
        return Err(format!(
            "Branch target '{}' must be 2-byte aligned",
            parts[3]
        ));
    }
    if offset > 4095 || offset < -4096 {
        return Err(format!(
            "Branch target '{}' is too far ({} bytes), must be within -4096 to +4095 bytes",
            parts[3], offset
        ));
    }

    let operands = Operands {
        rs1: parse_register(parts[1])
            .map_err(|e| format!("Invalid first source register: {}", e))?,
        rs2: parse_register(parts[2])
            .map_err(|e| format!("Invalid second source register: {}", e))?,
        imm: offset,
        ..Default::default()
    };
    Ok(Instruction::from_def_operands(def, operands))
}

fn parse_u_type(parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
    if parts.len() != 3 {
        return Err(format!(
            "U-type instruction '{}' requires a destination register and immediate value, got {} operands",
            parts[0],
            parts.len() - 1
        ));
    }

    let imm = parse_immediate(parts[2]).map_err(|e| format!("Invalid immediate value: {}", e))?;
    let imm_value = ((imm as u32) & 0xFFFFF) << 12;

    let operands = Operands {
        rd: parse_register(parts[1]).map_err(|e| format!("Invalid destination register: {}", e))?,
        imm: imm_value as i32,
        ..Default::default()
    };
    Ok(Instruction::from_def_operands(def, operands))
}

fn parse_j_type(
    parts: &[&str],
    def: InstructionDefinition,
    labels: &HashMap<String, u32>,
    current_address: u32,
) -> Result<Instruction, String> {
    if parts.len() != 3 {
        return Err(format!(
            "Jump instruction '{}' requires a destination register and a label/offset, got {} operands",
            parts[0],
            parts.len() - 1
        ));
    }

    let offset = if let Ok(imm) = parse_immediate(parts[2]) {
        if imm & 1 != 0 {
            return Err(format!("Jump offset {} must be 2-byte aligned", imm));
        }
        imm
    } else {
        let target = labels
            .get(parts[2])
            .ok_or(format!("Jump target '{}' is not a defined label", parts[2]))?;
        let offset = (*target as i32) - (current_address as i32);
        if offset & 1 != 0 {
            return Err(format!("Jump target '{}' must be 2-byte aligned", parts[2]));
        }
        if offset > 1048575 || offset < -1048576 {
            return Err(format!(
                "Jump target '{}' is too far ({} bytes), must be within -1048576 to +1048575 bytes",
                parts[2], offset
            ));
        }
        offset
    };

    let operands = Operands {
        rd: parse_register(parts[1]).map_err(|e| format!("Invalid destination register: {}", e))?,
        imm: offset,
        ..Default::default()
    };
    Ok(Instruction::from_def_operands(def, operands))
}

fn parse_mem_address(addr: &str) -> Result<(i32, u32), String> {
    let parts: Vec<&str> = addr
        .split(|c| c == '(' || c == ')')
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() != 2 {
        return Err(format!(
            "Memory address must be in format 'offset(register)', got: {}",
            addr
        ));
    }

    let offset = parse_immediate(parts[0])?;
    if offset > 2047 || offset < -2048 {
        return Err(format!(
            "Memory offset {} is out of range (-2048 to 2047)",
            offset
        ));
    }

    let reg = parse_register(parts[1])?;

    Ok((offset, reg))
}

fn parse_register(reg: &str) -> Result<u32, String> {
    let reg = reg.trim().to_lowercase();
    if !reg.starts_with('x') {
        return Err(format!("Invalid register (must start with 'x'): {}", reg));
    }

    match reg[1..].parse::<u32>() {
        Ok(num) if num < 32 => Ok(num),
        _ => Err(format!("Invalid register number (must be 0-31): {}", reg)),
    }
}

fn parse_immediate(value: &str) -> Result<i32, String> {
    let value = value.trim();
    let (is_negative, value) = if value.starts_with('-') {
        (true, &value[1..])
    } else {
        (false, value)
    };

    let abs_value = if value.starts_with("0x") {
        i32::from_str_radix(&value[2..], 16)
            .map_err(|_| format!("Invalid hexadecimal immediate value: {}", value))?
    } else {
        value
            .parse::<i32>()
            .map_err(|_| format!("Invalid decimal immediate value: {}", value))?
    };

    if is_negative {
        Ok(-abs_value)
    } else {
        Ok(abs_value)
    }
}
