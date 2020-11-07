use std::path::PathBuf;

use super::{Rule, Statement};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub start: (usize, (usize, usize)),
    pub end: (usize, (usize, usize)),
    pub line: String,
    pub statement: Statement,
}

type ResultNodeVec = Result<Vec<Node>, String>;

impl Node {
    pub fn from_expr(
        wd: PathBuf,
        source: &str,
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<Node, String> {
        let node = Node {
            start: (
                pair.as_span().start_pos().pos(),
                pair.as_span().start_pos().line_col(),
            ),
            end: (
                pair.as_span().end_pos().pos(),
                pair.as_span().end_pos().line_col(),
            ),
            line: pair.as_span().as_str().to_string(),
            statement: match pair.as_rule() {
                Rule::config => Statement::Config(
                    pair.into_inner()
                        .map(|x| Node::from_expr(wd.clone(), source, x))
                        .collect::<ResultNodeVec>()?,
                ),
                Rule::class => {
                    let mut parts = pair.into_inner();
                    Statement::Class {
                        ident: Box::new({
                            Node::from_expr(wd.clone(), source, parts.next().unwrap())?
                        }),
                        extends: None,
                        props: parts
                            .map(|x| Node::from_expr(wd.clone(), source, x))
                            .collect::<ResultNodeVec>()?,
                    }
                }
                Rule::classextends => {
                    let mut parts = pair.into_inner();
                    Statement::Class {
                        ident: Box::new({
                            Node::from_expr(wd.clone(), source, parts.next().unwrap())?
                        }),
                        extends: Some(Box::new({
                            Node::from_expr(wd.clone(), source, parts.next().unwrap())?
                        })),
                        props: parts
                            .map(|x| Node::from_expr(wd.clone(), source, x))
                            .collect::<ResultNodeVec>()?,
                    }
                }
                Rule::classdef => Statement::ClassDef(Box::new({
                    Node::from_expr(wd, source, pair.into_inner().next().unwrap())?
                })),
                Rule::classdelete => Statement::ClassDelete(Box::new({
                    Node::from_expr(wd, source, pair.into_inner().next().unwrap())?
                })),
                Rule::prop => {
                    let mut parts = pair.into_inner();
                    Statement::Property {
                        ident: Box::new({
                            Node::from_expr(wd.clone(), source, parts.next().unwrap())?
                        }),
                        value: Box::new(Node::from_expr(wd, source, parts.next().unwrap())?),
                        expand: false,
                    }
                }
                Rule::propexpand => {
                    let mut parts = pair.into_inner();
                    Statement::Property {
                        ident: Box::new({
                            Node::from_expr(wd.clone(), source, parts.next().unwrap())?
                        }),
                        value: Box::new(Node::from_expr(wd, source, parts.next().unwrap())?),
                        expand: true,
                    }
                }
                Rule::bool => Statement::Bool(pair.as_str() == "true"),
                Rule::array => Statement::Array(
                    pair.into_inner()
                        .map(|x| Node::from_expr(wd.clone(), source, x))
                        .collect::<ResultNodeVec>()?,
                ),
                Rule::float => Statement::Float(pair.as_str().parse().unwrap()),
                Rule::integer => Statement::Integer(pair.as_str().parse().unwrap()),
                Rule::string => Statement::Str(String::from(pair.as_str())),
                Rule::ident => Statement::Ident(String::from(pair.as_str())),
                Rule::identarray => {
                    Statement::IdentArray(String::from(pair.into_inner().next().unwrap().as_str()))
                }
                Rule::char => Statement::Char(pair.as_str().chars().next().unwrap()),
                Rule::unquoted => Statement::Unquoted(
                    pair.into_inner()
                        .map(|x| Node::from_expr(wd.clone(), source, x))
                        .collect::<ResultNodeVec>()?,
                ),
                // Special
                Rule::special => match pair.as_str() {
                    "__FILE__" => Statement::FILE,
                    "__LINE__" => Statement::LINE,
                    _ => panic!("Special was not handled. Please report this to ArmaLint"),
                },
                // Ignored
                Rule::EOI => Statement::Gone,
                Rule::file => unimplemented!(),
                Rule::string_wrapper => unimplemented!(),
                Rule::item => unimplemented!(),
                Rule::value => unimplemented!(),
                Rule::COMMENT => unimplemented!(),
                Rule::WHITESPACE => unimplemented!(),
            },
        };
        Ok(node)
    }
}
