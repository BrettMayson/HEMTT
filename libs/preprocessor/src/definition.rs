use std::sync::Arc;

use hemtt_workspace::reporting::{FunctionDefinition, Token};
use peekmore::{PeekMore, PeekMoreIterator};

pub trait FunctionDefinitionStream {
    #[must_use]
    /// Get the body as a stream
    fn stream(&self) -> PeekMoreIterator<impl Iterator<Item = Arc<Token>>>;
}

impl FunctionDefinitionStream for FunctionDefinition {
    fn stream(&self) -> PeekMoreIterator<impl Iterator<Item = Arc<Token>>> {
        self.body
            .clone()
            .into_iter()
            .filter(|t| !t.symbol().is_join())
            .peekmore()
    }
}
