use super::Project;

use crate::HEMTTError;

impl Project {
    /// Should the private key be saved to disk and reused for future versions
    pub fn reuse_private_key(&self) -> bool {
        self.reuse_private_key.is_some() && self.reuse_private_key.unwrap()
    }

    /// Get the name for .bikey & .biprivatekey files
    pub fn get_key_name(&self) -> Result<String, HEMTTError> {
        Ok(if self.key_name.is_empty() {
            if self.reuse_private_key() {
                self.prefix.clone()
            } else if self.prefix.is_empty() {
                self.version()?
            } else {
                format!("{}_{}", &self.prefix, &self.version()?)
            }
        } else {
            self.render_safe(&self.key_name, Some("project:key_name"))?
        })
    }

    /// Get the name for .bisign files
    pub fn get_sig_name(&self, pbo: &str) -> Result<String, HEMTTError> {
        if self.prefix.is_empty() {
            Ok(format!("{}.pbo.{}.bisign", pbo, self.get_authority()?))
        } else {
            Ok(format!("{}_{}.pbo.{}.bisign", &self.prefix, pbo, self.get_authority()?))
        }
    }

    /// Get the signing authority
    pub fn get_authority(&self) -> Result<String, HEMTTError> {
        Ok(if self.authority.is_empty() {
            self.version()?
        } else {
            self.render_safe(&self.authority, Some("project:authority"))?
        })
    }

    fn match_ver(&self, v: u8) -> bisign::BISignVersion {
        match v {
            3 => bisign::BISignVersion::V3,
            2 => bisign::BISignVersion::V2,
            _ => {
                warn!("Invalid Sig Version `{}`", v);
                self.match_ver(crate::project::default_sig_version())
            }
        }
    }

    /// BISignVersion to use for signing
    pub fn get_sig_version(&self) -> bisign::BISignVersion {
        self.match_ver(self.sig_version)
    }
}
