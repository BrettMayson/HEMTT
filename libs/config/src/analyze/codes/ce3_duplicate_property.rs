use hemtt_common::reporting::{Code, Diagnostic, Label, Processed};

use crate::Ident;

pub struct DuplicateProperty {
    conflicts: Vec<Ident>,
    diagnostic: Option<Diagnostic>,
}

impl Code for DuplicateProperty {
    fn ident(&self) -> &'static str {
        "CE3"
    }

    fn message(&self) -> String {
        "property was defined more than once".to_string()
    }

    fn label_message(&self) -> String {
        "duplicate property".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl DuplicateProperty {
    pub fn new(conflicts: Vec<Ident>, processed: &Processed) -> Self {
        Self {
            conflicts,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(
            &self,
            self.conflicts
                .last()
                .expect("conflicts should have at least one element if it was created with new")
                .span
                .clone(),
            processed,
        );
        if let Some(diag) = &mut self.diagnostic {
            for conflict in self.conflicts.iter().rev().skip(1) {
                let map = processed
                    .mapping(conflict.span.start)
                    .expect("mapping should exist");
                let file = processed.source(map.source()).expect("source should exist");
                diag.labels.push(
                    Label::secondary(
                        file.0.clone(),
                        map.original_start()..map.original_start() + conflict.span.len(),
                    )
                    .with_message("also defined here"),
                );
            }
        }
        self
    }
}
