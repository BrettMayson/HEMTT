use super::Node;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Config(Vec<Node>),
    Array(Vec<Node>),
    Float(f32),
    Integer(i32),
    Str(String),
    Bool(bool),
    Char(char),
    Unquoted(Vec<Node>),
    Property {
        ident: Box<Node>,
        value: Box<Node>,
        expand: bool,
    },
    Class {
        ident: Box<Node>,
        extends: Option<Box<Node>>,
        props: Vec<Node>,
    },
    ClassDef(Box<Node>),
    ClassDelete(Box<Node>),
    Ident(String),
    IdentArray(String),

    // Special
    FILE,
    LINE,

    Gone,
}
