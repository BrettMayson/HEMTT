use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Code, Processed};

use crate::Class;

pub struct ParentCase {
    class: Class,
    parent: Class,
}

impl ParentCase {
    pub const fn new(class: Class, parent: Class) -> Self {
        Self { class, parent }
    }
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
        "class's parent does not match parent defintion case".to_string()
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "change the parent case to match the parent definition: `{}`",
            self.parent.name().as_str()
        ))
    }

    fn generate_processed_report(&self, processed: &Processed) -> Option<String> {
        let class_parent = self.class.parent()?;
        let map = processed.mapping(self.class.name().span.start).unwrap();
        let token = map.token();
        let class_parent_map = processed.mapping(class_parent.span.start).unwrap();
        let class_parent_token = class_parent_map.token();
        let parent_map = processed.mapping(self.parent.name().span.start).unwrap();
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
            self.parent.name().as_str().fg(color_parent)
        ))
        .finish()
        .write_for_stdout(sources(processed.sources_adrianne()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {}
}
