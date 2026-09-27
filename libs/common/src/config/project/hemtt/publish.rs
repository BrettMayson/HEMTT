use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for `hemtt release`
pub struct PublishOptions {
    description: Option<PathBuf>,
    changelog: Option<PathBuf>,
}

impl PublishOptions {
    /// Get the description source
    pub const fn description(&self) -> Option<&PathBuf> {
        self.description.as_ref()
    }

    /// Get the changelog source
    pub const fn changelog(&self) -> Option<&PathBuf> {
        self.changelog.as_ref()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Release specific configuration
pub struct PublishOptionsFile {
    #[serde(default)]
    description: Option<String>,

    #[serde(default)]
    changelog: Option<String>,
}

impl PublishOptionsFile {
    pub fn into_config(self) -> Result<PublishOptions, Error> {
        Ok(PublishOptions {
            description: {
                let path = self.description.map(PathBuf::from);
                if let Some(ref p) = path
                    && !p.exists()
                {
                    return Err(Error::ConfigInvalid(format!(
                        "[hemtt.publish] description path does not exist: {}",
                        p.display()
                    )));
                }
                path
            },
            changelog: {
                let path = self.changelog.map(PathBuf::from);
                if let Some(ref p) = path
                    && !p.exists()
                {
                    return Err(Error::ConfigInvalid(format!(
                        "[hemtt.publish] changelog path does not exist: {}",
                        p.display()
                    )));
                }
                path
            },
        })
    }
}
