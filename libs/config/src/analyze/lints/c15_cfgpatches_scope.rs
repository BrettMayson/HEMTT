use std::{cmp, ops::Range, sync::Arc};

use crate::{Class, Config, Item, Number, Property, Value, analyze::LintData};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use indexmap::IndexMap;

crate::analyze::lint!(LintC15CfgPatchesScope);

impl Lint<LintData> for LintC15CfgPatchesScope {
    fn ident(&self) -> &'static str {
        "cfgpatches_scope"
    }
    fn sort(&self) -> u32 {
        150
    }
    fn description(&self) -> &'static str {
        "Reports on CfgPatches entries that do not match public items in CfgVehicles and CfgWeapons. This ensures items are available in Zeus."
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
By default it will only check classnames that start with the project's prefix. Use `*` to check all.

```toml
[lints.config.cfgpatches_scope]
options.check_prefixes = ["abe", "abx"]

"#
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::warning().with_enabled(hemtt_common::config::LintEnabled::Pedantic)
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
        if let Some(toml::Value::Array(prefixes)) = config.option("check_prefixes") {
            for item in prefixes {
                if let toml::Value::String(s) = item {
                    check_prefixes.push(s.to_lowercase());
                }
            }
        } else if let Some(project) = project {
            check_prefixes.push(project.prefix().to_lowercase());
        }

        let (patch_units, patch_weapons) = get_patch_arrays(target);
        let all_vehicles = get_defined("cfgvehicles", target);
        let all_weapons = get_defined("cfgweapons", target);

        let accept_all = check_prefixes.iter().any(|p| p == "*");
        let fnc_should_check = |name: &str| accept_all || check_prefixes.iter().any(|p| name.starts_with(p));

        // Check for public items not listed in CfgPatches (that start with project's prefix)
        for (unit, (public, span, _, _)) in &all_vehicles {
            if *public && !patch_units.contains_key(unit) && fnc_should_check(unit) {
                codes.push(Arc::new(Code15CfgPatchPublicItemNotListed::new(
                    unit.clone(),
                    PatchType::Vehicle,
                    span.clone(),
                    processed,
                    config.severity(),
                )));
            }
        }
        for (weapon, (public, span, _, _)) in &all_weapons {
            if *public && !patch_weapons.contains_key(weapon) && fnc_should_check(weapon) {
                codes.push(Arc::new(Code15CfgPatchPublicItemNotListed::new(
                    weapon.clone(),
                    PatchType::Weapon,
                    span.clone(),
                    processed,
                    config.severity(),
                )));
            }
        }
        // Check for items in CfgPatches that are not defined in CfgVehicles/CfgWeapons
        for (unit, span) in &patch_units {
            if !all_vehicles.contains_key(unit) {
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
            if !all_weapons.contains_key(weapon) {
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

type DefinedMap = IndexMap<String, (bool, Range<usize>, Option<i32>, Option<i32>)>;
fn get_defined(base_path: &str, target: &Config) -> DefinedMap  {
    fn get_number(properties: &[Property], key: &str) -> Option<i32> {
        if let Some(property) = properties.iter().find(|p| p.name().value.eq_ignore_ascii_case(key))
            && let Property::Entry { value, .. } = property
            && let Value::Number(number) = value
            && let Number::Int32 { value, .. } = number
        {
            return Some(*value);
        }
        None
    }
    let mut defined: DefinedMap = IndexMap::new();
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
            let name_lower = name.as_str().to_ascii_lowercase();
            let (parent_scope, parent_scope_curator) = parent.as_ref().map_or((None, None), |parent| {
                let parent_lower = parent.as_str().to_ascii_lowercase();
                defined.get(&parent_lower).map_or((None, None), |parent_def| (parent_def.2, parent_def.3))
            });
            
            let cfg_scope = get_number(properties, "scope").or(parent_scope);
            let cfg_scope_curator = get_number(properties, "scopeCurator").or(parent_scope_curator);
            let public_known = cmp::max(cfg_scope.unwrap_or(0), cfg_scope_curator.unwrap_or(0)) > 1 && cfg_scope_curator.is_none_or(|c| c > 1);
            defined.insert(name_lower, (public_known, name.span(), cfg_scope, cfg_scope_curator));
        }
    }
    defined
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

macro_rules! define_cfgpatch_code {
    ($name:ident, $ident:expr, $message_fn:expr, $label_fn:expr) => {
        pub struct $name {
            classname: String,
            patch_type: PatchType,
            span: Range<usize>,
            severity: Severity,
            diagnostic: Option<Diagnostic>,
        }

        impl Code for $name {
            fn ident(&self) -> &'static str {
                $ident
            }
            fn link(&self) -> Option<&str> {
                Some("/lints/config.html#cfgpatches_scope")
            }
            fn severity(&self) -> Severity {
                self.severity
            }
            fn message(&self) -> String {
                $message_fn(&self.classname, &self.patch_type)
            }
            fn label_message(&self) -> String {
                $label_fn(&self.patch_type)
            }
            fn diagnostic(&self) -> Option<Diagnostic> {
                self.diagnostic.clone()
            }
        }

        impl $name {
            #[must_use]
            pub fn new(
                classname: String,
                patch_type: PatchType,
                span: Range<usize>,
                processed: &Processed,
                severity: Severity,
            ) -> Self {
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
                self.diagnostic =
                    Diagnostic::from_code_processed(&self, self.span.clone(), processed);
                self
            }
        }
    };
}

define_cfgpatch_code!(
    Code15CfgPatchItemNotFound,
    "L-C15-MISSING-CLASS",
    |classname: &str, patch_type: &PatchType| format!(
        "CfgPatches {}s[] class `{}` not found",
        patch_type.singular(),
        classname
    ),
    |patch_type: &PatchType| format!("not defined in {}", patch_type.base())
);

define_cfgpatch_code!(
    Code15CfgPatchPublicItemNotListed,
    "L-C15-NOT-IN-PATCHES",
    |classname: &str, patch_type: &PatchType| format!(
        "Public {} class `{}` not listed in CfgPatches",
        patch_type.base(),
        classname
    ),
    |patch_type: &PatchType| format!("has scope=2 but not in {}s[]", patch_type.singular())
);
