use std::{collections::HashMap, ops::Range, sync::Arc};

use crate::{Class, Config, Item, Property, Value, analyze::LintData};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Label, Processed, Severity, get_span_info},
};

crate::analyze::lint!(LintC18CfgPatchesDuplicateClass);

impl Lint<LintData> for LintC18CfgPatchesDuplicateClass {
    fn ident(&self) -> &'static str {
        "cfgpatches_duplicate_class"
    }

    fn sort(&self) -> u32 {
        180
    }

    fn description(&self) -> &'static str {
        "Reports on CfgPatches entries that are duplicated in the same addon"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example
**Incorrect**
```hpp
class CfgPatches {
    class my_patch {
        units[] = {
            "MyVehicle",
            "MyVehicle"
        };
    };
};
```

**Correct**
```hpp
class CfgPatches {
    class my_patch {
        units[] = {
            "MyVehicle"
        };
    };
};

### Explanation
This lint checks for duplicate entries in the `units[]` and `weapons[]` arrays of `CfgPatches` classes. Duplicate entries won't cause errors in the game, but they are unnecessary and can be confusing."
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
        _project: Option<&ProjectConfig>,
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

        let (patch_units, patch_weapons) = get_patch_arrays(target);

        let mut seen: HashMap<String, Range<usize>> = HashMap::new();
        for (second_class, second_span) in &patch_units {
            if let Some(first_span) = seen.get(second_class) {
                codes.push(Arc::new(Code18CfgPatchesDuplicateClass::new(
                    second_class.clone(),
                    first_span.clone(),
                    second_span.clone(),
                    processed,
                    config.severity(),
                )));
            } else {
                seen.insert(second_class.clone(), second_span.clone());
            }
        }

        seen.clear();
        for (second_class, second_span) in &patch_weapons {
            if let Some(first_span) = seen.get(second_class) {
                codes.push(Arc::new(Code18CfgPatchesDuplicateClass::new(
                    second_class.clone(),
                    first_span.clone(),
                    second_span.clone(),
                    processed,
                    config.severity(),
                )));
            } else {
                seen.insert(second_class.clone(), second_span.clone());
            }
        }
        codes
    }
}

type PatchArray = Vec<(String, Range<usize>)>;
fn get_patch_arrays(target: &Config) -> (PatchArray, PatchArray) {
    fn get_array_property(key: &str, properties: &[Property]) -> Vec<(String, Range<usize>)> {
        let mut patch_classes = Vec::new();
        for property in properties {
            if let Property::Entry { name, value, .. } = property
                && name.as_str().eq_ignore_ascii_case(key)
                && let Value::Array(elements) = value
            {
                for item in &elements.items {
                    if let Item::Str(s) = item {
                        let key = s.value.to_ascii_lowercase();
                        patch_classes.push((key, s.span.clone()));
                    }
                }
            }
        }
        patch_classes
    }
    let mut patch_units = Vec::new();
    let mut patch_weapons = Vec::new();
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

#[allow(clippy::module_name_repetitions)]
pub struct Code18CfgPatchesDuplicateClass {
    severity: Severity,
    class: String,
    first: Range<usize>,
    second: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code18CfgPatchesDuplicateClass {
    fn ident(&self) -> &'static str {
        "L-C18"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/config.html#cfgpatches_duplicate_class")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!("class is duplicated: {}", self.class)
    }
    fn label_message(&self) -> String {
        "duplicate class".to_string()
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code18CfgPatchesDuplicateClass {
    #[must_use]
    pub fn new(
        class: String,
        first: Range<usize>,
        second: Range<usize>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            severity,
            class,
            first,
            second,
            diagnostic: None,
        }.generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(mut diag) = Diagnostic::from_code_processed(&self, self.second.clone(), processed)
        else {
            return self;
        };

        if let Some((path, span)) = get_span_info(&self.first, processed) {
            diag = diag.with_label(
                Label::secondary(path, span)
                    .with_message("first class here"),
            );
        }
        self.diagnostic = Some(diag);
        self
    }
}

