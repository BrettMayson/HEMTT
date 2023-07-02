use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_preprocessor::Processed;

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

    fn warnings(&self, processed: &Processed) -> Vec<String> {
        match self {
            Self::Entry { value, .. } => value.warnings(processed),
            Self::Class(c) => c.warnings(processed),
            Self::Delete(_) => vec![],
            Self::MissingSemicolon(_, _) => vec![], // TODO: warning
        }
    }

    fn errors(&self, processed: &Processed) -> Vec<String> {
        match self {
            Self::Entry { value, .. } => {
                let mut errors = value.errors(processed);
                errors.extend(unexpected_array(self, processed));
                errors.extend(expected_array(self, processed));
                errors
            }
            Self::Class(c) => c.errors(processed),
            Self::Delete(_) => vec![],
            Self::MissingSemicolon(_, span) => vec![missing_semicolon(span, processed)],
        }
    }
}

fn missing_semicolon(span: &Range<usize>, processed: &Processed) -> String {
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

fn unexpected_array(property: &Property, processed: &Processed) -> Vec<String> {
    let Property::Entry { name, value: Value::UnexpectedArray(array), .. } = property else {
        return vec![];
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
    .with_code(Codes::UnexpectedArray)
    .with_message(Codes::UnexpectedArray.message())
    .with_label(
        #[allow(clippy::range_plus_one)] // not supported by ariadne
        Label::new((
            array_file.0.clone(),
            array_start.original_column()..array_end.original_column(),
        ))
        .with_message(Codes::UnexpectedArray.label_message())
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
    vec![String::from_utf8(out).unwrap()]
}

fn expected_array(property: &Property, processed: &Processed) -> Vec<String> {
    let Property::Entry { name, value, expected_array } = property else {
        return vec![];
    };
    if !expected_array {
        return vec![];
    }
    if let Value::Array(_) = value {
        return vec![];
    }
    let ident_start = processed.original_col(name.span.start).unwrap();
    let ident_file = processed.source(ident_start.source()).unwrap();
    let ident_end = processed.original_col(name.span.end).unwrap();
    let value_start = processed.original_col(value.span().start).unwrap();
    let value_end = processed.original_col(value.span().end).unwrap();
    let mut out = Vec::new();
    let mut colors = ColorGenerator::new();
    let a = colors.next();
    let b = colors.next();
    Report::build(
        ariadne::ReportKind::Error,
        ident_file.0.clone(),
        ident_start.original_column(),
    )
    .with_code(Codes::ExpectedArray)
    .with_message(Codes::ExpectedArray.message())
    .with_label(
        Label::new((
            ident_file.0.clone(),
            value_start.original_column()..value_end.original_column(),
        ))
        .with_message(Codes::ExpectedArray.label_message())
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
    vec![String::from_utf8(out).unwrap()]
}
