// use crate::rapify::Rapify;

use crate::Property;

use super::Ident;

#[derive(Debug, Clone, PartialEq)]
/// A class definition
pub enum Class {
    /// The root class definition
    Root {
        /// The children of the class
        properties: Vec<Property>,
    },
    /// A local class definition
    ///
    /// ```cpp
    /// class CfgPatches {
    ///     ...
    /// };
    /// ```
    Local {
        /// The name of the class
        name: Ident,
        /// The parent class
        ///
        /// ```cpp
        /// class MyClass: MyParent {
        ///    ...
        /// };
        /// ```
        parent: Option<Ident>,
        /// The children of the class
        properties: Vec<Property>,
        /// Was the class missing {}
        err_missing_braces: bool,
    },
    /// An external class definition
    ///
    /// ```cpp
    /// class CfgPatches;
    /// ```
    External {
        /// The name of the class
        name: Ident,
    },
}

impl Class {
    #[must_use]
    /// Get the name of the class
    pub const fn name(&self) -> Option<&Ident> {
        match self {
            Self::External { name } | Self::Local { name, .. } => Some(name),
            Self::Root { .. } => None,
        }
    }

    #[must_use]
    /// Get the parent of the class
    pub const fn parent(&self) -> Option<&Ident> {
        match self {
            Self::External { .. } | Self::Root { .. } => None,
            Self::Local { parent, .. } => parent.as_ref(),
        }
    }

    #[must_use]
    /// Get the properties of the class
    pub fn properties(&self) -> &[Property] {
        match self {
            Self::Root { properties } | Self::Local { properties, .. } => properties,
            Self::External { .. } => &[],
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Class {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            Self::Root { properties } => {
                let mut state = serializer.serialize_map(Some(properties.len()))?;
                for property in properties {
                    state.serialize_entry(property.name().as_str(), property)?;
                }
                state.end()
            }
            Self::Local {
                properties, parent, ..
            } => {
                // Account for __parent entry in map size when parent exists
                let size = properties.len() + usize::from(parent.is_some());
                let mut state = serializer.serialize_map(Some(size))?;

                if let Some(parent) = parent {
                    state.serialize_entry("__parent", parent.as_str())?;
                }
                for property in properties {
                    state.serialize_entry(property.name().as_str(), property)?;
                }
                state.end()
            }
            Self::External { .. } => {
                // External class declarations serialize to null
                serializer.serialize_none()
            }
        }
    }
}
