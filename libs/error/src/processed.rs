use vfs::VfsPath;

use crate::{
    tokens::{LineCol, Symbol, Token},
    Code,
};

/// Output of preprocessing a file
pub struct Processed {
    sources: Vec<(String, (Option<VfsPath>, String))>,
    mappings: Vec<Vec<Mapping>>,
    output: String,
    warnings: Vec<Box<dyn Code>>,
}

impl Processed {
    #[must_use]
    /// Get the output of preprocessing a file
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Source processed tokens for use in futher tools
    pub fn from_tokens(tokens: Vec<Token>, warnings: Vec<Box<dyn Code>>) -> Self {
        let mut sources: Vec<(String, (Option<VfsPath>, String))> = Vec::new();
        let mut mappings = Vec::new();
        let mut output = String::new();
        let mut mapping = Vec::new();
        let mut line = 1;
        let mut col = 1;
        for token in tokens {
            let source = token.source();
            let symbol = token.symbol();
            let render = symbol.output();
            if render.is_empty() {
                continue;
            }
            let original = *source.start();
            let source = sources
                .iter()
                .position(|(name, _)| name == &source.path_or_builtin())
                .map_or_else(
                    || {
                        sources.push((
                            source.path_or_builtin(),
                            (source.path().cloned(), {
                                if source.path().is_none() {
                                    String::new()
                                } else {
                                    source.path().unwrap().read_to_string().unwrap()
                                }
                            }),
                        ));
                        sources.len() - 1
                    },
                    |index| index,
                );
            if symbol == &Symbol::Newline {
                mappings.push(mapping);
                mapping = Vec::new();
                line += 1;
                col = 1;
            } else {
                mapping.push(Mapping {
                    processed: LineCol(output.len(), (line, col)),
                    source,
                    original,
                    token: token.clone(),
                });
            }
            output.push_str(render.as_str());
            col += render.len();
        }
        Self {
            sources,
            mappings,
            output,
            warnings,
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
    pub fn source(&self, index: usize) -> Option<&(String, (Option<VfsPath>, String))> {
        self.sources.get(index)
    }

    #[must_use]
    /// Get the files used in preprocessing
    pub fn sources(&self) -> Vec<(String, String)> {
        self.sources
            .clone()
            .into_iter()
            .map(|(a, (_, b))| (a, b))
            .collect()
    }

    #[must_use]
    /// Get the warnings generated during preprocessing
    pub fn warnings(&self) -> &[Box<dyn Code>] {
        &self.warnings
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Mapping of a processed token to its source
pub struct Mapping {
    source: usize,
    processed: LineCol,
    original: LineCol,
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
        self.processed.0
    }

    #[must_use]
    /// Get the column of the original token
    pub const fn original_column(&self) -> usize {
        self.original.0
    }

    #[must_use]
    /// Get the processed position of the token
    pub const fn processed(&self) -> LineCol {
        self.processed
    }

    pub const fn original(&self) -> LineCol {
        self.original
    }

    #[must_use]
    /// Get the original token
    pub const fn token(&self) -> &Token {
        &self.token
    }
}
