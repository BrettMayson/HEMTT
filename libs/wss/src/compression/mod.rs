use crate::Error;

mod byte;
mod nibble;
mod none;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Compression type for WSS files
///
/// Reccomended to use either `Compression::Byte` or `Compression::None`
pub enum Compression {
    /// No compression
    None,
    /// Nibble compression, IMA ADPCM-inspired, not recommended
    Nibble,
    /// Byte compression, recommended
    #[default]
    Byte,
}

impl Compression {
    /// Create a new compression type from a u32
    ///
    /// # Errors
    /// [`Error::InvalidCompressionValue`] if the value is not 0, 4, or 8
    pub const fn from_u32(value: u32) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::None),
            4 => Ok(Self::Nibble),
            8 => Ok(Self::Byte),
            _ => Err(Error::InvalidCompressionValue(value)),
        }
    }

    #[must_use]
    pub const fn to_u32(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Nibble => 4,
            Self::Byte => 8,
        }
    }

    #[must_use]
    pub fn decompress(&self, data: &[u8]) -> Vec<i16> {
        match self {
            Self::None => none::decompress(data),
            Self::Nibble => nibble::decompress(data),
            Self::Byte => byte::decompress(data),
        }
    }

    #[must_use]
    pub fn compress(&self, data: &[i16]) -> Vec<u8> {
        match self {
            Self::None => none::compress(data),
            Self::Nibble => nibble::compress(data),
            Self::Byte => byte::compress(data),
        }
    }
}
