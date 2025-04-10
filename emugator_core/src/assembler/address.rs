use super::Section;
use ibig::IBig;

#[derive(Debug, Clone)]
pub struct Address(pub Section, pub IBig);

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
        write!(f, "{} ({})", self.1, self.0)
    }
}
