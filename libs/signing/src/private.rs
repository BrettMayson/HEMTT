use std::io::{Read, Seek, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt};
use hemtt_pbo::{BISignVersion, ReadablePbo};
use rsa::{
    BoxedUint, RsaPrivateKey,
    traits::{PrivateKeyParts, PublicKeyParts},
};

use crate::{
    encrypted::KDFParams, error::Error, generate_hashes, modpow, public::BIPublicKey,
    signature::BISign,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// A private key for signing PBOs
pub struct BIPrivateKey {
    authority: String,
    length: u32,
    exponent: BoxedUint,
    n: BoxedUint,
    p: BoxedUint,
    q: BoxedUint,
    dp: BoxedUint,
    dq: BoxedUint,
    qinv: BoxedUint,
    d: BoxedUint,
}

impl BIPrivateKey {
    #[must_use]
    /// Returns the authority name of the private key.
    pub fn authority(&self) -> &str {
        &self.authority
    }

    #[must_use]
    /// Returns the length of the private key in bits.
    pub const fn length(&self) -> u32 {
        self.length
    }

    // It won't panic, but clippy doesn't know that.
    // If the precompute fails it return at that point, and won't reach the unwraps.
    #[allow(clippy::missing_panics_doc)]
    /// Generate a new private key.
    ///
    /// # Errors
    /// If RSA generation fails.
    pub fn generate(length: u32, authority: &str) -> Result<Self, Error> {
        let mut rng = rand::rng();
        let mut rsa = RsaPrivateKey::new(&mut rng, length as usize)?;
        rsa.precompute()?;
        let primes = rsa.primes();
        let qinv = rsa.qinv().expect(
            "qinv should be precomputed, if it's not, the precompute failed and we should return",
        ).to_montgomery();
        Ok(Self {
            authority: authority.to_string(),
            length,
            exponent: rsa.e().clone(),
            n: rsa.n().clone().get(),
            p: primes[0].clone(),
            q: primes[1].clone(),
            dp: rsa.dp().expect(
                "dp should be precomputed, if it's not, the precompute failed and we should return",
            ).clone(),
            dq: rsa.dq().expect(
                "dq should be precomputed, if it's not, the precompute failed and we should return",
            ).clone(),
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
            BoxedUint::from_le_slice_vartime(&buffer)
        };

        let n = {
            let mut buffer = vec![0; (length / 8) as usize];
            input.read_exact(&mut buffer)?;
            BoxedUint::from_le_slice_vartime(&buffer)
        };

        let p = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BoxedUint::from_le_slice_vartime(&buffer)
        };

        let q = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BoxedUint::from_le_slice_vartime(&buffer)
        };

        let dp = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BoxedUint::from_le_slice_vartime(&buffer)
        };

        let dq = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BoxedUint::from_le_slice_vartime(&buffer)
        };

        let qinv = {
            let mut buffer = vec![0; (length / 16) as usize];
            input.read_exact(&mut buffer)?;
            BoxedUint::from_le_slice_vartime(&buffer)
        };

        let d = {
            let mut buffer = vec![0; (length / 8) as usize];
            input.read_exact(&mut buffer)?;
            BoxedUint::from_le_slice_vartime(&buffer)
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

    /// Reads an encrypted private key from the given input.
    ///
    /// # Errors
    /// If the input fails to read or decrypt.
    pub fn read_encrypted<I: Read>(input: &mut I, password: &str) -> Result<Self, Error> {
        let mut data = Vec::new();
        input.read_to_end(&mut data)?;
        let decrypted = crate::encrypted::decrypt(&data, password)?;
        let mut cursor = std::io::Cursor::new(decrypted);
        Self::read(&mut cursor)
    }

    /// Write encrypted private key to output.
    ///
    /// # Errors
    /// If the output fails to write or encryption fails.
    pub fn write_encrypted<O: Write>(
        &self,
        output: &mut O,
        password: &str,
        kdf_params: KDFParams,
    ) -> Result<(), Error> {
        let mut buffer = Vec::new();
        self.write_danger(&mut buffer)?;
        let encrypted = crate::encrypted::encrypt(&buffer, password, kdf_params)?;
        output.write_all(&encrypted)?;
        Ok(())
    }

    /// Computes the validation hash of the private key.
    ///
    /// # Errors
    /// If computing the hash fails.
    pub fn validation_hash(&self) -> Result<String, Error> {
        let public_key = self.to_public_key();
        let hash_bytes = public_key.hash()?;
        Ok(BoxedUint::from_be_slice_vartime(&hash_bytes)
            .to_string_radix_vartime(16)
            .to_lowercase())
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

        let sig1 = modpow(&hash1, &self.d, &self.n);
        let sig2 = modpow(&hash2, &self.d, &self.n);
        let sig3 = modpow(&hash3, &self.d, &self.n);

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
        super::write_boxeduint(output, &self.exponent, 4)?;
        // output.write_all(&self.exponent.to_bytes_le())?;
        super::write_boxeduint(output, &self.n, (self.length / 8) as usize)?;
        // output.write_all(&self.n.to_bytes_le())?;
        super::write_boxeduint(output, &self.p, (self.length / 16) as usize)?;
        // output.write_all(&self.p.to_bytes_le())?;
        super::write_boxeduint(output, &self.q, (self.length / 16) as usize)?;
        // output.write_all(&self.q.to_bytes_le())?;
        super::write_boxeduint(output, &self.dp, (self.length / 16) as usize)?;
        // output.write_all(&self.dp.to_bytes_le())?;
        super::write_boxeduint(output, &self.dq, (self.length / 16) as usize)?;
        // output.write_all(&self.dq.to_bytes_le())?;
        super::write_boxeduint(output, &self.qinv, (self.length / 16) as usize)?;
        // output.write_all(&self.qinv.to_bytes_le())?;
        super::write_boxeduint(output, &self.d, (self.length / 8) as usize)?;
        // output.write_all(&self.d.to_bytes_le())?;
        Ok(())
    }
}
