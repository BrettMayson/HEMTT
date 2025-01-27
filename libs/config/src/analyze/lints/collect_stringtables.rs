use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Codes, Processed}
};

use crate::{analyze::LintData, Value};

crate::analyze::lint!(LintC12ConfigStringtable);

impl Lint<LintData> for LintC12ConfigStringtable {
    fn display(&self) -> bool {
        false
    }

    fn ident(&self) -> &'static str {
        "config_stringtable"
    }

    fn sort(&self) -> u32 {
        0
    }

    fn description(&self) -> &'static str {
        "config stringtable entriy does not exist"
    }

    fn documentation(&self) -> &'static str {
        r"### Explanation
Strings should exist...
"
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
    type Target = Value;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Value,
        data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return vec![];
        };
        let Value::Str(cstring_data) = target else {
            return vec![];
        };
        let cstring_value = cstring_data.value();

        if cstring_value.to_lowercase().starts_with("str_") || cstring_value.to_lowercase().starts_with("$str_") {
            let mut locations = data.localizations.lock().expect("mutex safety");
            let pos = if let Some(mapping) = processed.mapping(target.span().start) {
                mapping.token().position().clone()
            } else {
                // No position found for token
                return vec![];
            };
            locations.push((cstring_value.trim_start_matches('$').to_lowercase(), pos));
        }

        vec![]
    }
}
