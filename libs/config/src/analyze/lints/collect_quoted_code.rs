use std::sync::Arc;

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Processed},
};

use crate::{Class, Config, Item, Property, Str, Value, analyze::LintData};

crate::analyze::lint!(LintCollectQuotedCode);

impl Lint<LintData> for LintCollectQuotedCode {
    fn display(&self) -> bool {
        false
    }
    fn ident(&self) -> &'static str {
        "collect_quoted_code"
    }
    fn sort(&self) -> u32 {
        0
    }
    fn description(&self) -> &'static str {
        "collect_quoted_code"
    }
    fn documentation(&self) -> &'static str {
        r"This should not be visable"
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
    type Target = Config;
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Config,
        data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        let Some(project) = project else {
            return vec![];
        };
        let prefix = project.prefix().to_lowercase();
        for p in &target.0 {
            check_property(p, processed, data, get_ignores_generic, &prefix);
        }
        vec![]
    }
}

fn check_property(
    property: &Property,
    processed: &Processed,
    data: &LintData,
    fn_ignore: fn(&str) -> Option<&'static str>,
    prefix: &str,
) {
    match property {
        Property::Class(Class::Local { name, parent:_, properties, err_missing_braces:_ }) => {
            let Some(fn_ignore) = get_ignore_fn(name.as_str().to_lowercase().as_str(), fn_ignore) else {
                return;
            };
            for p in properties {
                check_property(p, processed, data, fn_ignore, prefix);
            }
        }
        Property::Entry { name, value:Value::Str(value), expected_array:_ } => {
            // based on entry name, see if there is something extra needs to be injected (special vars to ignore)
            // Some means the entry should always contain code 
            // None means unknown, so look for a func call
            let source_extra_inject = fn_ignore(name.as_str().to_lowercase().as_str());
            check_string(value, processed, data, prefix, source_extra_inject);
        }
        Property::Entry { name, value:Value::Array(array), expected_array:_ } => {
            let source_extra_inject = fn_ignore(name.as_str().to_lowercase().as_str());
            for item in &array.items {
                check_item(item, processed, data, prefix, source_extra_inject);
            }
        }
        _ => { }
    }
}
fn check_string(value: &Str, processed: &Processed, data: &LintData, prefix: &str, source_extra_inject: Option<&'static str>) {
    let value_lower = value.value().to_lowercase();
    if value_lower.is_empty() || value_lower.starts_with("http") || value_lower.starts_with("$str") {
        return
    }
    if !is_project_func(&value_lower, prefix) && source_extra_inject.is_none() 
        && !(value_lower.contains("_fnc_") && (value_lower.contains("call "))) {     
        return
    }
    let range = value.span().clone();
    let (output, _boundaries) = unescape_quoted(&processed.extract(&range));
    
    let processed = Arc::new(processed.select_sub_region(output, &range, source_extra_inject));
    data.quoted_code.lock().expect("mutex").push(processed);
}
fn check_item(item: &Item, processed: &Processed, data: &LintData, prefix: &str, source_extra_inject: Option<&'static str>) {
    match item {
        Item::Str(str) => {
            check_string(str, processed, data, prefix, source_extra_inject);
        }
        Item::Array(array) => {
            for item in array {
                check_item(item, processed, data, prefix, source_extra_inject);
            }
        }
        _ => {}
    }
}
#[must_use]
fn is_project_func(var_lower: &str, prefix: &str,) -> bool {
    var_lower.starts_with(prefix) && var_lower.contains("_fnc_")
}
#[must_use]
fn get_ignore_fn(classname: &str, current: fn(&str) -> Option<&'static str>) -> Option<fn(&str) -> Option<&'static str>> {
    match classname.to_lowercase().as_str() {
        // ACE Actions on CfgVehicles, Zeus at root
        "ace_actions" | "ace_selfactions" | "ace_zeusactions" | "ace_interaction_anims" => Some(get_ignores_cfgvehicles),
        // ace-like - https://github.com/zen-mod/ZEN/blob/master/addons/context_menu/
        "zen_context_menu_actions" => Some(get_ignores_zen),
        // 3den at root and Attributes on CfgVehicles
        "cfg3den" | "attributes" => Some(get_ignores_3den),
        // MFD has simple expressions that will fail (e.g. `speed`)
        "mfd" => None,
        _ => Some(current),
    }
}
#[must_use]
fn get_ignores_generic(name_lower: &str) -> Option<&'static str> {
    match name_lower {
        "action" | "ondraw" | "onmousebuttonup" | "onmousebuttondown" | "onunload" | "onload" | "onsetfocus" | "onkillfocus"
        | "onmouseenter" | "onmouseexit" | "onmousemoving" | "onmouseholding" | "onmousebuttondblclick" | "onmousebuttonclick" 
        | "onsliderposchanged" | "ontreedblclick"| "ontoolboxselchanged"
        | "onbuttonclick"  | "onlbdblclick" | "onlbselchanged" | "onkeyup" | "onkeydown" => Some(""),
        _ => None,
    }
}
#[must_use]
fn get_ignores_cfgvehicles(name_lower: &str) -> Option<&'static str> {
    match name_lower {
        "statement" | "condition" | "insertchildren" | "modifierfunction" => Some(r#"#pragma hemtt ignore_variables ["_target", "_player", "_actionParams"]"#),
        "position" | "positions" => Some(r#"#pragma hemtt ignore_variables ["_target"]"#),
        _ => get_ignores_generic(name_lower),
    }
}
#[must_use]
fn get_ignores_3den(name_lower: &str) -> Option<&'static str> {
    match name_lower {
        "attributeload" | "attributesave" | "expression" => Some(r#"#pragma hemtt ignore_variables ["_value"]"#),
        _ => get_ignores_generic(name_lower),
    }
}
#[must_use]
fn get_ignores_zen(name_lower: &str) -> Option<&'static str> {
    match name_lower {
        "statement" | "condition" | "insertchildren" | "modifierfunction" => Some(r#"#pragma hemtt ignore_variables ["_args", "_position", "_objects", "_groups", "_waypoints", "_markers", "_hoveredEntity"]"#),
        _ => get_ignores_generic(name_lower),
    }
}
#[must_use]
fn unescape_quoted(source: &str) -> (String, Vec<usize>) {
    let source_chars = source.chars().collect::<Vec<_>>();
    let quoted = source_chars.len() >= 2
        && source_chars.first() == Some(&'"')
        && source_chars.last() == Some(&'"');
    assert!(quoted, "quoted config string must start and end with a double quote");
    let content_start = 1;
    let content_end = source_chars.len() - 1;
    let mut boundaries = vec![0; source_chars.len() + 1];
    let mut output = String::new();
    let mut output_offset = 0;
    let mut source_offset = 0;
    while source_offset < source_chars.len() {
        boundaries[source_offset] = output_offset;
        let consumed = if source_offset < content_start || source_offset >= content_end {
            1
        } else if source_chars[source_offset] == '"'
            && source_offset + 1 < content_end
            && source_chars[source_offset + 1] == '"'
        {
            output.push('"');
            output_offset += 1;
            boundaries[source_offset + 1] = output_offset;
            2
        } else {
            output.push(source_chars[source_offset]);
            output_offset += 1;
            1
        };
        source_offset += consumed;
        boundaries[source_offset] = output_offset;
    }
    (output, boundaries)
}
