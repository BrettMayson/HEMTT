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
        .report_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
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
                panic!("ParentCase::report_generate_processed called on class without parent");
            };
            let map = processed
                .mapping(
                    self.parent
                        .name()
                        .expect("parent existed to create error")
                        .span
                        .start,
                )
                .unwrap();
            let file = processed.source(map.source()).unwrap();
            diag.labels.push(
                Label::secondary(
                    file.0.clone(),
                    map.original_column()..map.original_column() + parent.span.len(),
                )
                .with_message("parent definition here"),
            );
        }
        self
        // let Some(parent) = self.class.parent() else {
        //     panic!("ParentCase::report_generate_processed called on class without parent");
        // };
        // let map = processed
        //     .mapping(
        //         self.class
        //             .name()
        //             .expect("parent existed to create error")
        //             .span
        //             .start,
        //     )
        //     .unwrap();
        // let token = map.token();
        // let class_parent_map = processed.mapping(parent.span.start).unwrap();
        // let class_parent_token = class_parent_map.token();
        // let parent_map = processed
        //     .mapping(
        //         self.parent
        //             .name()
        //             .expect("parent existed to create error")
        //             .span
        //             .start,
        //     )
        //     .unwrap();
        // let parent_token = parent_map.token();
        // let mut out = Vec::new();
        // let mut colors = ColorGenerator::new();
        // let color_class = colors.next();
        // let color_parent = colors.next();
        // Report::build(
        //     ariadne::ReportKind::Warning,
        //     token.position().path().as_str(),
        //     map.original_column(),
        // )
        // .with_code(self.ident())
        // .with_message(self.message())
        // .with_label(
        //     Label::new((
        //         class_parent_token.position().path().to_string(),
        //         class_parent_token.position().start().0..class_parent_token.position().end().0,
        //     ))
        //     .with_message(self.label_message())
        //     .with_color(color_class),
        // )
        // .with_label(
        //     Label::new((
        //         parent_token.position().path().to_string(),
        //         parent_token.position().start().0..parent_token.position().end().0,
        //     ))
        //     .with_message("parent definition here")
        //     .with_color(color_parent),
        // )
        // .with_help(format!(
        //     "change the {} to match the parent definition `{}`",
        //     "parent case".fg(color_class),
        //     self.parent
        //         .name()
        //         .expect("parent existed to create error")
        //         .as_str()
        //         .fg(color_parent)
        // ))
        // .finish()
        // .write_for_stdout(sources(processed.sources_adrianne()), &mut out)
        // .unwrap();
        // self.diagnostic = Some(String::from_utf8(out).unwrap());
        // self
    }
}
