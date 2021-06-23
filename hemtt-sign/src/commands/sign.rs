use std::convert::TryInto;
use std::fs::File;
use std::path::PathBuf;

use super::Command;
use crate::{BIPrivateKey, BISignError};

pub struct Sign {}
impl Command for Sign {
    fn register(&self) -> clap::App {
        clap::App::new("sign")
            .arg(
                clap::Arg::with_name("private")
                    .help("Private key to sign with")
                    .required(true),
            )
            .arg(
                clap::Arg::with_name("file")
                    .help("PBO file to sign")
                    .required(true),
            )
            .arg(
                clap::Arg::with_name("out")
                    .help("Output location of signature")
                    .short("o")
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name("version")
                    .help("BISignVersion")
                    .default_value("3")
                    .possible_values(&["2", "3"])
                    .short("v"),
            )
    }

    fn run(&self, args: &clap::ArgMatches) -> Result<(), BISignError> {
        let pbo_path = PathBuf::from(args.value_of("file").unwrap());
        let private_key = BIPrivateKey::read(
            &mut File::open(args.value_of("private").unwrap()).expect("Failed to open private key"),
        )
        .expect("Failed to read private key");
        let sig_path = match args.value_of("out") {
            Some(sig) => PathBuf::from(sig),
            None => {
                let mut pbo_path = pbo_path.clone();
                pbo_path.set_extension(format!("pbo.{}.bisign", private_key.authority));
                pbo_path
            }
        };
        let sig = crate::sign(
            pbo_path,
            &private_key,
            args.value_of("version")
                .unwrap()
                .parse::<u32>()
                .unwrap()
                .try_into()
                .unwrap(),
        )?;
        sig.write(&mut File::create(&sig_path).expect("Failed to open signature file"))
            .expect("Failed to write signature");
        Ok(())
    }
}
