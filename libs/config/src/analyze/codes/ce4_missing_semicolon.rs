use std::ops::Range;

use ariadne::{sources, ColorGenerator, Label, Report};
use hemtt_error::{processed::Processed, Code};

pub struct MissingSemicolon {
    span: Range<usize>,
}

impl MissingSemicolon {
    pub const fn new(span: Range<usize>) -> Self {
        Self { span }
    }
}

impl Code for MissingSemicolon {
    fn ident(&self) -> &'static str {
        "CE4"
    }

    fn message(&self) -> String {
        "property is missing a semicolon".to_string()
    }

    fn label_message(&self) -> String {
        "missing semicolon".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("add a semicolon `;` to the end of the property".to_string())
    }

    fn generate_processed_report(&self, processed: &Processed) -> Option<String> {
        let haystack = processed.output()[self.span.start..self.span.end]
            .chars()
            .rev()
            .collect::<String>();
        let semicolon_index =
            self.span.end - haystack.chars().position(|c| !c.is_whitespace()).unwrap() - 1;
        let map = processed.original_col(semicolon_index + 1).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            map_file.0.clone(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            #[allow(clippy::range_plus_one)] // not supported by ariadne
            Label::new((
                map_file.0.clone(),
                map.original_column()..(map.original_column() + 1),
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_help(self.help().unwrap())
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }
}
