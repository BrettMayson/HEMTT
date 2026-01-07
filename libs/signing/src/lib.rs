//! HEMTT - Arma 3 Signing

// Parts of the following code is derivative work of the code from the armake2 project by KoffeinFlummi,
// which is licensed GPLv2. This code therefore is also licensed under the terms
// of the GNU Public License, verison 2.

// The original code can be found here:
// https://github.com/KoffeinFlummi/armake2/blob/4b736afc8c615cf49a0d1adce8f6b9a8ae31d90f/src/sign.rs

use std::io::{Read, Seek, Write};

use crypto_bigint::{
    Odd, Resize,
    modular::{BoxedMontyForm, BoxedMontyParams},
};
use hemtt_common::BISignVersion;
use hemtt_pbo::ReadablePbo;
use rsa::BoxedUint;
use sha1::{Digest, Sha1};

mod encrypted;
mod error;
mod private;
mod public;
mod signature;

pub use encrypted::KDFParams;
pub use error::Error;
pub use private::{BIPrivateKey, HEMTTPrivateKey};
pub use public::BIPublicKey;
pub use signature::BISign;

/// Writes a [`BoxedUint`] to the given output.
///
/// # Errors
/// If the output fails to write.
pub fn write_boxeduint<O: Write>(output: &mut O, bn: &BoxedUint, size: usize) -> Result<(), Error> {
    let mut vec: Vec<u8> = bn.to_le_bytes().to_vec();
    vec.resize(size, 0);
    output.write_all(&vec).map_err(std::convert::Into::into)
}

fn display_hashes(a: &BoxedUint, b: &BoxedUint) -> (String, String) {
    let hex_a = a.to_string_radix_vartime(16).to_lowercase();
    let hex_b = b.to_string_radix_vartime(16).to_lowercase();

    if hex_a.len() != hex_b.len() || hex_a.len() <= 40 {
        return (hex_a, hex_b);
    }

    let (padding_a, hash_a) = hex_a.split_at(hex_a.len() - 40);
    let (paddingb, hash_b) = hex_b.split_at(hex_b.len() - 40);

    if padding_a == paddingb {
        (hash_a.to_string(), hash_b.to_string())
    } else {
        (hex_a, hex_b)
    }
}

/// Generate the hashes for a PBO
///
/// # Errors
/// If the PBO cannot be read
pub fn generate_hashes<I: Seek + Read>(
    pbo: &mut ReadablePbo<I>,
    version: BISignVersion,
    length: u32,
) -> Result<(BoxedUint, BoxedUint, BoxedUint), Error> {
    let mut hasher = Sha1::new();
    let hash1 = pbo.gen_checksum()?;

    hasher.update(hash1.as_bytes());
    hasher.update(pbo.hash_filenames()?);
    if let Some(prefix) = pbo.properties().get("prefix") {
        hasher.update(prefix.as_bytes());
        if !prefix.ends_with('\\') {
            hasher.update(b"\\");
        }
    }
    let hash2 = &*hasher.finalize().to_vec();

    let mut hasher = Sha1::new();
    hasher.update(pbo.hash_files(version)?);
    hasher.update(pbo.hash_filenames()?);
    if let Some(prefix) = pbo.properties().get("prefix") {
        hasher.update(prefix.as_bytes());
        if !prefix.ends_with('\\') {
            hasher.update(b"\\");
        }
    }
    let hash3 = &*hasher.finalize().to_vec();

    Ok((
        pad_hash(hash1.as_bytes(), (length / 8) as usize),
        pad_hash(hash2, (length / 8) as usize),
        pad_hash(hash3, (length / 8) as usize),
    ))
}

#[must_use]
/// Pad a hash to the given size
pub fn pad_hash(hash: &[u8], size: usize) -> BoxedUint {
    let mut vec: Vec<u8> = vec![0, 1];
    vec.resize(size - 36, 255);
    vec.extend(b"\x00\x30\x21\x30\x09\x06\x05\x2b");
    vec.extend(b"\x0e\x03\x02\x1a\x05\x00\x04\x14");
    vec.extend(hash);

    BoxedUint::from_be_slice_vartime(&vec)
}

#[must_use]
pub fn modpow(base: &BoxedUint, exponent: &BoxedUint, modulus: &BoxedUint) -> BoxedUint {
    let n_params = BoxedMontyParams::new(Odd::new(modulus.clone()).unwrap());
    pow_mod_params(base, exponent, &n_params)
}

fn pow_mod_params(base: &BoxedUint, exp: &BoxedUint, n_params: &BoxedMontyParams) -> BoxedUint {
    let base = reduce_vartime(base, n_params);
    base.pow(exp).retrieve()
}

fn reduce_vartime(n: &BoxedUint, p: &BoxedMontyParams) -> BoxedMontyForm {
    let modulus = p.modulus().as_nz_ref().clone();
    let n_reduced = n.rem_vartime(&modulus).resize_unchecked(p.bits_precision());
    BoxedMontyForm::new(n_reduced, p.clone())
}

#[cfg(test)]
mod tests {
    use rsa::BoxedUint;

    #[test]
    fn display_hashes() {
        let bu = &BoxedUint::from_words([
            3_383_022_893_987_068_657,
            211_522_787_039_626_673,
            12_924_607_435_213_790_771,
            4_642_736_248_734_124_677,
            13_049_545_899_981_164_527,
            5_836_844_033_225_426_751,
            18_151_108_490_666_601_265,
            12_542_211_595_622_881_391,
            9_775_904_686_761_608_895,
            9_316_370_910_833_152_348,
            14_627_999_956_071_527_320,
            12_883_383_326_514_718_719,
            15_374_746_912_982_504_272,
            4_911_298_651_162_881_918,
            2_378_468_947_387_679_438,
            13_201_642_397_579_307_866,
        ]);
        let (a, b) = super::display_hashes(bu, bu);
        assert_eq!(a, "1d4a5e3302ef7adaa765c5b12ef2e965e71cfaf1");
        assert_eq!(a, b);
    }
}
