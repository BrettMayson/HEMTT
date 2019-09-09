use std::path::{Path, PathBuf};

use crate::{Addon, AddonList, HEMTTError, Project, Report, Task};
use armake2::{pbo::PBO, sign::BIPrivateKey};

#[derive(Clone)]
pub struct Sign {}
impl Task for Sign {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project) -> AddonList {
        let keyname = p.get_key_name()?;
        let key = if p.reuse_private_key() {
            warn!("`Reuse Private Key` is enabled. This should be disabled unless you know what you are doing.");
            if !Path::new(&format!("keys/{}.bikey", keyname)).exists() {
                // Generate and write the keypair to disk in the current directory
                armake2::sign::cmd_keygen(PathBuf::from(&keyname))?;
                rename_file!(format!("{}.bikey", keyname), format!("keys/{}.bikey", keyname))?;
                rename_file!(format!("{}.biprivatekey", keyname), format!("keys/{}.biprivatekey", keyname))?;
            }

            BIPrivateKey::read(&mut open_file!(format!("keys/{}.biprivatekey", keyname))?)
                .expect("Failed to read private key")
        } else {
            BIPrivateKey::generate(1024, keyname)
        };

        let release_folder = p.release_dir()?;

        for d in &addons {
            let (_, addon) = d.as_ref().unwrap();
            let release_target = addon.release_target(&release_folder, p);
            let pbo = PBO::read(&mut open_file!(release_target)?)?;
            let sig = key.sign(&pbo, p.get_sig_version());
            sig.write(&mut create_file!(format!("{}.bisign", release_target.to_str().unwrap()))?)?;
        }
        Ok(addons)
    }
}
