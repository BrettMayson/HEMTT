use std::fs::File;
use std::path::PathBuf;

use super::Command;
use crate::{BIPrivateKey, BISignError};

pub struct Keygen {}
impl Command for Keygen {
    fn register(&self) -> clap::App {
        clap::App::new("keygen").arg(
            clap::Arg::with_name("keyname")
                .help("name of the key")
                .required(true),
        )
    }

    fn run(&self, args: &clap::ArgMatches) -> Result<(), BISignError> {
        let keyname = PathBuf::from(args.value_of("keyname").unwrap());

        let private_key =
            BIPrivateKey::generate(1024, keyname.file_name().unwrap().to_str().unwrap());
        let public_key = private_key.to_public_key();
        let name = keyname.file_name().unwrap().to_str().unwrap();

        let mut private_key_path = keyname.clone();
        private_key_path.set_file_name(format!("{}.biprivatekey", name));
        private_key
            .write(&mut File::create(private_key_path).unwrap())
            .expect("Failed to write private key");

        let mut public_key_path = keyname.clone();
        public_key_path.set_file_name(format!("{}.bikey", name));
        public_key
            .write(&mut File::create(public_key_path).unwrap())
            .expect("Failed to write public key");

        Ok(())
    }
}
