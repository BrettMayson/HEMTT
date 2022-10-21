use std::{io::Cursor, iter::Peekable};

use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_tokens::{symbol::Symbol, whitespace, Token};

use crate::{
    error::Error,
    model::Array,
    rapify::{compressed_int_len, Rapify, WriteExt},
};

use super::{Entry, Ident, Parse};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Class {
    pub name: Ident,
    pub parent: String,
    pub external: bool,
    pub children: Children,
}

impl Parse for Class {
    fn parse(
        tokens: &mut Peekable<impl Iterator<Item = hemtt_tokens::Token>>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Some(token) = tokens.next() {
            if let Symbol::Word(word) = token.symbol() {
                if word != "class" {
                    return Err(Error::UnexpectedToken {
                        token: token.clone(),
                        expected: vec![Symbol::Word("class".into())],
                    });
                }
                word.to_string()
            } else {
                return Err(Error::UnexpectedToken {
                    token: token.clone(),
                    expected: vec![Symbol::Word("class".into())],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF);
        };
        // get name
        whitespace::skip(tokens);
        let name = Ident::parse(tokens)?;
        // Check for : and parent
        whitespace::skip(tokens);
        let parent = if let Some(token) = tokens.peek() {
            if token.symbol() == &Symbol::Colon {
                tokens.next();
                whitespace::skip(tokens);
                if let Some(token) = tokens.next() {
                    if let Symbol::Word(word) = token.symbol() {
                        word.to_string()
                    } else {
                        return Err(Error::ExpectedIdent { token });
                    }
                } else {
                    return Err(Error::UnexpectedEOF);
                }
            } else {
                "".to_string()
            }
        } else {
            return Err(Error::UnexpectedEOF);
        };
        // read children
        whitespace::skip_newline(tokens);
        let children = Children::parse(tokens)?;
        Ok(Self {
            name,
            parent,
            external: false,
            children,
        })
    }
}

impl Rapify for Class {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        if self.children.0 .0.is_empty() {
            return Ok(0);
        }
        output.write_cstring(&self.parent)?;
        let mut written = self.parent.len() + 1;
        written += output.write_compressed_int(self.children.0 .0.len() as u32)?;

        let children_len = self
            .children
            .0
             .0
            .iter()
            .map(|(n, p)| n.len() + 1 + p.rapified_length())
            .sum::<usize>();
        let mut class_offset = offset + written + children_len;
        let mut class_bodies: Vec<Cursor<Box<[u8]>>> = Vec::new();
        let pre_children = written;

        for (name, property) in &self.children.0 .0 {
            let pre_write = written;
            let code = property.property_code();
            output.write_all(&code)?;
            written += code.len();
            output.write_cstring(name)?;
            written += name.len() + 1;
            match property {
                Property::Entry(e) => {
                    written += e.rapify(output, offset)?;
                }
                Property::Class(c) => {
                    if c.external {
                        continue;
                    }
                    output.write_u32::<LittleEndian>(class_offset as u32)?;
                    written += 4;

                    let buffer: Box<[u8]> = vec![0; c.rapified_length()].into_boxed_slice();
                    let mut cursor = Cursor::new(buffer);
                    class_offset += c.rapify(&mut cursor, class_offset)?;

                    class_bodies.push(cursor);
                }
            }
            assert_eq!(
                written - pre_write,
                property.rapified_length() + name.len() + 1
            );
        }

        assert_eq!(written - pre_children, children_len);

        for cursor in class_bodies {
            output.write_all(cursor.get_ref())?;
            written += cursor.get_ref().len();
        }

        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        if self.children.0 .0.is_empty() {
            return 0;
        }
        self.parent.len()
            + 1
            + compressed_int_len(self.children.0 .0.len() as u32)
            + self
                .children
                .0
                 .0
                .iter()
                .map(|(n, p)| {
                    n.len()
                        + 1
                        + p.rapified_length()
                        + match p {
                            Property::Class(c) => c.rapified_length(),
                            _ => 0,
                        }
                })
                .sum::<usize>()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Children(pub Properties);

impl Parse for Children {
    fn parse(
        tokens: &mut Peekable<impl Iterator<Item = hemtt_tokens::Token>>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Some(token) = tokens.next() {
            if token.symbol() != &Symbol::LeftBrace {
                return Err(Error::UnexpectedToken {
                    token,
                    expected: vec![Symbol::LeftBrace],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF);
        }
        let children = Properties::parse(tokens)?;
        if let Some(token) = tokens.next() {
            if token.symbol() != &Symbol::RightBrace {
                return Err(Error::UnexpectedToken {
                    token,
                    expected: vec![Symbol::RightBrace],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF);
        }
        Ok(Self(children))
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Properties(pub Vec<(String, Property)>);

impl Parse for Properties {
    fn parse(
        tokens: &mut Peekable<impl Iterator<Item = hemtt_tokens::Token>>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut entries = Vec::new();
        loop {
            let ident = match tokens.peek() {
                Some(token) => match token.symbol() {
                    Symbol::Word(ref ident) => {
                        let ident = ident.to_string();
                        if ident != "class" {
                            tokens.next();
                        }
                        ident
                    }
                    Symbol::RightBrace | Symbol::Eoi => break,
                    Symbol::Whitespace(_) | Symbol::Newline => {
                        tokens.next();
                        continue;
                    }
                    _ => {
                        return Err(Error::ExpectedIdent {
                            token: token.clone(),
                        })
                    }
                },
                None => return Err(Error::UnexpectedEOF),
            };
            whitespace::skip(tokens);
            match ident.as_str() {
                "class" => {
                    let class = Class::parse(tokens)?;
                    entries.push((class.name.to_string(), Property::Class(class)));
                }
                "delete" => unimplemented!("delete"),
                _ => {
                    // Check for array
                    let array_expand = if tokens.peek().unwrap().symbol() == &Symbol::LeftBracket {
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
                            token: tokens.next().unwrap(),
                            expected: vec![Symbol::Assignment],
                        });
                    }
                    tokens.next();
                    whitespace::skip(tokens);
                    let entry = if array_expand {
                        let array = Array::parse(tokens)?;
                        Entry::Array(Array {
                            elements: array.elements,
                            expand: true,
                        })
                    } else {
                        Entry::parse(tokens)?
                    };
                    entries.push((ident, Property::Entry(entry)));
                }
            }
            let t = tokens.peek().map_or(Token::builtin(), |t| t.clone());
            if t.symbol() == &Symbol::Semicolon {
                tokens.next();
            } else {
                return Err(Error::UnexpectedToken {
                    token: t,
                    expected: vec![Symbol::Semicolon],
                });
            }
        }
        Ok(Self(entries))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    Entry(Entry),
    Class(Class),
}

impl Property {
    pub fn property_code(&self) -> Vec<u8> {
        match self {
            Self::Entry(e) => match e {
                Entry::Str(s) => vec![1, s.rapified_code().unwrap()],
                Entry::Number(n) => vec![1, n.rapified_code().unwrap()],
                Entry::Array(a) => {
                    if a.expand {
                        vec![5, 1, 0, 0, 0]
                    } else {
                        vec![2]
                    }
                }
            },
            Self::Class(c) => {
                if c.external {
                    vec![3]
                } else {
                    vec![0]
                }
            }
        }
    }
}

impl Rapify for Property {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        unreachable!()
    }

    fn rapified_length(&self) -> usize {
        self.property_code().len()
            + match self {
                Self::Entry(e) => {
                    match e {
                        Entry::Str(s) => s.rapified_length(),
                        Entry::Number(n) => n.rapified_length(),
                        Entry::Array(a) => a.rapified_length(),
                        // Entry::Array(a) => {
                        //     compressed_int_len(a.elements.len() as u32)
                        //         + a.elements
                        //             .iter()
                        //             .map(Rapify::rapified_length)
                        //             .sum::<usize>()
                        // }
                    }
                }
                Self::Class(c) => {
                    if c.external {
                        0
                    } else {
                        4
                    }
                }
            }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{class::Property, Array, Children, Entry, Number, Parse, Str};

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
        .peekable();
        let children = Children::parse(&mut tokens).unwrap();
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
        .peekable();
        let class = super::Class::parse(&mut tokens).unwrap();
        assert_eq!(class.name.to_string(), "HEMTT");
        assert_eq!(class.parent, "");
        assert_eq!(
            class.children.0 .0,
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
        .peekable();
        let class = super::Class::parse(&mut tokens).unwrap();
        assert_eq!(class.name.to_string(), "HEMTT");
        assert_eq!(class.parent, "CfgPatches");
        assert_eq!(
            class.children.0 .0,
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
