use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_common::io::WriteExt;
use hemtt_pbo::BISignVersion;
use rsa::BigUint;

use crate::Error;

#[derive(Debug)]
/// A signature for a PBO
pub struct BISign {
    pub(crate) version: BISignVersion,
    pub(crate) authority: String,
    pub(crate) length: u32,
    pub(crate) exponent: BigUint,
    pub(crate) n: BigUint,
    pub(crate) sig1: BigUint,
    pub(crate) sig2: BigUint,
    pub(crate) sig3: BigUint,
}

impl BISign {
    /// Write the signature to a writer
    ///
    /// # Errors
    /// If the writer fails to write
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        output.write_cstring(&self.authority)?;
        output.write_u32::<LittleEndian>(self.length / 8 + 20)?;
        output.write_all(b"\x06\x02\x00\x00\x00\x24\x00\x00")?;
        output.write_all(b"RSA1")?;
        output.write_u32::<LittleEndian>(self.length)?;
        crate::write_biguint(output, &self.exponent, 4)?;
        crate::write_biguint(output, &self.n, (self.length / 8) as usize)?;
        output.write_u32::<LittleEndian>(self.length / 8)?;
        crate::write_biguint(output, &self.sig1, (self.length / 8) as usize)?;
        output.write_u32::<LittleEndian>(self.version.into())?;
        output.write_u32::<LittleEndian>(self.length / 8)?;
        crate::write_biguint(output, &self.sig2, (self.length / 8) as usize)?;
        output.write_u32::<LittleEndian>(self.length / 8)?;
        crate::write_biguint(output, &self.sig3, (self.length / 8) as usize)?;
        Ok(())
    }
}
