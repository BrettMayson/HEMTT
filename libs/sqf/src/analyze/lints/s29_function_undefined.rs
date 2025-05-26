use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
    vec,
};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    addons::Addon,
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Severity},
};
use toml::Value;

use crate::{analyze::LintData, BinaryCommand, Expression, Statement};

crate::analyze::lint!(CollectFunctions);

impl Lint<LintData> for CollectFunctions {
    fn display(&self) -> bool {
        false
    }

    fn ident(&self) -> &'static str {
        "function_undefined"
    }

    fn sort(&self) -> u32 {
        290
    }

    fn description(&self) -> &'static str {
        "Using undefined functions is bad"
    }

    fn documentation(&self) -> &'static str {
        r#"### Configuration

- **ignore**: An funcs to ignore

```toml
[lints.sqf.function_undefined]
options.ignore = [
    "myproject_fnc_piano",
]
```

### Explanation

Checks for usage of undefined functions"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![
            Box::new(RunnerExpression),
            Box::new(RunnerStatement),
            Box::new(RunnerFinal),
        ]
    }
}

fn is_project_func(var: &str, project: &ProjectConfig) -> bool {
    let prefix = project.prefix();
    var.starts_with(prefix) && var.contains("_fnc_")
}
/// Runner for Expression (Var usage and calls to CBA's compile funcs)
struct RunnerExpression;
impl LintRunner<LintData> for RunnerExpression {
    type Target = crate::Expression;

    fn run(
        &self,
        project: Option<&hemtt_common::config::ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Some(project) = project else {
            return Vec::new();
        };
        match target {
            Expression::Variable(var_name, var_span) => {
                let var_name = var_name.to_lowercase();
                if is_project_func(&var_name, project) {
                    let pos = if let Some(mapping) = processed.mapping(var_span.start) {
                        mapping.token().position().clone()
                    } else {
                        // No position found for token?
                        return vec![];
                    };
                    data.functions_used.push((var_name, pos));
                }
            }
            Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, rhs, _span) => {
                if cmd.to_lowercase() != "call" {
                    return Vec::new();
                }
                let Expression::Variable(rhs_name, _) = &**rhs else {
                    return vec![];
                };
                let Expression::Array(lhs_arr, _) = &**lhs else {
                    return vec![];
                };
                // funcs have different arg order but do have same arg length
                if lhs_arr.len() < 2 {
                    return vec![];
                }
                let func_name = match rhs_name.to_lowercase().as_str() {
                    "cba_fnc_compilefinal" => {
                        let Expression::String(func_name, _, _) = &lhs_arr[0] else {
                            return vec![];
                        };
                        func_name
                    }
                    "cba_fnc_compilefunction" | "slx_xeh_compile_new" => {
                        let Expression::String(func_name, _, _) = &lhs_arr[1] else {
                            return vec![];
                        };
                        func_name
                    }
                    _ => {
                        return vec![];
                    }
                };
                let func_name = func_name.to_lowercase();
                if is_project_func(&func_name, project) {
                    data.functions_defined.push(func_name);
                }
            }
            _ => {}
        }

        vec![]
    }
}

/// Runner for Statement (Var Assignment)
struct RunnerStatement;
impl LintRunner<LintData> for RunnerStatement {
    type Target = crate::Statement;

    fn run(
        &self,
        project: Option<&hemtt_common::config::ProjectConfig>,
        _config: &LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        data: &LintData,
    ) -> Codes {
        let Some(project) = project else {
            return Vec::new();
        };
        let Statement::AssignGlobal(func_name, _, _) = target else {
            return Vec::new();
        };
        let func_name = func_name.to_lowercase();
        if is_project_func(&func_name, project) {
            data.functions_defined.push(func_name);
        }

        vec![]
    }
}

/// Runner for finale during `pre_build`
struct RunnerFinal;
impl LintRunner<LintData> for RunnerFinal {
    type Target = Vec<Addon>;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let mut codes: Codes = Vec::new();
        let mut all_defined = HashSet::new();
        for addon in target {
            let defined = addon
                .build_data()
                .functions_defined()
                .lock()
                .expect("not romeo")
                .clone();
            all_defined.extend(defined);
        }

        if let Some(toml::Value::Array(ignore)) = config.option("ignore") {
            for i in ignore {
                if let Value::String(i) = i {
                    all_defined.insert(i.to_lowercase());
                }
            }
        }

        let mut all_missing = BTreeMap::new();
        for addon in target {
            let used = addon
                .build_data()
                .functions_used()
                .lock()
                .expect("not juliet")
                .clone();
            for (func, span) in used {
                if !all_defined.contains(&func) {
                    all_missing.entry(func).or_insert(Vec::new()).push(span);
                }
            }
        }

        for (func, spans) in all_missing {
            let joined = spans
                .iter()
                .map(|s| format!("{}:{}:{}", s.path(), s.start().line(), s.start().column()))
                .collect::<Vec<_>>()
                .join("\n");

            codes.push(Arc::new(Code29FunctionUndefined::new(
                func,
                joined,
                config.severity(),
            )));
        }

        codes
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct Code29FunctionUndefined {
    func_name: String,
    spans: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code29FunctionUndefined {
    fn ident(&self) -> &'static str {
        "L-S29"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#function_undefined")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!("Undefined Function: {}", self.func_name)
    }
    fn note(&self) -> Option<String> {
        Some(format!("Used in:\n{}", self.spans))
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code29FunctionUndefined {
    #[must_use]
    pub fn new(func_name: String, spans: String, severity: Severity) -> Self {
        Self {
            func_name,
            spans,
            severity,
            diagnostic: None,
        }
        .generate_processed()
    }

    fn generate_processed(mut self) -> Self {
        self.diagnostic = Some(Diagnostic::from_code(&self));
        self
    }
}
