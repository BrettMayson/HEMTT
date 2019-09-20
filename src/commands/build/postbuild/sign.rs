use std::path::Path;

use crate::{Addon, AddonList, HEMTTError, Project, Report, Stage, Task};
use armake2::{BIPrivateKey, PBO};

#[derive(Clone)]
pub struct Sign {}
impl Task for Sign {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project, _: &Stage) -> AddonList {
        create_dir!("keys/")?;
        let keyname = p.get_key_name()?;
        let key = if p.reuse_private_key() {
            warn!("`Reuse Private Key` is enabled. This should be disabled unless you know what you are doing.");
            if Path::new(&format!("keys/{}.biprivatekey", keyname)).exists() {
                BIPrivateKey::read(&mut open_file!(format!("keys/{}.biprivatekey", keyname))?)
                    .expect("Failed to read private key")
            } else {
                // Generate and write the keypair to disk in the current directory
                let privatekey = BIPrivateKey::generate(1024, keyname.clone());
                privatekey.write(&mut create_file!(format!("keys/{}.biprivatekey", keyname))?)?;
                privatekey
            }
        } else {
            BIPrivateKey::generate(1024, keyname.clone())
        };

        let release_folder = p.release_dir()?;

        // Generate a public key to match the private key
        key.to_public_key()
            .write(&mut create_file!(format!("keys/{}.bikey", &keyname))?)?;

        // Copy public key to specific release dir
        copy_file!(format!("keys/{}.bikey", keyname), {
            let mut bikey = release_folder.clone();
            bikey.push("keys");
            bikey.push(format!("{}.bikey", &keyname));
            bikey
        })?;

        for d in &addons {
            let (_, addon) = d.as_ref().unwrap();
            let pbo = PBO::read(&mut open_file!(addon.release_target(&release_folder, p))?)?;
            let sig = key.sign(&pbo, p.get_sig_version());
            let sig_name = p.get_sig_name(&addon.name)?;
            let mut location = addon.release_location(&release_folder);
            location.push(sig_name);
            sig.write(&mut create_file!(location)?)?;
        }
        Ok(addons)
    }
}
