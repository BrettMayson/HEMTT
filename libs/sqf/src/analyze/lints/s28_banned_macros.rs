use std::{ops::Range, sync::Arc};

use hemtt_common::config::{BuildInfo, LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Statements};

crate::analyze::lint!(LintS28BannedMacros);

impl Lint<LintData> for LintS28BannedMacros {
    fn ident(&self) -> &'static str {
        "banned_macros"
    }

    fn sort(&self) -> u32 {
        280
    }

    fn description(&self) -> &'static str {
        "Checks for banned macro in release builds."
    }

    fn documentation(&self) -> &'static str {
        r#"### Configuration

- **banned**: macros to check for

```toml
[lints.sqf.banned_macros]
options.banned = [
    "DEBUG_MODE_FULL",
]
```

### Explanation

Checks for usage of banned macros for release"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn minimum_severity(&self) -> Severity {
        Severity::Warning
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = Statements;

    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        build_info: Option<&hemtt_common::config::BuildInfo>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        if !build_info.is_some_and(BuildInfo::is_release) {
            return Vec::new();
        }
        let macros = processed.macros();
        if let Some(toml::Value::Array(banned)) = config.option("banned") {
            for ban in banned {
                let Some(ban_name) = ban.as_str() else {
                    continue;
                };
                if macros.contains_key(ban_name) {
                    return vec![Arc::new(CodeS28BannedMacros::new(
                        ban_name,
                        target.span(),
                        processed,
                        config.severity(),
                    ))];
                }
            }
        }
        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS28BannedMacros {
    macro_name: String,
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS28BannedMacros {
    fn ident(&self) -> &'static str {
        "L-S280"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#banned_macros")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("Macro `{}` is banned on release builds", self.macro_name)
    }

    fn label_message(&self) -> String {
        "macro defined inside this scope".into()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS28BannedMacros {
    #[must_use]
    pub fn new(
        macro_name: &str,
        span: Range<usize>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            macro_name: macro_name.into(),
            span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic =
            Diagnostic::from_code_processed_skip_macros(&self, self.span.clone(), processed);
        self
    }
}
