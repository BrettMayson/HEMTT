use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Code, Processed};

use crate::Class;

pub struct MissingParrent {
    class: Class,
}

impl MissingParrent {
    pub const fn new(class: Class) -> Self {
        Self { class }
    }
}

// TODO: maybe we could have a `did you mean` here without too much trouble?

impl Code for MissingParrent {
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

    fn generate_processed_report(&self, processed: &Processed) -> Option<String> {
        let parent = self.class.parent()?;
        let map = processed.mapping(self.class.name().span.start).unwrap();
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
        Some(String::from_utf8(out).unwrap())
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {}
}
