// use crate::rapify::Rapify;

use crate::Property;

use super::Ident;

#[derive(Debug, Clone, PartialEq)]
/// A class definition
pub enum Class {
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
    pub const fn name(&self) -> &Ident {
        match self {
            Self::External { name } | Self::Local { name, .. } => name,
        }
    }

    #[must_use]
    /// Get the parent of the class
    pub const fn parent(&self) -> Option<&Ident> {
        match self {
            Self::External { .. } => None,
            Self::Local { parent, .. } => parent.as_ref(),
        }
    }
}
