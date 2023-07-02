use crate::tokens::{Symbol, Token};

/// Output of preprocessing a file
pub struct Processed {
    sources: Vec<(String, String)>,
    mappings: Vec<Vec<Mapping>>,
    output: String,
}

impl Processed {
    #[must_use]
    /// Get the output of preprocessing a file
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Source processed tokens for use in futher tools
    pub fn from_tokens(tokens: Vec<Token>) -> Self {
        let mut sources: Vec<(String, String)> = Vec::new();
        let mut mappings = Vec::new();
        let mut output = String::new();
        let mut mapping = Vec::new();
        for token in tokens {
            let source = token.source();
            let symbol = token.symbol();
            let render = symbol.output();
            if render.is_empty() {
                continue;
            }
            let original_column = source.start().0;
            let source = sources
                .iter()
                .position(|(name, _)| name == &source.path_or_builtin())
                .map_or_else(
                    || {
                        sources.push((source.path_or_builtin(), {
                            if source.path().is_none() {
                                String::new()
                            } else {
                                source.path().unwrap().read_to_string().unwrap()
                            }
                        }));
                        sources.len() - 1
                    },
                    |index| index,
                );
            if symbol == &Symbol::Newline {
                mappings.push(mapping);
                mapping = Vec::new();
            } else {
                mapping.push(Mapping {
                    processed_column: output.len(),
                    source,
                    original_column,
                    token: token.clone(),
                });
            }
            output.push_str(render.as_str());
        }
        Self {
            sources,
            mappings,
            output,
        }
    }

    #[must_use]
    /// Get the token at a given position
    pub fn original_col(&self, column: usize) -> Option<&Mapping> {
        let mut last_map = None;
        for mapping in self.mappings.iter().flatten() {
            if mapping.processed_column() > column {
                return last_map;
            }
            if mapping.processed_column() == column {
                return Some(mapping);
            }
            last_map = Some(mapping);
        }
        last_map
    }

    #[must_use]
    /// Get the file at a given index
    pub fn source(&self, index: usize) -> Option<&(String, String)> {
        self.sources.get(index)
    }

    #[must_use]
    /// Get the files used in preprocessing
    pub fn sources(&self) -> Vec<(String, String)> {
        self.sources.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Mapping of a processed token to its source
pub struct Mapping {
    source: usize,
    processed_column: usize,
    original_column: usize,
    token: Token,
}

impl Mapping {
    #[must_use]
    /// Get the source of the processed token
    pub const fn source(&self) -> usize {
        self.source
    }

    #[must_use]
    /// Get the column of the processed token
    pub const fn processed_column(&self) -> usize {
        self.processed_column
    }

    #[must_use]
    /// Get the column of the original token
    pub const fn original_column(&self) -> usize {
        self.original_column
    }

    #[must_use]
    /// Get the original token
    pub const fn token(&self) -> &Token {
        &self.token
    }
}
