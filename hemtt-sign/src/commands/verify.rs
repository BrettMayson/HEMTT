use std::path::PathBuf;
use std::{fs::File, io::Cursor};

use super::Command;
use crate::{BIPublicKey, BISign, BISignError};

use hemtt_pbo::sync::{ReadablePbo, WritablePbo};

pub struct Verify {}
impl Command for Verify {
    fn register(&self) -> clap::App {
        clap::App::new("verify")
            .arg(
                clap::Arg::with_name("public")
                    .help("Public key to verify with")
                    .required(true),
            )
            .arg(
                clap::Arg::with_name("file")
                    .help("PBO file to verify")
                    .required(true),
            )
            .arg(
                clap::Arg::with_name("signature")
                    .help("Signature to verify against")
                    .short("s")
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name("hashes")
                    .help("Display full hash stages")
                    .short("e")
                    .takes_value(false),
            )
    }

    fn run(&self, args: &clap::ArgMatches) -> Result<(), BISignError> {
        let mut publickey_file =
            File::open(&args.value_of("public").unwrap()).expect("Failed to open public key");
        let publickey = BIPublicKey::read(&mut publickey_file).expect("Failed to read public key");

        let pbo_path = args.value_of("file").unwrap();
        let mut pbo_file = File::open(&pbo_path).expect("Failed to open PBO");
        let pbo_size = pbo_file.metadata().unwrap().len();
        let mut pbo = ReadablePbo::from(&mut pbo_file).expect("Failed to read PBO");

        let sig_path = match args.value_of("signature") {
            Some(path) => PathBuf::from(path),
            None => {
                let mut path = PathBuf::from(pbo_path);
                path.set_extension(format!("pbo.{}.bisign", publickey.authority));
                path
            }
        };

        let sig = BISign::read(&mut File::open(&sig_path).expect("Failed to open signature"))
            .expect("Failed to read signature");

        println!();
        println!("Public Key: {:?}", &args.value_of("public").unwrap());
        println!("\tAuthority: {}", publickey.authority);
        println!("\tLength: {}", publickey.length);
        println!("\tExponent: {}", publickey.exponent);

        println!();
        println!("PBO: {:?}", pbo_path);
        let stored = pbo.checksum();
        let actual = pbo.gen_checksum().unwrap();
        println!("\tStored Hash:  {:?}", stored);
        let sorted = pbo.is_sorted();
        if let Err((_, files_sorted)) = sorted {
            println!("\tInvalid Hash: {:?}", actual);
            let mut new_pbo = WritablePbo::<Cursor<Vec<u8>>>::new();
            for f in files_sorted {
                new_pbo
                    .add_file_header(
                        f.filename(),
                        pbo.retrieve(f.filename()).unwrap(),
                        f.to_owned(),
                    )
                    .unwrap();
            }
            println!("\tActual Hash:  {:?}", new_pbo.checksum().unwrap());
        } else {
            println!("\tActual Hash:  {:?}", actual);
        }
        println!("\tExtensions");
        for ext in pbo.extensions() {
            println!("\t\t{}: {}", ext.0, ext.1);
        }
        println!("\tFiles");
        for ext in pbo.files() {
            println!("\t\t{}: {}", ext.filename(), ext.size());
        }
        println!("\tSize: {}", pbo_size);
        if args.is_present("hashes") {
            println!("\tHash Stages");
            let (h1, h2, h3) =
                crate::types::generate_hashes(&mut pbo, sig.version, publickey.length);
            println!("\t\t{:?}", h1);
            println!("\t\t{:?}", h2);
            println!("\t\t{:?}", h3);
        }

        if !pbo.extensions().contains_key("prefix") {
            println!("Verification Failed: PBO is missing a prefix header")
        } else if stored != actual {
            println!("Verification Warning: PBO reports an invalid hash");
        }

        println!();
        println!("Signature: {:?}", sig_path);
        println!("\tAuthority: {}", sig.authority);
        println!("\tVersion: {}", sig.version.to_string());
        println!("\tLength: {}", sig.length);
        println!("\tExponent: {}", sig.exponent);
        if args.is_present("hashes") {
            println!("\tHash Stages");
            let (signed_hash1, signed_hash2, signed_hash3) = publickey.get_hashes(&sig);
            println!("\t\t{:?}", signed_hash1);
            println!("\t\t{:?}", signed_hash2);
            println!("\t\t{:?}", signed_hash3);
        }

        println!();

        match publickey.verify(&mut pbo, &sig) {
            Ok(()) => println!("Verified!"),
            Err(BISignError::AuthorityMismatch { .. }) => {
                println!("Verification Failed: Authority does not match");
            }
            Err(BISignError::HashMismatch { .. }) => {
                println!("Verification Failed: Signature does not match");
            }
            Err(BISignError::UknownBISignVersion(v)) => {
                println!("Verification Failed: Unknown BI Signature Version: {}", v);
            }
            Err(BISignError::IOError(e)) => {
                println!("Verification Failed: Encountered IO error: {}", e);
            }
            Err(BISignError::InvalidFileSorting) => {
                if pbo.extension("Mikero").is_some() {
                    println!(
                        "Verification Failed: Invalid file sorting. This is a bug in Mikero tools."
                    );
                } else {
                    println!("Verification Failed: Invalid file sorting");
                }
            }
        }

        Ok(())
    }
}
