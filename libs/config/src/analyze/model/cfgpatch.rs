use hemtt_common::version::Version;

use crate::Ident;

#[derive(Debug, Clone)]
pub struct CfgPatch {
    name: Ident,
    required_version: Version,
}

impl CfgPatch {
    pub const fn new(name: Ident, required_version: Version) -> Self {
        Self {
            name,
            required_version,
        }
    }

    pub const fn name(&self) -> &Ident {
        &self.name
    }

    pub const fn required_version(&self) -> &Version {
        &self.required_version
    }
}
