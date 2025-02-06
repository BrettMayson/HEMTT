use hemtt_workspace::reporting::{Code, Diagnostic, Severity};

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableUnusedFile {
    count: u64,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableUnusedFile {
    fn ident(&self) -> &'static str {
        "L-L02U"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("There are {} unused keys in stringtables.", self.count)
    }

    fn note(&self) -> Option<String> {
        Some(String::from(
            "A list has been generated in .hemttout/unused_stringtables.txt",
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableUnusedFile {
    #[must_use]
    pub fn new(count: u64, severity: Severity) -> Self {
        Self {
            count,
            severity,
            diagnostic: None,
        }
        .generate_processed()
    }

    fn generate_processed(mut self) -> Self {
        self.diagnostic = Some(Diagnostic::from_code(&self));
        self
    }
}
