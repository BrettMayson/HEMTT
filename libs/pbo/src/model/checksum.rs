use crate::ReadPbo;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Checksum([u8; 20]);

impl Checksum {
    #[must_use]
    pub const fn new() -> Self {
        Self([0; 20])
    }

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl From<Vec<u8>> for Checksum {
    fn from(bytes: Vec<u8>) -> Self {
        let mut checksum = [0; 20];
        checksum.copy_from_slice(&bytes);
        Self(checksum)
    }
}

impl ReadPbo for Checksum {
    fn read_pbo<I: std::io::Read>(input: &mut I) -> Result<(Self, usize), crate::error::Error> {
        let mut checksum = [0; 20];
        input.read_exact(&mut checksum)?;
        Ok((Self(checksum), 20))
    }
}
