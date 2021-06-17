use std::path::PathBuf;

use crate::{Addon, HEMTTError};

pub trait Template {
    /// Initialize the project in the provided path
    ///
    /// Arguments:
    /// * `path`: Location to create template
    fn init(&self) -> Result<(), HEMTTError>;

    // Addons
    /// Generate a new addon folder
    ///
    /// Arguments:
    /// * `addon`: Location of the addon
    fn new_addon(&self, addon: &Addon) -> Result<(), HEMTTError>;

    // Functions
    /// Generate a new function file
    ///
    /// Arguments:
    /// * `addon`: Location of the addon
    /// * `name`: function name
    fn new_function(&self, addon: &Addon, name: &str) -> Result<PathBuf, HEMTTError>;
}
