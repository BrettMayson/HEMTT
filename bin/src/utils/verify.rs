use std::path::PathBuf;

use hemtt_pbo::ReadablePbo;

use crate::{
    Error,
    utils::inspect::{bikey, bisign},
};

#[derive(clap::Parser)]
#[command(arg_required_else_help = true, verbatim_doc_comment)]
/// Verify a signed PBO against a public key
///
/// Validates that a PBO file has been properly signed and can be trusted.
///
/// ## Verification Checks
///
/// - **Authority matches**: Signature authority matches the public key
/// - **PBO correctly sorted**: Files are in the correct order
/// - **Hashes match**: PBO content hasn't been tampered with
/// - **Prefix present**: PBO has required prefix property
///
/// Returns a success message if all checks pass, or specific error details if verification fails.
///
/// ## Usage
///
/// ```bash
/// hemtt utils verify my_addon.pbo my_key.bikey
/// ```
pub struct Command {
    /// Path to the PBO to check. The corresponding .bisign file must be present.
    pbo: String,
    /// Path to the public key (.bikey) used to verify the signature.
    bikey: String,
}

/// Execute the verify command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<(), Error> {
    let pbo_path = PathBuf::from(&cmd.pbo);
    let bikey_path = PathBuf::from(&cmd.bikey);

    debug!("Reading PBO: {:?}", &pbo_path);
    let mut pbo = ReadablePbo::from(fs_err::File::open(&pbo_path)?)?;
    debug!("Reading BIKey: {:?}", &bikey_path);
    let publickey = bikey(std::fs::File::open(&bikey_path)?, &bikey_path)?;

    let signature_path = {
        let mut pbo_path = pbo_path.clone();
        pbo_path.set_extension(format!("pbo.{}.bisign", publickey.authority()));
        pbo_path
    };
    debug!("Reading Signature: {:?}", &signature_path);
    println!();
    let signature = bisign(std::fs::File::open(&signature_path)?, &signature_path)?;

    println!();
    println!("PBO: {}", pbo_path.display());
    let stored = *pbo.checksum();
    println!("  - Stored SHA1 Hash:  {}", stored.hex());
    let actual = pbo.gen_checksum()?;
    println!("  - Actual SHA1 Hash:  {}", actual.hex());
    println!("  - Properties");
    for ext in pbo.properties() {
        println!("      - {}: {}", ext.0, ext.1);
    }
    println!("  - Size: {}", fs_err::metadata(&pbo_path)?.len());

    if actual != stored {
        warn!("Verification Warning: PBO has an invalid hash stored");
    }

    if !pbo.properties().contains_key("prefix") {
        println!("Verification Failed: PBO is missing a prefix header");
    } else if stored != actual {
        println!("Verification Warning: PBO reports an invalid hash");
    }

    match publickey.verify(&mut pbo, &signature) {
        Ok(()) => println!("Verified!"),
        Err(hemtt_signing::Error::AuthorityMismatch { .. }) => {
            error!("Verification Failed: Authority does not match");
        }
        Err(hemtt_signing::Error::HashMismatch { .. }) => {
            error!("Verification Failed: Signature does not match");
        }
        Err(hemtt_signing::Error::InvalidMagic) => {
            error!("Verification Failed: Invalid private key format");
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
