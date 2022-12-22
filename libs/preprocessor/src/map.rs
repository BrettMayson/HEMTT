use std::path::PathBuf;

use hemtt_tokens::{Symbol, Token};
use serde::Serialize;

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

    #[must_use]
    /// Get the source map for the processed file
    /// Work in progress, does not produce a valid source map yet
    ///
    /// # Panics
    /// Panics if the processed file is not in the same directory as the source file
    pub fn get_source_map(&self, processed: PathBuf) -> String {
        #[derive(Serialize)]
        struct Intermediate {
            version: u8,
            file: PathBuf,
            sources: Vec<String>,
            names: Vec<()>,
            mappings: Vec<Vec<(usize, usize, usize, usize)>>,
        }
        serde_json::to_string(&Intermediate {
            version: 3,
            names: Vec::new(),
            file: processed,
            sources: self.sources.iter().map(|(path, _)| path.clone()).collect(),
            mappings: {
                self.mappings
                    .iter()
                    .map(|o| {
                        o.iter()
                            .map(|i| {
                                (
                                    i.processed_column,
                                    i.source,
                                    i.original_line,
                                    i.original_column,
                                )
                            })
                            .collect::<Vec<(usize, usize, usize, usize)>>()
                    })
                    .collect::<Vec<Vec<(usize, usize, usize, usize)>>>()
            },
        })
        .unwrap()
    }
}

#[allow(clippy::fallible_impl_from)] // TODO
impl From<Vec<Token>> for Processed {
    fn from(tokens: Vec<Token>) -> Self {
        let mut sources: Vec<(String, String)> = Vec::new();
        let mut mappings = Vec::new();
        let mut output = String::new();
        let mut mapping = Vec::new();
        let mut next_offset = 0;
        for token in tokens {
            let source = token.source();
            let symbol = token.symbol();
            let render = symbol.output();
            if render.is_empty() {
                continue;
            }
            let original_line = source.start().1 .0;
            let original_column = source.start().1 .1;
            let source = sources
                .iter()
                .position(|(name, _)| name == &source.path().to_string())
                .map_or_else(
                    || {
                        sources.push((source.path().to_string(), {
                            if source.path() == "%builtin%" {
                                String::new()
                            } else {
                                std::fs::read_to_string(source.path()).unwrap()
                            }
                        }));
                        sources.len() - 1
                    },
                    |index| index,
                );
            if symbol == &Symbol::Newline {
                mappings.push(mapping);
                mapping = Vec::new();
                next_offset = 0;
            } else {
                mapping.push(Mapping {
                    processed_column: next_offset,
                    source,
                    original_line,
                    original_column,
                });
                next_offset = render.len();
            }
            output.push_str(render.as_str());
        }
        Self {
            sources,
            mappings,
            output,
        }
    }
}

/// Mapping of a processed token to its source
pub struct Mapping {
    processed_column: usize,
    source: usize,
    original_line: usize,
    original_column: usize,
}

impl Mapping {
    #[must_use]
    /// Get the column of the processed token
    pub const fn processed_column(&self) -> usize {
        self.processed_column
    }

    #[must_use]
    /// Get the source of the processed token
    pub const fn source(&self) -> usize {
        self.source
    }

    #[must_use]
    /// Get the line of the original token
    pub const fn original_line(&self) -> usize {
        self.original_line
    }

    #[must_use]
    /// Get the column of the original token
    pub const fn original_column(&self) -> usize {
        self.original_column
    }
}
