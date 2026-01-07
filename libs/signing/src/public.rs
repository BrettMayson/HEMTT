use std::io::{Read, Seek, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt};
use hemtt_pbo::ReadablePbo;
use rsa::BoxedUint;
use sha1::Digest as _;

use crate::{BISign, Error, generate_hashes};

#[derive(Debug)]
/// A public key
pub struct BIPublicKey {
    pub(crate) authority: String,
    pub(crate) length: u32,
    pub(crate) exponent: BoxedUint,
    pub(crate) n: BoxedUint,
}

impl BIPublicKey {
    #[must_use]
    /// Returns the authority of the public key
    pub fn authority(&self) -> &str {
        &self.authority
    }

    #[must_use]
    /// Returns the length of the public key
    pub const fn length(&self) -> u32 {
        self.length
    }

    #[must_use]
    /// Returns the exponent of the public key
    pub const fn exponent(&self) -> &BoxedUint {
        &self.exponent
    }

    #[must_use]
    /// Returns the modulus of the public key
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

    /// Computes the SHA-1 hash of the public key
    ///
    /// # Errors
    /// If writing the public key fails
    pub fn hash(&self) -> Result<Vec<u8>, Error> {
        let mut hasher = sha1::Sha1::new();
        let mut data = Vec::new();
        self.write(&mut data)?;
        hasher.update(&data);
        Ok(hasher.finalize().to_vec())
    }

    /// Write the public key to a writer
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
        Ok(())
    }

    /// Read a public key from a reader
    ///
    /// # Errors
    /// If the reader fails to read
    ///
    /// # Panics
    /// If the public key is invalid
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

        Ok(Self {
            authority,
            length,
            exponent,
            n,
        })
    }

    /// Verifies a signature against this public key.
    ///
    /// # Errors
    /// If the signature is invalid
    pub fn verify<I: Seek + Read>(
        &self,
        pbo: &mut ReadablePbo<I>,
        signature: &BISign,
    ) -> Result<(), Error> {
        if self.authority != signature.authority {
            return Err(Error::AuthorityMismatch {
                sig: signature.authority.clone(),
                key: self.authority.clone(),
            });
        }

        if pbo.is_sorted().is_err() {
            return Err(Error::InvalidFileSorting);
        }

        let (real_hash1, real_hash2, real_hash3) =
            generate_hashes(pbo, signature.version, self.length)?;

        let (signed_hash1, signed_hash2, signed_hash3) = signature.signatures_modpow();

        if real_hash1 != signed_hash1 {
            let (s, r) = super::display_hashes(&signed_hash1, &real_hash1);
            return Err(Error::HashMismatch { sig: s, real: r });
        }

        if real_hash2 != signed_hash2 {
            let (s, r) = super::display_hashes(&signed_hash2, &real_hash2);
            return Err(Error::HashMismatch { sig: s, real: r });
        }

        if real_hash3 != signed_hash3 {
            let (s, r) = super::display_hashes(&signed_hash3, &real_hash3);
            return Err(Error::HashMismatch { sig: s, real: r });
        }

        Ok(())
    }
}
