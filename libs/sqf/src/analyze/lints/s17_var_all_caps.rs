use std::{ops::Range, sync::Arc};

use hemtt_common::{config::LintConfig, similar_values};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed, Severity, Symbol},
};

use crate::{analyze::LintData, Expression};

crate::analyze::lint!(LintS17VarAllCaps);

impl Lint<LintData> for LintS17VarAllCaps {
    fn ident(&self) -> &'static str {
        "var_all_caps"
    }

    fn sort(&self) -> u32 {
        170
    }

    fn description(&self) -> &'static str {
        "Checks for global variables that are ALL_CAPS and may actually be a undefined macro"
    }

    fn documentation(&self) -> &'static str {
        r#"### Configuration

- **ignore**: An array of vars to ignore

```toml
[lints.sqf.var_all_caps]
options.ignore = [
    "XMOD_TEST", "MYMOD_*",
]
```

### Example

**Incorrect**
```sqf
private _z = _y + DO_NOT_EXIST;
```

### Explanation

Variables that are all caps are usually reserved for macros. This should should help prevent any accidental typos or uses before definitions when using macros.
."#
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
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
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::Variable(var, span) = target else {
            return Vec::new();
        };
        if var.starts_with('_') || &var.to_ascii_uppercase() != var || var == "SLX_XEH_COMPILE_NEW"
        {
            return Vec::new();
        }
        if let Some(toml::Value::Array(ignore)) = config.option("ignore") {
            if ignore.iter().any(|i| {
                let s = i.as_str().unwrap_or_default();
                s == var || (s.ends_with('*') && var.starts_with(&s[0..s.len() - 1]))
            }) {
                return Vec::new();
            }
        }
        vec![Arc::new(CodeS17VarAllCaps::new(
            span.clone(),
            var.clone(),
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS17VarAllCaps {
    span: Range<usize>,
    ident: String,
    similar: Vec<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS17VarAllCaps {
    fn ident(&self) -> &'static str {
        "L-S17"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#var_all_caps")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("Variable should not be all caps: {}", self.ident)
    }

    fn note(&self) -> Option<String> {
        Some("All caps variables are usually reserved for macros".to_string())
    }

    fn label_message(&self) -> String {
        "all caps variable".to_string()
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("did you mean `{}`?", self.similar.join("`, `")))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS17VarAllCaps {
    #[must_use]
    pub fn new(span: Range<usize>, ident: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            similar: similar_values(
                &ident,
                &processed.macros().keys().map(std::string::String::as_str).collect::<Vec<_>>(),
            )
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
            span,
            ident,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(mut diagnostic) = Diagnostic::from_code_processed(&self, self.span.clone(), processed) else {
            return self;
        };
        self.diagnostic = Some(diagnostic.clone());
        let mut mappings = processed.mappings(self.span.start);
        mappings.pop();
        let symbol = Symbol::Word(self.ident.clone());
        let Some(mapping) = mappings
            .iter()
            .find(|m| {
                m.token().symbol() == &symbol
            }) else {
            return self;
            };
        if let Some(l) = diagnostic.labels.get_mut(0) { *l = l.clone().with_message("used in macro here"); }
        diagnostic.labels.push(
            Label::primary(
                mapping.original().path().clone(),
                mapping.original().span(),
            )
            .with_message("all caps variable"),
        );
        self.diagnostic = Some(diagnostic);
        self
    }
}
