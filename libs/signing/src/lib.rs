#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]

//! HEMTT - Arma 3 Signing

// Parts of the following code is derivative work of the code from the armake2 project by KoffeinFlummi,
// which is licensed GPLv2. This code therefore is also licensed under the terms
// of the GNU Public License, verison 2.

// The original code can be found here:
// https://github.com/KoffeinFlummi/armake2/blob/4b736afc8c615cf49a0d1adce8f6b9a8ae31d90f/src/sign.rs

use std::io::{Read, Seek, Write};

use hemtt_common::BISignVersion;
use hemtt_pbo::ReadablePbo;
use rsa::BigUint;
use sha1::{Digest, Sha1};

mod error;
mod private;
mod public;
mod signature;

pub use error::Error;
pub use private::BIPrivateKey;
pub use public::BIPublicKey;
pub use signature::BISign;

/// Writes a [`BigUint`] to the given output.
///
/// # Errors
/// If the output fails to write.
pub fn write_biguint<O: Write>(output: &mut O, bn: &BigUint, size: usize) -> Result<(), Error> {
    let mut vec: Vec<u8> = bn.to_bytes_le();
    vec.resize(size, 0);
    output.write_all(&vec).map_err(std::convert::Into::into)
}

fn display_hashes(a: &BigUint, b: &BigUint) -> (String, String) {
    let hex_a = a.to_str_radix(16).to_lowercase();
    let hex_b = b.to_str_radix(16).to_lowercase();

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
) -> Result<(BigUint, BigUint, BigUint), Error> {
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
pub fn pad_hash(hash: &[u8], size: usize) -> BigUint {
    let mut vec: Vec<u8> = vec![0, 1];
    vec.resize(size - 36, 255);
    vec.extend(b"\x00\x30\x21\x30\x09\x06\x05\x2b");
    vec.extend(b"\x0e\x03\x02\x1a\x05\x00\x04\x14");
    vec.extend(hash);

    BigUint::from_bytes_be(&vec)
}
