use chumsky::span::Spanned;
use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Codes, Processed},
};

use crate::{analyze::LintData, Class, Config, Property, Value};

crate::analyze::lint!(LintColectCfgFunctions);

impl Lint<LintData> for LintColectCfgFunctions {
    fn display(&self) -> bool {
        false
    }

    fn ident(&self) -> &'static str {
        "collect_cfgfunctions"
    }

    fn sort(&self) -> u32 {
        0
    }

    fn description(&self) -> &'static str {
        "collect_cfgfunctions"
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
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        _processed: Option<&Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Config,
        data: &LintData,
    ) -> Codes {
        let Some(Property::Class(Spanned{ inner: Class::Local {
            properties: prefices_properties,
            ..
        }, ..})) = target
            .0
            .iter()
            .find(|p| p.name().is_some_and(|name| name.0.eq_ignore_ascii_case("cfgfunctions")))
            .map(|f| &f.inner)
        else {
            return Vec::new();
        };
        for prefix in prefices_properties {
            let Property::Class(Spanned{ inner: Class::Local { name: tag_name, properties: tag_properties, .. }, .. }) = &prefix.inner else { continue };
            let mut prefix_real = tag_name.as_str();
            for p in tag_properties {
                let Property::Entry { name, value, .. } = &p.inner else { continue };
                if !name.as_str().eq_ignore_ascii_case("tag") { continue; }
                let Value::Str(value) = &value.inner else { continue; };
                prefix_real = value.value();
            }
            for p in tag_properties {
                let Property::Class(Spanned{ inner: class, .. }) = &p.inner else { continue };
                let Class::Local { properties: properties_category, .. } = class else { continue };
                for function in properties_category {
                    let Property::Class(Spanned{ inner: func_class, .. }) = &function.inner else { continue };
                    let Some(class_name) = func_class.name() else { continue; }; 
                    let func_name = format!("{prefix_real}_fnc_{}",class_name.as_str());
                    let func_name_lower = func_name.to_lowercase();
                    let mut functions_defined = data.functions_defined.lock().expect("mutex safety");
                    functions_defined.insert((func_name_lower, func_name.into()));
                }
            }
        }
        Vec::new()
    }
}
