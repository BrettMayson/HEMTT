use std::io::{Error, Read, Seek, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_io::*;
use hemtt_pbo::sync::ReadablePbo;
use openssl::bn::{BigNum, BigNumContext};

use crate::{BISign, BISignError};

#[derive(Debug)]
pub struct BIPublicKey {
    pub authority: String,
    pub length: u32,
    pub exponent: u32,
    pub n: BigNum,
}

impl BIPublicKey {
    /// Reads a public key from the given input.
    pub fn read<I: Read>(input: &mut I) -> Result<Self, Error> {
        let authority = input.read_cstring()?;
        let temp = input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        let length = input.read_u32::<LittleEndian>()?;
        let exponent = input.read_u32::<LittleEndian>()?;

        assert_eq!(temp, length / 8 + 20);

        let mut buffer = vec![0; (length / 8) as usize];
        input.read_exact(&mut buffer)?;
        buffer = buffer.iter().rev().cloned().collect();
        let n = BigNum::from_slice(&buffer).unwrap();

        Ok(Self {
            authority,
            length,
            exponent,
            n,
        })
    }

    /// Verifies a signature against this public key.
    pub fn verify<I: Seek + Read>(
        &self,
        pbo: &mut ReadablePbo<I>,
        signature: &BISign,
    ) -> Result<(), BISignError> {
        if self.authority != signature.authority {
            return Err(BISignError::AuthorityMismatch {
                signed: signature.authority.clone(),
                real: self.authority.clone(),
            });
        }

        if pbo.is_sorted().is_err() {
            return Err(BISignError::InvalidFileSorting);
        }

        let (real_hash1, real_hash2, real_hash3) =
            crate::types::generate_hashes(pbo, signature.version, self.length);

        let (signed_hash1, signed_hash2, signed_hash3) = self.get_hashes(signature);

        if real_hash1 != signed_hash1 {
            let (s, r) = crate::types::display_hashes(signed_hash1, real_hash1);
            return Err(BISignError::HashMismatch { signed: s, real: r });
        }

        if real_hash2 != signed_hash2 {
            let (s, r) = crate::types::display_hashes(signed_hash2, real_hash2);
            return Err(BISignError::HashMismatch { signed: s, real: r });
        }

        if real_hash3 != signed_hash3 {
            let (s, r) = crate::types::display_hashes(signed_hash3, real_hash3);
            return Err(BISignError::HashMismatch { signed: s, real: r });
        }

        Ok(())
    }

    pub fn get_hashes(&self, signature: &BISign) -> (BigNum, BigNum, BigNum) {
        let mut ctx = BigNumContext::new().unwrap();
        let exponent = BigNum::from_u32(signature.exponent).unwrap();
        let mut signed_hash1: BigNum = BigNum::new().unwrap();
        signed_hash1
            .mod_exp(&signature.sig1, &exponent, &signature.n, &mut ctx)
            .unwrap();
        let mut signed_hash2: BigNum = BigNum::new().unwrap();
        signed_hash2
            .mod_exp(&signature.sig2, &exponent, &signature.n, &mut ctx)
            .unwrap();
        let mut signed_hash3: BigNum = BigNum::new().unwrap();
        signed_hash3
            .mod_exp(&signature.sig3, &exponent, &signature.n, &mut ctx)
            .unwrap();
        (signed_hash1, signed_hash2, signed_hash3)
    }

    /// Write public key to output.
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        output.write_cstring(&self.authority)?;
        output.write_u32::<LittleEndian>(self.length / 8 + 20)?;
        output.write_all(b"\x06\x02\x00\x00\x00\x24\x00\x00")?;
        output.write_all(b"RSA1")?;
        output.write_u32::<LittleEndian>(self.length)?;
        output.write_u32::<LittleEndian>(self.exponent)?;
        crate::types::write_bignum(output, &self.n, (self.length / 8) as usize)?;
        Ok(())
    }
}
