use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{
    BinaryCommand::{self}, Expression, analyze::LintData,
};

crate::analyze::lint!(LintS53SelectSubstringLen);

impl Lint<LintData> for LintS53SelectSubstringLen {
    fn ident(&self) -> &'static str {
        "select_substring_len"
    }
    fn sort(&self) -> u32 {
        530
    }
    fn description(&self) -> &'static str {
        "Checks for substring length mismatch when using `select`"
    }
    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
if (((currentWeapon player) select [0, 3]) == "ABC_") then {};
```

**Correct**
```sqf
if (((currentWeapon player) select [0, 4]) == "ABC_") then {};
```
"#
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Expression;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        fn match_pair(e1: &Expression, e2: &Expression) -> Option<(usize, usize)> {
            let Expression::String(str, _, _) = e1 else {
                return None;
            };
            let Expression::BinaryCommand(BinaryCommand::Named(cmd), _sel_lhs, sel_rhs, _) = e2 else {
                return None;
            };
            if !cmd.eq_ignore_ascii_case("select") {
                return None;
            }
            let Expression::Array(arr, _) = sel_rhs.as_ref() else {
                return None;
            };
            if arr.len() != 2 {
                return None;
            }
            let Expression::Number(sel_len, _) = arr[1] else {
                return None;
            };
            let str_len = str.len();
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let sel_len = sel_len.0.round() as usize;
            println!("str_len: {str_len}, sel_len: {sel_len}");
            if str_len == sel_len {
                return None;
            }
            Some((str_len, sel_len))
        }

        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::BinaryCommand(bcmd, lhs, rhs, span) = target else {
            return Vec::new();
        };
        if !(bcmd == &BinaryCommand::Eq || bcmd == &BinaryCommand::NotEq || bcmd.as_str().eq_ignore_ascii_case("isEqualTo") || bcmd.as_str().eq_ignore_ascii_case("isNotEqualTo")) {
            return Vec::new();
        }
        let len_pair = match_pair(lhs, rhs).or_else(|| match_pair(rhs, lhs));
        let Some((str_len, sel_len)) = len_pair else {
            return Vec::new();
        };
        vec![Arc::new(CodeS53SelectSubstringLen::new(
            span.clone(),
            str_len, sel_len,
            config.severity(),
            processed,
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS53SelectSubstringLen {
    span: Range<usize>,
    str_len: usize,
    sel_len: usize,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS53SelectSubstringLen {
    fn ident(&self) -> &'static str {
        "L-S53"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#select_substring_len")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        "Select substring length does not match string length".to_string()
    }
    fn label_message(&self) -> String {
        format!("substring is length {} | string is length {}", self.sel_len, self.str_len)
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS53SelectSubstringLen {
    #[must_use]
    pub fn new(span: Range<usize>, str_len: usize, sel_len: usize, severity: Severity, processed: &Processed) -> Self {
        Self {
            span,
            str_len,
            sel_len,
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
