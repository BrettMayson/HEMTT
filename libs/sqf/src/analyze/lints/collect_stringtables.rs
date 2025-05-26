use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::Codes,
};

use crate::{analyze::LintData, Expression};

crate::analyze::lint!(LocalizeStringtable);

impl Lint<LintData> for LocalizeStringtable {
    fn display(&self) -> bool {
        false
    }

    fn ident(&self) -> &'static str {
        "collect_stringtable"
    }

    fn sort(&self) -> u32 {
        0
    }

    fn description(&self) -> &'static str {
        "collect_stringtable"
    }

    fn documentation(&self) -> &'static str {
        r"This should not be visible"
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
        _config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::String(lstring, range, _) = target else {
            return Vec::new();
        };
        
        // `localize`` strings are case-insensitive and optionally can start with $
        if lstring.to_lowercase().starts_with("str_") || lstring.to_lowercase().starts_with("$str_") {
            let pos = if let Some(mapping) = processed.mapping(range.start) {
                mapping.token().position().clone()
            } else {
                // No position found for token
                return vec![];
            };
            data.localizations.push((lstring.trim_start_matches('$').to_lowercase(), pos));
        }

        vec![]
    }
}
