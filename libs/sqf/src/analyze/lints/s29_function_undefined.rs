use std::{
    collections::{BTreeMap, HashSet}, sync::Arc, vec
};

use hemtt_common::{config::{LintConfig, ProjectConfig}, similar_values};
use hemtt_workspace::{
    addons::Addon, lint::{AnyLintRunner, Lint, LintRunner}, position::Position, reporting::{Code, Codes, Diagnostic, Label, Mapping, Severity}, WorkspacePath
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
        "Reports on undefined functions using the project's prefix"
    }

    fn documentation(&self) -> &'static str {
        r#"### Configuration

- **ignore**: Functions to ignore

```toml
[lints.sqf.function_undefined]
options.ignore = [
    "myproject_fnc_piano",
]
```"#
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

fn is_project_func(var_lower: &str, project: &ProjectConfig) -> bool {
    let prefix = project.prefix().to_lowercase();
    var_lower.starts_with(&prefix) && var_lower.contains("_fnc_")
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
        _runtime: &hemtt_common::config::RuntimeArguments,
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
                    let mut used_functions = data.functions_used.lock().expect("mutex safety");
                    let Some(map_start) = processed.mapping(var_span.start) else {
                        return vec![];
                    };
                    let Some(map_end) = processed.mapping(var_span.end) else {
                        return vec![];
                    };
                    let Some(map_file) = processed.source(map_start.source()) else {
                        return vec![];
                    };
                    used_functions.push((var_name, pos, map_start.to_owned(), map_end.to_owned(), map_file.0.clone()));
                }
            }
            Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, rhs, _span) => {
                if !cmd.eq_ignore_ascii_case("call") {
                    return vec![];
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
                let func_name_lower = func_name.to_lowercase();
                if is_project_func(&func_name_lower, project) {
                    let mut functions_defined =
                        data.functions_defined.lock().expect("mutex safety");
                    functions_defined.insert((func_name_lower, func_name.clone()));
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
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        data: &LintData,
    ) -> Codes {
        let Some(project) = project else {
            return Vec::new();
        };
        let Statement::AssignGlobal(func_name, _, _) = target else {
            return Vec::new();
        };
        let func_name_lower = func_name.to_lowercase();
        if is_project_func(&func_name_lower, project) {
            let mut functions_defined = data.functions_defined.lock().expect("mutex safety");
            functions_defined.insert((func_name_lower, func_name.clone().into()));
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
        runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let mut codes: Codes = Vec::new();
        if runtime.is_just() { // --just build will be missing EFUNCS
            return codes;
        }
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
                    all_defined.insert((i.to_lowercase(), i.clone().into()));
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
            for (func, position, start, end, file) in used {
                if !all_defined.iter().any(|(s, _)| s == &func) {
                    all_missing.entry(func).or_insert(Vec::new()).push((position, start, end, file));
                }
            }
        }

        for (func, positions) in all_missing {
            let similar = similar_values(&func, &all_defined.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>())
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect();
            codes.push(Arc::new(Code29FunctionUndefined::new(
                func,
                positions,
                similar,
                config.severity(),
            )));
        }

        codes
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct Code29FunctionUndefined {
    name: String,
    usage: Vec<(Position, Mapping, Mapping, WorkspacePath)>,
    suggestions: Vec<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code29FunctionUndefined {
    fn ident(&self) -> &'static str {
        "L-S29"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#function_undefined")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!("Undefined Function: {}", self.name)
    }
    fn label_message(&self) -> String {
        "undefined function".to_string()
    }
    fn note(&self) -> Option<String> {
        if self.usage.len() == 1 {
            return None;
        }
        Some(format!("Used in:\n{}", self.usage
            .iter()
            .map(|s| {
                let s = &s.0;
                format!("{}:{}:{}", s.path(), s.start().line(), s.start().column())
            })
            .collect::<Vec<_>>()
            .join("\n")))
    }
    fn help(&self) -> Option<String> {
        if self.suggestions.is_empty() {
            None
        } else {
            Some(format!(
                "Did you mean: `{}`?",
                self.suggestions.join("`, `")
            ))
        }
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code29FunctionUndefined {
    #[must_use]
    pub fn new(name: String, usage: Vec<(Position, Mapping, Mapping, WorkspacePath)>, suggestions: Vec<String>, severity: Severity) -> Self {
        Self {
            name,
            usage,
            suggestions,
            severity,
            diagnostic: None,
        }
        .generate_processed()
    }

    fn generate_processed(mut self) -> Self {
        let mut diag = Diagnostic::from_code(&self);
        if let Some((_, map_start, map_end, map_file)) = self.usage.first() {
            diag.labels.push(
                Label::primary(
                    map_file.clone(),
                    map_start.original_start()..map_end.original_start(),
                )
                .with_message(self.label_message()),
            );
        }
        self.diagnostic = Some(diag);
        self
    }
}
