use hemtt_common::version::Version;

use crate::{analyze::CfgPatch, Class, Number, Property, Value};

#[derive(Debug, PartialEq)]
/// A config file
pub struct Config(pub Vec<Property>);

impl Config {
    #[must_use]
    pub fn to_class(&self) -> Class {
        Class::Root {
            properties: self.0.clone(),
        }
    }
}

impl Config {
    #[must_use]
    /// Get the patches
    pub fn get_patches(&self) -> Vec<CfgPatch> {
        let mut patches = Vec::new();
        for property in &self.0 {
            if let Property::Class(Class::Local {
                name, properties, ..
            }) = property
            {
                if name.as_str() == "CfgPatches" {
                    for patch in properties {
                        if let Property::Class(Class::Local {
                            name, properties, ..
                        }) = patch
                        {
                            let mut required_version = Version::new(0, 0, 0, None);
                            for property in properties {
                                if let Property::Entry { name, value, .. } = property {
                                    if name.as_str() == "requiredVersion" {
                                        if let Value::Number(Number::Float32 { value, .. }) = value
                                        {
                                            required_version = Version::from(*value);
                                        }
                                    }
                                }
                            }
                            patches.push(CfgPatch::new(name.clone(), required_version));
                        }
                    }
                }
            }
        }
        patches
    }
}
