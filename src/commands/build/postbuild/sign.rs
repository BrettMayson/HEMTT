use std::path::Path;

use crate::{Addon, AddonList, HEMTTError, Project, Report, Stage, Task};
use bisign::BIPrivateKey;

#[derive(Clone)]
pub struct Sign {}
impl Task for Sign {
    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project, _: &Stage) -> AddonList {
        create_dir!("keys/")?;
        let key_name = p.get_key_name()?;
        let key = if p.reuse_private_key() {
            warn!("`Reuse Private Key` is enabled. This should be disabled unless you know what you are doing.");
            if Path::new(&format!("keys/{}.biprivatekey", key_name)).exists() {
                BIPrivateKey::read(&mut open_file!(format!("keys/{}.biprivatekey", key_name))?)
                    .expect("Failed to read private key")
            } else {
                // Generate and write the keypair to disk in the current directory
                warn!("Generating a new private key to disk");
                let privatekey = BIPrivateKey::generate(1024, p.get_authority()?);
                privatekey.write(&mut create_file!(format!("keys/{}.biprivatekey", key_name))?)?;
                privatekey
            }
        } else {
            BIPrivateKey::generate(1024, p.get_authority()?)
        };

        let release_folder = p.release_dir()?;

        // Generate a public key to match the private key
        key.to_public_key()
            .write(&mut create_file!(format!("keys/{}.bikey", &key_name))?)?;

        // Copy public key to specific release dir
        copy_file!(format!("keys/{}.bikey", key_name), {
            let mut bikey = release_folder.clone();
            bikey.push("keys");
            bikey.push(format!("{}.bikey", &key_name));
            bikey
        })?;

        for d in &addons {
            let (_, addon) = d.as_ref().unwrap();
            // TODO deal with Result properly
            let sig = bisign::sign(addon.release_target(&release_folder, p), &key, p.get_sig_version()).unwrap();
            let sig_name = p.get_sig_name(&addon.name)?;
            let mut location = addon.release_location(&release_folder, p);
            location.push(sig_name);
            sig.write(&mut create_file!(location)?)?;
        }
        Ok(addons)
    }
}
