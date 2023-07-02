use std::collections::HashMap;

use hemtt_error::tokens::Token;
use strsim::levenshtein;

use crate::Definition;

/// `HashMap` of all current defines
pub type Defines = HashMap<String, (Token, Definition)>;

/// Helper functions for Defines
pub trait DefinitionLibrary {
    /// Find similar functions with a certain number of args
    ///
    /// Args can be Some(1) to only find macros that take 1 argument
    /// or None to check any number of arguments
    fn similar_function(&self, search: &str, args: Option<usize>) -> Vec<&str>;
    /// Find similar values
    ///
    /// This is used for error reporting
    fn similar_values(&self, search: &str) -> Vec<&str>;
}

impl DefinitionLibrary for Defines {
    fn similar_function(&self, search: &str, args: Option<usize>) -> Vec<&str> {
        let mut similar = self
            .iter()
            .filter(|(_, (_, def))| {
                let Definition::Function(func) = def else {
                    return false;
                };
                args.map_or(true, |args| func.parameters().len() == args)
            })
            .map(|(name, _)| (name.as_str(), levenshtein(name, search)))
            .collect::<Vec<_>>();
        similar.sort_by_key(|(_, v)| *v);
        similar.retain(|s| s.1 <= 3);
        if similar.len() > 3 {
            similar.truncate(3);
        }
        similar.into_iter().map(|(n, _)| n).collect::<Vec<_>>()
    }

    fn similar_values(&self, search: &str) -> Vec<&str> {
        let mut similar = self
            .iter()
            .filter(|(_, (_, def))| {
                let Definition::Value(_) = def else {
                    return false;
                };
                true
            })
            .map(|(name, _)| (name.as_str(), levenshtein(name, search)))
            .collect::<Vec<_>>();
        similar.sort_by_key(|(_, v)| *v);
        similar.retain(|s| s.1 <= 3);
        if similar.len() > 3 {
            similar.truncate(3);
        }
        similar.into_iter().map(|(n, _)| n).collect::<Vec<_>>()
    }
}
