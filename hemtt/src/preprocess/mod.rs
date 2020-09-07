//! Functions for preprocessing Arma configs and scripts

use std::clone::Clone;
use std::collections::HashMap;
use std::iter::Sum;
use std::path::PathBuf;

use crate::error::{PreprocessError, PreprocessParseError};
use crate::HEMTTError;

mod fs;

/// Can an extension be preprocessed
///
/// Arguments
/// * `ext`: the file extension ("cpp")
pub fn can_preprocess_ext(ext: &str) -> bool {
    ["cpp", "rvmat", "ext"].contains(&ext)
}

/// Can a file be preprocessed
///
/// Arguments
/// * `path`: path to the file
pub fn can_preprocess_file<P: Into<PathBuf>>(path: P) -> bool {
    can_preprocess_ext(
        &path
            .into()
            .extension()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_str()
            .unwrap(),
    )
}

pub mod grammar {
    #![allow(deprecated)]
    #![allow(missing_docs)]
    include!(concat!(env!("OUT_DIR"), "/preprocess.rs"));
}

/// Macro definition
#[derive(Clone, Debug)]
pub struct Definition {
    name: String,
    parameters: Option<Vec<String>>,
    value: Vec<Token>,
    local: bool,
}

/// Preprocessor directive
#[derive(Debug)]
pub enum Directive {
    /// `#include` directive containing the given path
    IncludeDirective(String),
    /// `#define` directive containing the definition
    DefineDirective(Definition),
    /// `#undef` directive containing the name of the macro
    UndefDirective(String),
    /// `#ifdef` directive containing the name of the macro
    IfDefDirective(String),
    /// `#ifndef` directive containing the name of the macro
    IfNDefDirective(String),
    /// `#else` directive
    ElseDirective,
    /// `#endif` directive
    EndIfDirective,
}

/// Potential macro invocation
#[derive(Debug)]
pub struct Macro {
    name: String,
    arguments: Option<Vec<String>>,
    original: String,
    quoted: bool,
}

/// Preprocessor token
#[derive(Debug)]
pub enum Token {
    /// Non-macro token
    RegularToken(String),
    /// Non-macro token that contains a number of newlines
    NewlineToken(String, u32),
    /// Potential macro token
    MacroToken(Macro),
    /// Comment token containing a number of newlines
    CommentToken(u32),
    /// Token for the concatenation operator (`##`)
    ConcatToken,
}

/// Preprocessor line
#[derive(Debug)]
pub enum Line {
    /// Directive line
    DirectiveLine(Directive, u32),
    /// Non-directive line of tokens
    TokenLine(Vec<Token>),
}

/// Struct for additional information about preprocessor output. Contains import stack used for
/// loop detection and the origins of all the lines in the output.
#[derive(Debug)]
pub struct PreprocessInfo {
    /// For every line in the output, `line_origins` contains a line number (starting at 1) and a
    /// `PathBuf` to the file where the line was found. The path may be `None` if the line was in the
    /// original input to `preprocess` and `origin` was not given.
    pub line_origins: Vec<(u32, Option<PathBuf>)>,
    import_stack: Vec<PathBuf>,
}

fn parse_macro(input: &str) -> Macro {
    let without_original: Macro = grammar::macro_proper(input).unwrap();

    Macro {
        original: String::from(input),
        ..without_original
    }
}

impl Clone for Macro {
    fn clone(&self) -> Macro {
        Macro {
            name: self.name.clone(),
            arguments: self.arguments.clone(),
            original: self.original.clone(),
            quoted: self.quoted,
        }
    }
}

impl Clone for Token {
    fn clone(&self) -> Token {
        match self {
            Token::RegularToken(s) => Token::RegularToken(s.clone()),
            Token::NewlineToken(s, n) => Token::NewlineToken(s.clone(), *n),
            Token::MacroToken(m) => Token::MacroToken(m.clone()),
            Token::CommentToken(n) => Token::CommentToken(*n),
            Token::ConcatToken => Token::ConcatToken,
        }
    }
}

impl Definition {
    fn value(
        &self,
        arguments: &Option<Vec<String>>,
        def_map: &HashMap<String, Definition>,
        stack: &[Definition],
    ) -> Result<Option<Vec<Token>>, HEMTTError> {
        let params = self.parameters.clone().unwrap_or_default();
        let args = arguments.clone().unwrap_or_default();

        if params.len() != args.len() {
            return Ok(None);
        }

        let mut tokens = self.value.clone();

        if stack.iter().any(|d| d.name == self.name) {
            return Ok(Some(tokens));
        }

        let mut stack_new: Vec<Definition> = stack.to_vec();
        stack_new.push(self.clone());

        if !params.is_empty() {
            let mut local_map: HashMap<String, Definition> = HashMap::new();

            for (key, value) in def_map.iter() {
                local_map.insert(key.clone(), value.clone());
            }

            // @todo: handle these errors properly
            for (param, arg) in params.iter().zip(args.iter()) {
                let mut tokens = grammar::tokens(&arg).expect("Failed to parse macro argument");
                let stack: Vec<Definition> = Vec::new();
                tokens = Macro::resolve_all(&tokens, &def_map, &stack)
                    .expect("Failed to resolve macro arguments");

                local_map.insert(
                    param.clone(),
                    Definition {
                        name: param.clone(),
                        parameters: None,
                        value: tokens,
                        local: true,
                    },
                );
            }

            tokens = Macro::resolve_all(&tokens, &local_map, &stack_new)?;
        } else {
            tokens = Macro::resolve_all(&tokens, &def_map, &stack_new)?;
        }

        Ok(Some(tokens))
    }
}

impl Macro {
    fn resolve_pseudoargs(
        &self,
        def_map: &HashMap<String, Definition>,
        stack: &[Definition],
    ) -> Result<Vec<Token>, HEMTTError> {
        let mut tokens: Vec<Token> = Vec::new();
        tokens.push(Token::RegularToken(self.name.clone()));

        if self.arguments.is_none() {
            return Ok(tokens);
        }

        let (_, without_name) = self.original.split_at(self.name.len());
        let mut arg_tokens =
            grammar::tokens(&without_name).expect("Failed to parse macro arguments.");

        arg_tokens = Macro::resolve_all(&arg_tokens, &def_map, &stack)?;
        for t in arg_tokens {
            tokens.push(t);
        }

        Ok(tokens)
    }

    fn resolve(
        &self,
        def_map: &HashMap<String, Definition>,
        stack: &[Definition],
    ) -> Result<Vec<Token>, HEMTTError> {
        match def_map.get(&self.name) {
            Some(def) => {
                let value = def.value(&self.arguments, def_map, stack)?;

                if !def.local && self.quoted {
                    // @todo: complain
                }

                if let Some(tokens) = value {
                    if self.quoted {
                        let (concatted, newlines) = Token::concat(&tokens);
                        let mut tokens: Vec<Token> = Vec::new();
                        tokens.push(Token::NewlineToken(
                            format!("\"{}\"", concatted.trim()),
                            newlines,
                        ));
                        Ok(tokens)
                    } else {
                        Ok(tokens)
                    }
                } else {
                    self.resolve_pseudoargs(def_map, stack)
                }
            }
            None => self.resolve_pseudoargs(def_map, stack),
        }
    }

    fn resolve_all(
        tokens: &[Token],
        def_map: &HashMap<String, Definition>,
        stack: &[Definition],
    ) -> Result<Vec<Token>, HEMTTError> {
        let mut result: Vec<Token> = Vec::new();

        for token in tokens {
            match token {
                Token::MacroToken(ref m) => {
                    let resolved = m.resolve(def_map, stack)?;
                    for t in resolved {
                        result.push(t);
                    }
                }
                _ => {
                    result.push(token.clone());
                }
            }
        }

        Ok(result)
    }
}

impl Token {
    fn concat(tokens: &[Token]) -> (String, u32) {
        let mut output = String::new();
        let mut newlines = 0;

        for token in tokens {
            match token {
                Token::RegularToken(s) => {
                    output += &s;
                }
                Token::NewlineToken(s, n) => {
                    output += &s;
                    newlines += n;
                }
                Token::MacroToken(m) => {
                    output += &m.original;
                }
                Token::CommentToken(n) => {
                    newlines += n;
                }
                _ => {}
            }
        }

        (output, newlines)
    }
}

fn preprocess_rec<F>(
    input: String,
    origin: Option<PathBuf>,
    definition_map: &mut HashMap<String, Definition>,
    info: &mut PreprocessInfo,
    includefolders: &[PathBuf],
    fileread: F,
) -> Result<String, HEMTTError>
where
    F: Fn(&PathBuf) -> Result<String, HEMTTError>,
    F: Copy,
{
    let lines = grammar::file(&input).map_err(|source| {
        HEMTTError::Parse(PreprocessParseError {
            path: Some(
                origin
                    .clone()
                    .unwrap_or_else(PathBuf::new)
                    .to_string_lossy()
                    .to_string(),
            ),
            message: input,
            source,
        })
    })?;
    let mut output = String::from("");
    let mut original_lineno = 1;
    let mut level = 0;
    let mut level_true = 0;

    for line in lines {
        match line {
            Line::DirectiveLine(dir, newlines) => {
                original_lineno += newlines;

                match dir {
                    Directive::IncludeDirective(path) => {
                        if level > level_true {
                            continue;
                        }

                        //let import_tree = &mut info.import_tree;
                        //let includer = import_tree.get(&path);
                        //if let Some(path) = includer {
                        //    // @todo: complain
                        //}

                        let file_path =
                            fs::find_include_file(&path, origin.as_ref(), includefolders)?;

                        info.import_stack.push(file_path.clone());

                        let content = fileread(&file_path)?;
                        let result = preprocess_rec(
                            content,
                            Some(file_path),
                            definition_map,
                            info,
                            includefolders,
                            fileread,
                        )
                        .map_err(|e| match e {
                            HEMTTError::Preprocess(p) => HEMTTError::Preprocess(p),
                            _ => HEMTTError::Preprocess(PreprocessError {
                                message: "Failed to process include".to_string(),
                                path: Some(path),
                                source: Box::new(e),
                            }),
                        })?;

                        info.import_stack.pop();

                        output += &result;
                    }
                    Directive::DefineDirective(def) => {
                        original_lineno += u32::sum(def.value.iter().map(|t| match t {
                            Token::NewlineToken(_s, n) => *n,
                            Token::CommentToken(n) => *n,
                            _ => 0,
                        }));

                        if level > level_true {
                            continue;
                        }

                        if definition_map.remove(&def.name).is_some() {
                            // @todo: warn about redefine
                        }

                        definition_map.insert(def.name.clone(), def);
                    }
                    Directive::UndefDirective(name) => {
                        if level > level_true {
                            continue;
                        }

                        definition_map.remove(&name);
                    }
                    Directive::IfDefDirective(name) => {
                        level_true += if level_true == level && definition_map.contains_key(&name) {
                            1
                        } else {
                            0
                        };
                        level += 1;
                    }
                    Directive::IfNDefDirective(name) => {
                        level_true += if level_true == level && !definition_map.contains_key(&name)
                        {
                            1
                        } else {
                            0
                        };
                        level += 1;
                    }
                    Directive::ElseDirective => {
                        if level_true + 1 == level {
                            level_true = level;
                        } else if level_true == level {
                            level_true -= 1;
                        }
                    }
                    Directive::EndIfDirective => {
                        assert!(level > 0);
                        level -= 1;
                        if level_true > level {
                            level_true -= 1;
                        }
                    }
                }
            }
            Line::TokenLine(tokens) => {
                let stack: Vec<Definition> = Vec::new();
                let resolved =
                    Macro::resolve_all(&tokens, &definition_map, &stack).map_err(|e| {
                        HEMTTError::Preprocess(PreprocessError {
                            message: "Failed to process macros".to_string(),
                            path: None,
                            source: Box::new(e),
                        })
                    })?;

                let (mut result, newlines) = Token::concat(&resolved);
                result = result.replace("\r\n", "\n");
                original_lineno += newlines;

                let before = result.len();
                result = result.replace("\\\n", "");

                if level > level_true {
                    continue;
                }

                output += &result;
                output += "\n";

                info.line_origins.push((original_lineno, origin.clone()));
                original_lineno += (before - result.len()) as u32 / 2;
            }
        }
        original_lineno += 1;

        if level > 0 {
            // @todo: complain
        }
    }

    Ok(output)
}

/// Reads input string and returns preprocessed string with an info struct containing the origins
/// of the lines in the output.
///
/// `path` is the path to the input if it is known and is used for relative includes and error
/// messages. `includefolders` are the folders searched for absolute includes and should usually at
/// least include the current working directory.
///
/// # Examples
///
/// ```
/// # use hemtt::preprocess::preprocess;
/// let input = String::from("
/// #define QUOTE(x) #x
/// #define DOUBLES(x,y) x##_##y
///
/// foo = QUOTE(DOUBLES(abc, xyz));
/// ");
///
/// let (output, _) = preprocess(input, None, &Vec::new(), |path| std::fs::read_to_string(&path).map_err(|e| e.into())).expect("Failed to preprocess");
///
/// assert_eq!("foo = \"abc_xyz\";", output.trim());
/// ```
pub fn preprocess<F>(
    mut input: String,
    origin: Option<PathBuf>,
    includefolders: &[PathBuf],
    fileread: F,
) -> Result<(String, PreprocessInfo), HEMTTError>
where
    F: Fn(&PathBuf) -> Result<String, HEMTTError>,
    F: Copy,
{
    if input[..3].as_bytes() == [0xef, 0xbb, 0xbf] {
        input = input[3..].to_string();
    }

    let mut info = PreprocessInfo {
        line_origins: Vec::new(),
        import_stack: Vec::new(),
    };

    if let Some(ref path) = origin {
        info.import_stack.push(path.clone());
    }

    let mut def_map: HashMap<String, Definition> = HashMap::new();

    match preprocess_rec(
        input,
        origin,
        &mut def_map,
        &mut info,
        includefolders,
        fileread,
    ) {
        Ok(result) => Ok((result, info)),
        Err(e) => Err(e),
    }
}
