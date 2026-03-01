use chumsky::span::Spanned;
use hemtt_common::version::Version;

use crate::Ident;

#[derive(Debug, Clone)]
pub struct CfgPatch {
    name: Spanned<Ident>,
    required_version: Version,
}

impl CfgPatch {
    #[must_use]
    pub const fn new(name: Spanned<Ident>, required_version: Version) -> Self {
        Self {
            name,
            required_version,
        }
    }

    #[must_use]
    pub const fn name(&self) -> &Spanned<Ident> {
        &self.name
    }

    #[must_use]
    pub const fn required_version(&self) -> &Version {
        &self.required_version
    }
}
