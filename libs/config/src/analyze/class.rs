use std::collections::HashMap;

use ariadne::{sources, ColorGenerator, Label, Report};
use hemtt_preprocessor::Processed;

use crate::{analyze::codes::Codes, Class, Ident};

use super::Analyze;

impl Analyze for Class {
    fn valid(&self) -> bool {
        match self {
            Self::External { .. } => true,
            Self::Local { properties, .. } => properties.iter().all(Analyze::valid),
        }
    }

    fn warnings(&self, processed: &Processed) -> Vec<String> {
        match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } => properties
                .iter()
                .flat_map(|p| p.warnings(processed))
                .collect::<Vec<_>>(),
        }
    }

    fn errors(&self, processed: &Processed) -> Vec<String> {
        match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } => {
                let mut errors = properties
                    .iter()
                    .flat_map(|p| p.errors(processed))
                    .collect::<Vec<_>>();
                errors.extend(self.duplicate_properties(processed));
                errors
            }
        }
    }
}

impl Class {
    fn duplicate_properties(&self, processed: &Processed) -> Vec<String> {
        match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } => {
                let mut errors = Vec::new();
                let mut seen = Vec::new();
                let mut conflicts = HashMap::new();
                for property in properties {
                    if let Some(b) = seen
                        .iter()
                        .find(|b: &&Ident| b.value == property.name().value)
                    {
                        conflicts
                            .entry(b.as_str().to_string())
                            .or_insert_with(|| vec![b.clone()])
                            .push(property.name().clone());
                        continue;
                    }
                    seen.push(property.name().clone());
                }
                for conflict in conflicts.values() {
                    errors.push(duplicate_error(conflict, processed));
                }
                errors
            }
        }
    }
}

fn duplicate_error(conflicts: &[Ident], processed: &Processed) -> String {
    let first = conflicts.first().unwrap();
    let first_map = processed.original_col(first.span.start).unwrap();
    let first_file = processed.source(first_map.source()).unwrap();
    let mut out = Vec::new();
    let mut colors = ColorGenerator::new();
    Report::build(
        ariadne::ReportKind::Error,
        first_file.0.clone(),
        first.span.start,
    )
    .with_code(Codes::DuplicateProperty)
    .with_message(Codes::DuplicateProperty.message())
    .with_labels(conflicts.iter().map(|b| {
        let map = processed.original_col(b.span.start).unwrap();
        let file = processed.source(map.source()).unwrap();
        Label::new((
            file.0.clone(),
            map.original_column()..(map.original_column() + b.value.len()),
        ))
        .with_color(colors.next())
        .with_message(if b == first {
            "first defined here"
        } else {
            "also defined here"
        })
    }))
    .finish()
    .write_for_stdout(sources(processed.sources()), &mut out)
    .unwrap();
    String::from_utf8(out).unwrap()
}
