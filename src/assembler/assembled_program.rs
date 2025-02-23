use bimap::BiBTreeMap;
use ibig::IBig;
use std::{
    collections::{BTreeMap, HashMap}, ops::Add, path::Display, str::FromStr
};

#[derive(Debug)]
pub struct AssembledProgram {
    /// Map of instruction memory addresses to instruction bytes
    pub instruction_memory: BTreeMap<u32, u8>,

    /// Map of data memory addresses to data bytes
    pub data_memory: BTreeMap<u32, u8>,

    /// Map of line numbers (left) to instruction addresses (right)
    pub source_map: BiBTreeMap<u32, usize>,

    /// Map of instruction labels to addresses
    pub symbol_table: HashMap<String, Address>,
}

impl AssembledProgram {
    pub fn new() -> Self {
        AssembledProgram {
            instruction_memory: BTreeMap::new(),
            data_memory: BTreeMap::new(),
            source_map: BiBTreeMap::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn get_section_start(&self, section: Section) -> u32 {
        match section {
            Section::Text => self.source_map.left_values().next().copied().unwrap_or(0),
            Section::Data => self.data_memory.keys().next().copied().unwrap_or(0),
            _ => todo!(), // TODO: Add support for other sections and user-defined sections
        }
    }

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

#[derive(Debug, Clone)]
pub struct Address(pub Section, pub IBig);

impl Address {
    pub fn new(section: Option<Section>, offset: IBig) -> Self {
        Address(section.unwrap_or(Section::Absolute), offset)
    }
}

impl std::ops::Neg for Address {
    type Output = Result<Address, String>;

    fn neg(self) -> Self::Output {
        if self.0 == Section::Absolute {
            Ok(Address(Section::Absolute, -self.1))
        } else {
            Err(format!("Cannot negate address in section {}", self.0))
        }
    }
}

impl std::ops::Not for Address {
    type Output = Result<Address, String>;

    fn not(self) -> Self::Output {
        if self.0 == Section::Absolute {
            Ok(Address(Section::Absolute, !self.1))
        } else {
            Err(format!("Cannot bitwise not address in section {}", self.0))
        }
    }
}

impl std::ops::Mul for Address {
    type Output = Result<Address, String>;

    fn mul(self, other: Address) -> Self::Output {
        if self.0 == Section::Absolute && other.0 == Section::Absolute {
            Ok(Address(Section::Absolute, self.1 * other.1))
        } else {
            Err(format!(
                "Cannot multiply addresses from sections {} * {}",
                self.0, other.0
            ))
        }
    }
}

impl std::ops::Div for Address {
    type Output = Result<Address, String>;

    fn div(self, other: Address) -> Self::Output {
        if self.0 == Section::Absolute && other.0 == Section::Absolute {
            Ok(Address(Section::Absolute, self.1 / other.1))
        } else {
            Err(format!(
                "Cannot divide addresses from sections {} / {}",
                self.0, other.0
            ))
        }
    }
}

impl std::ops::Rem for Address {
    type Output = Result<Address, String>;

    fn rem(self, other: Address) -> Self::Output {
        if self.0 == Section::Absolute && other.0 == Section::Absolute {
            Ok(Address(Section::Absolute, self.1 % other.1))
        } else {
            Err(format!(
                "Cannot modulo addresses from sections {} % {}",
                self.0, other.0
            ))
        }
    }
}

impl std::ops::Shl for Address {
    type Output = Result<Address, String>;

    fn shl(self, other: Address) -> Self::Output {
        if self.0 == Section::Absolute && other.0 == Section::Absolute {
            Ok(Address(
                Section::Absolute,
                self.1 << usize::try_from(&other.1).map_err(|e| e.to_string())?,
            ))
        } else {
            Err(format!(
                "Cannot left shift addresses from sections {} << {}",
                self.0, other.0
            ))
        }
    }
}

impl std::ops::Shr for Address {
    type Output = Result<Address, String>;

    fn shr(self, other: Address) -> Self::Output {
        if self.0 == Section::Absolute && other.0 == Section::Absolute {
            Ok(Address(
                Section::Absolute,
                self.1 >> usize::try_from(&other.1).map_err(|e| e.to_string())?,
            ))
        } else {
            Err(format!(
                "Cannot right shift addresses from sections {} >> {}",
                self.0, other.0
            ))
        }
    }
}

impl std::ops::BitOr for Address {
    type Output = Result<Address, String>;

    fn bitor(self, other: Address) -> Self::Output {
        if self.0 == Section::Absolute && other.0 == Section::Absolute {
            Ok(Address(Section::Absolute, self.1 | other.1))
        } else {
            Err(format!(
                "Cannot bitwise or addresses from sections {} | {}",
                self.0, other.0
            ))
        }
    }
}

impl std::ops::BitAnd for Address {
    type Output = Result<Address, String>;

    fn bitand(self, other: Address) -> Self::Output {
        if self.0 == Section::Absolute && other.0 == Section::Absolute {
            Ok(Address(Section::Absolute, self.1 & other.1))
        } else {
            Err(format!(
                "Cannot bitwise and addresses from sections {} & {}",
                self.0, other.0
            ))
        }
    }
}

impl std::ops::BitXor for Address {
    type Output = Result<Address, String>;

    fn bitxor(self, other: Address) -> Self::Output {
        if self.0 == Section::Absolute && other.0 == Section::Absolute {
            Ok(Address(Section::Absolute, self.1 ^ other.1))
        } else {
            Err(format!(
                "Cannot bitwise xor addresses from sections {} ^ {}",
                self.0, other.0
            ))
        }
    }
}

impl std::ops::Add<Address> for Address {
    type Output = Result<Address, String>;

    fn add(self, other: Address) -> Self::Output {
        match (self.0, other.0) {
            (Section::Absolute, section) | (section, Section::Absolute) => {
                Ok(Address(section, self.1 + other.1))
            }
            (section, other_section) if section == other_section => {
                Ok(Address(section, self.1 + other.1))
            }
            (section, other_section) => Err(format!(
                "Cannot add addresses from different sections {} + {}",
                section, other_section
            )),
        }
    }
}

impl std::ops::Sub<Address> for Address {
    type Output = Result<Address, String>;

    fn sub(self, other: Address) -> Self::Output {
        match (self.0, other.0) {
            (section, Section::Absolute) => Ok(Address(section, self.1 - other.1)),
            (section, other_section) if section == other_section => {
                Ok(Address(Section::Absolute, self.1 - other.1))
            }
            (section, other_section) => Err(format!(
                "Cannot subtract addresses from different sections {} - {}",
                section, other_section
            )),
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.1, self.0)
    }
}
