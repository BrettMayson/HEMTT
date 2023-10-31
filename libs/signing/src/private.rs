use std::io::{Read, Seek, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt};
use hemtt_pbo::{BISignVersion, ReadablePbo};
use rsa::{
    traits::{PrivateKeyParts, PublicKeyParts},
    BigUint, RsaPrivateKey,
};

use crate::{error::Error, generate_hashes, public::BIPublicKey, signature::BISign};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// A private key for signing PBOs
pub struct BIPrivateKey {
    authority: String,
    length: u32,
    exponent: BigUint,
    n: BigUint,
    p: BigUint,
    q: BigUint,
    dp: BigUint,
    dq: BigUint,
    qinv: BigUint,
    d: BigUint,
}

impl BIPrivateKey {
    // It won't panic, but clippy doesn't know that.
    // If the precompute fails it return at that point, and won't reach the unwraps.
    #[allow(clippy::missing_panics_doc)]
    /// Generate a new private key.
    ///
    /// # Errors
    /// If RSA generation fails.
    pub fn generate(length: u32, authority: &str) -> Result<Self, Error> {
        let mut rng = rand::thread_rng();
        let mut rsa = RsaPrivateKey::new(&mut rng, length as usize)?;
        rsa.precompute()?;
        let primes = rsa.primes();
        let Some(qinv) = rsa.qinv().unwrap().to_biguint() else {
            return Err(Error::Rsa(rsa::errors::Error::Internal));
        };
        Ok(Self {
            authority: authority.to_string(),
            length,
            exponent: rsa.e().clone(),
            n: rsa.n().clone(),
            p: primes[0].clone(),
            q: primes[1].clone(),
            dp: rsa.dp().unwrap().clone(),
            dq: rsa.dq().unwrap().clone(),
            qinv,
            d: rsa.d().clone(),
        })
    }

    #[must_use]
    /// Returns the public key for this private key.
    pub fn to_public_key(&self) -> BIPublicKey {
        BIPublicKey {
            authority: self.authority.clone(),
            length: self.length,
            exponent: self.exponent.clone(),
            n: self.n.clone(),
        }
    }

    /// Reads a private key from the given input.
    ///
    /// # Errors
    /// If the input fails to read.
    pub fn read<I: Read>(input: &mut I) -> Result<Self, Error> {
        let authority = input.read_cstring()?;
        let temp = input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        input.read_u32::<LittleEndian>()?;
        let length = input.read_u32::<LittleEndian>()?;

        if temp != length / 16 * 9 + 20 {
            return Err(Error::InvalidLength);
        }

        let exponent = {
            let mut buffer = vec![0; 4];
            input.read_exact(&mut buffer)?;
            BigUint::from_bytes_le(&buffer)
        };

        let n = {
            let mut buffer = vec![0; (length / 8) as usize];
            input.read_exact(&mut buffer)?;
            BigUint::from_bytes_le(&buffer)
        };

        let p = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BigUint::from_bytes_le(&buffer)
        };

        let q = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BigUint::from_bytes_le(&buffer)
        };

        let dp = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BigUint::from_bytes_le(&buffer)
        };

        let dq = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BigUint::from_bytes_le(&buffer)
        };

        let qinv = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BigUint::from_bytes_le(&buffer)
        };

        let d = {
            let mut buffer = vec![0; (length / 8) as usize];
            input.read_exact(&mut buffer)?;
            BigUint::from_bytes_le(&buffer)
        };

        Ok(Self {
            authority,
            length,
            exponent,
            n,
            p,
            q,
            dp,
            dq,
            qinv,
            d,
        })
    }

    /// Sign a PBO.
    ///
    /// # Errors
    /// If the PBO fails to read.
    pub fn sign<I: Seek + Read>(
        &self,
        pbo: &mut ReadablePbo<I>,
        version: BISignVersion,
    ) -> Result<BISign, Error> {
        let (hash1, hash2, hash3) = generate_hashes(pbo, version, self.length)?;

        let sig1 = hash1.modpow(&self.d, &self.n);
        let sig2 = hash2.modpow(&self.d, &self.n);
        let sig3 = hash3.modpow(&self.d, &self.n);

        Ok(BISign {
            version,
            authority: self.authority.clone(),
            length: self.length,
            exponent: self.exponent.clone(),
            n: self.n.clone(),
            sig1,
            sig2,
            sig3,
        })
    }

    /// Write private key to output.
    ///
    /// # Errors
    /// If the output fails to write.
    ///
    /// # Panics
    /// If the qinv sign is not `NoSign`.
    pub fn write_danger<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        output.write_cstring(&self.authority)?;
        output.write_u32::<LittleEndian>(self.length / 16 * 9 + 20)?;
        output.write_all(b"\x07\x02\x00\x00\x00\x24\x00\x00")?;
        output.write_all(b"RSA2")?;
        output.write_u32::<LittleEndian>(self.length)?;
        super::write_biguint(output, &self.exponent, 4)?;
        // output.write_all(&self.exponent.to_bytes_le())?;
        super::write_biguint(output, &self.n, (self.length / 8) as usize)?;
        // output.write_all(&self.n.to_bytes_le())?;
        super::write_biguint(output, &self.p, (self.length / 16) as usize)?;
        // output.write_all(&self.p.to_bytes_le())?;
        super::write_biguint(output, &self.q, (self.length / 16) as usize)?;
        // output.write_all(&self.q.to_bytes_le())?;
        super::write_biguint(output, &self.dp, (self.length / 16) as usize)?;
        // output.write_all(&self.dp.to_bytes_le())?;
        super::write_biguint(output, &self.dq, (self.length / 16) as usize)?;
        // output.write_all(&self.dq.to_bytes_le())?;
        super::write_biguint(output, &self.qinv, (self.length / 16) as usize)?;
        // output.write_all(&self.qinv.to_bytes_le())?;
        super::write_biguint(output, &self.d, (self.length / 8) as usize)?;
        // output.write_all(&self.d.to_bytes_le())?;
        Ok(())
    }
}
