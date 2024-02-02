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
            self.conflicts.last().unwrap().span.clone(),
            processed,
        );
        if let Some(diag) = &mut self.diagnostic {
            for conflict in self.conflicts.iter().rev().skip(1) {
                let map = processed.mapping(conflict.span.start).unwrap();
                let file = processed.source(map.source()).unwrap();
                diag.labels.push(
                    Label::secondary(
                        file.0.clone(),
                        map.original_column()..map.original_column() + conflict.span.len(),
                    )
                    .with_message("also defined here"),
                );
            }
        }
        self
    }
}
