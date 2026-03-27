use crate::position::Position;

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
    #[must_use]
    /// Create a new macro frame
    pub const fn new(
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

    #[must_use]
    /// Get the nesting depth (1 = top-level, 2 = nested one level, etc.)
    pub const fn depth(&self, total_stack_len: usize) -> usize {
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
    #[must_use]
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

    #[must_use]
    /// Get the current expansion depth (0 = not in macro)
    pub const fn depth(&self) -> usize {
        self.stack.len()
    }

    #[must_use]
    /// Get the current macro frame without consuming it
    pub fn current(&self) -> Option<&MacroFrame> {
        self.stack.last()
    }

    #[must_use]
    /// Get the full stack
    pub fn stack(&self) -> &[MacroFrame] {
        &self.stack
    }

    #[must_use]
    /// Check if we're currently inside a macro expansion
    pub const fn in_expansion(&self) -> bool {
        !self.stack.is_empty()
    }

    #[must_use]
    /// Check if a macro is already being expanded (prevent infinite recursion)
    pub fn is_expanding(&self, macro_name: &str) -> bool {
        self.stack.iter().any(|frame| frame.name == macro_name)
    }

    #[must_use]
    /// Get the maximum nesting depth seen
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    #[must_use]
    /// Get the total number of expansions processed
    pub const fn total_expansions(&self) -> usize {
        self.total_expansions
    }

    /// Clear the stack (for error recovery or cleanup)
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    #[must_use]
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
