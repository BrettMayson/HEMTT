//! Macro expansion system
//!
//! Handles the complex logic of expanding macros while tracking nested expansion
//! history for error reporting and source mapping.

use crate::position::Position;

use super::macros::{MacroExpansionContext, MacroFrame};

/// Metadata captured when a macro is expanded
#[derive(Debug, Clone)]
pub struct ExpansionMetadata {
    /// Name of the macro that was expanded
    pub macro_name: String,

    /// Where the macro is defined
    pub definition_location: Position,

    /// Byte span of the macro name in the definition file
    pub definition_span: std::ops::Range<usize>,

    /// Where the macro was called
    pub callsite: Position,

    /// Number of arguments passed
    pub argument_count: usize,

    /// The full expansion stack at the time of expansion
    pub expansion_stack: Vec<MacroFrame>,
}

impl ExpansionMetadata {
    #[must_use]
    /// Create new expansion metadata
    pub const fn new(
        macro_name: String,
        definition_location: Position,
        definition_span: std::ops::Range<usize>,
        callsite: Position,
        argument_count: usize,
        expansion_stack: Vec<MacroFrame>,
    ) -> Self {
        Self {
            macro_name,
            definition_location,
            definition_span,
            callsite,
            argument_count,
            expansion_stack,
        }
    }

    #[must_use]
    /// Check if this expansion is nested inside another macro
    pub const fn is_nested(&self) -> bool {
        !self.expansion_stack.is_empty()
    }

    #[must_use]
    /// Get the nesting depth (0 = top-level, 1 = one level deep, etc.)
    pub const fn nesting_depth(&self) -> usize {
        self.expansion_stack.len()
    }

    #[must_use]
    /// Return a formatted representation for error messages
    pub fn format_for_error(&self) -> String {
        if self.is_nested() {
            format!(
                "expanded from macro '{}' (nested {} level{})",
                self.macro_name,
                self.nesting_depth(),
                if self.nesting_depth() == 1 { "" } else { "s" }
            )
        } else {
            format!("expanded from macro '{}'", self.macro_name)
        }
    }
}

/// Macro expansion handler
///
/// Responsible for:
/// - Tracking nested macro expansions
/// - Preventing infinite recursion
/// - Capturing expansion metadata
pub struct MacroExpander {
    expansion_context: MacroExpansionContext,
}

impl MacroExpander {
    #[must_use]
    /// Create a new macro expander
    pub fn new() -> Self {
        Self {
            expansion_context: MacroExpansionContext::new(),
        }
    }

    #[must_use]
    /// Get the current expansion depth
    pub const fn depth(&self) -> usize {
        self.expansion_context.depth()
    }

    #[must_use]
    /// Check if a macro is already being expanded (infinite recursion check)
    pub fn is_expanding(&self, name: &str) -> bool {
        self.expansion_context.is_expanding(name)
    }

    /// Push a macro expansion onto the stack
    ///
    /// Should be called before expanding a macro, paired with `pop()`
    pub fn push_expansion(
        &mut self,
        name: String,
        definition_location: Position,
        callsite: Position,
        argument_count: usize,
    ) {
        let frame = MacroFrame::new(name, definition_location, callsite, argument_count);
        self.expansion_context.push(frame);
    }

    /// Pop the current macro expansion from the stack
    ///
    /// Should be called after finishing expansion, paired with `push_expansion()`
    pub fn pop_expansion(&mut self) -> Option<MacroFrame> {
        self.expansion_context.pop()
    }

    #[must_use]
    /// Capture expansion metadata in the current state
    ///
    /// Returns `ExpansionMetadata` with the full expansion stack history
    pub fn capture_metadata(
        &self,
        macro_name: String,
        definition_location: Position,
        definition_span: std::ops::Range<usize>,
        callsite: Position,
        argument_count: usize,
    ) -> ExpansionMetadata {
        ExpansionMetadata::new(
            macro_name,
            definition_location,
            definition_span,
            callsite,
            argument_count,
            self.expansion_context.stack().to_vec(),
        )
    }

    #[must_use]
    /// Get the current macro frame without consuming it
    pub fn current_frame(&self) -> Option<&MacroFrame> {
        self.expansion_context.current()
    }

    #[must_use]
    /// Get the maximum nesting depth seen
    pub const fn max_depth(&self) -> usize {
        self.expansion_context.max_depth()
    }

    /// Clear the expansion stack (for error recovery)
    pub fn clear_stack(&mut self) {
        self.expansion_context.clear();
    }

    #[must_use]
    /// Get a formatted stack trace for debugging
    pub fn format_stack(&self) -> String {
        self.expansion_context.format_stack()
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::Workspace;

    use super::*;

    fn make_position(line: usize) -> Position {
        let workspace = Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .expect("Failed to create workspace for test");
        let path = workspace
            .join("test.hpp")
            .expect("Failed to create test path");
        Position::new(
            crate::position::LineCol(line, (1, line)),
            crate::position::LineCol(line + 1, (1, line + 1)),
            path,
        )
    }

    #[test]
    fn test_macro_expander_push_pop() {
        let mut expander = MacroExpander::new();
        assert_eq!(expander.depth(), 0);

        expander.push_expansion("MACRO1".to_string(), make_position(1), make_position(5), 0);
        assert_eq!(expander.depth(), 1);
        assert!(expander.is_expanding("MACRO1"));

        expander.push_expansion("MACRO2".to_string(), make_position(2), make_position(6), 1);
        assert_eq!(expander.depth(), 2);

        let frame = expander.pop_expansion();
        assert!(frame.is_some());
        assert_eq!(expander.depth(), 1);

        expander.pop_expansion();
        assert_eq!(expander.depth(), 0);
    }

    #[test]
    fn test_is_expanding() {
        let mut expander = MacroExpander::new();
        assert!(!expander.is_expanding("MACRO"));

        expander.push_expansion("MACRO".to_string(), make_position(1), make_position(2), 0);
        assert!(expander.is_expanding("MACRO"));
        assert!(!expander.is_expanding("OTHER"));
    }

    #[test]
    fn test_capture_metadata() {
        let mut expander = MacroExpander::new();
        expander.push_expansion("OUTER".to_string(), make_position(1), make_position(10), 1);

        let metadata = expander.capture_metadata(
            "INNER".to_string(),
            make_position(2),
            0..5,
            make_position(11),
            1,
        );

        assert_eq!(metadata.macro_name, "INNER");
        assert!(metadata.is_nested());
        assert_eq!(metadata.nesting_depth(), 1);
    }

    #[test]
    fn test_expansion_metadata_format() {
        let metadata = ExpansionMetadata::new(
            "TEST".to_string(),
            make_position(1),
            0..4,
            make_position(5),
            0,
            vec![],
        );

        let formatted = metadata.format_for_error();
        assert!(formatted.contains("TEST"));
        assert_eq!(metadata.nesting_depth(), 0);
    }
}
