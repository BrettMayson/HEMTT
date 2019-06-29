use std::collections::HashMap;
use std::sync::{Mutex, Arc};

#[macro_use]
pub mod macros;

pub mod build;
pub mod commands;
pub mod error;
pub mod files;
pub mod flow;
pub mod project;
pub mod render;

pub use commands::Command;
pub use build::{Addon, AddonLocation};
pub use error::{HEMTTError, FileErrorLineNumber, IOPathError};
pub use files::{FileCache, RenderedFiles};
pub use flow::{Flow, Report, Task, Step};
pub use project::Project;

lazy_static::lazy_static! {
    static ref CACHED: Arc<Mutex<FileCache>> = Arc::new(Mutex::new(FileCache::new()));
    static ref RENDERED: Arc<Mutex<RenderedFiles>> = Arc::new(Mutex::new(RenderedFiles::new()));
    static ref REPORTS: Arc<Mutex<HashMap<String, Report>>> = Arc::new(Mutex::new(HashMap::new()));
}
