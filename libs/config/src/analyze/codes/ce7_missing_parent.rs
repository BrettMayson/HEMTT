use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

use crate::Class;

pub struct MissingParent {
    class: Class,
    report: Option<String>,
    annotations: Vec<Annotation>,
}

// TODO: maybe we could have a `did you mean` here without too much trouble?

impl Code for MissingParent {
    fn ident(&self) -> &'static str {
        "CE7"
    }

    fn message(&self) -> String {
        "class's parent is not present".to_string()
    }

    fn label_message(&self) -> String {
        "not present in config".to_string()
    }

    fn help(&self) -> Option<String> {
        self.class.parent().map(|parent| {
            format!(
                "add `class {};` to the config to declare it as external",
                parent.as_str(),
            )
        })
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {}
}

impl MissingParent {
    pub fn new(class: Class, processed: &Processed) -> Self {
        Self {
            class,
            report: None,
            annotations: vec![],
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let Some(parent) = self.class.parent() else {
            panic!("MissingParent::report_generate_processed called on class without parent");
        };
        let map = processed
            .mapping(
                self.class
                    .name()
                    .expect("parent existed to create error")
                    .span
                    .start,
            )
            .unwrap();
        let token = map.token();
        let parent_map = processed.mapping(parent.span.start).unwrap();
        let parent_token = parent_map.token();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            token.position().path().as_str(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                parent_token.position().path().to_string(),
                parent_token.position().start().0..parent_token.position().end().0,
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_help(format!(
            "add `class {};` to the config to declare it as external",
            parent.as_str().fg(a),
        ))
        .finish()
        .write_for_stdout(sources(processed.sources_adrianne()), &mut out)
        .unwrap();
        self.report = Some(String::from_utf8(out).unwrap());
        self
    }

    fn ci_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed
            .mapping(self.class.parent().unwrap().span.start)
            .unwrap();
        let map_file = processed.source(map.source()).unwrap();
        self.annotations = vec![self.annotation(
            AnnotationLevel::Error,
            map_file.0.as_str().to_string(),
            map.original(),
        )];
        self
    }
}
