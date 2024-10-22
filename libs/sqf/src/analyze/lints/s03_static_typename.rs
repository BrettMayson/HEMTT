use std::{ops::Range, sync::Arc};

use float_ord::FloatOrd;
use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Diagnostic, Processed, Severity}};

use crate::{analyze::SqfLintData, Expression, NularCommand, UnaryCommand};

crate::analyze::lint!(LintS03StaticTypename);

impl Lint<SqfLintData> for LintS03StaticTypename {
    fn ident(&self) -> &str {
        "static_typename"
    }

    fn sort(&self) -> u32 {
        30
    }

    fn description(&self) -> &str {
        "Checks for `typeName` on static values, which can be replaced with the string type directly"
    }

    fn documentation(&self) -> &str {
r#"### Example

**Incorrect**
```sqf
if (typeName _myVar == typeName "") then {
    hint "String";
};
```
**Correct**
```sqf
if (typeName _myVar == "STRING") then {
    hint "String";
};
```

### Explanation

`typeName` is a command that returns the type of a variable. When used on a constant value, it is slower than using the type directly."#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = Expression;
    
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        _data: &SqfLintData,
    ) -> hemtt_workspace::reporting::Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::UnaryCommand(UnaryCommand::Named(name), expresssion, _) = target else {
            return Vec::new();
        };
        if name.to_lowercase() != "typename" {
            return Vec::new();
        }
        let target_span = expresssion.span();
        let (constant_type, span, length) = match &**expresssion {
            Expression::String(s, span, _) => ("STRING", span, s.len() + 2),
            Expression::Number(FloatOrd(s), span) => ("SCALAR", span, s.to_string().len()),
            Expression::Boolean(bool, span) => ("BOOL", span, bool.to_string().len()),
            Expression::Code(statements) if statements.content().is_empty() => {
                ("CODE", &target_span, statements.span().len())
            }
            Expression::Array(array, span) if array.is_empty() => {
                ("ARRAY", &target_span, span.len().max(2))
            }
            Expression::NularCommand(NularCommand { name }, span) => {
                let (a, b) = match name.as_str() {
                    "scriptnull" => ("SCRIPT", span),
                    "objnull" => ("OBJECT", span),
                    "grpnull" => ("GROUP", span),
                    "controlnull" => ("CONTROL", span),
                    "teammembernull" => ("TEAM_MEMBER", span),
                    "displaynull" => ("DISPLAY", span),
                    "tasknull" => ("TASK", span),
                    "locationnull" => ("LOCATION", span),
                    "sideunknown" => ("SIDE", span),
                    "configfile" | "confignull" => ("CONFIG", span),
                    "missionnamespace" | "profilenamespace" | "uinamespace" | "parsingnamespace" => {
                        ("NAMESPACE", span)
                    }
                    "diaryrecordnull" => ("DIARY_RECORD", span),
                    "createhashmap" => ("HASHMAP", span),
                    _ => return Vec::new(),
                };
                (a, b, name.len())
            }
            Expression::UnaryCommand(UnaryCommand::Named(name), _, span) => {
                if name == "text" {
                    ("TEXT", span, name.len())
                } else {
                    return Vec::new();
                }
            }
            _ => return Vec::new(),
        };
        vec![Arc::new(CodeS03StaticTypename::new(
            target.full_span(),
            (constant_type.to_string(), span.clone(), length),
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS03StaticTypename {
    span: Range<usize>,
    constant: (String, Range<usize>, usize),

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS03StaticTypename {
    fn ident(&self) -> &'static str {
        "L-S03"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#static_typename")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("using `typeName` on a constant is slower than using the type directly")
    }

    fn label_message(&self) -> String {
        "`typeName` on a constant".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("\"{}\"", self.constant.0))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS03StaticTypename {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        constant: (String, Range<usize>, usize),
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            constant,

            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
