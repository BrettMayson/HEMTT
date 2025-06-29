// use crate::rapify::Rapify;

use crate::Property;

use super::Ident;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
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
