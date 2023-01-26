#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]

//! HEMTT - Arma 3 Signing

// Parts of the following code is derivative work of the code from the armake2 project by KoffeinFlummi, 
// which is licensed GPLv2. This code therefore is also licensed under the terms 
// of the GNU Public License, verison 2.

// The original code can be found here:
// https://github.com/KoffeinFlummi/armake2/blob/4b736afc8c615cf49a0d1adce8f6b9a8ae31d90f/src/sign.rs

use std::io::Write;

use rsa::BigUint;

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
