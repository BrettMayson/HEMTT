/// Context structures for preprocessing
///
/// Organizes state and configuration into logical units:
/// - PreprocessingContext: Global preprocessing state
/// - LineContext: State for processing a single line
/// - MacroExpansionContext: Tracks nested macro expansions

use hemtt_workspace::WorkspacePath;
use hemtt_workspace::position::Position;

#[cfg(feature = "lsp")]
use std::collections::HashMap;

use crate::defines::Defines;
use crate::ifstate::IfStates;
use crate::processor::pragma::Pragma;

/// Global preprocessing context holding all permanent state
#[derive(Default)]
pub struct PreprocessingContext {
    /// All macro definitions
    pub defines: Defines,

    /// Conditional compilation state stack
    pub if_states: IfStates,

    /// Stack of files being processed (for include tracking)
    pub file_stack: Vec<WorkspacePath>,

    /// All files that were included
    pub included_files: Vec<WorkspacePath>,

    /// Macro call site tracking for LSP
    #[cfg(feature = "lsp")]
    pub declarations: HashMap<Position, Position>,

    #[cfg(feature = "lsp")]
    pub usage: HashMap<Position, Vec<Position>>,

    /// Flag to skip rapification if preprocessor has warnings/issues
    pub no_rapify: bool,

    /// Number of tokens processed so far
    pub token_count: usize,

    /// Number of consecutive backslashes (for line continuation)
    pub backslashes: usize,
}

impl PreprocessingContext {
    /// Create a new preprocessing context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an include to the tracking lists
    pub fn add_include(&mut self, path: WorkspacePath) -> Result<(), crate::Error> {
        use crate::codes::pe29_circular_include::CircularInclude;

        if self.file_stack.contains(&path) {
            return Err(CircularInclude::code(Vec::new(), self.file_stack.clone()));
        }
        self.file_stack.push(path.clone());
        self.included_files.push(path);
        Ok(())
    }

    /// Check if we're at the top level (root file)
    pub fn is_root(&self) -> bool {
        self.file_stack.len() <= 1
    }

    /// Get the current file being processed
    pub fn current_file(&self) -> Option<&WorkspacePath> {
        self.file_stack.last()
    }
}

/// State for processing a single line of input
#[derive(Debug, Clone)]
pub struct LineContext {
    /// Pragma settings that apply to this line
    pub pragma: Pragma,

    /// Whether we're at the start of a line (before first non-whitespace)
    pub at_line_start: bool,

    /// Whether we're inside quotes (prevents directive processing)
    pub in_quotes: bool,

    /// Type of quote we're in ('"' or '\'')
    pub quote_char: Option<char>,
}

impl LineContext {
    /// Create a new line context
    pub fn new(pragma: Pragma) -> Self {
        Self {
            pragma,
            at_line_start: true,
            in_quotes: false,
            quote_char: None,
        }
    }

    /// Enter into a quote
    pub fn enter_quote(&mut self, quote_char: char) {
        self.in_quotes = true;
        self.quote_char = Some(quote_char);
    }

    /// Exit the current quote
    pub fn exit_quote(&mut self) {
        self.in_quotes = false;
        self.quote_char = None;
    }

    /// Mark that we've seen content (not just whitespace)
    pub fn mark_content_seen(&mut self) {
        self.at_line_start = false;
    }

    /// Reset for the next line
    pub fn reset_line(&mut self, new_pragma: Pragma) {
        self.pragma = new_pragma;
        self.at_line_start = true;
        self.in_quotes = false;
        self.quote_char = None;
    }
}

/// Information about a macro expansion in the stack
#[derive(Debug, Clone)]
pub struct MacroFrame {
    /// Name of the macro being expanded
    pub name: String,

    /// Where the macro is defined
    pub definition_location: Position,

    /// Where the macro is being called from
    pub callsite: Position,

    /// Arguments passed to this macro (if function-like)
    pub argument_count: usize,
}

impl MacroFrame {
    /// Create a new macro frame
    pub fn new(
        name: String,
        definition_location: Position,
        callsite: Position,
        argument_count: usize,
    ) -> Self {
        Self {
            name,
            definition_location,
            callsite,
            argument_count,
        }
    }

    /// Get the nesting depth (1 = top-level, 2 = nested one level, etc.)
    pub fn depth(&self, total_stack_len: usize) -> usize {
        total_stack_len
    }
}

/// Tracks nested macro expansions
#[derive(Default, Debug, Clone)]
pub struct MacroExpansionContext {
    /// Stack of macro expansions (innermost at the end)
    stack: Vec<MacroFrame>,

    /// Maximum nesting depth detected
    max_depth: usize,

    /// Total number of expansions processed
    total_expansions: usize,
}

impl MacroExpansionContext {
    /// Create a new expansion context
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a macro expansion onto the stack
    pub fn push(&mut self, frame: MacroFrame) {
        self.total_expansions += 1;
        let new_depth = self.stack.len() + 1;
        if new_depth > self.max_depth {
            self.max_depth = new_depth;
        }
        self.stack.push(frame);
    }

    /// Pop the current macro expansion from the stack
    pub fn pop(&mut self) -> Option<MacroFrame> {
        self.stack.pop()
    }

    /// Get the current expansion depth (0 = not in macro)
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Get the current macro frame without consuming it
    pub fn current(&self) -> Option<&MacroFrame> {
        self.stack.last()
    }

    /// Get the full stack
    pub fn stack(&self) -> &[MacroFrame] {
        &self.stack
    }

    /// Check if we're currently inside a macro expansion
    pub fn in_expansion(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Check if a macro is already being expanded (prevent infinite recursion)
    pub fn is_expanding(&self, macro_name: &str) -> bool {
        self.stack.iter().any(|frame| frame.name == macro_name)
    }

    /// Get the maximum nesting depth seen
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Get the total number of expansions processed
    pub fn total_expansions(&self) -> usize {
        self.total_expansions
    }

    /// Clear the stack (for error recovery or cleanup)
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Get a formatted representation of the expansion stack for debugging
    pub fn format_stack(&self) -> String {
        if self.stack.is_empty() {
            return "(not in macro)".to_string();
        }

        self.stack
            .iter()
            .enumerate()
            .map(|(idx, frame)| {
                format!(
                    "{}. {} (called from {}:{})",
                    idx,
                    frame.name,
                    frame.callsite.path().as_str(),
                    frame.callsite.start().line()
                )
            })
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessing_context() {
        let ctx = PreprocessingContext::new();
        assert!(ctx.is_root());
        assert_eq!(ctx.token_count, 0);
        assert_eq!(ctx.included_files.len(), 0);
    }

    #[test]
    fn test_line_context() {
        let pragma = Pragma::root();
        let mut ctx = LineContext::new(pragma.clone());

        assert!(ctx.at_line_start);
        assert!(!ctx.in_quotes);

        ctx.mark_content_seen();
        assert!(!ctx.at_line_start);

        ctx.enter_quote('"');
        assert!(ctx.in_quotes);
        assert_eq!(ctx.quote_char, Some('"'));

        ctx.exit_quote();
        assert!(!ctx.in_quotes);
    }

    #[test]
    fn test_macro_expansion_context() {
        let workspace = hemtt_workspace::Workspace::builder()
            .memory()
            .finish(
                None,
                false,
                &hemtt_common::config::PDriveOption::Disallow,
            )
            .unwrap();
        let path = workspace.join("test.hpp").unwrap();
        let pos = hemtt_workspace::position::Position::new(
            hemtt_workspace::position::LineCol(0, (1, 0)),
            hemtt_workspace::position::LineCol(1, (1, 1)),
            path.clone(),
        );

        let mut ctx = MacroExpansionContext::new();
        assert!(!ctx.in_expansion());
        assert_eq!(ctx.depth(), 0);

        let frame1 = MacroFrame::new("MACRO1".to_string(), pos.clone(), pos.clone(), 0);
        ctx.push(frame1);
        assert!(ctx.in_expansion());
        assert_eq!(ctx.depth(), 1);
        assert!(ctx.is_expanding("MACRO1"));

        let frame2 = MacroFrame::new("MACRO2".to_string(), pos.clone(), pos.clone(), 1);
        ctx.push(frame2);
        assert_eq!(ctx.depth(), 2);
        assert_eq!(ctx.max_depth(), 2);

        ctx.pop();
        assert_eq!(ctx.depth(), 1);

        ctx.pop();
        assert!(!ctx.in_expansion());
    }
}
