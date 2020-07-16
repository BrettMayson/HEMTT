use std::path::PathBuf;

use hemtt::{Addon, HEMTTError};

mod replace;
pub use replace::{replace, Vars};

pub trait Template {
    /// Guess if the files in the provided path are this template
    ///
    /// Arguments:
    /// * `path`: Location to check
    fn detect(path: PathBuf) -> Result<bool, HEMTTError>;

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
