#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum Mime {
    Vers,
    Cprs,
    Enco,
    #[default]
    Blank,
}

impl Mime {
    #[must_use]
    pub const fn as_u32(&self) -> u32 {
        match self {
            Self::Vers => 0x5665_7273,
            Self::Cprs => 0x4370_7273,
            Self::Enco => 0x456e_6372,
            Self::Blank => 0x0000_0000,
        }
    }

    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x5665_7273 => Some(Self::Vers),
            0x4370_7273 => Some(Self::Cprs),
            0x456e_6372 => Some(Self::Enco),
            0x0000_0000 => Some(Self::Blank),
            _ => None,
        }
    }
}
