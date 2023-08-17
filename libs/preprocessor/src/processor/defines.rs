use std::{collections::HashMap, rc::Rc};

use hemtt_common::position::Position;
use peekmore::{PeekMore, PeekMoreIterator};

use crate::{definition::Definition, output::Output, symbol::Symbol, token::Token, Error};

use super::Processor;

impl Processor {
    /// Reads the arguments of a macro call
    ///
    /// Expects the stream to be at the left parenthesis
    ///
    /// The stream is left after the closing parenthesis
    pub(crate) fn call_read_args(
        &mut self,
        callsite: &Position,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<Option<Vec<Vec<Token>>>, Error> {
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
                    self.define_use(callsite, stream, &mut inner)?;
                    arg.append(
                        &mut inner
                            .into_iter()
                            .map(std::convert::Into::into)
                            .collect::<Vec<Vec<Token>>>()
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
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<Vec<Token>, Error> {
        if !stream
            .next()
            .expect("peeked by caller")
            .symbol()
            .is_left_paren()
        {
            panic!("expected left parenthesis");
        }
        let mut need_comma = false;
        let mut args = Vec::new();
        while let Some(token) = stream.peek() {
            let symbol = token.symbol();
            if symbol.is_word() {
                if need_comma {
                    panic!("expected comma");
                }
                args.push(stream.next().expect("peeked above"));
                need_comma = true;
            } else if symbol.is_comma() {
                need_comma = false;
                stream.next();
                continue;
            } else if symbol.is_right_paren() {
                stream.next();
                break;
            } else if symbol.is_whitespace() {
                stream.next();
            } else {
                panic!("expected word or comma");
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
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<Vec<Token>, Error> {
        self.skip_whitespace(stream, None);
        let mut body = Vec::new();
        for token in stream.by_ref() {
            let symbol = token.symbol();
            if symbol.is_newline() {
                if body
                    .last()
                    .map_or(false, |t: &Token| t.symbol().is_escape())
                {
                    // remove the backslash
                    body.pop();
                } else {
                    return Ok(body);
                }
            }
            if !symbol.is_eoi() {
                body.push(token);
            }
        }
        Ok(body)
    }

    /// A define was used
    ///
    /// Expects the stream to be at the ident
    ///
    /// The stream is left on the next token after the end of the call
    pub(crate) fn define_use(
        &mut self,
        callsite: &Position,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        let Some(ident) = self.current_word(stream) else {
            panic!("expected ident for call");
        };
        let ident_string = ident.to_string();
        let Some((_source, body)) = self.defines.get(&ident, callsite) else {
            panic!("unknown macro, caller should check");
        };
        match body {
            Definition::Function(function) => {
                let Some(args) = self.call_read_args(callsite, stream)? else {
                    buffer.push(Output::Direct(ident));
                    return Ok(());
                };
                if args.len() != function.args().len() {
                    for arg in &args {
                        println!(
                            "- {}",
                            arg.iter()
                                .map(std::string::ToString::to_string)
                                .collect::<String>()
                        );
                    }
                    panic!(
                        "wrong number of arguments ({}) for {} at {:?} from {:?}",
                        args.len(),
                        ident_string,
                        ident.source().start(),
                        callsite,
                    );
                }
                let mut arg_defines = HashMap::new();
                for (arg, value) in function.args().iter().zip(args) {
                    arg_defines.insert(
                        Rc::from(arg.to_string().as_str()),
                        (arg.clone(), Definition::Value(value)),
                    );
                }
                self.defines.push(&ident_string, arg_defines);
                let mut layer = Vec::new();
                self.walk(
                    Some(callsite),
                    Some(&ident_string),
                    &mut function.stream(),
                    &mut layer,
                )?;
                buffer.push(Output::Macro(ident, layer));
                self.defines.pop();
                Ok(())
            }
            Definition::Value(body) => {
                let mut layer = Vec::new();
                self.walk(
                    Some(callsite),
                    Some(&ident_string),
                    &mut body.into_iter().peekmore(),
                    &mut layer,
                )?;
                buffer.push(Output::Macro(ident, layer));
                Ok(())
            }
            Definition::Void => Ok(()),
            Definition::Unit => panic!("unit macro used as value / function"),
        }
    }
}

#[cfg(test)]
mod tests {
    use hemtt_common::position::Position;

    use crate::{
        processor::{tests, Processor},
        symbol::Symbol,
        whitespace::Whitespace,
    };

    #[test]
    fn single_arg_single_word() {
        let mut stream = tests::setup("(hello)");
        let mut processor = Processor::default();
        let args = processor
            .call_read_args(&Position::builtin(), &mut stream)
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
            .call_read_args(&Position::builtin(), &mut stream)
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
            .call_read_args(&Position::builtin(), &mut stream)
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
            .call_read_args(&Position::builtin(), &mut stream)
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
            .call_read_args(&Position::builtin(), &mut stream)
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
            .call_read_args(&Position::builtin(), &mut stream)
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
            .call_read_args(&Position::builtin(), &mut stream)
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
        let body = processor.define_read_body(&mut stream).unwrap();
        assert_eq!(body.len(), 1);
        assert_eq!(*body[0].symbol(), Symbol::Word("hello".to_string()));
    }

    #[test]
    fn body_multi_word() {
        let mut stream = tests::setup("hello world");
        let mut processor = Processor::default();
        let body = processor.define_read_body(&mut stream).unwrap();
        assert_eq!(body.len(), 3);
        assert_eq!(*body[0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(*body[1].symbol(), Symbol::Whitespace(Whitespace::Space));
        assert_eq!(*body[2].symbol(), Symbol::Word("world".to_string()));
    }

    #[test]
    fn body_multi_line_no_escape() {
        let mut stream = tests::setup("hello\nworld");
        let mut processor = Processor::default();
        let body = processor.define_read_body(&mut stream).unwrap();
        assert_eq!(body.len(), 1);
        assert_eq!(*body[0].symbol(), Symbol::Word("hello".to_string()));
    }

    #[test]
    fn body_multi_line_with_escape() {
        let mut stream = tests::setup("hello \\\nworld");
        let mut processor = Processor::default();
        let body = processor.define_read_body(&mut stream).unwrap();
        assert_eq!(body.len(), 4);
        assert_eq!(*body[0].symbol(), Symbol::Word("hello".to_string()));
        assert_eq!(*body[1].symbol(), Symbol::Whitespace(Whitespace::Space));
        assert_eq!(*body[2].symbol(), Symbol::Newline);
        assert_eq!(*body[3].symbol(), Symbol::Word("world".to_string()));
    }
}
