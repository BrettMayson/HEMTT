use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Processed, Severity}};

use crate::{analyze::LintData, Expression};

crate::analyze::lint!(LintS04CommandCase);

impl Lint<LintData> for LintS04CommandCase {
    fn ident(&self) -> &'static str {
        "command_case"
    }

    fn sort(&self) -> u32 {
        40
    }

    fn description(&self) -> &'static str {
        "Checks command usage for casing that matches the wiki"
    }

    fn documentation(&self) -> &'static str {
r#"### Configuration

- **ignore**: An array of commands to ignore

```toml
[lints.sqf.command_case]
options.ignore = [
    "ASLtoAGL",
    "AGLtoASL",
]
```

### Example

**Incorrect**
```sqf
private _leaky = getwaterleakiness vehicle player;
```
**Correct**
```sqf
private _leaky = getWaterLeakiness vehicle player;
```"#
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
    type Target = Expression;
    
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &Self::Target,
        data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Some(command) = target.command_name() else {
            return Vec::new();
        };
        if let Some(toml::Value::Array(ignore)) = config.option("ignore") {
            if ignore.iter().any(|i| i.as_str().map(str::to_lowercase) == Some(command.to_lowercase())) {
                return Vec::new();
            }
        }
        let Some(wiki) = data.database.wiki().commands().get(command) else {
            return Vec::new();
        };
        if command != wiki.name() {
            return vec![Arc::new(CodeS04CommandCase::new(
                target.span(),
                command.to_string(),
                wiki.name().to_string(),
                processed,
                config.severity(),
            ))];
        }
        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS04CommandCase {
    span: Range<usize>,
    used: String,
    wiki: String,

    include: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS04CommandCase {
    fn ident(&self) -> &'static str {
        "L-S04"
    }

    fn include(&self) -> bool {
        self.include
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#command_case")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("`{}` does not match the wiki's case", self.used)
    }

    fn label_message(&self) -> String {
        "non-standard command case".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("\"{}\"", self.wiki))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS04CommandCase {
    #[must_use]
    pub fn new(span: Range<usize>, used: String, wiki: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            include: processed.mappings(span.end).first().is_some_and(|mapping| {
                mapping.original().path().is_include()
            }),
            severity,
            diagnostic: None,
            
            span,
            used,
            wiki,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
