use std::{ops::Range, sync::Arc};

use crate::{Class, Config, Item, Number, Property, Value, analyze::LintData};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use indexmap::{IndexMap, IndexSet};

crate::analyze::lint!(LintC15CfgPatchesScope);

impl Lint<LintData> for LintC15CfgPatchesScope {
    fn ident(&self) -> &'static str {
        "cfgpatches_scope"
    }
    fn sort(&self) -> u32 {
        150
    }
    fn description(&self) -> &'static str {
        "Reports on CfgPatches entries that do not match public items in CfgVehicles and CfgWeapons"
    }
    fn documentation(&self) -> &'static str {
        r#"### Example
**Incorrect**
```hpp
class CfgPatches {
    class my_patch {
        units[] = { "MissingVehicle" }; // Does not exist in CfgVehicles
    };
};
class CfgVehicles {
    class MyVehicle { scope = 2; }; // Not in CfgPatches's units[]
};
### Configuration

- **check_prefixes**: only consider classes that start with any of these prefixes
By default it will only check the project's prefix. Use `*` to check all classes.

```toml
[lints.config.cfgpatches_scope]
options.check_prefixes = ["abe", "abx"]

"#
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
        config: &LintConfig,
        processed: Option<&Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Config,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let mut codes: Codes = Vec::new();

        let mut check_prefixes = Vec::new();
        if let Some(toml::Value::Array(ignore)) = config.option("check_prefixes") {
            for item in ignore {
                if let toml::Value::String(s) = item {
                    check_prefixes.push(s.to_lowercase());
                }
            }
        } else if let Some(project) = project {
            check_prefixes.push(project.prefix().to_lowercase());
        }

        let (patch_units, patch_weapons) = get_patch_arrays(target);
        let (all_vehicles, public_vehicles) = get_defined("cfgvehicles", target);
        let (all_weapons, public_weapons) = get_defined("cfgweapons", target);

        // Check for public items not listed in CfgPatches (that start with our prefix)
        for (unit, span) in &public_vehicles {
            if !patch_units.contains_key(unit) && check_prefixes.iter().any(|p| p == "*" || unit.starts_with(p)) {
                codes.push(Arc::new(Code15CfgPatchPublicItemNotListed::new(
                    unit.clone(),
                    PatchType::Vehicle,
                    span.clone(),
                    processed,
                    config.severity(),
                )));
            }
        }
        for (weapon, span) in &public_weapons {
            if !patch_weapons.contains_key(weapon) && check_prefixes.iter().any(|p| p == "*" ||weapon.starts_with(p)) {
                codes.push(Arc::new(Code15CfgPatchPublicItemNotListed::new(
                    weapon.clone(),
                    PatchType::Weapon,
                    span.clone(),
                    processed,
                    config.severity(),
                )));
            }
        }
        // Check for CfgPatches items not defined
        for (unit, span) in &patch_units {
            if !all_vehicles.contains(unit) {
                codes.push(Arc::new(Code15CfgPatchItemNotFound::new(
                    unit.clone(),
                    PatchType::Vehicle,
                    span.clone(),
                    processed,
                    config.severity(),
                )));
            }
        }
        for (weapon, span) in &patch_weapons {
            if !all_weapons.contains(weapon) {
                codes.push(Arc::new(Code15CfgPatchItemNotFound::new(
                    weapon.clone(),
                    PatchType::Weapon,
                    span.clone(),
                    processed,
                    config.severity(),
                )));
            }
        }
        codes
    }
}

fn get_defined(base_path: &str, target: &Config) -> (IndexSet<String>, IndexMap<String, Range<usize>>) {
    fn get_number(properties: &[Property], key: &str, default: i32) -> i32 {
        if let Some(property) = properties.iter().find(|p| p.name().value.eq_ignore_ascii_case(key))
            && let Property::Entry { value, .. } = property
            && let Value::Number(number) = value
            && let Number::Int32 { value, .. } = number
        {
            return *value;
        }
        default
    }
    let mut set_exist = IndexSet::new();
    let mut map_public = IndexMap::new();
    if let Some(Property::Class(Class::Local { properties, .. })) =
        target.0.iter().find(|p| p.name().value.eq_ignore_ascii_case(base_path))
    {
        for class in properties {
            let Property::Class(Class::Local {
                name, parent, properties, ..
            }) = class
            else {
                continue;
            };
            let lower_name = name.as_str().to_ascii_lowercase();
            set_exist.insert(lower_name.clone());
            let scope = get_number(properties, "scope", -1);
            let inherited = parent.as_ref().is_some_and(|parent| {
                let key = parent.as_str().to_ascii_lowercase();
                map_public.contains_key(&key)
            });
            if scope == 2 || (inherited && scope != 1) {
                map_public.insert(lower_name, name.span());
            }
        }
    }
    (set_exist, map_public)
}

fn get_patch_arrays(target: &Config) -> (IndexMap<String, Range<usize>>, IndexMap<String, Range<usize>>) {
    fn get_array_property(key: &str, properties: &[Property]) -> IndexMap<String, Range<usize>> {
        let mut patch_classes = IndexMap::new();
        for property in properties {
            if let Property::Entry { name, value, .. } = property
                && name.as_str().eq_ignore_ascii_case(key)
                && let Value::Array(elements) = value
            {
                for item in &elements.items {
                    if let Item::Str(s) = item {
                        let key = s.value.to_ascii_lowercase();
                        patch_classes.insert(key, s.span.clone());
                    }
                }
            }
        }
        patch_classes
    }
    let mut patch_units = IndexMap::new();
    let mut patch_weapons = IndexMap::new();
    if let Some(Property::Class(Class::Local { properties, .. })) =
        target.0.iter().find(|p| p.name().value.eq_ignore_ascii_case("cfgpatches"))
    {
        for patch in properties {
            let Property::Class(Class::Local { properties, .. }) = patch else {
                continue;
            };
            patch_units.extend(get_array_property("units", properties));
            patch_weapons.extend(get_array_property("weapons", properties));
        }
    }
    (patch_units, patch_weapons)
}

pub enum PatchType {
    Vehicle,
    Weapon,
}
impl PatchType {
    const fn singular(&self) -> &str {
        match self {
            Self::Vehicle => "unit",
            Self::Weapon => "weapon",
        }
    }
    const fn base(&self) -> &str {
        match self {
            Self::Vehicle => "CfgVehicles",
            Self::Weapon => "CfgWeapons",
        }
    }
}

pub struct Code15CfgPatchItemNotFound {
    classname: String,
    patch_type: PatchType,
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code15CfgPatchItemNotFound {
    fn ident(&self) -> &'static str {
        "L-C15a"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/config.html#cfgpatches_scope")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!("CfgPatches {}s[] class `{}` not found", self.patch_type.singular(), self.classname)
    }
    fn label_message(&self) -> String {
        format!("not defined in {}", self.patch_type.base())
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code15CfgPatchItemNotFound {
    #[must_use]
    pub fn new(classname: String, patch_type: PatchType, span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            classname,
            patch_type,
            span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }
    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
pub struct Code15CfgPatchPublicItemNotListed {
    classname: String,
    patch_type: PatchType,
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code15CfgPatchPublicItemNotListed {
    fn ident(&self) -> &'static str {
        "L-C15b"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/config.html#cfgpatches_scope")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!(
            "Public {} class `{}` not listed in CfgPatches",
            self.patch_type.base(),
            self.classname
        )
    }
    fn label_message(&self) -> String {
        format!("has scope=2 but not in {}s[]", self.patch_type.singular())
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code15CfgPatchPublicItemNotListed {
    #[must_use]
    pub fn new(classname: String, patch_type: PatchType, span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            classname,
            patch_type,
            span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }
    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
