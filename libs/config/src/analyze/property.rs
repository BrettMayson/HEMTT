use std::ops::Range;

use ariadne::{sources, ColorGenerator, Label, Report};

use crate::{Property, Value};

use super::{codes::Codes, Analyze};

impl Analyze for Property {
    fn valid(&self) -> bool {
        !matches!(
            self,
            Self::Entry {
                value: Value::Invalid(_),
                ..
            } | Self::MissingSemicolon(_, _)
        )
    }

    fn warnings(&self, processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        match self {
            Self::Entry { value, .. } => value.warnings(processed),
            Self::Class(c) => c.warnings(processed),
            Self::Delete(_) => vec![],
            Self::MissingSemicolon(_, _) => vec![], // TODO: warning
        }
    }

    fn errors(&self, processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        match self {
            Self::Entry { value, .. } => value.errors(processed),
            Self::Class(c) => c.errors(processed),
            Self::Delete(_) => vec![],
            Self::MissingSemicolon(_, span) => vec![missing_semicolon(span, processed)],
        }
    }
}

fn missing_semicolon(span: &Range<usize>, processed: &hemtt_preprocessor::Processed) -> String {
    let haystack = processed.output()[span.start..span.end]
        .chars()
        .rev()
        .collect::<String>();
    let semicolon_index = span.end - haystack.chars().position(|c| !c.is_whitespace()).unwrap() - 1;
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
    .with_code(Codes::MissingSemicolon)
    .with_message(Codes::MissingSemicolon.message())
    .with_label(
        #[allow(clippy::range_plus_one)] // not supported by ariadne
        Label::new((
            map_file.0.clone(),
            map.original_column()..(map.original_column() + 1),
        ))
        .with_message(Codes::MissingSemicolon.label_message())
        .with_color(a),
    )
    .with_help(Codes::MissingSemicolon.help().unwrap())
    .finish()
    .write_for_stdout(sources(processed.sources()), &mut out)
    .unwrap();
    String::from_utf8(out).unwrap()
}
