use hemtt_tokens::{whitespace, Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{Array, Children, Class, Entry, Error, Ident, Properties, Property};

use super::{Options, Parse};

impl Parse for Class {
    fn parse(
        options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Some(token) = tokens.next() {
            if let Symbol::Word(word) = token.symbol() {
                if word != "class" {
                    return Err(Error::UnexpectedToken {
                        token: Box::new(token.clone()),
                        expected: vec![Symbol::Word("class".into())],
                    });
                }
                word.to_string()
            } else {
                return Err(Error::UnexpectedToken {
                    token: Box::new(token.clone()),
                    expected: vec![Symbol::Word("class".into())],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF {
                token: Box::new(from.clone()),
            });
        };
        // get name
        whitespace::skip(tokens);
        let name = Ident::parse(options, tokens, from)?;
        // Check for : and parent
        let skipped = whitespace::skip(tokens);
        let last = skipped.last().cloned();
        let parent = if let Some(token) = tokens.peek() {
            if token.symbol() == &Symbol::Colon {
                tokens.next();
                let skipped = whitespace::skip(tokens);
                let last = skipped.last().cloned();
                Some(Ident::parse(
                    options,
                    tokens,
                    &last.unwrap_or_else(|| from.clone()),
                )?)
            } else {
                None
            }
        } else {
            return Err(Error::UnexpectedEOF {
                token: Box::new(last.unwrap_or_else(|| from.clone())),
            });
        };
        // read children
        let skipped = whitespace::skip_newline(tokens);
        let last = skipped.last().cloned();
        if let Some(token) = tokens.peek() {
            if token.symbol() == &Symbol::Semicolon {
                return Ok(Self::External { name });
            }
        }
        let children = Children::parse(options, tokens, &last.unwrap_or_else(|| from.clone()))?;
        Ok(Self::Local {
            name,
            parent,
            children,
        })
    }
}

impl Parse for Children {
    fn parse(
        options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Some(token) = tokens.next() {
            if token.symbol() != &Symbol::LeftBrace {
                return Err(Error::UnexpectedToken {
                    token: Box::new(token),
                    expected: vec![Symbol::LeftBrace, Symbol::Semicolon],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF {
                token: Box::new(from.clone()),
            });
        }
        let children = Properties::parse(options, tokens, from)?;
        if let Some(token) = tokens.next() {
            if token.symbol() != &Symbol::RightBrace {
                return Err(Error::UnexpectedToken {
                    token: Box::new(token),
                    expected: vec![Symbol::RightBrace],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF {
                token: Box::new(from.clone()),
            });
        }
        Ok(Self(children))
    }
}

impl Parse for Properties {
    #[allow(clippy::too_many_lines)]
    fn parse(
        options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut entries = Vec::new();
        loop {
            let ident = match tokens.peek() {
                Some(token) => match token.symbol() {
                    Symbol::Word(ref ident) => {
                        if ident == "class" {
                            None
                        } else {
                            let token = token.clone();
                            Some(Ident::parse(options, tokens, &token)?)
                        }
                    }
                    Symbol::Digit(_) => {
                        let token = token.clone();
                        Some(Ident::parse(options, tokens, &token)?)
                    }
                    Symbol::RightBrace | Symbol::Eoi => break,
                    Symbol::Whitespace(_) | Symbol::Newline | Symbol::Comment(_) => {
                        tokens.next();
                        continue;
                    }
                    _ => {
                        return Err(Error::ExpectedIdent {
                            token: Box::new(token.clone()),
                        });
                    }
                },
                None => {
                    return Err(Error::UnexpectedEOF {
                        token: Box::new(from.clone()),
                    });
                }
            };
            whitespace::skip(tokens);
            'read_ident: {
                match ident {
                    None => {
                        let class = Class::parse(options, tokens, from)?;
                        entries.push((class.name().to_string(), Property::Class(class)));
                    }
                    Some(ident) => {
                        if ident.to_string() == "delete" {
                            let skipped = whitespace::skip(tokens);
                            let last = skipped.last().cloned();
                            let ident = Ident::parse(
                                options,
                                tokens,
                                &last.unwrap_or_else(|| from.clone()),
                            )?;
                            entries.push((ident.to_string(), Property::Delete(ident)));
                            break 'read_ident;
                        }
                        // Check for array
                        let array_expand =
                            if tokens.peek().unwrap().symbol() == &Symbol::LeftBracket {
                                tokens.next();
                                tokens.next();
                                whitespace::skip(tokens);
                                match tokens.peek().map_or(Symbol::Void, |t| t.symbol().clone()) {
                                    Symbol::Plus => {
                                        tokens.next();
                                        true
                                    }
                                    _ => false,
                                }
                            } else {
                                whitespace::skip(tokens);
                                false
                            };
                        whitespace::skip(tokens);
                        if tokens.peek().map_or(Symbol::Void, |t| t.symbol().clone())
                            != Symbol::Assignment
                        {
                            return Err(Error::UnexpectedToken {
                                token: Box::new(tokens.next().unwrap()),
                                expected: vec![Symbol::Assignment],
                            });
                        }
                        tokens.next();
                        let skipped = whitespace::skip(tokens);
                        let last = skipped.last().cloned();
                        let entry = if array_expand {
                            let array = Array::parse(options, tokens, from)?;
                            Entry::Array(Array {
                                elements: array.elements,
                                expand: true,
                            })
                        } else {
                            Entry::parse(options, tokens, &last.unwrap_or_else(|| from.clone()))?
                        };
                        entries.push((ident.to_string(), Property::Entry(entry)));
                    }
                }
            }
            let t = tokens
                .peek()
                .map_or_else(|| Token::builtin(None), std::clone::Clone::clone);
            if t.symbol() == &Symbol::Semicolon {
                tokens.next();
            } else {
                return Err(Error::UnexpectedToken {
                    token: Box::new(t),
                    expected: vec![Symbol::Semicolon],
                });
            }
        }
        Ok(Self(entries))
    }
}

#[cfg(test)]
mod tests {
    use hemtt_tokens::Token;
    use peekmore::PeekMore;

    use crate::{
        model::{Array, Children, Entry, Number, Property, Str},
        parse::Parse,
        Class,
    };

    #[test]
    fn children() {
        let mut tokens = hemtt_preprocessor::preprocess_string(
            r#"{
    alpha = "Alpha";
    version = 10;
    points[] = {
        {1,2,3},
        {4,5,6}
    };
    extra[] += {7,8,9};
    scale = 1.5;
}"#,
        )
        .unwrap()
        .into_iter()
        .peekmore();
        let children = Children::parse(
            &super::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(
            children.0 .0,
            vec![
                (
                    "alpha".to_string(),
                    Property::Entry(Entry::Str(Str("Alpha".to_string())))
                ),
                (
                    "version".to_string(),
                    Property::Entry(Entry::Number(Number::Int32(10)))
                ),
                (
                    "points".to_string(),
                    Property::Entry(Entry::Array(Array {
                        expand: false,
                        elements: vec![
                            Entry::Array(Array {
                                expand: false,
                                elements: vec![
                                    Entry::Number(Number::Int32(1)),
                                    Entry::Number(Number::Int32(2)),
                                    Entry::Number(Number::Int32(3)),
                                ]
                            }),
                            Entry::Array(Array {
                                expand: false,
                                elements: vec![
                                    Entry::Number(Number::Int32(4)),
                                    Entry::Number(Number::Int32(5)),
                                    Entry::Number(Number::Int32(6)),
                                ]
                            }),
                        ]
                    }))
                ),
                (
                    "extra".to_string(),
                    Property::Entry(Entry::Array(Array {
                        expand: true,
                        elements: vec![
                            Entry::Number(Number::Int32(7)),
                            Entry::Number(Number::Int32(8)),
                            Entry::Number(Number::Int32(9)),
                        ]
                    }))
                ),
                (
                    "scale".to_string(),
                    Property::Entry(Entry::Number(Number::Float32(1.5)))
                ),
            ]
        );
    }

    #[test]
    fn class() {
        let mut tokens = hemtt_preprocessor::preprocess_string(
            r#"class HEMTT {
    alpha = "Alpha";
    version = 10;
}"#,
        )
        .unwrap()
        .into_iter()
        .peekmore();
        let Class::Local {name, parent, children} = super::Class::parse(&super::Options::default(), &mut tokens, &Token::builtin(None)).unwrap() else {
            panic!("Expected local class");
        };
        assert_eq!(name.to_string(), "HEMTT");
        assert_eq!(parent, None);
        assert_eq!(
            children.0 .0,
            vec![
                (
                    "alpha".to_string(),
                    Property::Entry(Entry::Str(Str("Alpha".to_string())))
                ),
                (
                    "version".to_string(),
                    Property::Entry(Entry::Number(Number::Int32(10)))
                ),
            ]
        );
    }

    #[test]
    fn class_parent() {
        let mut tokens = hemtt_preprocessor::preprocess_string(
            r#"class HEMTT: CfgPatches 
{
    alpha = "Alpha";
    version = 10;
}"#,
        )
        .unwrap()
        .into_iter()
        .peekmore();
        let Class::Local {name, parent, children} = super::Class::parse(&super::Options::default(), &mut tokens, &Token::builtin(None)).unwrap() else {
            panic!("Expected local class");
        };
        assert_eq!(name.to_string(), "HEMTT");
        assert_eq!(parent.unwrap().to_string(), "CfgPatches");
        assert_eq!(
            children.0 .0,
            vec![
                (
                    "alpha".to_string(),
                    Property::Entry(Entry::Str(Str("Alpha".to_string())))
                ),
                (
                    "version".to_string(),
                    Property::Entry(Entry::Number(Number::Int32(10)))
                ),
            ]
        );
    }

    #[test]
    fn class_parent_join() {
        let mut tokens = hemtt_preprocessor::preprocess_string(
            r#"#define DOUBLES(a,b) a##b
class HEMTT: DOUBLES(hello,world) 
{
    alpha = "Alpha";
    version = 10;
}"#,
        )
        .unwrap()
        .into_iter()
        .peekmore();
        let Class::Local {name, parent, children} = super::Class::parse(&super::Options::default(), &mut tokens, &Token::builtin(None)).unwrap() else {
            panic!("Expected local class");
        };
        assert_eq!(name.to_string(), "HEMTT");
        assert_eq!(parent.unwrap().to_string(), "helloworld");
        assert_eq!(
            children.0 .0,
            vec![
                (
                    "alpha".to_string(),
                    Property::Entry(Entry::Str(Str("Alpha".to_string())))
                ),
                (
                    "version".to_string(),
                    Property::Entry(Entry::Number(Number::Int32(10)))
                ),
            ]
        );
    }
}
