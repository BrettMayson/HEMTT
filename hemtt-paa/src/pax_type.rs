use std::io::Read;

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

    pub fn from_bytes(bytes: [u8; 2]) -> Option<Self> {
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
}

impl Into<image::dxt::DXTVariant> for PaXType {
    fn into(self) -> image::dxt::DXTVariant {
        match self {
            Self::DXT1 => image::dxt::DXTVariant::DXT1,
            Self::DXT3 => image::dxt::DXTVariant::DXT3,
            Self::DXT5 => image::dxt::DXTVariant::DXT5,
            _ => unimplemented!(),
        }
    }
}
