use std::path::PathBuf;

use clap::{ArgMatches, Command};
use hemtt_pbo::ReadablePbo;
use hemtt_signing::{BIPublicKey, BISign};

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("verify")
        .about("Verify a signed PBO")
        .long_about("Check a .bisign file against a public key and PBO")
        .arg(clap::Arg::new("pbo").help("PBO to verify").required(true))
        .arg(
            clap::Arg::new("bikey")
                .help("BIKey to verify against")
                .required(true),
        )
}

/// Execute the verify command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let pbo_path = PathBuf::from(matches.get_one::<String>("pbo").expect("required"));
    let bikey_path = PathBuf::from(matches.get_one::<String>("bikey").expect("required"));

    debug!("Reading PBO: {:?}", &pbo_path);
    let mut pbo = ReadablePbo::from(std::fs::File::open(&pbo_path)?)?;
    debug!("Reading BIKey: {:?}", &bikey_path);
    let publickey = BIPublicKey::read(&mut std::fs::File::open(&bikey_path)?)?;

    let signature_path = {
        let mut pbo_path = pbo_path.clone();
        pbo_path.set_extension(format!("pbo.{}.bisign", publickey.authority()));
        pbo_path
    };
    debug!("Reading Signature: {:?}", &signature_path);
    let signature = BISign::read(&mut std::fs::File::open(&signature_path)?)?;

    println!("Public Key: {:?}", &bikey_path);
    println!("  - Authority: {}", publickey.authority());
    println!("  - Length: {}", publickey.length());
    println!("  - Exponent: {}", publickey.exponent());
    println!("  - Modulus: {}", publickey.modulus_display(17));

    println!();
    println!("PBO: {pbo_path:?}");
    let stored = *pbo.checksum();
    println!("  - Stored Hash:  {stored:?}");
    let actual = pbo.gen_checksum().unwrap();
    println!("  - Actual Hash:  {actual:?}");
    println!("  - Properties");
    for ext in pbo.properties() {
        println!("      - {}: {}", ext.0, ext.1);
    }
    println!("  - Size: {}", pbo_path.metadata()?.len());

    if actual != stored {
        warn!("Verification Warning: PBO has an invalid hash stored");
    }

    if !pbo.properties().contains_key("prefix") {
        println!("Verification Failed: PBO is missing a prefix header");
    } else if stored != actual {
        println!("Verification Warning: PBO reports an invalid hash");
    }

    println!();
    println!("Signature: {signature_path:?}");
    println!("  - Authority: {}", signature.authority());
    println!("  - Version: {}", signature.version());
    println!("  - Length: {}", signature.length());
    println!("  - Exponent: {}", signature.exponent());
    println!("  - Modulus: {}", signature.modulus_display(17));

    match publickey.verify(&mut pbo, &signature) {
        Ok(()) => println!("Verified!"),
        Err(hemtt_signing::Error::AuthorityMismatch { .. }) => {
            error!("Verification Failed: Authority does not match");
        }
        Err(hemtt_signing::Error::HashMismatch { .. }) => {
            error!("Verification Failed: Signature does not match");
        }
        Err(hemtt_signing::Error::UknownBISignVersion(v)) => {
            error!("Verification Failed: Unknown BI Signature Version: {v}");
        }
        Err(hemtt_signing::Error::Io(e)) => {
            error!("Verification Failed: Encountered IO error: {e}");
        }
        Err(hemtt_signing::Error::Pbo(e)) => {
            error!("Verification Failed: Encountered PBO error: {e}");
        }
        Err(hemtt_signing::Error::Rsa(e)) => {
            error!("Verification Failed: Encountered RSA error: {e}");
        }
        Err(hemtt_signing::Error::InvalidLength) => {
            error!("Verification Failed: Invalid length");
        }
        Err(hemtt_signing::Error::AuthorityMissing) => {
            error!("Verification Failed: Missing authority");
        }
        Err(hemtt_signing::Error::InvalidFileSorting) => {
            if pbo.properties().contains_key("Mikero") {
                error!("Verification Failed: Invalid file sorting. This is a bug in Mikero tools.");
            } else {
                error!("Verification Failed: Invalid file sorting");
            }
        }
    }

    Ok(())
}
