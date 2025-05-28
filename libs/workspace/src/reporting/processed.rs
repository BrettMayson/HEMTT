use std::{collections::HashMap, ops::Range, sync::Arc};
use tracing::warn;

use crate::{
    Error, WorkspacePath,
    position::{LineCol, Position},
};

use super::{Code, Codes, Output, Token, definition::Definition};

pub type Sources = Vec<(WorkspacePath, String)>;

#[derive(Debug)]
/// A processed file
pub struct Processed {
    sources: Sources,

    output: String,
    clean_output: String,
    clean_output_line_indexes: Vec<(usize, usize)>,
    total_chars: usize,

    /// character offset for each line
    line_offsets: HashMap<WorkspacePath, HashMap<usize, usize>>,

    /// string offset(start, stop), source, source position
    mappings: Vec<Mapping>,
    mappings_interval: intervaltree::IntervalTree<usize, usize>,

    macros: HashMap<String, Vec<(Position, Definition)>>,

    #[allow(dead_code)]
    #[cfg(feature = "lsp")]
    /// Map of token definition to usage
    /// (definition, usages)
    usage: HashMap<Position, Vec<Position>>,

    /// Warnings
    warnings: Codes,

    /// The preprocessor was able to check the file, but it should not be rapified
    no_rapify: bool,
}

#[derive(Default, Debug)]
struct Processing {
    sources: Sources,

    output: String,

    /// character offset for each line
    line_offsets: HashMap<WorkspacePath, HashMap<usize, usize>>,

    /// string offset(start, stop), source, source position
    mappings: Vec<Mapping>,

    line: usize,
    col: usize,
    total_chars: usize,
}

#[allow(clippy::too_many_lines)]
fn append_token(
    processing: &mut Processing,
    string_stack: &mut Vec<char>,
    next_is_escape: &mut Option<Arc<Token>>,
    token: Arc<Token>,
) -> Result<(), Error> {
    if token.symbol().is_newline() && next_is_escape.is_some() {
        *next_is_escape = None;
        return Ok(());
    }
    if token.symbol().is_escape() {
        if *next_is_escape == Some(token.clone()) {
            *next_is_escape = None;
        } else if let Some(escape_token) = next_is_escape.clone() {
            *next_is_escape = None;
            append_token(processing, string_stack, next_is_escape, escape_token)?;
        } else {
            *next_is_escape = Some(token);
            return Ok(());
        }
    }
    if let Some(escape_token) = next_is_escape.clone() {
        append_token(processing, string_stack, next_is_escape, escape_token)?;
    }
    let path = token.position().path().clone();
    let source = processing
        .sources
        .iter()
        .position(|(s, _)| s == &path)
        .map_or_else(
            || {
                let content = path.read_to_string()?;
                processing.sources.push((path.clone(), content));
                Ok::<usize, Error>(processing.sources.len() - 1)
            },
            Ok,
        )?;
    if token.symbol().is_double_quote() {
        if string_stack.is_empty() {
            string_stack.push('"');
        } else if string_stack.last().expect("string stack is empty") == &'"' {
            string_stack.pop();
        } else {
            string_stack.push('"');
        }
    } else if token.symbol().is_single_quote() {
        if string_stack.is_empty() {
            string_stack.push('\'');
        } else if string_stack.last().expect("string stack is empty") == &'\''
            && token.position().start().0 != token.position().end().0
        {
            string_stack.pop();
        } else {
            string_stack.push('\'');
        }
    }
    if token.symbol().is_newline() {
        processing.line_offsets.entry(path).or_default().insert(
            token.position().end().line() - 1,
            token.position().end().offset(),
        );
        processing.output.push('\n');
        processing.mappings.push(Mapping {
            processed: (
                LineCol(processing.total_chars, (processing.line, processing.col)),
                {
                    processing.line += 1;
                    processing.col = 0;
                    processing.total_chars += 1;
                    LineCol(processing.total_chars, (processing.line, processing.col))
                },
            ),
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
        processing.mappings.push(Mapping {
            processed: (
                LineCol(processing.total_chars, (processing.line, processing.col)),
                {
                    let chars = str.chars().count();
                    processing.col += chars;
                    processing.total_chars += chars;
                    processing.output.push_str(&str);
                    LineCol(
                        processing.total_chars + chars,
                        (processing.line, processing.col + chars),
                    )
                },
            ),
            source,
            original: token.position().clone(),
            token,
            was_macro: false,
        });
    }
    Ok(())
}

fn append_output(
    processing: &mut Processing,
    string_stack: &mut Vec<char>,
    next_is_escape: &mut Option<Arc<Token>>,
    output: Vec<Output>,
) -> Result<(), Error> {
    for o in output {
        match o {
            Output::Direct(t) => {
                append_token(processing, string_stack, next_is_escape, t)?;
            }
            Output::Macro(root, o) => {
                let start = processing.total_chars;
                let line = processing.line;
                let col = processing.col;
                append_output(processing, string_stack, next_is_escape, o)?;
                let end = processing.total_chars;
                let path = root.position().path().clone();
                let source = processing
                    .sources
                    .iter()
                    .position(|(s, _)| s.as_str() == path.as_str())
                    .map_or_else(
                        || {
                            let content = path.read_to_string().expect("file should exist if used");
                            processing.sources.push((path, content));
                            processing.sources.len() - 1
                        },
                        |i| i,
                    );
                processing.mappings.push(Mapping {
                    processed: (
                        LineCol(start, (line, col)),
                        LineCol(end, (processing.line, processing.col)),
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

pub fn clean_output(processed: &mut Processed) {
    let mut comitted_file = String::new();
    let mut comitted_line = 0;
    let mut lines = processed.output.lines();
    let mut output = String::new();
    let mut indexes = Vec::new();
    let mut cursor_offset = 0;
    let mut clean_cursor = 0;
    let mut pending_empty = 0;
    loop {
        let Some(line) = lines.next() else {
            break;
        };
        if line.trim().is_empty() {
            cursor_offset += line.chars().count() + 1;
            pending_empty += 1;
            continue;
        }

        let Some(map) = processed.mapping(cursor_offset + 1) else {
            panic!("mapping not found for offset {cursor_offset}");
        };

        let pending_line = comitted_line + pending_empty;
        let linenum = map.original().start().line();
        let file = map
            .original()
            .path()
            .as_virtual_str()
            .to_string()
            .replace('/', "\\");
        if file != comitted_file || pending_line != linenum {
            comitted_file = file;
            comitted_line = linenum;
            let line = format!("#line {linenum} \"{comitted_file}\"\n");
            output.push_str(&line);
            clean_cursor += line.chars().count();
            pending_empty = 0;
        }
        if pending_empty > 0 {
            for _ in 0..pending_empty {
                indexes.push((cursor_offset, clean_cursor));
                output.push('\n');
                clean_cursor += 1;
            }
            comitted_line += pending_empty;
            pending_empty = 0;
        }
        indexes.push((cursor_offset, clean_cursor));
        output.push_str(line);
        output.push('\n');
        let chars = line.chars().count() + 1;
        cursor_offset += chars;
        clean_cursor += chars;
        comitted_line += 1;
    }
    processed.clean_output = output;
    processed.clean_output_line_indexes = indexes;
}

impl Processed {
    /// Process the output of the preprocessor
    ///
    /// # Errors
    /// [`Error::Workspace`] if a workspace path could not be read
    pub fn new(
        output: Vec<Output>,
        macros: HashMap<String, Vec<(Position, Definition)>>,
        #[cfg(feature = "lsp")] usage: HashMap<Position, Vec<Position>>,
        warnings: Codes,
        no_rapify: bool,
    ) -> Result<Self, Error> {
        let mut processing = Processing::default();
        let mut string_stack = Vec::new();
        let mut next_is_escape = None;
        append_output(
            &mut processing,
            &mut string_stack,
            &mut next_is_escape,
            output,
        )?;

        let mut processed = Self {
            sources: processing.sources,
            output: processing.output,
            clean_output: String::new(),
            clean_output_line_indexes: Vec::new(),
            line_offsets: processing.line_offsets,
            mappings_interval: processing
                .mappings
                .iter()
                .enumerate()
                .map(|(idx, map)| {
                    (
                        map.processed_start().offset()..map.processed_end().offset(),
                        idx,
                    )
                })
                .collect(),
            mappings: processing.mappings,
            total_chars: processing.total_chars,
            macros,
            #[cfg(feature = "lsp")]
            usage,
            warnings,
            no_rapify,
        };

        clean_output(&mut processed);
        Ok(processed)
    }

    #[must_use]
    /// Get the output suitable for further processing
    /// Ignores certain tokens
    pub fn as_str(&self) -> &str {
        &self.output
    }

    #[must_use]
    // Get the length of the output in characters
    pub const fn output_chars(&self) -> usize {
        self.total_chars
    }

    #[must_use]
    /// Character offset for a line
    pub fn line_offset(&self, source: &WorkspacePath, line: usize) -> Option<usize> {
        self.line_offsets.get(source)?.get(&line).copied()
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
        let mut mappings = self
            .mappings_interval
            .query_point(offset)
            .collect::<Vec<_>>();
        mappings.sort_by_key(|item| item.value);
        mappings
            .iter()
            .map(|item| &self.mappings[item.value])
            .collect()
    }

    #[must_use]
    /// Get the deepest tree mapping at a position in the stringified output
    pub fn mapping(&self, offset: usize) -> Option<&Mapping> {
        self.mappings_interval
            .query_point(offset)
            .max_by_key(|item| item.value)
            .map(|item| &self.mappings[item.value])
    }

    #[must_use]
    /// Get the deepest tree mapping at a position in the stringified output
    pub fn mapping_no_macros(&self, offset: usize) -> Option<&Mapping> {
        self.mappings(offset)
            .into_iter()
            .rev()
            .find(|m| !m.was_macro)
    }

    #[must_use]
    /// Get the macros defined
    pub const fn macros(&self) -> &HashMap<String, Vec<(Position, Definition)>> {
        &self.macros
    }

    #[must_use]
    /// Returns the warnings
    pub fn warnings(&self) -> &[Arc<dyn Code>] {
        &self.warnings
    }

    #[must_use]
    /// Returns whether the file should not be rapified
    pub const fn no_rapify(&self) -> bool {
        self.no_rapify
    }

    #[must_use]
    /// Return a string with the source from the span
    pub fn extract(&self, span: Range<usize>) -> Arc<str> {
        if span.start == span.end {
            warn!("tried to extract an invalid span");
            return Arc::from("");
        }
        let mut real_start = 0;
        let mut real_end = 0;
        self.output.chars().enumerate().for_each(|(p, c)| {
            if p < span.start {
                real_start += c.len_utf8();
            }
            if p < span.end {
                real_end += c.len_utf8();
            }
        });
        Arc::from(&self.output[real_start..real_end])
    }

    #[must_use]
    /// Return the entire clean output
    pub fn clean_output(&self) -> &str {
        &self.clean_output
    }

    #[must_use]
    /// Return a string with the source from the span
    pub fn clean_span(&self, span: &Range<usize>) -> Range<usize> {
        fn find_point(processed: &Processed, target: usize) -> usize {
            let mut last_start = (0, 0);
            for (original, clean) in &processed.clean_output_line_indexes {
                if original > &target {
                    break;
                }
                last_start = (*original, *clean);
            }
            processed
                .clean_output
                .chars()
                .take(last_start.1 - 1 + (target - last_start.0))
                .map(char::len_utf8)
                .sum::<usize>()
                + 1
        }
        let start = find_point(self, span.start);
        let end = find_point(self, span.end);
        start..end
    }

    #[cfg(feature = "lsp")]
    #[must_use]
    pub const fn usage(&self) -> &HashMap<Position, Vec<Position>> {
        &self.usage
    }

    #[cfg(feature = "lsp")]
    #[must_use]
    pub fn cache(self) -> CacheProcessed {
        CacheProcessed {
            output: self.clean_output,
            macros: self.macros,
            usage: self.usage,
        }
    }
}

pub struct CacheProcessed {
    pub output: String,
    pub macros: HashMap<String, Vec<(Position, Definition)>>,
    pub usage: HashMap<Position, Vec<Position>>,
}

#[derive(Clone, Debug)]
/// A mapping from the stringified output to the original source
pub struct Mapping {
    source: usize,
    processed: (LineCol, LineCol),
    original: Position,
    token: Arc<Token>,
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
    pub const fn original_start(&self) -> usize {
        self.original.start().0
    }

    #[must_use]
    /// Get the end column of the original token
    pub const fn original_end(&self) -> usize {
        self.original.end().0
    }

    #[must_use]
    /// Get the original position
    pub const fn original(&self) -> &Position {
        &self.original
    }

    #[must_use]
    /// Get the original token
    pub const fn token(&self) -> &Arc<Token> {
        &self.token
    }

    #[must_use]
    /// Get whether the token came from a macro
    pub const fn was_macro(&self) -> bool {
        self.was_macro
    }
}
