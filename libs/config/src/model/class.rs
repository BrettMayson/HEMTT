use crate::rapify::Rapify;

use super::{Entry, Ident};

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
        children: Children,
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
}

#[derive(Debug, Clone, PartialEq, Default)]
/// The list of children of a class
pub struct Children(pub Properties);

#[derive(Debug, Clone, PartialEq, Default)]
/// Properties of a class
pub struct Properties(pub Vec<(String, Property)>);

#[derive(Debug, Clone, PartialEq)]
/// A property of a class
pub enum Property {
    /// A property entry
    Entry(Entry),
    /// A sub-class
    Class(Class),
    /// A class deletion
    Delete(Ident),
}

impl Property {
    /// Get the code of the property
    #[must_use]
    pub fn property_code(&self) -> Vec<u8> {
        match self {
            Self::Entry(e) => match e {
                Entry::Str(s) => vec![1, s.rapified_code()],
                Entry::Number(n) => vec![1, n.rapified_code()],
                Entry::Array(a) => {
                    if a.expand {
                        vec![5, 1, 0, 0, 0]
                    } else {
                        vec![2]
                    }
                }
            },
            Self::Class(c) => match c {
                Class::Local { .. } => vec![0],
                Class::External { .. } => vec![3],
            },
            Self::Delete(_) => {
                vec![4]
            }
        }
    }
}
