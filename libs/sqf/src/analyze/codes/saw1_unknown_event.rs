use std::{ops::Range, sync::Arc};

use arma3_wiki::model::EventHandlerNamespace;
use hemtt_common::similar_values;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::parser::database::Database;

pub struct UnknownEvent {
    span: Range<usize>,
    command: String,
    id: Arc<str>,

    similar: Vec<String>,

    diagnostic: Option<Diagnostic>,
}

impl Code for UnknownEvent {
    fn ident(&self) -> &'static str {
        "SAW1"
    }

    fn severity(&self) -> Severity {
        if self.id.to_lowercase() == "damaged" {
            Severity::Error
        } else {
            Severity::Warning
        }
    }

    fn message(&self) -> String {
        format!("Using `{}` with unknown event `{}`", self.command, self.id)
    }

    fn label_message(&self) -> String {
        format!("unknown event `{}`", self.id)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("Did you mean: `{}`?", self.similar.join("`, `")))
        }
    }

    // fn suggestion(&self) -> Option<String> {
    //     Some(format!("\"{}\"", self.constant.0))
    // }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone().map(|d| {
            if self.id.to_lowercase() == "damaged" {
                d.with_help("Damaged is a common typo for `Dammaged`. An error has been raised to prevent accidental usage.")
            } else {
                d
            }
        })
    }
}

impl UnknownEvent {
    #[must_use]
    pub fn new(
        nss: &[EventHandlerNamespace],
        span: Range<usize>,
        command: String,
        id: Arc<str>,
        processed: &Processed,
        database: &Database,
    ) -> Self {
        Self {
            span,
            command,

            similar: {
                let mut haystack = Vec::new();
                for (dns, ehs) in database.wiki().event_handlers() {
                    if !nss.contains(dns) {
                        continue;
                    }
                    for eh in ehs {
                        haystack.push(eh.id());
                    }
                }
                let mut similar: Vec<String> = similar_values(&id, &haystack)
                    .into_iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                similar.sort();
                similar.dedup();
                similar
            },

            id,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
