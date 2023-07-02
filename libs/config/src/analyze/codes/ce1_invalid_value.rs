use std::ops::Range;

use ariadne::{sources, ColorGenerator, Label, Report};
use hemtt_error::Code;

pub struct InvalidValue {
    span: Range<usize>,
}

impl InvalidValue {
    pub const fn new(span: Range<usize>) -> Self {
        Self { span }
    }
}

impl Code for InvalidValue {
    fn ident(&self) -> &'static str {
        "CE1"
    }

    fn message(&self) -> String {
        "property's value could not be parsed.".to_string()
    }

    fn label_message(&self) -> String {
        "invalid value".to_string()
    }

    fn help(&self) -> Option<String> {
        Some(
            "use quotes `\"` around the value, or a QUOTE macro if it contains #define values"
                .to_string(),
        )
    }

    fn generate_processed_report(
        &self,
        processed: &hemtt_error::processed::Processed,
    ) -> Option<String> {
        let map = processed.original_col(self.span.start).unwrap();
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
            Label::new((
                map_file.0.clone(),
                map.original_column()..map.original_column() + self.span.len(),
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
