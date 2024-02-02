use std::ops::Range;

use arma3_wiki::model::Version;
use hemtt_common::{
    reporting::{Code, Diagnostic, Label, Processed},
    workspace::WorkspacePath,
};

pub struct InsufficientRequiredVersion {
    command: String,
    span: Range<usize>,
    version: Version,
    required: (Version, WorkspacePath, Range<usize>),
    stable: Version,

    diagnostic: Option<Diagnostic>,
}

impl Code for InsufficientRequiredVersion {
    fn ident(&self) -> &'static str {
        "SAE1"
    }

    fn message(&self) -> String {
        format!(
            "command `{}` requires version {}",
            self.command, self.version
        )
    }

    fn label_message(&self) -> String {
        format!("requires version {}", self.version)
    }

    fn note(&self) -> Option<String> {
        if self.version > self.stable {
            Some(format!(
                "Current stable version is {}. Using {} will require the development branch.",
                self.stable, self.version
            ))
        } else {
            None
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl InsufficientRequiredVersion {
    #[must_use]
    pub fn new(
        command: String,
        span: Range<usize>,
        version: Version,
        required: (Version, WorkspacePath, Range<usize>),
        stable: Version,
        processed: &Processed,
    ) -> Self {
        Self {
            command,
            span,
            version,
            required,
            stable,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(diag) = Diagnostic::new_for_processed(&self, self.span.clone(), processed) else {
            return self;
        };
        self.diagnostic = Some(diag.with_label(
            Label::secondary(self.required.1.clone(), self.required.2.clone()).with_message(
                format!("CfgPatch only requires version {}", self.required.0),
            ),
        ));
        self
    }
}
