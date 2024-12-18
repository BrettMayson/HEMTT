use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Processed, Severity}};

use crate::{analyze::LintData, Expression};

crate::analyze::lint!(LintS09BannedCommand);

impl Lint<LintData> for LintS09BannedCommand {
    fn ident(&self) -> &'static str {
        "banned_commands"
    }

    fn sort(&self) -> u32 {
        90
    }

    fn description(&self) -> &'static str {
        "Checks for broken or banned commands."
    }

    fn documentation(&self) -> &'static str {
r#"### Configuration

- **banned**: Additional commands to check for
- **ignore**: An array of commands to ignore

```toml
[lints.sqf.banned_commands]
options.banned = [
    "execVM",
]
options.ignore = [
    "echo",
]
```

### Example

**Incorrect**
```sqf
echo "Hello World"; // Doesn't exist in the retail game
```

### Explanation

Checks for usage of broken or banned commands."#
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
        let Some(wiki) = data.1.wiki().commands().get(command) else {
            return Vec::new();
        };
        if wiki.groups().contains(&String::from("Broken Commands")) {
            return vec![Arc::new(CodeS09BannedCommand::new(
                target.span(),
                command.to_string(),
                processed,
                config.severity(),
                true,
            ))];
        }
        if let Some(toml::Value::Array(banned)) = config.option("banned") {
            if banned.iter().any(|i| i.as_str().map(str::to_lowercase) == Some(command.to_lowercase())) {
                return vec![Arc::new(CodeS09BannedCommand::new(
                    target.span(),
                    command.to_string(),
                    processed,
                    config.severity(),
                    false,
                ))];
            }
        }
        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS09BannedCommand {
    span: Range<usize>,
    command: String,
    from_wiki: bool,

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS09BannedCommand {
    fn ident(&self) -> &'static str {
        "L-S09"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#banned_commands")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        if self.from_wiki {
            format!("`{}` is marked as a broken command on the wiki", self.command)
        } else {
            format!("`{}` is banned by the project", self.command)
        }
    }

    fn label_message(&self) -> String {
        if self.from_wiki {
            "broken command".to_string()
        } else {
            "banned command".to_string()
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS09BannedCommand {
    #[must_use]
    pub fn new(span: Range<usize>, command: String, processed: &Processed, severity: Severity, from_wiki: bool) -> Self {
        Self {
            span,
            command,

            severity,
            diagnostic: None,
            from_wiki,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
