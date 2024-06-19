use hemtt_workspace::reporting::{Code, Diagnostic, Label, Processed};

use crate::Class;

pub struct DuplicateClasses {
    classes: Vec<Class>,
    diagnostic: Option<Diagnostic>,
}

impl Code for DuplicateClasses {
    fn ident(&self) -> &'static str {
        "CE8"
    }

    fn message(&self) -> String {
        "class defined multiple times".to_string()
    }

    fn label_message(&self) -> String {
        "defined multiple times".to_string()
    }

    fn help(&self) -> Option<String> {
        self.classes
            .first()
            .expect("at least one class")
            .name()
            .map(|parent| {
                format!(
                    "remove all but the first definition of `class {};`",
                    parent.as_str(),
                )
            })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl DuplicateClasses {
    pub fn new(classes: Vec<Class>, processed: &Processed) -> Self {
        Self {
            classes,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(name) = self.classes[0].name() else {
            panic!("DuplicateClasses::generate_processed called on class without name");
        };
        self.diagnostic = Diagnostic::new_for_processed(&self, name.span.clone(), processed);
        if let Some(diag) = &mut self.diagnostic {
            for class in self.classes.iter().skip(1) {
                let map = processed
                    .mapping(class.name().expect("class should have name").span.start)
                    .expect("mapping should exist");
                let file = processed.source(map.source()).expect("source should exist");
                diag.labels.push(
                    Label::secondary(
                        file.0.clone(),
                        map.original_start()
                            ..map.original_start()
                                + class.name().expect("class should have name").span.len(),
                    )
                    .with_message("also defined here"),
                );
            }
        }
        self
    }
}
