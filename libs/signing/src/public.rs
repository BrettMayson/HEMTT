use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_io::WriteExt;
use rsa::BigUint;

use crate::Error;

#[derive(Debug)]
pub struct BIPublicKey {
    pub(crate) authority: String,
    pub(crate) length: u32,
    pub(crate) exponent: BigUint,
    pub(crate) n: BigUint,
}

impl BIPublicKey {
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        output.write_cstring(&self.authority)?;
        output.write_u32::<LittleEndian>(self.length / 8 + 20)?;
        output.write_all(b"\x06\x02\x00\x00\x00\x24\x00\x00")?;
        output.write_all(b"RSA1")?;
        output.write_u32::<LittleEndian>(self.length)?;
        crate::write_biguint(output, &self.exponent, 4)?;
        crate::write_biguint(output, &self.n, (self.length / 8) as usize)?;
        Ok(())
    }
}
