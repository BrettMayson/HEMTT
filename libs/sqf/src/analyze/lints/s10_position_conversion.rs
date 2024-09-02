use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Processed, Severity}};

use crate::{analyze::SqfLintData, BinaryCommand, Expression, UnaryCommand};

crate::lint!(LintS10PositionConversion);

impl Lint<SqfLintData> for LintS10PositionConversion {
    fn ident(&self) -> &str {
        "unnecessary_position_conversion"
    }

    fn description(&self) -> &str {
        "Unnecessary position conversion on a command with a variant"
    }

    fn documentation(&self) -> &str {
        todo!()
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

static COMBINATIONS: [[&str; 3]; 6] = [
    // ["agltoasl",     "getpos",          "getPosASL"], // AGL vs AGLS :|
    // ["agltoatl",     "getpos",          "getPosATL"],
    // ["asltoagl",     "getposasl",       "getPos"],
    // ["atltoagl",     "getposatl",       "getPos"],
    ["asltoatl",    "getposasl",       "getPosATL"],
    ["atltoasl",    "getposatl",       "getPosASL"],
    // ["agltoasl",    "visibleposition", "visiblePositionASL"],
    ["setpos",      "asltoagl",        "setPosASL"],
    ["setpos",      "atltoagl",        "setPosATL"],
    ["setposatl",   "asltoatl",        "setPosATL"],
    ["setposasl",   "atltoasl",        "setPosASL"],
];

struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = Expression;
    
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &Self::Target,
        _data: &SqfLintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let (
            Expression::BinaryCommand(BinaryCommand::Named(name), _, expression, span)
            | Expression::UnaryCommand(UnaryCommand::Named(name), expression, span)
        ) = target else {
            return Vec::new()
        };
        let binary = matches!(target, Expression::BinaryCommand(_, _, _, _));

        if binary && target.command_name().map(str::to_lowercase) != Some("getpos".to_string()) {
            return Vec::new();
        }

        let Some(conversion) = expression.command_name() else {
            return Vec::new();
        };

        let name = name.to_lowercase();
        let conversion = conversion.to_lowercase();

        let Some(combination) = COMBINATIONS.iter().find(|c| c[0] == name && c[1] == conversion) else {
            return Vec::new();
        };
        
        let span = span.start .. expression.span().end;

        vec![Arc::new(CodeS10PositionConversion::new(span, combination[2].to_string(), processed, config.severity()))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS10PositionConversion {
    span: Range<usize>,
    suggestion: String,

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS10PositionConversion {
    fn ident(&self) -> &'static str {
        "L-S10"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#unnecessary_position_conversion")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Unnecessary position conversion on a command with a variant".to_string()
    }

    fn label_message(&self) -> String {
        "Unnecessary conversion".to_string()
    }

    /// In order to be a suggestion, it must deal with parenthesis and left side arguments
    fn help(&self) -> Option<String> {
        Some(self.suggestion.clone())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS10PositionConversion {
    #[must_use]
    pub fn new(span: Range<usize>, suggestion: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            suggestion,

            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}

