use std::{collections::HashMap, sync::Arc};

use hemtt_workspace::{
    position::Position,
    reporting::{Code, Definition, Output, Symbol, Token},
};
use peekmore::{PeekMore, PeekMoreIterator};

use crate::{
    codes::{
        pe10_function_as_value::FunctionAsValue,
        pe11_expected_function_or_value::ExpectedFunctionOrValue,
        pe1_unexpected_token::UnexpectedToken, pe5_define_multitoken_argument::DefineMissingComma,
        pe9_function_call_argument_count::FunctionCallArgumentCount, pw3_padded_arg::PaddedArg,
    },
    defines::DefineSource,
    definition::FunctionDefinitionStream,
    Error,
};

use super::{
    pragma::{Flag, Pragma, Suppress},
    Processor,
};

impl Processor {
    /// Reads the arguments of a macro call
    ///
    /// Expects the stream to be at the left parenthesis
    ///
    /// The stream is left after the closing parenthesis
    pub(crate) fn call_read_args(
        &mut self,
        callsite: &Position,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<Option<Vec<Vec<Arc<Token>>>>, Error> {
        if !stream
            .peek()
            .expect("peeked by caller")
            .symbol()
            .is_left_paren()
        {
            return Ok(None);
        }
        stream.next().expect("peeked above");
        let mut quotes = false;
        let mut depth = 0;
        let mut args = Vec::new();
        let mut arg = Vec::new();
        while let Some(token) = stream.peek() {
            let symbol = token.symbol();
            if quotes {
                if symbol.is_double_quote() {
                    quotes = false;
                }
                arg.push(stream.next().expect("peeked above"));
                continue;
            }
            if let Symbol::Word(word) = symbol {
                if self.defines.contains_key(word) {
                    let mut inner = Vec::new();
                    self.define_use(callsite, pragma, stream, &mut inner)?;
                    arg.append(
                        &mut inner
                            .into_iter()
                            .map(std::convert::Into::into)
                            .collect::<Vec<Vec<Arc<Token>>>>()
                            .concat(),
                    );
                    continue;
                }
            } else if symbol.is_left_paren() {
                depth += 1;
            } else if symbol.is_right_paren() {
                if depth == 0 {
                    stream.next();
                    break;
                }
                depth -= 1;
            } else if symbol.is_comma() {
                args.push(arg);
                arg = Vec::new();
                stream.next();
                continue;
            } else if symbol.is_double_quote() {
                quotes = true;
            }
            arg.push(stream.next().expect("peeked above"));
        }
        if !arg.is_empty() {
            args.push(arg);
        }
        Ok(Some(args))
    }

    /// Reads the arguments of a macro definition
    ///
    /// Expects the stream to be at the left parenthesis
    ///
    /// The stream is left after the closing parenthesis
    pub(crate) fn define_read_args(
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<Vec<Arc<Token>>, Error> {
        if !stream
            .next()
            .expect("peeked by caller")
            .symbol()
            .is_left_paren()
        {
            panic!(
                "define_read_args called without left parenthesis as first token, found {:?}",
                stream.peek().expect("peeked above").symbol()
            );
        }
        let mut args: Vec<Arc<Token>> = Vec::new();
        let mut comma_next = false;
        while let Some(token) = stream.peek() {
            let symbol = token.symbol();
            if symbol.is_word() {
                if comma_next {
                    return Err(DefineMissingComma::code(
                        stream.next().expect("peeked above").as_ref().clone(),
                        args.last().expect("peeked above").as_ref().clone(),
                    ));
                }
                args.push(stream.next().expect("peeked above"));
                comma_next = true;
            } else if symbol.is_comma() {
                if !comma_next {
                    return Err(UnexpectedToken::code(
                        stream.next().expect("peeked above").as_ref().clone(),
                        vec!["{variable}".to_string()],
                    ));
                }
                stream.next();
                comma_next = false;
                continue;
            } else if symbol.is_right_paren() {
                stream.next();
                break;
            } else if symbol.is_whitespace() {
                stream.next();
            } else if comma_next {
                return Err(UnexpectedToken::code(
                    stream.next().expect("peeked above").as_ref().clone(),
                    vec![",".to_string(), ")".to_string()],
                ));
            } else {
                return Err(UnexpectedToken::code(
                    stream.next().expect("peeked above").as_ref().clone(),
                    vec!["{variable}".to_string(), ",".to_string(), ")".to_string()],
                ));
            }
        }
        Ok(args)
    }

    /// Reads the body of a macro definition
    ///
    /// Expects the stream to be right after ident (and arguments if function macro)
    ///
    /// The stream is left at the start of the next line
    pub(crate) fn define_read_body(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Vec<Arc<Token>> {
        self.skip_whitespace(stream, None);
        let mut body = Vec::new();
        for token in stream.by_ref() {
            let symbol = token.symbol();
            if symbol.is_newline() {
                if body
                    .last()
                    .is_some_and(|t: &Arc<Token>| t.symbol().is_escape())
                {
                    // remove the backslash
                    body.pop();
                    // remove the newline
                    continue;
                }
                return body;
            }
            if !symbol.is_eoi() {
                body.push(token);
            }
        }
        body
    }

    #[allow(clippy::too_many_lines)]
    /// A define was used
    ///
    /// Expects the stream to be at the ident
    ///
    /// The stream is left on the next token after the end of the call
    pub(crate) fn define_use(
        &mut self,
        callsite: &Position,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        let ident = Self::current_word(stream)?;
        let ident_string = ident.to_string();
        let Some((source, body, define_source)) = self.defines.get_with_gen(&ident, Some(callsite))
        else {
            buffer.push(Output::Direct(ident));
            return Ok(());
        };
        match body {
            Definition::Function(function) => {
                let Some(args) = self.call_read_args(callsite, pragma, stream)? else {
                    #[allow(clippy::redundant_clone)] // behind hls feature flag
                    return Err(FunctionAsValue::code(
                        ident.as_ref().clone(),
                        source.as_ref().clone(),
                    ));
                };
                if args.len() != function.args().len() {
                    return Err(FunctionCallArgumentCount::code(
                        ident.as_ref().clone(),
                        function.args().len(),
                        args.len(),
                        &self.defines.clone(),
                    ));
                }
                let mut arg_defines = HashMap::new();
                for (arg, value) in function.args().iter().zip(args) {
                    if !pragma.is_suppressed(&Suppress::Pw3PaddedArg)
                        && (!pragma.is_flagged(&Flag::Pw3IgnoreFormat) || {
                            [
                                "ARR_", "TRACE_", "INFO_", "WARNING_", "ERROR_", "DEBUG_",
                                "FORMAT_",
                            ]
                            .iter()
                            .all(|s| !ident_string.starts_with(s))
                        })
                    {
                        for token in [value.first(), value.last()] {
                            if token.is_some_and(|t| t.symbol().is_whitespace()) {
                                let warning = PaddedArg::new(
                                    Box::new(
                                        (**token.expect("token exists from map_or check")).clone(),
                                    ),
                                    ident_string.clone(),
                                );

                                if !self.warnings.iter().any(|w| {
                                    w.ident() == warning.ident() && w.token() == warning.token()
                                }) {
                                    self.warnings.push(Arc::new(warning));
                                }
                            }
                        }
                    }
                    arg_defines.insert(
                        Arc::from(arg.to_string().as_str()),
                        (
                            arg.clone(),
                            Definition::Value(Arc::new(value)),
                            DefineSource::Argument,
                        ),
                    );
                }
                self.defines.push(&ident_string, arg_defines);
                let mut layer = Vec::new();
                self.walk(
                    Some(callsite),
                    Some(&ident_string),
                    pragma,
                    &mut function.stream(),
                    &mut layer,
                )?;
                buffer.push(Output::Macro(ident.clone(), layer));
                self.defines.pop();
            }
            #[allow(clippy::needless_collect)] // causes recursion at runtime otherwise
            Definition::Value(body) => {
                if define_source == DefineSource::Argument {
                    // prevent infinite recursion
                    buffer.push(Output::Macro(
                        ident.clone(),
                        body.iter().map(|t| Output::Direct(t.clone())).collect(),
                    ));
                } else {
                    let mut layer = Vec::new();
                    let body: Vec<_> = body
                        .iter()
                        .filter(|t| !t.symbol().is_join())
                        .cloned()
                        .collect();
                    self.walk(
                        Some(callsite),
                        Some(&ident_string),
                        pragma,
                        &mut body.into_iter().peekmore(),
                        &mut layer,
                    )?;
                    buffer.push(Output::Macro(ident.clone(), layer));
                }
            }
            Definition::Void => return Ok(()),
            Definition::Unit => {
                #[allow(clippy::redundant_clone)] // behind hls feature flag
                return Err(ExpectedFunctionOrValue::code(
                    ident.as_ref().clone(),
                    source.as_ref().clone(),
                    stream
                        .peek()
                        .expect("peeked by caller")
                        .symbol()
                        .is_left_paren(),
                ));
            }
        };
        #[cfg(feature = "lsp")]
        self.usage.get_mut(source.position()).map_or_else(
            || {
                // println!("missing {:?}", ident.position());
            },
            |usage| {
                usage.push(ident.position().clone());
            },
        );
        #[cfg(feature = "lsp")]
        self.declarations
            .insert(ident.position().clone(), source.position().clone());
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use hemtt_workspace::reporting::{Symbol, Whitespace};

    use crate::processor::{pragma::Pragma, tests, Processor};

    #[test]
    fn single_arg_single_word() {
        let mut stream = tests::setup("(hello)");
        let mut processor = Processor::default();
        let args = processor
            .call_read_args(
                &stream.peek().unwrap().position().clone(),
                &mut Pragma::root(),
                &mut stream,
            )
            .unwrap()
            .unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].len(), 1);
        assert_eq!(*args[0][0].symbol(), Symbol::Word("hello".to_string()));
    }

    #[test]
    fn single_arg_multi_word() {
        let mut stream = tests::setup("(hello world)");
        let mut processor = Processor::default();
        let args = processor
            .call_read_args(
                &stream.peek().unwrap().position().clone(),
                &mut Pragma::root(),
                &mut stream,
            )
            .unwrap()
            .unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].len(), 3);
        assert_eq!(*args[0][0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(*args[0][1].symbol(), Symbol::Whitespace(Whitespace::Space));
        assert_eq!(*args[0][2].symbol(), Symbol::Word("world".to_string()));
    }

    #[test]
    fn multi_arg_single_word() {
        let mut stream = tests::setup("(hello,world)");
        let mut processor = Processor::default();
        let args = processor
            .call_read_args(
                &stream.peek().unwrap().position().clone(),
                &mut Pragma::root(),
                &mut stream,
            )
            .unwrap()
            .unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].len(), 1);
        assert_eq!(*args[0][0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(args[1].len(), 1);
        assert_eq!(*args[1][0].symbol(), Symbol::Word("world".to_string()));
    }

    #[test]
    fn multi_arg_single_word_whitespace() {
        let mut stream = tests::setup("(hello, world)");
        let mut processor = Processor::default();
        let args = processor
            .call_read_args(
                &stream.peek().unwrap().position().clone(),
                &mut Pragma::root(),
                &mut stream,
            )
            .unwrap()
            .unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].len(), 1);
        assert_eq!(*args[0][0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(args[1].len(), 2);
        assert_eq!(*args[1][0].symbol(), Symbol::Whitespace(Whitespace::Space));
        assert_eq!(*args[1][1].symbol(), Symbol::Word("world".to_string()));
    }

    #[test]
    fn multi_arg_multi_word() {
        let mut stream = tests::setup("(hello world,world hello)");
        let mut processor = Processor::default();
        let args = processor
            .call_read_args(
                &stream.peek().unwrap().position().clone(),
                &mut Pragma::root(),
                &mut stream,
            )
            .unwrap()
            .unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].len(), 3);
        assert_eq!(*args[0][0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(*args[0][1].symbol(), Symbol::Whitespace(Whitespace::Space));
        assert_eq!(*args[0][2].symbol(), Symbol::Word("world".to_string()));
        assert_eq!(args[1].len(), 3);
        assert_eq!(*args[1][0].symbol(), Symbol::Word("world".to_string()));
        assert_eq!(*args[1][1].symbol(), Symbol::Whitespace(Whitespace::Space));
        assert_eq!(*args[1][2].symbol(), Symbol::Word("hello".to_string()));
    }

    #[test]
    fn multi_arg_nested() {
        let mut stream = tests::setup("(hello(world),world(hello))");
        let mut processor = Processor::default();
        let args = processor
            .call_read_args(
                &stream.peek().unwrap().position().clone(),
                &mut Pragma::root(),
                &mut stream,
            )
            .unwrap()
            .unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].len(), 4);
        assert_eq!(*args[0][0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(*args[0][1].symbol(), Symbol::LeftParenthesis);
        assert_eq!(*args[0][2].symbol(), Symbol::Word("world".to_string()));
        assert_eq!(*args[0][3].symbol(), Symbol::RightParenthesis);
        assert_eq!(args[1].len(), 4);
        assert_eq!(*args[1][0].symbol(), Symbol::Word("world".to_string()));
        assert_eq!(*args[1][1].symbol(), Symbol::LeftParenthesis);
        assert_eq!(*args[1][2].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(*args[1][3].symbol(), Symbol::RightParenthesis);
    }

    #[test]
    fn multi_arg_awkward_comma() {
        let mut stream = tests::setup("(set(1,2),set(3,4))");
        let mut processor = Processor::default();
        let args = processor
            .call_read_args(
                &stream.peek().unwrap().position().clone(),
                &mut Pragma::root(),
                &mut stream,
            )
            .unwrap()
            .unwrap();
        assert_eq!(args.len(), 4);
        assert_eq!(args[0].len(), 3);
        assert_eq!(*args[0][0].symbol(), Symbol::Word("set".to_string()));
        assert_eq!(*args[0][1].symbol(), Symbol::LeftParenthesis);
        assert_eq!(*args[0][2].symbol(), Symbol::Digit(1));
        assert_eq!(args[1].len(), 2);
        assert_eq!(*args[1][0].symbol(), Symbol::Digit(2));
        assert_eq!(*args[1][1].symbol(), Symbol::RightParenthesis);
        assert_eq!(args[2].len(), 3);
        assert_eq!(*args[2][0].symbol(), Symbol::Word("set".to_string()));
        assert_eq!(*args[2][1].symbol(), Symbol::LeftParenthesis);
        assert_eq!(*args[2][2].symbol(), Symbol::Digit(3));
        assert_eq!(args[3].len(), 2);
        assert_eq!(*args[3][0].symbol(), Symbol::Digit(4));
        assert_eq!(*args[3][1].symbol(), Symbol::RightParenthesis);
    }

    #[test]
    fn body_single_word() {
        let mut stream = tests::setup("hello");
        let mut processor = Processor::default();
        let body = processor.define_read_body(&mut stream);
        assert_eq!(body.len(), 1);
        assert_eq!(*body[0].symbol(), Symbol::Word("hello".to_string()));
    }

    #[test]
    fn body_multi_word() {
        let mut stream = tests::setup("hello world");
        let mut processor = Processor::default();
        let body = processor.define_read_body(&mut stream);
        assert_eq!(body.len(), 3);
        assert_eq!(*body[0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(*body[1].symbol(), Symbol::Whitespace(Whitespace::Space));
        assert_eq!(*body[2].symbol(), Symbol::Word("world".to_string()));
    }

    #[test]
    fn body_multi_line_no_escape() {
        let mut stream = tests::setup("hello\nworld");
        let mut processor = Processor::default();
        let body = processor.define_read_body(&mut stream);
        assert_eq!(body.len(), 1);
        assert_eq!(*body[0].symbol(), Symbol::Word("hello".to_string()));
    }

    #[test]
    fn body_multi_line_with_escape() {
        let mut stream = tests::setup("hello \\\nworld");
        let mut processor = Processor::default();
        let body = processor.define_read_body(&mut stream);
        assert_eq!(body.len(), 3);
        assert_eq!(*body[0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(*body[1].symbol(), Symbol::Whitespace(Whitespace::Space));
        assert_eq!(*body[2].symbol(), Symbol::Word("world".to_string()));
    }
}
