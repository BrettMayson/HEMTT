use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_error::processed::Processed;
use hemtt_error::Code;
use lsp_types::Diagnostic;

use crate::model::Value;
use crate::Property;

pub struct UnexpectedArray {
    property: Property,
}

impl UnexpectedArray {
    pub const fn new(property: Property) -> Self {
        Self { property }
    }
}

impl Code for UnexpectedArray {
    fn ident(&self) -> &'static str {
        "CE5"
    }

    fn message(&self) -> String {
        "property was not expected to be an array".to_string()
    }

    fn label_message(&self) -> String {
        "unexpected array".to_string()
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_processed_report(&self, processed: &Processed) -> Option<String> {
        let Property::Entry { name, value: Value::UnexpectedArray(array), .. } = &self.property else {
            return None;
        };
        let array_start = processed.original_col(array.span.start).unwrap();
        let array_file = processed.source(array_start.source()).unwrap();
        let array_end = processed.original_col(array.span.end).unwrap();
        let ident_start = processed.original_col(name.span.start).unwrap();
        let ident_end = processed.original_col(name.span.end).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        let b = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            array_file.0.clone(),
            array_start.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            #[allow(clippy::range_plus_one)] // not supported by ariadne
            Label::new((
                array_file.0.clone(),
                array_start.original_column()..array_end.original_column(),
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_label(
            Label::new((
                array_file.0.clone(),
                ident_start.original_column()..ident_end.original_column(),
            ))
            .with_message(format!(
                "expected `{}` here",
                format!("{}[]", name.as_str()).fg(b)
            ))
            .with_color(b),
        )
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {
        let Property::Entry { value: Value::UnexpectedArray(array), .. } = &self.property else {
            return vec![];
        };
        let array_start = processed.original_col(array.span.start).unwrap();
        let array_file = processed.source(array_start.source()).unwrap();
        let array_end = processed.original_col(array.span.end).unwrap();
        let Some(path) = array_file.1.0.clone() else {
            return vec![];
        };
        vec![(
            path,
            self.diagnostic(lsp_types::Range::new(
                array_start.original().to_lsp(),
                array_end.original().to_lsp(),
            )),
        )]
    }
}
