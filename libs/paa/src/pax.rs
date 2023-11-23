use std::{fmt::Display, io::Read};

use texpresso::Format;

#[derive(Debug, Clone)]
pub enum PaXType {
    DXT1,
    DXT2,
    DXT3,
    DXT4,
    DXT5,
    RGBA4,
    RGBA5,
    RGBA8,
    GRAYA,
}

impl PaXType {
    pub fn from_stream<I: Read>(stream: &mut I) -> Option<Self> {
        let mut bytes = [0; 2];
        if stream.read_exact(&mut bytes).is_ok() {
            Self::from_bytes(bytes)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 2]) -> Option<Self> {
        match bytes {
            [1, 255] => Some(Self::DXT1),    // 0x01FF
            [2, 255] => Some(Self::DXT2),    // 0x02FF
            [3, 255] => Some(Self::DXT3),    // 0x03FF
            [4, 255] => Some(Self::DXT4),    // 0x04FF
            [5, 255] => Some(Self::DXT5),    // 0x05FF
            [68, 68] => Some(Self::RGBA4),   // 0x4444
            [21, 85] => Some(Self::RGBA5),   // 0x1555
            [136, 136] => Some(Self::RGBA8), // 0x8888
            [128, 128] => Some(Self::GRAYA), // 0x8080
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_bytes(&self) -> [u8; 2] {
        match self {
            Self::DXT1 => [1, 255],
            Self::DXT2 => [2, 255],
            Self::DXT3 => [3, 255],
            Self::DXT4 => [4, 255],
            Self::DXT5 => [5, 255],
            Self::RGBA4 => [68, 68],
            Self::RGBA5 => [21, 85],
            Self::RGBA8 => [136, 136],
            Self::GRAYA => [128, 128],
        }
    }
}

impl From<PaXType> for Format {
    fn from(pax: PaXType) -> Self {
        match pax {
            PaXType::DXT1 => Self::Bc1,
            PaXType::DXT3 => Self::Bc2,
            PaXType::DXT5 => Self::Bc3,
            _ => unimplemented!(),
        }
    }
}

impl From<Format> for PaXType {
    fn from(pax: Format) -> Self {
        match pax {
            Format::Bc1 => Self::DXT1,
            Format::Bc2 => Self::DXT3,
            Format::Bc3 => Self::DXT5,
            _ => unimplemented!(),
        }
    }
}

impl Display for PaXType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DXT1 => write!(f, "DXT1"),
            Self::DXT2 => write!(f, "DXT2"),
            Self::DXT3 => write!(f, "DXT3"),
            Self::DXT4 => write!(f, "DXT4"),
            Self::DXT5 => write!(f, "DXT5"),
            Self::RGBA4 => write!(f, "RGBA4"),
            Self::RGBA5 => write!(f, "RGBA5"),
            Self::RGBA8 => write!(f, "RGBA8"),
            Self::GRAYA => write!(f, "GRAYA"),
        }
    }
}
