use hemtt_common::reporting::{Code, Diagnostic, Label, Processed};

use crate::Class;

pub struct ParentCase {
    class: Class,
    parent: Class,

    diagnostic: Option<Diagnostic>,
}

// TODO: maybe we could have a `did you mean` here without too much trouble?

impl Code for ParentCase {
    fn ident(&self) -> &'static str {
        "CW1"
    }

    fn message(&self) -> String {
        "parent case does not match parent definition".to_string()
    }

    fn label_message(&self) -> String {
        "parent does not match definition case".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("change the parent case to match the parent definition".to_string())
    }

    fn suggestion(&self) -> Option<String> {
        Some(
            self.parent
                .name()
                .expect("parent existed to create error")
                .as_str()
                .to_string(),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl ParentCase {
    pub fn new(class: Class, parent: Class, processed: &Processed) -> Self {
        Self {
            class,
            parent,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(
            &self,
            self.class
                .parent()
                .expect("parent existed to create error")
                .span
                .clone(),
            processed,
        );
        if let Some(diag) = &mut self.diagnostic {
            let Some(parent) = self.class.parent() else {
                panic!("ParentCase::generate_processed called on class without parent");
            };
            let map = processed
                .mapping(
                    self.parent
                        .name()
                        .expect("parent existed to create error")
                        .span
                        .start,
                )
                .expect("mapping should exist");
            let file = processed.source(map.source()).expect("source should exist");
            diag.labels.push(
                Label::secondary(
                    file.0.clone(),
                    map.original_start()..map.original_start() + parent.span.len(),
                )
                .with_message("parent definition here"),
            );
        }
        self
    }
}
