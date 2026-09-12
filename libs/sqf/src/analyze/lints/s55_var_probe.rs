use std::{
    collections::HashSet, vec
};

use hemtt_common::{config::{LintConfig, ProjectConfig}};
use hemtt_workspace::{
     addons::{Addon, VarUsed}, lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Codes}
};

use crate::{BinaryCommand::{self}, Expression, NularCommand, Statement, UnaryCommand, analyze::LintData};

crate::analyze::lint!(LintS55VarProbe);

impl Lint<LintData> for LintS55VarProbe {
    fn ident(&self) -> &'static str {
        "var_probe"
    }
    fn sort(&self) -> u32 {
        550
    }
    fn description(&self) -> &'static str {
        "Reports on undefined variables using the project's prefix"
    }
    fn documentation(&self) -> &'static str {
        r#"### Configuration

- **ignore**: Functions to ignore

```toml
[lints.sqf.var_probe]
options.ignore = [
    "myproject_fnc_piano", // todo?
]
```"#
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![
            Box::new(RunnerStatement),
            Box::new(RunnerExpression),
            Box::new(RunnerFinal),
        ]
    }
}
/// does the variable belong to the project
fn is_project_var(var_lower: &str, project: &ProjectConfig) -> bool {
    let prefix = project.prefix().to_lowercase();
    var_lower.starts_with(&prefix)
}
/// Gets var name for setVariable/getVariable
fn xvar_extract_name(expr: &Expression, project: &ProjectConfig) -> Option<String> {
    let var_lower = match expr {
        Expression::String(var, _, _) => var.to_lowercase(),
        Expression::Array(arr, _) => {
            let Some(Expression::String(var, _, _)) = arr.first() else {
                return None;
            };
            var.to_lowercase()
        }
        _ => return None,
    };
    if !is_project_var(&var_lower, project) {
        return None;
    }
    Some(var_lower)
}
fn is_mission_namespace(expr: &Expression) -> Option<bool> {
    if let Expression::NularCommand(NularCommand { name }, _) = expr {
        match name.to_lowercase().as_str() {
            "uinamespace" => return None,
            "missionnamespace" => return Some(true),
            _ => {}
        }
    }
    Some(false)
}


/// Runner for Statement (Simple Var Assignment)
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
        let Statement::AssignGlobal(var, _, _) = target else {
            return Vec::new();
        };
        let var_lower = var.to_lowercase();
        if is_project_var(&var_lower, project) {
            data.variables_used.lock().expect("mutex").push(VarUsed::MissionAssignemnt(var_lower));
        }
        vec![]
    }
}


/// Runner for Expression (Various)
struct RunnerExpression;
impl LintRunner<LintData> for RunnerExpression {
    type Target = crate::Expression;

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
        let result = match target {
            Expression::Variable(var, _span) => {
                let var_lower = var.to_lowercase();
                if !is_project_var(&var_lower, project) {
                    return vec![];
                }
                VarUsed::MissionAccess(var_lower)
            }
            Expression::UnaryCommand(UnaryCommand::Named(ucmd), rhs, _span) => {
                if !ucmd.eq_ignore_ascii_case("isNil") {
                    return vec![];
                }
                let Expression::String(var, _, _) = rhs.as_ref() else {
                    return vec![];
                };
                let var_lower = var.to_lowercase();
                if !is_project_var(&var_lower, project) {
                    return vec![];
                }
                VarUsed::MissionAccessWeak(var_lower)
            }
            Expression::BinaryCommand(BinaryCommand::Named(bcmd), lhs, rhs, _span) => {
                // fully ignore uiNamespace as most will be set from gui configs
                let Some(lhs_is_mission) = is_mission_namespace(lhs.as_ref()) else {
                    return vec![];
                };
                match bcmd.to_lowercase().as_str() {
                    "getvariable" => {
                        let Some(var_lower) = xvar_extract_name(rhs.as_ref(), project) else {
                            return vec![];
                        };
                        if lhs_is_mission {
                            VarUsed::MissionAccessWeak(var_lower)
                        } else {
                            VarUsed::ThingAccessWeak(var_lower)
                        }
                    }
                    "setvariable" => {
                        let Some(var_lower) = xvar_extract_name(rhs.as_ref(), project) else {
                            return vec![];
                        };
                        if lhs_is_mission {
                            VarUsed::MissionAssignemnt(var_lower)
                        } else {
                            VarUsed::ThingAssignemnt(var_lower)
                        }
                    }
                    "call" => {
                        let Expression::Variable(func, _span) = rhs.as_ref() else {
                            return vec![];
                        };
                        match func.to_lowercase().as_str() {
                            "cba_fnc_addsetting" | "cba_settings_fnc_init" => {
                                let Expression::Array(func_args, _span) = lhs.as_ref() else {
                                    return vec![];
                                };
                                let Some(Expression::String(var, _, _)) = func_args.first() else {
                                    return vec![];
                                };
                                let var_lower = var.to_lowercase();
                                if !is_project_var(&var_lower, project) {
                                    return vec![];
                                }
                                VarUsed::MissionAssignemnt(var_lower)
                            }
                            _ => {
                                return vec![];
                            }
                        }
                    }
                    "addpublicvariableeventhandler" => {
                        let Expression::String(var, _, _) = lhs.as_ref() else {
                            return vec![];
                        };
                        let var_lower = var.to_lowercase();
                        if !is_project_var(&var_lower, project) {
                            return vec![];
                        }
                        VarUsed::MissionAccessWeak(var_lower)
                    }
                    _ => {
                        return vec![];
                    }
                }
            }
            _ => {
                return vec![];
            }
        };
        data.variables_used.lock().expect("mutex").push(result);
        vec![]
    }
}

/// Runner for finale during `pre_build`
struct RunnerFinal;
impl LintRunner<LintData> for RunnerFinal {
    type Target = Vec<Addon>;

    fn run(
        &self,
        project: Option<&hemtt_common::config::ProjectConfig>,
        _config: &LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        if runtime.is_just() { // --just build will be missing full picture
            return vec![];
        }
        let Some(project) = project else {
            return Vec::new();
        };
        let mut all_addons = HashSet::new();
        let mut mission_defined = HashSet::new();
        let mut mission_used = HashSet::new();
        let mut mission_used_weak = HashSet::new();
        let mut thing_defined = HashSet::new();
        let mut thing_used_weak = HashSet::new();
        for addon in target {
            mission_defined.extend(addon.build_data().functions_defined().lock().expect("mutex").iter().map(|(name, _)| name.to_lowercase()));
            all_addons.insert(format!("{}_{}", project.prefix().to_lowercase(), addon.name().to_lowercase()));
            let var_usage = addon
                .build_data()
                .variables_used()
                .lock()
                .expect("mutex not poisoned").clone();
            for gv in &var_usage {
                match gv {
                    VarUsed::MissionAssignemnt(var) => {
                        mission_defined.insert(var.to_owned());
                    }
                    VarUsed::MissionAccess(var) =>{
                        mission_used.insert(var.to_owned());
                    }
                    VarUsed::MissionAccessWeak(var) =>{
                        mission_used_weak.insert(var.to_owned());
                    }
                    VarUsed::ThingAssignemnt(var) => {
                        thing_defined.insert(var.to_owned());
                    }
                    VarUsed::ThingAccessWeak(var) =>{
                        thing_used_weak.insert(var.to_owned());
                    }
                }
            }          
        }
        mission_used_weak = &mission_used_weak - &mission_used;
        let mut mission_unused = mission_defined.iter().filter(|eh| 
            !eh.contains("_fnc_") && !mission_used.contains(*eh) && !mission_used_weak.contains(*eh) && !all_addons.contains(*eh)
        ).collect::<Vec<_>>();
        let mut mission_undefined_weak = mission_used_weak.iter().filter(|eh| !mission_defined.contains(*eh)).collect::<Vec<_>>();
        let mut mission_undefined = mission_used.iter().filter(|eh| !mission_defined.contains(*eh)).collect::<Vec<_>>();
        mission_unused.sort_unstable();
        mission_undefined_weak.sort_unstable();
        mission_undefined.sort_unstable();
        for eh in &mission_unused {
            println!("[Mission] Not Used: {eh}");
        }
        for eh in &mission_undefined_weak {
            println!("[Mission] Undefined Weak: {eh}");
        }
        for eh in &mission_undefined {
            println!("[Mission] Undefined: {eh}");
        }
        let mut thing_unused = thing_defined.iter().filter(|eh| !thing_used_weak.contains(*eh)).collect::<Vec<_>>();
        let mut thing_undefined_weak = thing_used_weak.iter().filter(|eh| !thing_defined.contains(*eh)).collect::<Vec<_>>();
        thing_unused.sort_unstable();
        thing_undefined_weak.sort_unstable();
        for eh in &thing_unused {
            println!("[Thing] Not Used: {eh}");
        }
        for eh in &thing_undefined_weak {
            println!("[Thing] Undefined Weak: {eh}");
        }
        vec![]
    }
}

// maybe dump to a txt like unused_stringtables?
