use std::{
    f32::consts::PI,
    ops::Range,
    sync::{Arc, Mutex, OnceLock},
};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::Expression;

static IGNORED_IN_ARRAYS: OnceLock<Mutex<Vec<Range<usize>>>> = OnceLock::new();
// Detects manual use of pi values (3.14...) and suggests using the pi command

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    fn check_number(target: &Expression) -> bool {
        if let Expression::Number(value, _) = target {
            // Allow for some floating point tolerance
            // We check if the number is a reasonable approximation of pi
            // (between 3.14 and 3.142 to catch common approximations)
            #[allow(clippy::approx_constant)]
            return (value.0 - PI).abs() < 0.002 && value.0 >= 3.14;
        }
        false
    }
    let mut codes = Vec::new();
    if let Expression::Array(elements, _) = target {
        for element in elements {
            if check_number(element) {
                let mutex_vec = IGNORED_IN_ARRAYS.get_or_init(|| Mutex::new(Vec::new()));
                if let Ok(mut lock) = mutex_vec.lock() {
                    lock.push(element.full_span());
                }
            }
        }
    }
    if check_number(target) {
        let mutex_vec = IGNORED_IN_ARRAYS.get_or_init(|| Mutex::new(Vec::new()));
        if let Ok(lock) = mutex_vec.lock()
            && !lock.contains(&target.full_span())
        {
            codes.push(Arc::new(CodeS33ReimplementingCommandPi::new(
                target.full_span(),
                processed,
                config.severity(),
            )) as Arc<dyn Code>);
        }
    }

    codes
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandPi {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandPi {
    fn ident(&self) -> &'static str {
        "L-S33-PI"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("use `pi` command instead of manual pi value")
    }

    fn label_message(&self) -> String {
        String::from("use `pi`")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS33ReimplementingCommandPi {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
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
