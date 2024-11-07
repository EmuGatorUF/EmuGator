#[cfg(test)]
mod tests;

use std::{collections::{BTreeMap, HashMap}, str::FromStr};

use crate::isa::{Instruction, InstructionDefinition, InstructionFormat, Operands, ISA};

#[derive(Debug, PartialEq, Clone, Copy)]
enum Section {
    Data,
    Text,
}

#[derive(Debug)]
struct DataItem {
    size: usize, // in bytes
    values: Vec<u8>,
}


struct Assembler();

impl Assembler {
    fn new() -> Self {
        Assembler {}
    }

    fn parse_section_directive(&self, line: &str) -> Option<(Section, u32)> {
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

    fn assemble(&self, program: &str) -> Result<AssembledProgram, String> {
        let mut assembled = AssembledProgram::new();
        let mut current_section = Section::Text;
        let mut text_address = 0;
        let mut data_address = 0;
    
        // First pass: collect labels and process data
        for (line_num, line) in program.lines().enumerate() {
            let line = self.clean_line(line);
            if line.is_empty() {
                continue;
            }
    
            let (label_opt, content) = self.split_label_and_content(&line);
    
            // Handle section directives with optional address
            if let Some((section, address)) = self.parse_section_directive(&content) {
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
                        assembled.add_label(label, text_address, false);
                    }
                    Section::Data => {
                        assembled.add_label(label, data_address, true);
                    }
                }
            }
    
            // If there's no content after the label, continue to next line
            if content.is_empty() {
                continue;
            }
    
            // Handle data directives
            if content.starts_with('.') {
                if current_section == Section::Data {
                    if let Ok(Some((_, data))) = self.parse_data_line(&content) {
                        assembled.add_data(data_address, &data.values);
                        data_address += data.size as u32;
                    }
                } else {
                    return Err(format!(
                        "Data directive '{}' outside of .data section on line {}",
                        content,
                        line_num + 1
                    ));
                }
                continue;
            }
    
            // Count instruction size for text section
            if current_section == Section::Text && !content.is_empty() {
                text_address += 4;
            }
        }
    
        // Second pass: assemble instructions
        current_section = Section::Text;
        text_address = assembled.get_section_start(Section::Text);
    
        for (line_num, line) in program.lines().enumerate() {
            let line = self.clean_line(line);
            if line.is_empty() {
                continue;
            }
    
            let (_, content) = self.split_label_and_content(&line);
            if content.is_empty() {
                continue;
            }
    
            // Handle section directives
            if let Some((section, address)) = self.parse_section_directive(&content) {
                current_section = section;
                match section {
                    Section::Text => text_address = address,
                    Section::Data => (),
                }
                continue;
            }
    
            if current_section == Section::Text && !content.starts_with('.') {
                match self.parse_instruction(
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
                    Err(e) => return Err(format!("Error on line {}: {}", line_num + 1, e)),
                }
            }
        }
    
        Ok(assembled)
    }

    fn clean_line(&self, line: &str) -> String {
        match line.split('#').next() {
            Some(l) => l.trim().to_string(),
            None => String::new(),
        }
    }

    fn split_label_and_content(&self, line: &str) -> (Option<String>, String) {
        if let Some(colon_pos) = line.find(':') {
            let (label, rest) = line.split_at(colon_pos);
            let content = rest[1..].trim().to_string();
            (Some(label.trim().to_string()), content)
        } else {
            (None, line.to_string())
        }
    }

    fn parse_data_line(&self, line: &str) -> Result<Option<(String, DataItem)>, String> {
        if line.ends_with(':') {
            return Ok(None);
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid data directive".to_string());
        }

        let directive = parts[0];
        let joined = parts[1..].join(" ");
        let values: Vec<&str> = joined.split(',').map(|s| s.trim()).collect();

        match directive {
            ".byte" => {
                let bytes = values
                    .iter()
                    .map(|v| self.parse_number(v))
                    .collect::<Result<Vec<u8>, _>>()?;
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
                    let word = self.parse_number(value)? as u32;
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
            _ => Err(format!("Unknown data directive: {}", directive)),
        }
    }

    fn parse_number(&self, value: &str) -> Result<u8, String> {
        let value = value.trim();
        if value.starts_with("0x") {
            u8::from_str_radix(&value[2..], 16)
        } else {
            value.parse::<u8>()
        }
        .map_err(|_| format!("Invalid numeric value: {}", value))
    }

    fn parse_instruction(
        &self,
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
        let def = ISA::from_str(&name).map_err(|_| format!("Unknown instruction: {}", name))?.definition();

        // For load/store instructions with labels, convert label to base+offset format
        if def.format == InstructionFormat::I && def.opcode == 0b0000011
            || def.format == InstructionFormat::S && def.opcode == 0b0100011
        {
            if parts.len() == 3 && data_labels.contains_key(parts[2]) {
                let base_addr = data_labels[parts[2]];
                let modified_addr = format!("{}(x0)", base_addr);
                let mut modified_parts = parts.to_vec();
                modified_parts[2] = &modified_addr;
                return match def.format {
                    InstructionFormat::I => self.parse_i_type(&modified_parts, def.clone()), // Use same instruction def
                    InstructionFormat::S => self.parse_s_type(&modified_parts, def.clone()),
                    _ => unreachable!(),
                };
            }
        }

        match def.format {
            InstructionFormat::R => self.parse_r_type(&parts, def),
            InstructionFormat::I => self.parse_i_type(&parts, def),
            InstructionFormat::S => self.parse_s_type(&parts, def),
            InstructionFormat::B => self.parse_b_type(&parts, def, text_labels, current_address),
            InstructionFormat::U => self.parse_u_type(&parts, def),
            InstructionFormat::J => self.parse_j_type(&parts, def, text_labels, current_address),
        }
    }

    fn parse_r_type(&self, parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
        if parts.len() != 4 {
            return Err("R-type instructions need 3 registers".to_string());
        }
        let operands = Operands {
            rd: self.parse_register(parts[1])?,
            rs1: self.parse_register(parts[2])?,
            rs2: self.parse_register(parts[3])?,
            imm: 0,
        };
        Ok(Instruction::from_def_operands(def, operands))
    }

    fn parse_i_type(&self, parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
        match def.opcode {
            0b0000011 => self.parse_load_type(&parts, def),
            0b1110011 => {
                // Special handling for ECALL/EBREAK
                if parts.len() != 1 {
                    return Err("ECALL/EBREAK instructions take no operands".to_string());
                }

                let operands = Operands {
                    rd: 0,   // x0
                    rs1: 0,  // x0
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
                // Special handling for FENCE
                if parts.len() != 1 {
                    return Err("FENCE instruction takes no operands".to_string());
                }

                let operands = Operands {
                    rd: 0,   // x0
                    rs1: 0,  // x0
                    imm: 0,
                    ..Default::default()
                };
                Ok(Instruction::from_def_operands(def, operands))
            }
            _ => {
                // Regular I-type instructions
                if parts.len() != 4 {
                    return Err("I-type instructions need 2 registers and an immediate".to_string());
                }

                // Special handling for shift instructions (SLLI, SRLI, SRAI)
                let mut imm = self.parse_immediate(parts[3])?;
                if let Some(funct7) = def.funct7 {
                    // For shift instructions, immediate is split into funct7 and shamt
                    if def.opcode == 0b0010011 && (def.funct3 == Some(0x1) || def.funct3 == Some(0x5)) {
                        // SLLI, SRLI, SRAI
                        let shamt = imm & 0x1F; // Bottom 5 bits only
                        imm = ((funct7 as i32) << 5) | shamt; // Combine funct7 and shamt
                    }
                }

                let operands = Operands {
                    rd: self.parse_register(parts[1])?,
                    rs1: self.parse_register(parts[2])?,
                    imm,
                    ..Default::default()
                };
                Ok(Instruction::from_def_operands(def, operands))
            }
        }
    }

    fn parse_load_type(
        &self,
        parts: &[&str],
        def: InstructionDefinition,
    ) -> Result<Instruction, String> {
        if parts.len() != 3 {
            return Err("Load instructions need a register and a memory address".to_string());
        }

        let (offset, base) = self.parse_mem_address(parts[2])?;

        let operands = Operands {
            rd: self.parse_register(parts[1])?,
            rs1: base,
            imm: offset,
            ..Default::default()
        };
        Ok(Instruction::from_def_operands(def, operands))
    }

    fn parse_s_type(&self, parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
        if parts.len() != 3 {
            return Err("Store instructions need a register and a memory address".to_string());
        }

        let (offset, base) = self.parse_mem_address(parts[2])?;

        let operands = Operands {
            rs1: base,
            rs2: self.parse_register(parts[1])?,
            imm: offset,
            ..Default::default()
        };
        Ok(Instruction::from_def_operands(def, operands))
    }

    fn parse_b_type(
        &self,
        parts: &[&str],
        def: InstructionDefinition,
        labels: &HashMap<String, u32>,
        current_address: u32,
    ) -> Result<Instruction, String> {
        if parts.len() != 4 {
            return Err("B-type instructions need 2 registers and a label".to_string());
        }

        let target = labels
            .get(parts[3])
            .ok_or(format!("Undefined label: {}", parts[3]))?;

        let offset = (*target as i32) - (current_address as i32);
        if offset & 1 != 0 {
            return Err("Branch target must be 2-byte aligned".to_string());
        }
        if offset > 4095 || offset < -4096 {
            return Err("Branch offset out of range (-4096 to +4095)".to_string());
        }

        let operands = Operands {
            rs1: self.parse_register(parts[1])?,
            rs2: self.parse_register(parts[2])?,
            imm: offset,
            ..Default::default()
        };
        Ok(Instruction::from_def_operands(def, operands))
    }

    fn parse_u_type(&self, parts: &[&str], def: InstructionDefinition) -> Result<Instruction, String> {
        if parts.len() != 3 {
            return Err("U-type instructions need a register and an immediate".to_string());
        }

        let operands = Operands {
            rd: self.parse_register(parts[1])?,
            imm: self.parse_immediate(parts[2])? << 12,
            ..Default::default()
        };
        Ok(Instruction::from_def_operands(def, operands))
    }

    fn parse_j_type(
        &self,
        parts: &[&str],
        def: InstructionDefinition,
        labels: &HashMap<String, u32>,
        current_address: u32,
    ) -> Result<Instruction, String> {
        if parts.len() != 3 {
            return Err("J-type instructions need a register and a label/offset".to_string());
        }

        let offset = if let Ok(imm) = self.parse_immediate(parts[2]) {
            if imm & 1 != 0 {
                return Err("Jump target must be 2-byte aligned".to_string());
            }
            imm
        } else {
            let target = labels
                .get(parts[2])
                .ok_or(format!("Undefined label: {}", parts[2]))?;
            let offset = (*target as i32) - (current_address as i32);
            if offset & 1 != 0 {
                return Err("Jump target must be 2-byte aligned".to_string());
            }
            if offset > 1048575 || offset < -1048576 {
                return Err("Jump offset out of range (-1048576 to +1048575)".to_string());
            }
            offset
        };

        let operands = Operands {
            rd: self.parse_register(parts[1])?,
            imm: offset,
            ..Default::default()
        };
        Ok(Instruction::from_def_operands(def, operands))
    }

    fn parse_mem_address(&self, addr: &str) -> Result<(i32, u32), String> {
        let parts: Vec<&str> = addr
            .split(|c| c == '(' || c == ')')
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() != 2 {
            return Err("Memory address must be in format: offset(register)".to_string());
        }

        let offset = self.parse_immediate(parts[0])?;
        let reg = self.parse_register(parts[1])?;

        Ok((offset, reg))
    }

    fn parse_register(&self, reg: &str) -> Result<u32, String> {
        let reg = reg.trim().to_lowercase();
        if !reg.starts_with('x') {
            return Err(format!("Invalid register: {}", reg));
        }

        match reg[1..].parse::<u32>() {
            Ok(num) if num < 32 => Ok(num),
            _ => Err(format!("Invalid register number: {}", reg)),
        }
    }

    fn parse_immediate(&self, imm: &str) -> Result<i32, String> {
        let imm = imm.trim();
        if imm.starts_with("0x") {
            i32::from_str_radix(&imm[2..], 16)
        } else {
            imm.parse::<i32>()
        }
        .map_err(|_| format!("Invalid immediate value: {}", imm))
    }
}

#[derive(Debug)]
struct AssembledProgram {
    instruction_memory: BTreeMap<u32, u8>,
    data_memory: BTreeMap<u32, u8>,
    source_map: BTreeMap<u32, usize>,
    labels: HashMap<String, u32>,
    data_labels: HashMap<String, u32>,
}

impl AssembledProgram {
    fn new() -> Self {
        AssembledProgram {
            instruction_memory: BTreeMap::new(),
            data_memory: BTreeMap::new(),
            source_map: BTreeMap::new(),
            labels: HashMap::new(),
            data_labels: HashMap::new(),
        }
    }

    fn get_section_start(&self, section: Section) -> u32 {
        match section {
            Section::Text => self.source_map.keys().next().copied().unwrap_or(0),
            Section::Data => self.data_memory.keys().next().copied().unwrap_or(0),
        }
    }

    fn add_label(&mut self, label: String, address: u32, is_data: bool) {
        if is_data {
            self.data_labels.insert(label, address);
        } else {
            self.labels.insert(label, address);
        }
    }

    fn add_instruction(&mut self, address: u32, encoded: u32, line_num: usize) {
        self.instruction_memory
            .insert(address, (encoded & 0xFF) as u8);
        self.instruction_memory
            .insert(address + 1, ((encoded >> 8) & 0xFF) as u8);
        self.instruction_memory
            .insert(address + 2, ((encoded >> 16) & 0xFF) as u8);
        self.instruction_memory
            .insert(address + 3, ((encoded >> 24) & 0xFF) as u8);

        self.source_map.insert(address, line_num);
    }

    fn add_data(&mut self, address: u32, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            self.data_memory.insert(address + i as u32, byte);
        }
    }
}

pub fn get_emulator_maps(
    program: &str,
) -> Result<(BTreeMap<u32, u8>, BTreeMap<u32, usize>, BTreeMap<u32, u8>), String> {
    let assembler = Assembler::new();
    match assembler.assemble(program) {
        Ok(assembled) => Ok((
            assembled.instruction_memory,
            assembled.source_map,
            assembled.data_memory,
        )),
        Err(e) => Err(e),
    }
}
