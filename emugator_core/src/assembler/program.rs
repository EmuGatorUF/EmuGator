use crate::assembler::Address;

use bimap::BiBTreeMap;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct AssembledProgram {
    /// Map of instruction memory addresses to instruction bytes
    pub instruction_memory: BTreeMap<u32, u8>,

    /// Map of initial data memory addresses to data bytes
    pub initial_data_memory: BTreeMap<u32, u8>,

    /// Map of instruction addresses (left) to line numbers (right)
    pub source_map: BiBTreeMap<u32, usize>,

    /// Map of instruction labels to addresses
    pub symbol_table: HashMap<String, Address>,
}

impl AssembledProgram {
    pub fn get_section_start(&self, section: Section) -> u32 {
        match section {
            Section::Text => self.source_map.left_values().next().copied().unwrap_or(0),
            Section::Data => self.initial_data_memory.keys().next().copied().unwrap_or(0),
            _ => todo!(), // TODO: Add support for other sections and user-defined sections
        }
    }

    pub fn empty() -> &'static Self {
        static EMPTY: OnceLock<AssembledProgram> = OnceLock::new();
        EMPTY.get_or_init(|| AssembledProgram {
            instruction_memory: BTreeMap::new(),
            initial_data_memory: BTreeMap::new(),
            source_map: BiBTreeMap::new(),
            symbol_table: HashMap::new(),
        })
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
            &self.initial_data_memory,
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

impl From<Section> for String {
    fn from(val: Section) -> Self {
        match val {
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
