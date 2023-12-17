use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

use crate::Class;

pub struct ParentCase {
    class: Class,
    parent: Class,

    report: Option<String>,
    annotations: Vec<Annotation>,
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
        Some(format!(
            "change the parent case to match the parent definition: `{}`",
            self.parent
                .name()
                .expect("parent existed to create error")
                .as_str()
        ))
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

impl ParentCase {
    pub fn new(class: Class, parent: Class, processed: &Processed) -> Self {
        Self {
            class,
            parent,

            report: None,
            annotations: vec![],
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let Some(parent) = self.class.parent() else {
            panic!("ParentCase::report_generate_processed called on class without parent");
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
        let class_parent_map = processed.mapping(parent.span.start).unwrap();
        let class_parent_token = class_parent_map.token();
        let parent_map = processed
            .mapping(
                self.parent
                    .name()
                    .expect("parent existed to create error")
                    .span
                    .start,
            )
            .unwrap();
        let parent_token = parent_map.token();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let color_class = colors.next();
        let color_parent = colors.next();
        Report::build(
            ariadne::ReportKind::Warning,
            token.position().path().as_str(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                class_parent_token.position().path().to_string(),
                class_parent_token.position().start().0..class_parent_token.position().end().0,
            ))
            .with_message(self.label_message())
            .with_color(color_class),
        )
        .with_label(
            Label::new((
                parent_token.position().path().to_string(),
                parent_token.position().start().0..parent_token.position().end().0,
            ))
            .with_message("parent definition here")
            .with_color(color_parent),
        )
        .with_help(format!(
            "change the {} to match the parent definition `{}`",
            "parent case".fg(color_class),
            self.parent
                .name()
                .expect("parent existed to create error")
                .as_str()
                .fg(color_parent)
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
            AnnotationLevel::Warning,
            map_file.0.as_str().to_string(),
            map.original(),
        )];
        self
    }
}
