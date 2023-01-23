use std::io::Cursor;

use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_tokens::{whitespace, Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{
    error::Error,
    model::Array,
    rapify::{compressed_int_len, Rapify, WriteExt},
    Options,
};

use super::{Entry, Ident, Parse};

#[derive(Debug, Clone, PartialEq)]
/// A class definition
pub enum Class {
    /// A local class definition
    ///
    /// ```cpp
    /// class CfgPatches {
    ///     ...
    /// };
    /// ```
    Local {
        /// The name of the class
        name: Ident,
        /// The parent class
        ///
        /// ```cpp
        /// class MyClass: MyParent {
        ///    ...
        /// };
        /// ```
        parent: Option<Ident>,
        /// The children of the class
        children: Children,
    },
    /// An external class definition
    ///
    /// ```cpp
    /// class CfgPatches;
    /// ```
    External {
        /// The name of the class
        name: Ident,
    },
}

impl Class {
    #[must_use]
    /// Get the name of the class
    pub const fn name(&self) -> &Ident {
        match self {
            Self::External { name } | Self::Local { name, .. } => name,
        }
    }
}

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
        let last = whitespace::skip(tokens);
        let parent = if let Some(token) = tokens.peek() {
            if token.symbol() == &Symbol::Colon {
                tokens.next();
                let last = whitespace::skip(tokens).unwrap_or_else(|| from.clone());
                Some(Ident::parse(options, tokens, &last)?)
            } else {
                None
            }
        } else {
            return Err(Error::UnexpectedEOF {
                token: Box::new(last.unwrap_or_else(|| from.clone())),
            });
        };
        // read children
        let last = whitespace::skip_newline(tokens);
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

impl Rapify for Class {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = 0;

        match self {
            Self::External { name } => {
                output.write_all(&[3])?;
                output.write_cstring(&name.to_string())?;
                written += 1;
            }
            Self::Local {
                name: _,
                parent,
                children,
            } => {
                if let Some(parent) = &parent {
                    output.write_cstring(parent.to_string())?;
                    written += parent.to_string().len() + 1;
                } else {
                    written += output.write(b"\0")?;
                }
                written += output.write_compressed_int(children.0 .0.len() as u32)?;

                let children_len = children
                    .0
                     .0
                    .iter()
                    .map(|(n, p)| n.len() + 1 + p.rapified_length())
                    .sum::<usize>();
                let mut class_offset = offset + written + children_len;
                let mut class_bodies: Vec<Cursor<Box<[u8]>>> = Vec::new();
                let pre_children = written;

                for (name, property) in &children.0 .0 {
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
                            if let Self::Local { .. } = c {
                                output.write_u32::<LittleEndian>(class_offset as u32)?;
                                written += 4;
                                let buffer: Box<[u8]> =
                                    vec![0; c.rapified_length()].into_boxed_slice();
                                let mut cursor = Cursor::new(buffer);
                                let body_size = c.rapify(&mut cursor, class_offset)?;
                                assert_eq!(body_size, c.rapified_length());
                                class_offset += body_size;
                                class_bodies.push(cursor);
                            }
                        }
                        Property::Delete(_) => continue,
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
            }
        }

        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::External { .. } => 0,
            Self::Local {
                name: _,
                parent,
                children,
            } => {
                let parent_length = parent.as_ref().map_or(0, |parent| parent.to_string().len());
                parent_length
                    + 1
                    + compressed_int_len(children.0 .0.len() as u32)
                    + children
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
                                    // Property::Delete(i) => i.to_string().len() + 1,
                                }
                        })
                        .sum::<usize>()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
/// The list of children of a class
pub struct Children(pub Properties);

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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Properties(pub Vec<(String, Property)>);

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
        // whitespace::skip(tokens);
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
                        })
                    }
                },
                None => {
                    return Err(Error::UnexpectedEOF {
                        token: Box::new(from.clone()),
                    })
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
                            let last = whitespace::skip(tokens);
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
                        let last = whitespace::skip(tokens);
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

#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    Entry(Entry),
    Class(Class),
    Delete(Ident),
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
            Self::Class(c) => match c {
                Class::Local { .. } => vec![0],
                Class::External { .. } => vec![3],
            },
            Self::Delete(_) => {
                vec![4]
            }
        }
    }
}

impl Rapify for Property {
    fn rapify<O: std::io::Write>(
        &self,
        _output: &mut O,
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
                Self::Class(c) => match c {
                    Class::Local { .. } => 4,
                    Class::External { .. } => 0,
                },
                Self::Delete(_) => 0,
            }
    }
}

#[cfg(test)]
mod tests {
    use hemtt_tokens::Token;
    use peekmore::PeekMore;

    use crate::{
        model::{class::Property, Array, Children, Entry, Number, Parse, Str},
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
            &crate::Options::default(),
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
        let Class::Local {name, parent, children} = super::Class::parse(&crate::Options::default(), &mut tokens, &Token::builtin(None)).unwrap() else {
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
        let Class::Local {name, parent, children} = super::Class::parse(&crate::Options::default(), &mut tokens, &Token::builtin(None)).unwrap() else {
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
        let Class::Local {name, parent, children} = super::Class::parse(&crate::Options::default(), &mut tokens, &Token::builtin(None)).unwrap() else {
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
