use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt};
use hemtt_pbo::BISignVersion;
use rsa::BoxedUint;

use crate::{Error, modpow};

#[derive(Debug)]
/// A signature for a PBO
pub struct BISign {
    pub(crate) version: BISignVersion,
    pub(crate) authority: String,
    pub(crate) length: u32,
    pub(crate) exponent: BoxedUint,
    pub(crate) n: BoxedUint,
    pub(crate) sig1: BoxedUint,
    pub(crate) sig2: BoxedUint,
    pub(crate) sig3: BoxedUint,
}

impl BISign {
    #[must_use]
    /// The version of the signature
    pub const fn version(&self) -> BISignVersion {
        self.version
    }

    #[must_use]
    /// The authority of the signature
    pub fn authority(&self) -> &str {
        &self.authority
    }

    #[must_use]
    /// The length of the signature
    pub const fn length(&self) -> u32 {
        self.length
    }

    #[must_use]
    /// The exponent of the signature
    pub const fn exponent(&self) -> &BoxedUint {
        &self.exponent
    }

    #[must_use]
    /// The modulus of the signature
    pub const fn modulus(&self) -> &BoxedUint {
        &self.n
    }

    #[must_use]
    /// Display the modules in rows of 20 characters
    pub fn modulus_display(&self, left_pad: u8) -> String {
        let mut out = String::new();
        for (i, c) in self.n.to_string_radix_vartime(16).chars().enumerate() {
            if i % 20 == 0 && i != 0 {
                out.push('\n');
                out.push_str(&" ".repeat(left_pad as usize));
            }
            out.push(c);
        }
        out
    }

    #[must_use]
    /// Returns the signatures
    pub const fn signatures(&self) -> (&BoxedUint, &BoxedUint, &BoxedUint) {
        (&self.sig1, &self.sig2, &self.sig3)
    }

    #[must_use]
    /// Returns the signatures modpow'd with the exponent
    pub fn signatures_modpow(&self) -> (BoxedUint, BoxedUint, BoxedUint) {
        let exponent = self.exponent();
        (
            modpow(&self.sig1, exponent, self.modulus()),
            modpow(&self.sig2, exponent, self.modulus()),
            modpow(&self.sig3, exponent, self.modulus()),
        )
    }

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
        crate::write_boxeduint(output, &self.exponent, 4)?;
        crate::write_boxeduint(output, &self.n, (self.length / 8) as usize)?;
        output.write_u32::<LittleEndian>(self.length / 8)?;
        crate::write_boxeduint(output, &self.sig1, (self.length / 8) as usize)?;
        output.write_u32::<LittleEndian>(self.version.into())?;
        output.write_u32::<LittleEndian>(self.length / 8)?;
        crate::write_boxeduint(output, &self.sig2, (self.length / 8) as usize)?;
        output.write_u32::<LittleEndian>(self.length / 8)?;
        crate::write_boxeduint(output, &self.sig3, (self.length / 8) as usize)?;
        Ok(())
    }

    /// Read a signature from a reader
    ///
    /// # Errors
    /// If the reader fails to read
    ///
    /// # Panics
    /// If the signature is invalid
    pub fn read<I: Read>(input: &mut I) -> Result<Self, Error> {
        let authority = input.read_cstring()?;
        let temp = input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        let length = input.read_u32::<LittleEndian>()?;
        let exponent = BoxedUint::from(input.read_u32::<LittleEndian>()?);

        assert_eq!(temp, length / 8 + 20);

        let mut buffer = vec![0; (length / 8) as usize];
        input.read_exact(&mut buffer)?;
        let n = BoxedUint::from_le_slice_vartime(&buffer);

        input.read_u32::<LittleEndian>()?;

        let mut buffer = vec![0; (length / 8) as usize];
        input.read_exact(&mut buffer)?;
        let sig1 = BoxedUint::from_le_slice_vartime(&buffer);

        let version = match input.read_u32::<LittleEndian>()? {
            2 => BISignVersion::V2,
            3 => BISignVersion::V3,
            v => {
                return Err(Error::UknownBISignVersion(v));
            }
        };

        input.read_u32::<LittleEndian>()?;

        let mut buffer = vec![0; (length / 8) as usize];
        input.read_exact(&mut buffer)?;
        let sig2 = BoxedUint::from_le_slice_vartime(&buffer);

        input.read_u32::<LittleEndian>()?;

        let mut buffer = vec![0; (length / 8) as usize];
        input.read_exact(&mut buffer)?;
        let sig3 = BoxedUint::from_le_slice_vartime(&buffer);

        Ok(Self {
            version,
            authority,
            length,
            exponent,
            n,
            sig1,
            sig2,
            sig3,
        })
    }
}
