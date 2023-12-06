#[cfg(feature = "lsp")]
use std::collections::HashMap;
use std::{ops::Range, rc::Rc};

use tracing::trace;

use crate::{
    position::{LineCol, Position},
    reporting::{Output, Token},
    workspace::WorkspacePath,
};

use super::{Code, Error};

#[derive(Debug, Default)]
/// A processed file
pub struct Processed {
    sources: Vec<(WorkspacePath, String)>,
    processed: String,

    /// character offset for each line
    line_offsets: Vec<usize>,

    /// string offset(start, stop), source, source position
    mappings: Vec<Mapping>,

    #[cfg(feature = "lsp")]
    /// Map of token usage to definition
    /// (token, definition)
    declarations: HashMap<Position, Position>,

    #[cfg(feature = "lsp")]
    /// Map of token definition to usage
    /// (definition, usages)
    usage: HashMap<Position, Vec<Position>>,

    line: usize,
    col: usize,
    total: usize,

    /// Warnings
    warnings: Vec<Box<dyn Code>>,

    /// The preprocessor was able to check the file, but it should not be rapified
    no_rapify: bool,
}

fn append_token(
    processed: &mut Processed,
    string_stack: &mut Vec<char>,
    token: Rc<Token>,
) -> Result<(), Error> {
    let path = token.position().path().clone();
    let source = processed
        .sources
        .iter()
        .position(|(s, _)| s == &path)
        .map_or_else(
            || {
                let content = path.read_to_string()?;
                processed.sources.push((path, content));
                Ok(processed.sources.len() - 1)
            },
            Ok,
        )
        .map_err(Error::Workspace)?;
    if token.symbol().is_double_quote() {
        if string_stack.is_empty() {
            string_stack.push('"');
        } else if string_stack.last().unwrap() == &'"' {
            string_stack.pop();
        } else {
            string_stack.push('"');
        }
    } else if token.symbol().is_single_quote() {
        if string_stack.is_empty() {
            string_stack.push('\'');
        } else if string_stack.last().unwrap() == &'\''
            && token.position().start().0 != token.position().end().0
        {
            string_stack.pop();
        } else {
            string_stack.push('\'');
        }
    }
    if token.symbol().is_newline() {
        processed.line_offsets.push(processed.processed.len());
        processed.processed.push('\n');
        processed.mappings.push(Mapping {
            processed: (LineCol(processed.total, (processed.line, processed.col)), {
                processed.line += 1;
                processed.col = 0;
                processed.total += 1;
                LineCol(processed.total, (processed.line, processed.col))
            }),
            source,
            original: token.position().clone(),
            token,
            was_macro: false,
        });
    } else {
        let str = token.to_source();
        if str.is_empty() {
            return Ok(());
        }
        if str == "##" && string_stack.is_empty() {
            return Ok(());
        }
        processed.mappings.push(Mapping {
            processed: (LineCol(processed.total, (processed.line, processed.col)), {
                processed.col += str.len();
                processed.total += str.len();
                processed.processed.push_str(&str);
                LineCol(
                    processed.total + str.len(),
                    (processed.line, processed.col + str.len()),
                )
            }),
            source,
            original: token.position().clone(),
            token,
            was_macro: false,
        });
    }
    Ok(())
}

fn append_output(
    processed: &mut Processed,
    string_stack: &mut Vec<char>,
    output: Vec<Output>,
) -> Result<(), Error> {
    for o in output {
        match o {
            Output::Direct(t) => {
                append_token(processed, string_stack, t)?;
            }
            Output::Macro(root, o) => {
                let start = processed.total;
                let line = processed.line;
                let col = processed.col;
                append_output(processed, string_stack, o)?;
                let end = processed.total;
                let path = root.position().path().clone();
                let source = processed
                    .sources
                    .iter()
                    .position(|(s, _)| s.as_str() == path.as_str())
                    .map_or_else(
                        || {
                            let content = path.read_to_string().unwrap();
                            processed.sources.push((path, content));
                            processed.sources.len() - 1
                        },
                        |i| i,
                    );
                processed.mappings.push(Mapping {
                    processed: (
                        LineCol(start, (line, col)),
                        LineCol(end, (processed.line, processed.col)),
                    ),
                    source,
                    original: root.position().clone(),
                    token: root,
                    was_macro: true,
                });
            }
        }
    }
    Ok(())
}

impl Processed {
    /// Process the output of the preprocessor
    ///
    /// # Errors
    /// [`Error::Workspace`] if a workspace path could not be read
    pub fn new(
        output: Vec<Output>,
        #[cfg(feature = "lsp")] usage: HashMap<Position, Vec<Position>>,
        #[cfg(feature = "lsp")] declarations: HashMap<Position, Position>,
        warnings: Vec<Box<dyn Code>>,
        no_rapify: bool,
    ) -> Result<Self, Error> {
        let mut processed = Self {
            #[cfg(feature = "lsp")]
            declarations,
            #[cfg(feature = "lsp")]
            usage,
            warnings,
            no_rapify,
            ..Default::default()
        };
        let mut string_stack = Vec::new();
        append_output(&mut processed, &mut string_stack, output)?;
        Ok(processed)
    }

    #[must_use]
    /// Get the output suitable for further processing
    /// Ignores certain tokens
    pub fn as_str(&self) -> &str {
        &self.processed
    }

    #[must_use]
    /// Character offset for a line
    pub fn line_offset(&self, line: usize) -> Option<usize> {
        self.line_offsets.get(line).copied()
    }

    #[must_use]
    /// Get the files used in preprocessing
    pub fn sources(&self) -> Vec<(WorkspacePath, String)> {
        self.sources.clone()
    }

    #[must_use]
    /// Get a source by index
    ///
    /// Returns `Some((path, content))` if the index is in bounds
    /// Returns `None` if the index is out of bounds
    pub fn source(&self, index: usize) -> Option<&(WorkspacePath, String)> {
        self.sources.get(index)
    }

    #[must_use]
    /// Get the sources for arianne
    pub fn sources_adrianne(&self) -> Vec<(String, String)> {
        self.sources
            .iter()
            .map(|(path, content)| (path.to_string(), content.clone()))
            .collect()
    }

    #[must_use]
    /// Get the tree mapping at a position in the stringified output
    pub fn mappings(&self, offset: usize) -> Vec<&Mapping> {
        self.mappings
            .iter()
            .filter(|map| {
                map.processed_start().offset() <= offset && map.processed_end().offset() > offset
            })
            .collect()
    }

    #[must_use]
    /// Get the code at a position in the stringified output
    ///
    /// # Panics
    /// Panics if a source does not exist
    pub fn code(&self, span: Range<usize>) -> String {
        if self.processed.is_empty() {
            return String::new();
        }
        self.processed
            .chars()
            .skip(span.start)
            .take(span.end - span.start)
            .collect::<String>()
    }

    #[must_use]
    /// Get the deepest tree mapping at a position in the stringified output
    pub fn mapping(&self, offset: usize) -> Option<&Mapping> {
        self.mappings(offset).last().copied()
    }

    #[must_use]
    /// Returns the warnings
    pub fn warnings(&self) -> &[Box<dyn Code>] {
        &self.warnings
    }

    #[must_use]
    /// Returns whether the file should not be rapified
    pub const fn no_rapify(&self) -> bool {
        self.no_rapify
    }
}

#[derive(Debug)]
/// A mapping from the stringified output to the original source
pub struct Mapping {
    source: usize,
    processed: (LineCol, LineCol),
    original: Position,
    token: Rc<Token>,
    was_macro: bool,
}

impl Mapping {
    #[must_use]
    /// Get the source of the processed token
    pub const fn source(&self) -> usize {
        self.source
    }

    #[must_use]
    /// Get the start of the processed token
    pub const fn processed_start(&self) -> LineCol {
        self.processed.0
    }

    #[must_use]
    /// Get the end of the processed token
    pub const fn processed_end(&self) -> LineCol {
        self.processed.1
    }

    #[must_use]
    /// Get the start column of the original token
    pub const fn original_column(&self) -> usize {
        self.original.start().0
    }

    #[must_use]
    /// Get the original position
    pub const fn original(&self) -> &Position {
        &self.original
    }

    #[must_use]
    /// Get the original token
    pub const fn token(&self) -> &Rc<Token> {
        &self.token
    }

    #[must_use]
    /// Get whether the token came from a macro
    pub const fn was_macro(&self) -> bool {
        self.was_macro
    }
}
