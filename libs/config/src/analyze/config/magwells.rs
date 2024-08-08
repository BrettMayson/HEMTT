use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::reporting::{Code, Processed};

use crate::{
    analyze::codes::cw2_magwell_missing_magazine::MagwellMissingMagazine, Class, Config, Item,
    Property, Str, Value,
};

pub fn missing_magazine(
    project: &ProjectConfig,
    config: &Config,
    processed: &Processed,
) -> Vec<Arc<dyn Code>> {
    let mut warnings: Vec<Arc<dyn Code>> = Vec::new();
    let mut classes = Vec::new();
    let Some(Property::Class(Class::Local {
        properties: magwells,
        ..
    })) = config
        .0
        .iter()
        .find(|p| p.name().value.to_lowercase() == "cfgmagazinewells")
    else {
        return warnings;
    };
    let Some(Property::Class(Class::Local {
        properties: magazines,
        ..
    })) = config
        .0
        .iter()
        .find(|p| p.name().value.to_lowercase() == "cfgmagazines")
    else {
        return warnings;
    };
    for property in magazines {
        if let Property::Class(Class::Local { name, .. }) = property {
            classes.push(name);
        }
    }
    for magwell in magwells {
        let Property::Class(Class::Local {
            properties: addons, ..
        }) = magwell
        else {
            continue;
        };
        for addon in addons {
            let Property::Entry {
                name,
                value: Value::Array(magazines),
                ..
            } = addon
            else {
                continue;
            };
            for mag in &magazines.items {
                let Item::Str(Str { value, span }) = mag else {
                    continue;
                };
                if !value
                    .to_lowercase()
                    .starts_with(&project.prefix().to_lowercase())
                {
                    continue;
                }
                if !classes.iter().any(|c| c.value == *value) {
                    warnings.push(Arc::new(MagwellMissingMagazine::new(
                        name.clone(),
                        span.clone(),
                        processed,
                    )));
                }
            }
        }
    }
    warnings
}
