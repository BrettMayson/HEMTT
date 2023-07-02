pub mod processed;
pub mod tokens;

use processed::Processed;
pub use thiserror;

pub trait Code {
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
}
