use ariadne::{sources, ColorGenerator, Fmt, Label, Report};

use crate::{analyze::codes::Codes, Value};

use super::Analyze;

impl Analyze for Value {
    fn valid(&self) -> bool {
        match self {
            Self::Str(s) => s.valid(),
            Self::Number(n) => n.valid(),
            Self::Array(a) => a.valid(),
            Self::UnexpectedArray(_) => false,
            Self::Invalid(_) => false,
        }
    }

    fn warnings(&self, processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        match self {
            Self::Str(s) => s.warnings(processed),
            Self::Number(n) => n.warnings(processed),
            Self::Array(a) => a.warnings(processed),
            Self::UnexpectedArray(a) => a.warnings(processed),
            Self::Invalid(_) => vec![],
        }
    }

    fn errors(&self, processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        match self {
            Self::Str(s) => s.errors(processed),
            Self::Number(n) => n.errors(processed),
            Self::Array(a) | Self::UnexpectedArray(a) => a.errors(processed),
            Self::Invalid(invalid) => {
                // An unquoted string or otherwise invalid value
                vec![{
                    let map = processed.original_col(invalid.start).unwrap();
                    let mut out = Vec::new();
                    let mut colors = ColorGenerator::new();
                    let a = colors.next();
                    let mut root = map.token();
                    let mut code = Codes::InvalidValue;
                    while let Some(parent) = root.parent() {
                        root = parent;
                        code = Codes::InvalidValueMacro;
                    }
                    let mut report = Report::build(
                        ariadne::ReportKind::Error,
                        root.source().path_or_builtin(),
                        root.source().start().0,
                    )
                    .with_code(code)
                    .with_message(code.message())
                    .with_label(
                        Label::new((
                            root.source().path_or_builtin(),
                            root.source().start().0..root.source().end().0,
                        ))
                        .with_message(code.label_message())
                        .with_color(a),
                    )
                    .with_help(code.help().unwrap());
                    if code == Codes::InvalidValueMacro {
                        report = report.with_note(format!(
                            "The processed output was `{}`",
                            &processed.output()[invalid.start..invalid.end].fg(a)
                        ));
                    }
                    // .with_note("This may be valid in some other programs, learn more at https://hemtt.io/unquoted")
                    report
                        .finish()
                        .write_for_stdout(sources(processed.sources()), &mut out)
                        .unwrap();
                    String::from_utf8(out).unwrap()
                }]
            }
        }
    }
}
