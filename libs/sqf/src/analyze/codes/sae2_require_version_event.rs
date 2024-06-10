use std::ops::Range;

use arma3_wiki::model::Version;
use hemtt_workspace::{
    reporting::{Code, Diagnostic, Label, Processed},
    WorkspacePath,
};

pub struct InsufficientRequiredVersionEvent {
    event: String,
    span: Range<usize>,
    version: Version,
    required: (Option<Version>, WorkspacePath, Range<usize>),
    stable: Version,

    diagnostic: Option<Diagnostic>,
}

impl Code for InsufficientRequiredVersionEvent {
    fn ident(&self) -> &'static str {
        "SAE1"
    }

    fn message(&self) -> String {
        format!("event `{}` requires version {}", self.event, self.version)
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

impl InsufficientRequiredVersionEvent {
    #[must_use]
    pub fn new(
        event: String,
        span: Range<usize>,
        version: Version,
        required: (Version, WorkspacePath, Range<usize>),
        stable: Version,
        processed: &Processed,
    ) -> Self {
        Self {
            event,
            span,
            version,
            required: {
                if required.0.major() == 0 && required.0.minor() == 0 {
                    (None, required.1, required.2)
                } else {
                    (Some(required.0), required.1, required.2)
                }
            },
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
                self.required.0.map_or_else(
                    || "CfgPatch doesn't specify `requiredVersion`".to_string(),
                    |required| format!("CfgPatch requires version {required}"),
                ),
            ),
        ));
        self
    }
}
