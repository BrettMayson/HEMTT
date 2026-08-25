#[allow(dead_code)]
use std::collections::HashMap;
use std::sync::Arc;

use hemtt_common::config::PreprocessorOptions;
use hemtt_workspace::{
    SourceDatabase, WorkspacePath,
    position::Position,
    reporting::{
        Codes, Definition, ExpansionMetadata, ExpansionMetadataStore, MacroExpander, Output,
        Processed, Symbol, Token,
    },
};
use peekmore::{PeekMore, PeekMoreIterator};

use crate::codes::pe3_expected_ident::ExpectedIdent;
use crate::codes::pw2_invalid_config_case::InvalidConfigCase;
use crate::codes::{
    pe2_unexpected_eof::UnexpectedEOF, pe26_unsupported_builtin::BuiltInNotSupported,
};
use crate::codes::{pe18_eoi_ifstate::EoiIfState, pe25_exec::ExecNotSupported};
use crate::defines::Defines;
use crate::ifstate::IfStates;
use crate::{Error, codes::pe29_circular_include::CircularInclude};

use self::pragma::Pragma;

mod defines;
mod directives;
pub mod pragma;
mod whitespace;

#[derive(Default)]
/// Arma 3 Preprocessor
pub struct Processor {
    ifstates: IfStates,
    defines: Defines,
    backslashes: usize,

    included_files: Vec<WorkspacePath>,
    file_stack: Vec<WorkspacePath>,

    /// Source database used to resolve file content (workspace/VFS or LSP
    /// overlay) and to memoize parsing across includes. See
    /// [`Processor::run_with_sources`].
    sources: SourceDatabase,

    pub(crate) token_count: usize,

    macros: HashMap<String, Vec<(Position, Definition)>>,

    #[cfg(feature = "lsp")]
    /// Map of token usage to definition
    /// (token, definition)
    pub(crate) declarations: HashMap<Position, Position>,

    #[cfg(feature = "lsp")]
    /// Map of token definition to usage
    /// (definition, usages)
    pub(crate) usage: HashMap<Position, Vec<Position>>,

    /// Warnings
    pub(crate) warnings: Codes,

    /// The preprocessor was able to run checks, but the output should not be rapified
    pub(crate) no_rapify: bool,

    /// Macro expander for tracking expansion history and recursion
    pub(crate) macro_expander: MacroExpander,

    /// Storage for expansion metadata captured during preprocessing
    pub(crate) expansion_metadata: ExpansionMetadataStore,

    /// Map from token position to expansion metadata
    /// Used to link macro tokens to their expansion info
    pub(crate) metadata_by_token:
        std::collections::HashMap<std::ops::Range<usize>, ExpansionMetadata>,
}

impl Processor {
    #[must_use]
    /// Returns the defines
    pub const fn defines(&self) -> &Defines {
        &self.defines
    }

    #[must_use]
    /// Returns the macro expansion metadata store
    pub const fn expansion_metadata(&self) -> &ExpansionMetadataStore {
        &self.expansion_metadata
    }

    #[must_use]
    /// Returns the macro expander
    pub const fn macro_expander(&self) -> &MacroExpander {
        &self.macro_expander
    }

    /// Preprocess a file
    ///
    /// Uses a throwaway [`SourceDatabase`] with no overlays. Prefer
    /// [`Processor::run_with_sources`] when a [`SourceDatabase`] is
    /// available (e.g. from an LSP), so that unsaved editor buffers are
    /// used, and so parsing is shared across a batch of files that include
    /// common headers.
    ///
    /// # Errors
    /// See [`Error`]
    pub fn run(
        path: &WorkspacePath,
        options: &PreprocessorOptions,
    ) -> Result<Processed, (Vec<WorkspacePath>, Error)> {
        Self::run_with_sources(path, options, &SourceDatabase::new())
    }

    /// Preprocess a file, resolving all file content (the root file and any
    /// `#include`d files) through the given [`SourceDatabase`].
    ///
    /// This is the entry point that makes the preprocessor source-agnostic:
    /// `sources` may serve content from the [`hemtt_workspace::Workspace`]/VFS,
    /// from an LSP overlay (unsaved editor buffer), or a mix of both, and the
    /// preprocessor does not need to know which.
    ///
    /// # Errors
    /// See [`Error`]
    pub fn run_with_sources(
        path: &WorkspacePath,
        options: &PreprocessorOptions,
        sources: &SourceDatabase,
    ) -> Result<Processed, (Vec<WorkspacePath>, Error)> {
        let mut processor = Self {
            sources: sources.clone(),
            ..Self::default()
        };

        processor.defines.option_runtime(options.runtime_macros());

        processor.file_stack.push(path.clone());

        // Drop any dependency edges recorded by a previous run of this file
        // (e.g. an earlier LSP preprocess before the include set changed),
        // so re-running doesn't accumulate stale forward/reverse edges.
        let root_id = processor.sources.file_id(path);
        processor.sources.clear_dependencies_of(root_id);

        let tokens = crate::parse::file_with_sources(path, &processor.sources)
            .map_err(|e| (processor.included_files.clone(), e))?;
        let mut pragma = Pragma::root();
        let mut buffer = Vec::with_capacity(tokens.len());
        let mut stream = tokens.iter().cloned().peekmore();

        processor
            .file(&mut pragma, &mut stream, &mut buffer)
            .map_err(|e| (processor.included_files.clone(), e))?;

        if let Some(state) = processor.ifstates.pop() {
            return Err((
                processor.included_files,
                EoiIfState::code(state.token().as_ref().clone()),
            ));
        }

        if path.filename() == "Config.cpp" {
            processor
                .warnings
                .push(Arc::new(InvalidConfigCase::new(path.clone())));
        }

        let mut processed = Processed::new(
            buffer,
            processor.macros,
            processor.included_files.clone(),
            &processor.sources,
            #[cfg(feature = "lsp")]
            processor.usage,
            processor.warnings,
            processor.no_rapify,
        )
        .map_err(|e| (processor.included_files, e.into()))?;

        // Set expansions on the processed struct
        let mut expansions_store = ExpansionMetadataStore::new();
        for (token_span, metadata) in processor.metadata_by_token {
            expansions_store.register(token_span, metadata);
        }
        expansions_store.build_interval_tree();
        processed.expansions = expansions_store;

        Ok(processed)
    }

    fn file(
        &mut self,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        loop {
            let first = stream.peek();
            if first.is_none() || first.expect("just checked").symbol().is_eoi() {
                return Ok(());
            }
            self.line(pragma, stream, buffer)?;
        }
    }

    fn line(
        &mut self,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        self.skip_whitespace(stream, Some(buffer));
        if self.directive(pragma, stream, buffer)? {
            return Ok(());
        }
        if self.ifstates.reading() {
            self.walk(None, None, pragma, stream, buffer)?;
        } else {
            self.skip_to_after_newline(stream, None);
        }
        pragma.clear_line();
        Ok(())
    }

    fn walk(
        &mut self,
        callsite: Option<&Position>,
        in_macro: Option<&str>,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        let mut in_quotes = false;
        let mut quote = None;
        let mut just_whitespace = true;
        while let Some(token) = stream.peek() {
            match (token.symbol(), in_quotes) {
                (Symbol::Word(w), false) => {
                    if w == "__EXEC" {
                        return Err(ExecNotSupported::code((**token).clone()));
                    }
                    if Defines::is_unsupported_builtin(w) {
                        return Err(BuiltInNotSupported::code((**token).clone()));
                    }
                    just_whitespace = false;
                    if Some(w.as_str()) != in_macro && self.defines.contains_key(w) {
                        let token = token.clone();
                        self.define_use(
                            callsite.unwrap_or_else(|| token.position()),
                            pragma,
                            stream,
                            buffer,
                        )?;
                    } else {
                        self.output(stream.next().expect("peeked above"), buffer);
                    }
                }
                (Symbol::Directive, false) => {
                    if just_whitespace && let Some(command) = stream.peek_forward(1) {
                        if [
                            "if", "else", "endif", "ifdef", "ifndef", "define", "undef", "include",
                            "pragma",
                        ]
                        .contains(&command.to_string().as_str())
                        {
                            let _ = stream.peek_backward(1);
                            self.directive(pragma, stream, buffer)?;
                            just_whitespace = true;
                            continue;
                        }
                        let _ = stream.peek_backward(1);
                    }
                    let token = stream.next().expect("peeked above");
                    if in_macro.is_some()
                    && stream.peek().is_some_and(|t| t.symbol().is_word() && self.defines.contains_key(&t.symbol().to_string()))
                        // check if the # token is from another file, or defined before the callsite, ie not in the root arguments
                        && (token.position().path() != callsite.expect(
                            "callsite should exist if in_macro is some"
                        ).path()
                            || token.position().start().0 < callsite.expect(
                            "callsite should exist if in_macro is some"
                            ).start().0)
                    {
                        self.output(
                            Arc::new(Token::new(Symbol::DoubleQuote, token.position().clone())),
                            buffer,
                        );
                        quote = Some(token.position().clone());
                        continue;
                    }
                    self.output(token, buffer);
                }
                (Symbol::Newline, false) => {
                    just_whitespace = true;
                    self.output(stream.next().expect("peeked above"), buffer);
                    if in_macro.is_none() {
                        return Ok(());
                    }
                }
                (Symbol::DoubleQuote, _) => {
                    just_whitespace = false;
                    in_quotes = !in_quotes;
                    self.output(stream.next().expect("peeked above"), buffer);
                }
                (Symbol::Eoi, _) => {
                    return Ok(());
                }
                (Symbol::Whitespace(_), _) => {
                    self.output(stream.next().expect("peeked above"), buffer);
                }
                (_, _) => {
                    just_whitespace = false;
                    self.output(stream.next().expect("peeked above"), buffer);
                }
            }
            if let Some(quote) = quote {
                self.output(Arc::new(Token::new(Symbol::DoubleQuote, quote)), buffer);
            }
            quote = None;
        }
        Ok(())
    }

    /// Returns the current word, consuming it from the stream
    ///
    /// # Errors
    /// - [`UnexpectedEOF`]: If the stream is at the end of the file
    /// - [`ExpectedIdent`]: If the stream is not at a word
    fn current_word(
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<Arc<Token>, Error> {
        if let Some(token) = stream.peek() {
            if token.symbol().is_word() {
                return Ok(stream.next().expect("just checked"));
            }
            if token.symbol().is_eoi() {
                return Err(UnexpectedEOF::code(token.as_ref().clone()));
            }
        }
        Err(ExpectedIdent::code(
            stream.next().expect("just checked").as_ref().clone(),
        ))
    }

    /// Skips whitespace, returning the next word and consuming it from the stream
    ///
    /// # Errors
    /// - [`UnexpectedEOF`]: If the stream is at the end of the file
    /// - [`ExpectedIdent`]: If the stream is not at a word
    fn next_word(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: Option<&mut Vec<Output>>,
    ) -> Result<Arc<Token>, Error> {
        self.skip_whitespace(stream, buffer);
        Self::current_word(stream)
    }

    // I might want this later, so for now I am leaving it here
    #[allow(dead_code)]
    /// Skips whitespace, returning the next value and consuming it from the stream
    ///
    /// # Errors
    /// - [`UnexpectedEOF`]: If the stream is at the end of the file
    fn next_value(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: Option<&mut Vec<Output>>,
    ) -> Result<Arc<Token>, Error> {
        self.skip_whitespace(stream, buffer);
        if let Some(token) = stream.peek()
            && token.symbol().is_eoi()
        {
            return Err(UnexpectedEOF::code(token.as_ref().clone()));
        }
        Ok(stream.next().expect("just checked"))
    }

    fn output(&mut self, token: Arc<Token>, buffer: &mut Vec<Output>) {
        if self.ifstates.reading() && !token.symbol().is_comment() {
            if token.symbol().is_newline() && self.backslashes % 2 == 1 {
                self.backslashes = 0;
                buffer.pop();
                return;
            }
            if token.symbol().is_escape() {
                self.backslashes += 1;
            } else {
                self.backslashes = 0;
            }
            self.token_count += 1;
            buffer.push(Output::Direct(token));
        }
    }

    /// Check if any two files are the same
    fn add_include(&mut self, path: WorkspacePath, token: Vec<Arc<Token>>) -> Result<(), Error> {
        if self.file_stack.contains(&path) {
            return Err(CircularInclude::code(token, self.file_stack.clone()));
        }
        self.file_stack.push(path.clone());
        self.included_files.push(path);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod tests {
    use std::sync::Arc;

    use hemtt_workspace::reporting::Token;
    use peekmore::{PeekMore, PeekMoreIterator};

    pub fn setup(content: &str) -> PeekMoreIterator<impl Iterator<Item = Arc<Token>>> {
        let workspace = hemtt_workspace::Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .unwrap();
        let test = workspace.join("test.hpp").unwrap();
        test.create_file()
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        crate::parse::file(&test).unwrap().into_iter().peekmore()
    }

    /// End-to-end proof that `Processor::run_with_sources` prefers LSP
    /// overlay content over the on-disk/VFS content, and that this holds
    /// for included files too, not just the root file being preprocessed.
    #[test]
    fn run_with_sources_prefers_overlay_over_workspace() {
        use hemtt_workspace::SourceDatabase;

        let workspace = hemtt_workspace::Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .unwrap();

        let included = workspace.join("included.hpp").unwrap();
        included
            .create_file()
            .unwrap()
            .write_all(b"disk_included_value")
            .unwrap();

        let root = workspace.join("root.hpp").unwrap();
        root.create_file()
            .unwrap()
            .write_all(b"disk_root_value #include \"included.hpp\"")
            .unwrap();

        let sources = SourceDatabase::new();
        let root_id = sources.file_id(&root);
        let included_id = sources.file_id(&included);
        sources.set_overlay(
            root_id,
            "overlay_root_value\n#include \"included.hpp\"\n",
            1,
        );
        sources.set_overlay(included_id, "overlay_included_value", 1);

        let processed = crate::Processor::run_with_sources(
            &root,
            &hemtt_common::config::PreprocessorOptions::default(),
            &sources,
        )
        .unwrap();

        let output = processed.as_str();
        assert!(output.contains("overlay_root_value"));
        assert!(output.contains("overlay_included_value"));
        assert!(!output.contains("disk_root_value"));
        assert!(!output.contains("disk_included_value"));

        // The include dependency edge should have been recorded on the
        // shared `SourceDatabase`, in both directions.
        assert_eq!(sources.dependencies_of(root_id), vec![included_id]);
        assert_eq!(sources.dependents_of(included_id), vec![root_id]);
    }

    /// `Processor::run_with_sources` clears stale forward dependency edges
    /// for the root file it's (re)processing before recording new ones, so
    /// repeated LSP-style re-preprocessing after an edit that removes an
    /// `#include` doesn't leave a dangling dependency edge behind.
    #[test]
    fn run_with_sources_clears_stale_dependencies_on_rerun() {
        use hemtt_workspace::SourceDatabase;

        let workspace = hemtt_workspace::Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .unwrap();

        let included = workspace.join("included.hpp").unwrap();
        included
            .create_file()
            .unwrap()
            .write_all(b"included_value")
            .unwrap();

        let root = workspace.join("root.hpp").unwrap();
        root.create_file()
            .unwrap()
            .write_all(b"root_value")
            .unwrap();

        let sources = SourceDatabase::new();
        let root_id = sources.file_id(&root);
        let included_id = sources.file_id(&included);

        // First run: root includes included.hpp.
        sources.set_overlay(root_id, "root_value\n#include \"included.hpp\"\n", 1);
        crate::Processor::run_with_sources(
            &root,
            &hemtt_common::config::PreprocessorOptions::default(),
            &sources,
        )
        .unwrap();
        assert_eq!(sources.dependencies_of(root_id), vec![included_id]);

        // Second run: the edit removed the #include.
        sources.set_overlay(root_id, "root_value only\n", 2);
        crate::Processor::run_with_sources(
            &root,
            &hemtt_common::config::PreprocessorOptions::default(),
            &sources,
        )
        .unwrap();
        assert_eq!(
            sources.dependencies_of(root_id),
            [] as [hemtt_workspace::FileId; 0]
        );
        assert_eq!(
            sources.dependents_of(included_id),
            [] as [hemtt_workspace::FileId; 0]
        );
    }
}
