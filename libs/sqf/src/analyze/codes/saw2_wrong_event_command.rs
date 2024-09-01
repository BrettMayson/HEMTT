use std::{ops::Range, sync::Arc};

use arma3_wiki::model::EventHandlerNamespace;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::parser::database::Database;

pub struct WrongEventCommand {
    span: Range<usize>,
    command: String,
    id: Arc<str>,
    target: Option<(String, bool)>,

    alternatives: Vec<(String, bool)>,

    diagnostic: Option<Diagnostic>,
}

impl Code for WrongEventCommand {
    fn ident(&self) -> &'static str {
        "SAW2"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn message(&self) -> String {
        format!(
            "Event `{}` was not expected for command `{}`",
            self.id, self.command
        )
    }

    fn label_message(&self) -> String {
        format!("not supported by command `{}`", self.command)
    }

    fn suggestion(&self) -> Option<String> {
        if self.alternatives.len() == 1 {
            if self.alternatives[0].1 {
                if let Some((target, _)) = &self.target {
                    Some(format!(
                        "{} {} [\"{}\", {{ …",
                        target, self.alternatives[0].0, self.id
                    ))
                } else {
                    Some(format!(
                        "{{target}} {} [\"{}\", {{ …",
                        self.alternatives[0].0, self.id
                    ))
                }
            } else {
                Some(self.alternatives[0].0.clone())
            }
        } else {
            None
        }
    }

    fn help(&self) -> Option<String> {
        if self.alternatives.is_empty() {
            None
        } else {
            Some(format!(
                "Did you mean: `{}`?",
                self.alternatives
                    .iter()
                    .map(|(a, _)| a.as_str())
                    .collect::<Vec<_>>()
                    .join("`, `")
            ))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl WrongEventCommand {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        command: String,
        id: Arc<str>,
        target: Option<(String, bool)>,
        namespaces: Vec<EventHandlerNamespace>,
        processed: &Processed,
        database: &Database,
    ) -> Self {
        let prefix = command.chars().take(3).collect::<String>();
        Self {
            span,
            command,
            id,
            target,
            alternatives: {
                let mut alternatives = Vec::new();
                for ns in namespaces {
                    println!("Possible alternatives: {:?}", ns.commands());
                    ns.commands()
                        .iter()
                        .filter(|c| c.contains(&prefix))
                        .for_each(|c| {
                            alternatives.push(((*c).to_string(), {
                                database.wiki().commands().get(c).map_or(false, |c| {
                                    c.syntax().first().map_or(false, |s| s.call().is_binary())
                                })
                            }));
                        });
                }
                alternatives.sort();
                alternatives.dedup();
                alternatives
            },
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
