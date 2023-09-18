use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Code, Processed};

use crate::{Property, Value};

pub struct ExpectedArray {
    property: Property,
}

impl ExpectedArray {
    pub const fn new(property: Property) -> Self {
        Self { property }
    }
}

impl Code for ExpectedArray {
    fn ident(&self) -> &'static str {
        "CE6"
    }

    fn message(&self) -> String {
        "property was expected to be an array".to_string()
    }

    fn label_message(&self) -> String {
        "expected array".to_string()
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_processed_report(&self, processed: &Processed) -> Option<String> {
        let Property::Entry {
            name,
            value,
            expected_array,
        } = &self.property
        else {
            return None;
        };
        if !expected_array {
            return None;
        }
        if let Value::Array(_) = value {
            return None;
        }
        let ident_start = processed.mapping(name.span.start).unwrap();
        let ident_file = processed.source(ident_start.source()).unwrap();
        let ident_end = processed.mapping(name.span.end).unwrap();
        let value_start = processed.mapping(value.span().start).unwrap();
        let value_end = processed.mapping(value.span().end).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        let b = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            ident_file.0.clone(),
            ident_start.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                ident_file.0.clone(),
                value_start.original_column()..value_end.original_column(),
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_label(
            Label::new((
                ident_file.0.clone(),
                (ident_end.original_column())..(ident_end.original_column() + 2),
            ))
            .with_message(format!("`{}` indicates an upcoming array", "[]".fg(b)))
            .with_color(b),
        )
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {
        let Property::Entry {
            value,
            expected_array,
            ..
        } = &self.property
        else {
            return vec![];
        };
        if !expected_array {
            return vec![];
        }
        if let Value::Array(_) = value {
            return vec![];
        }
        let value_start = processed.mapping(value.span().start).unwrap();
        let value_file = processed.source(value_start.source()).unwrap();
        let value_end = processed.mapping(value.span().end).unwrap();
        let Some(path) = value_file.1 .0.clone() else {
            return vec![];
        };
        vec![(
            path,
            self.diagnostic(lsp_types::Range::new(
                value_start.original().to_lsp(),
                value_end.original().to_lsp(),
            )),
        )]
    }
}
