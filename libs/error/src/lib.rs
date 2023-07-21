use std::fmt::Debug;

pub mod processed;
pub mod tokens;

use lsp_types::Diagnostic;
use processed::Processed;
pub use thiserror;
use vfs::VfsPath;

pub trait Code: Send + Sync {
    fn ident(&self) -> &'static str;
    fn message(&self) -> String;
    fn label_message(&self) -> String;
    fn help(&self) -> Option<String>;
    fn generate_report(&self) -> Option<String> {
        None
    }
    fn generate_processed_report(&self, _processed: &Processed) -> Option<String> {
        None
    }
    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        None
    }
    fn generate_processed_lsp(&self, _processed: &Processed) -> Vec<(VfsPath, Diagnostic)> {
        Vec::new()
    }
}

impl Debug for dyn Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident())
    }
}
