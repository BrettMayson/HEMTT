use std::fmt::Write;

use crate::ReadPbo;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
/// A checksum found at the end of a PBO
///
/// The checksum is a SHA1 hash of the PBO's properties & files
pub struct Checksum([u8; 20]);

impl Checksum {
    #[must_use]
    /// Create a new empty checksum
    pub const fn new() -> Self {
        Self([0; 20])
    }

    #[must_use]
    /// Create a new checksum from a byte array
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    #[must_use]
    /// Get the checksum as a byte array
    pub const fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    #[must_use]
    pub fn hex(&self) -> String {
        let mut out = String::new();
        for byte in &self.0 {
            write!(out, "{byte:02x}").expect("Failed to write hex");
        }
        out
    }
}

impl From<Vec<u8>> for Checksum {
    fn from(bytes: Vec<u8>) -> Self {
        let mut checksum = [0; 20];
        checksum.copy_from_slice(&bytes);
        Self(checksum)
    }
}

impl AsRef<[u8]> for Checksum {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl ReadPbo for Checksum {
    fn read_pbo<I: std::io::Read>(input: &mut I) -> Result<(Self, usize), crate::error::Error> {
        let mut checksum = [0; 20];
        input.read_exact(&mut checksum)?;
        Ok((Self(checksum), 20))
    }
}
