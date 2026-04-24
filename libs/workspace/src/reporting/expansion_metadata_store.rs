//! Expansion metadata storage and querying
//!
//! Tracks which portions of the output came from macro expansions,
//! enabling error reporting that can say "error came from expansion of macro X"

use std::collections::HashMap;
use std::ops::Range;

use crate::reporting::macro_expander::ExpansionMetadata;

/// Storage for macro expansion metadata indexed by output position ranges
#[derive(Debug, Default, Clone)]
pub struct ExpansionMetadataStore {
    /// Map from output character range to expansion metadata
    expansions: HashMap<Range<usize>, ExpansionMetadata>,
}

impl ExpansionMetadataStore {
    #[must_use]
    /// Create a new empty metadata store
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an expansion in the store
    ///
    /// The range is the character positions in the final output
    pub fn register(&mut self, range: Range<usize>, metadata: ExpansionMetadata) {
        self.expansions.insert(range, metadata);
    }

    #[must_use]
    /// Look up expansion metadata by output position
    ///
    /// Returns the metadata for any expansion that contains the given position
    pub fn get_at(&self, position: usize) -> Option<&ExpansionMetadata> {
        self.expansions
            .iter()
            .find(|(range, _)| range.contains(&position))
            .map(|(_, metadata)| metadata)
    }

    #[must_use]
    /// Get all expansions in a given range
    pub fn get_range(&self, range: &Range<usize>) -> Vec<(&Range<usize>, &ExpansionMetadata)> {
        self.expansions
            .iter()
            .filter(|(exp_range, _)| {
                // Overlapping ranges
                !(exp_range.end <= range.start || exp_range.start >= range.end)
            })
            .collect()
    }

    /// Iterate through all registr ed expansions
    pub fn iter(&self) -> impl Iterator<Item = (&Range<usize>, &ExpansionMetadata)> {
        self.expansions.iter()
    }

    #[must_use]
    /// Get count of registered expansions
    pub fn len(&self) -> usize {
        self.expansions.len()
    }

    #[must_use]
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.expansions.is_empty()
    }

    #[must_use]
    /// Get all expansions as a mapping
    pub const fn as_map(&self) -> &HashMap<Range<usize>, ExpansionMetadata> {
        &self.expansions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Workspace;
    use crate::position::{LineCol, Position};

    fn create_test_metadata(name: &str) -> ExpansionMetadata {
        let workspace = Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .expect("Failed to create workspace for test");
        let path = workspace
            .join("test.hpp")
            .expect("Failed to create test path");
        let pos = Position::new(LineCol(0, (1, 0)), LineCol(1, (1, 1)), path);
        ExpansionMetadata {
            macro_name: name.to_string(),
            definition_location: pos.clone(),
            definition_span: 0..name.len(),
            callsite: pos,
            argument_count: 0,
            expansion_stack: Vec::new(),
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut store = ExpansionMetadataStore::new();
        let metadata = create_test_metadata("TEST");

        store.register(0..10, metadata);
        assert!(store.get_at(5).is_some());
        assert!(store.get_at(15).is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_multiple_expansions() {
        let mut store = ExpansionMetadataStore::new();
        store.register(0..10, create_test_metadata("FIRST"));
        store.register(10..20, create_test_metadata("SECOND"));
        store.register(30..40, create_test_metadata("THIRD"));

        assert_eq!(store.len(), 3);
        assert_eq!(store.get_at(5).unwrap().macro_name, "FIRST");
        assert_eq!(store.get_at(15).unwrap().macro_name, "SECOND");
        assert_eq!(store.get_at(35).unwrap().macro_name, "THIRD");
    }

    #[test]
    fn test_overlapping_ranges() {
        let mut store = ExpansionMetadataStore::new();
        store.register(0..20, create_test_metadata("OUTER"));
        store.register(5..10, create_test_metadata("INNER"));

        let overlapping = store.get_range(&(7..12));
        assert_eq!(overlapping.len(), 2); // Both ranges overlap [7,12)
    }
}
