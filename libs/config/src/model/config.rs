use hemtt_common::version::Version;

use crate::{Class, Number, Property, Value, analyze::CfgPatch, model::property::PropertyVecExt};

#[derive(Clone, Debug, PartialEq)]
/// A config file
pub struct Config(pub Vec<Property>);

impl Config {
    #[must_use]
    pub fn to_class(&self) -> Class {
        Class::Root {
            properties: self.0.clone(),
        }
    }

    #[must_use]
    /// Get the patches
    pub fn get_patches(&self) -> Vec<CfgPatch> {
        let mut patches = Vec::new();
        for property in &self.0 {
            if let Property::Class(Class::Local {
                name, properties, ..
            }) = property
                && name.as_str().eq_ignore_ascii_case("cfgpatches")
            {
                for patch in properties {
                    if let Property::Class(Class::Local {
                        name, properties, ..
                    }) = patch
                    {
                        let mut required_version = Version::new(0, 0, 0, None);
                        for property in properties {
                            if let Property::Entry { name, value, .. } = property
                                && name.as_str().eq_ignore_ascii_case("requiredversion")
                                && let Value::Number(Number::Float32 { value, .. }) = value
                            {
                                required_version = Version::from(*value);
                            }
                        }
                        patches.push(CfgPatch::new(name.clone(), required_version));
                    }
                }
            }
        }
        patches
    }

    pub fn auto_units(&mut self, prefixes: &[&str]) -> usize {
        self.auto_add("CfgVehicles", "units", prefixes)
    }

    pub fn auto_weapons(&mut self, prefixes: &[&str]) -> usize {
        self.auto_add("CfgWeapons", "weapons", prefixes)
    }

    fn auto_add(&mut self, source_cfg: &str, target: &str, prefixes: &[&str]) -> usize {
        let mut classes_to_add = Vec::new();
        let Some(Property::Class(Class::Local {
            properties: cfg_vehicles,
            ..
        })) = self.0.child(source_cfg)
        else {
            tracing::debug!("{} not found", source_cfg);
            return 0;
        };
        let prefixes = prefixes
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>();
        for vehicle in cfg_vehicles {
            if let Property::Class(Class::Local { name, .. }) = vehicle
                && prefixes
                    .iter()
                    .any(|prefix| name.as_str().to_lowercase().starts_with(prefix))
            {
                classes_to_add.push(name.clone());
            }
        }
        let Some(Property::Class(Class::Local {
            properties: cfg_patches,
            ..
        })) = self.0.child_mut("CfgPatches")
        else {
            tracing::debug!("CfgPatches not found");
            return 0;
        };
        if cfg_patches.len() != 1 {
            tracing::debug!("Expected exactly one CfgPatch, found {}", cfg_patches.len());
            return 0;
        }
        let Some(Property::Class(Class::Local {
            properties: cfg_patch,
            ..
        })) = cfg_patches.first_mut()
        else {
            return 0;
        };
        let Some(Property::Entry { value, .. }) = cfg_patch.child_mut(target) else {
            tracing::debug!("{} not found in CfgPatches", target);
            return 0;
        };
        let len = classes_to_add.len();
        if let Value::Array(arr) = value {
            for unit in classes_to_add {
                arr.items.push(super::Item::Str(unit.to_str()));
            }
        }
        len
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_class().serialize(serializer)
    }
}
