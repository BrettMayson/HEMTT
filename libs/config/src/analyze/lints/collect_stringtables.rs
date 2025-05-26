use std::ops::Range;

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Codes, Processed},
};

use crate::{analyze::LintData, Item, Value};

crate::analyze::lint!(LintColectStringtables);

impl Lint<LintData> for LintColectStringtables {
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
    type Target = Value;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Value,
        data: &LintData,
    ) -> Codes {
        fn check_string(
            hstr: &crate::Str,
            span: &Range<usize>,
            processed: &Processed,
            data: &LintData,
        ) {
            let hstr_value = hstr.value();
            if hstr_value.starts_with("$STR") {
                // 4 char prefix is case-sensitive
                let mut locations = data.localizations.lock().expect("mutex safety");
                let pos = if let Some(mapping) = processed.mapping(span.start) {
                    mapping.token().position().clone()
                } else {
                    // No position found for token
                    return;
                };
                locations.push((hstr_value.trim_start_matches('$').to_lowercase(), pos));
            }

            
        }
        let Some(processed) = processed else {
            return vec![];
        };
        match target {
            Value::Array(array_data) => {
                for item in &array_data.items {
                    let Item::Str(item_data) = item else {
                        continue;
                    };
                    check_string(item_data, &item_data.span(), processed, data);
                }
            }
            Value::Str(cstring_data) => {
                check_string(cstring_data, &target.span(), processed, data);
            }
            _ => {}
        }

        vec![]
    }
}
