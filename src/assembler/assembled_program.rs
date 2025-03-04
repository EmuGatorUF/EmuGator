use super::Address;
use bimap::BiBTreeMap;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct AssembledProgram {
    /// Map of instruction memory addresses to instruction bytes
    pub instruction_memory: BTreeMap<u32, u8>,

    /// Map of data memory addresses to data bytes
    pub data_memory: BTreeMap<u32, u8>,

    /// Map of instruction addresses (left) to line numbers (right)
    pub source_map: BiBTreeMap<u32, usize>,

    /// Map of instruction labels to addresses
    #[allow(dead_code)]
    pub symbol_table: HashMap<String, Address>,
}

impl AssembledProgram {
    pub fn get_section_start(&self, section: Section) -> u32 {
        match section {
            Section::Text => self.source_map.left_values().next().copied().unwrap_or(0),
            Section::Data => self.data_memory.keys().next().copied().unwrap_or(0),
            _ => todo!(), // TODO: Add support for other sections and user-defined sections
        }
    }

    #[cfg(test)]
    pub fn emulator_maps(
        &self,
    ) -> (
        &BTreeMap<u32, u8>,
        &BiBTreeMap<u32, usize>,
        &BTreeMap<u32, u8>,
    ) {
        (
            &self.instruction_memory,
            &self.source_map,
            &self.data_memory,
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Section {
    Absolute,
    Data,
    Text,
    Bss,
    User(String),
}

impl From<&str> for Section {
    fn from(s: &str) -> Self {
        match s {
            "absolute" => Section::Absolute,
            "data" => Section::Data,
            "text" => Section::Text,
            "bss" => Section::Bss,
            _ => Section::User(s.to_string()),
        }
    }
}

impl Into<String> for Section {
    fn into(self) -> String {
        match self {
            Section::Absolute => "absolute".to_string(),
            Section::Data => "data".to_string(),
            Section::Text => "text".to_string(),
            Section::Bss => "bss".to_string(),
            Section::User(name) => name,
        }
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(self.clone()))
    }
}
