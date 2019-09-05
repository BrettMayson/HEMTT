use super::Project;

use crate::HEMTTError;

impl Project {
    /// Should the private key be saved to disk and reused for future versions
    pub fn reuse_private_key(&self) -> bool {
        self.reuse_private_key.is_some() && self.reuse_private_key.unwrap()
    }

    pub fn get_key_name(&self) -> Result<String, HEMTTError> {
        Ok(if self.keyname.is_empty() {
            if self.reuse_private_key() {
                self.prefix.clone()
            } else if self.prefix.is_empty() {
                self.version()?
            } else {
                format!("{}_{}", &self.prefix, &self.version()?)
            }
        } else {
            self.render(&self.keyname)?
        })
    }

    pub fn get_sig_name(&self, pbo: &str) -> Result<String, HEMTTError> {
        Ok(if self.sig_name.is_empty() {
            format!("{}.{}.bisign", pbo, self.version()?)
        } else {
            format!("{}.{}.bisign", pbo, self.render(&self.sig_name)?)
        })
    }

    fn match_ver(&self, v: u8) -> armake2::sign::BISignVersion {
        match v {
            3 => armake2::sign::BISignVersion::V3,
            2 => armake2::sign::BISignVersion::V2,
            _ => {
                warn!(format!("Invalid Sig Version `{}`", v));
                self.match_ver(crate::project::default_sig_version())
            }
        }
    }

    /// BISignVersion to use for signing
    pub fn get_sig_version(&self) -> armake2::sign::BISignVersion {
        self.match_ver(self.sig_version)
    }
}
