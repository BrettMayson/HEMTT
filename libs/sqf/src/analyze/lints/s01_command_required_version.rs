use std::{ops::Range, sync::Arc};

use arma3_wiki::model::Version;
use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed},
    WorkspacePath,
};

use crate::{analyze::LintData, Statements};

crate::analyze::lint!(LintS01CommandRequiredVersion);

impl Lint<LintData> for LintS01CommandRequiredVersion {
    fn ident(&self) -> &'static str {
        "required_version"
    }

    fn sort(&self) -> u32 {
        10
    }

    fn description(&self) -> &'static str {
        "Checks for command usage that requires a newer version than specified in CfgPatches"
    }

    fn documentation(&self) -> &'static str {
"### Example

**Incorrect**
```hpp
class CfgPatches {
    class MyAddon {
        units[] = {};
        weapons[] = {};
        requiredVersion = 2.00;
    };
};
```
```sqf
private _leaky = getWaterLeakiness vehicle player; // getWaterLeakiness requires 2.16
```

Check [the wiki](https://community.bistudio.com/wiki/Category:Introduced_with_Arma_3) to see what in version commands were introduced.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::fatal()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

pub struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = Statements;
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        _config: &hemtt_common::config::LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Statements,
        data: &LintData,
    ) -> hemtt_workspace::reporting::Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Some(required) = data.addon.as_ref().expect("addon will exist").build_data().required_version() else {
            // TODO what to do here?
            return Vec::new();
        };
        let mut errors: Codes = Vec::new();
        let wiki_version = arma3_wiki::model::Version::new(
            u8::try_from(required.0.major()).unwrap_or_default(),
            u8::try_from(required.0.minor()).unwrap_or_default(),
        );
        let required = (wiki_version, required.1, required.2);
        let (command, usage, usage_span) = target.required_version(&data.database);
        if wiki_version < usage {
            errors.push(Arc::new(CodeS01CommandRequiredVersion::new(
                command,
                usage_span,
                usage,
                required,
                *data.database.wiki().version(),
                processed,
            )));
        }
        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS01CommandRequiredVersion {
    command: String,
    span: Range<usize>,
    version: Version,
    required: (Option<Version>, WorkspacePath, Range<usize>),
    stable: Version,

    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS01CommandRequiredVersion {
    fn ident(&self) -> &'static str {
        "L-S01"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#required_version")
    }

    fn message(&self) -> String {
        format!(
            "command `{}` requires version {}",
            self.command, self.version
        )
    }

    fn label_message(&self) -> String {
        format!("requires version {}", self.version)
    }

    fn note(&self) -> Option<String> {
        if self.version > self.stable {
            Some(format!(
                "Current stable version is {}. Using {} will require the development branch.",
                self.stable, self.version
            ))
        } else {
            None
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS01CommandRequiredVersion {
    #[must_use]
    pub fn new(
        command: String,
        span: Range<usize>,
        version: Version,
        required: (Version, WorkspacePath, Range<usize>),
        stable: Version,
        processed: &Processed,
    ) -> Self {
        Self {
            command,
            span,
            version,
            required: {
                if required.0.major() == 0 && required.0.minor() == 0 {
                    (None, required.1, required.2)
                } else {
                    (Some(required.0), required.1, required.2)
                }
            },
            stable,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(diag) = Diagnostic::from_code_processed(&self, self.span.clone(), processed) else {
            return self;
        };
        self.diagnostic = Some(diag.with_label(
            Label::secondary(self.required.1.clone(), self.required.2.clone()).with_message(
                self.required.0.map_or_else(
                    || "CfgPatches entry doesn't specify `requiredVersion`".to_string(),
                    |required| format!("CfgPatches entry requires version {required}"),
                ),
            ),
        ));
        self
    }
}
