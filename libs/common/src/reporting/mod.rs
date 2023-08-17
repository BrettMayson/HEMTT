//! Reporting module

use std::fmt::Debug;

/// A coded error or warning
pub trait Code: Send + Sync {
    /// The code identifier
    fn ident(&self) -> &'static str;
    /// Message explaining the error
    fn message(&self) -> String;
    /// Message explaining the error, applied to the label
    fn label_message(&self) -> String;
    /// Help message, if any
    fn help(&self) -> Option<String>;
    /// A report for the CLI
    fn generate_report(&self) -> Option<String> {
        None
    }
    /// A report for the CLI, applied to the processed file
    fn generate_processed_report(&self, _processed: &str) -> Option<String> {
        None
    }
    #[cfg(feature = "lsp")]
    /// Generate a diagnostic for the LSP
    fn lsp_generate(&self) -> Option<(VfsPath, Diagnostic)> {
        None
    }
    #[cfg(feature = "lsp")]
    /// Generate a diagnostic for the LSP, applied to the processed file
    fn lsp_generate_processed(&self, _processed: &Processed) -> Vec<(VfsPath, Diagnostic)> {
        Vec::new()
    }
    #[cfg(feature = "lsp")]
    /// Helper to generate a diagnostic for the LSP
    fn lsp_diagnostic(&self, range: Range) -> Diagnostic {
        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(lsp_types::NumberOrString::String(self.ident().to_string())),
            code_description: None,
            source: Some(String::from("HEMTT")),
            message: self.label_message(),
            related_information: None,
            tags: None,
            data: None,
        }
    }
}

impl Debug for dyn Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident())
    }
}
