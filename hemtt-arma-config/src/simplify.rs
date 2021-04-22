use super::parser::{Node, Statement, AST};
use crate::ArmaConfigError;

#[derive(Debug)]
pub struct Config {
    pub root: Class,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub parent: String,
    pub external: bool,
    pub deletion: bool,
    pub entries: Vec<(String, Entry)>,
}

#[derive(Debug, Clone)]
pub enum Entry {
    Str(String),
    Float(f32),
    Int(i32),
    Array(Array),
    Class(Class),
    Invisible(Vec<(String, Entry)>),
}

impl From<Entry> for ArrayElement {
    fn from(e: Entry) -> Self {
        match e {
            Entry::Str(v) => Self::Str(v),
            Entry::Float(v) => Self::Float(v),
            Entry::Int(v) => Self::Int(v),
            Entry::Array(v) => Self::Array(v),
            _ => panic!("Invalid item was found in array: {:?}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    pub expand: bool,
    pub elements: Vec<ArrayElement>,
}

#[derive(Debug, Clone)]
pub enum ArrayElement {
    Str(String),
    Float(f32),
    Int(i32),
    Array(Array),
}

impl Config {
    pub fn from_ast(ast: AST) -> Result<Self, ArmaConfigError> {
        if let Statement::Config(inner) = ast.config.statement {
            Ok(Config {
                root: Class {
                    parent: String::new(),
                    external: false,
                    deletion: false,
                    entries: get_entries(inner)?,
                },
            })
        } else {
            Err(ArmaConfigError::NotRoot)
        }
    }
}

pub fn get_entries(nodes: Vec<Node>) -> Result<Vec<(String, Entry)>, ArmaConfigError> {
    let mut entries = Vec::new();
    for node in nodes {
        if let Some((ident, entry)) = get_entry(node)? {
            if let Entry::Invisible(e) = entry {
                for i in e {
                    entries.push(i);
                }
            } else {
                entries.push((ident, entry));
            }
        }
    }
    Ok(entries)
}

pub fn get_entry(node: Node) -> Result<Option<(String, Entry)>, ArmaConfigError> {
    Ok(match node.statement {
        Statement::Class {
            ident,
            extends,
            props,
        } => Some((
            if let Statement::Ident(i) = ident.statement {
                i
            } else {
                panic!()
            },
            Entry::Class(Class {
                parent: {
                    if let Some(ex) = extends {
                        if let Statement::Ident(i) = ex.statement {
                            i
                        } else {
                            panic!()
                        }
                    } else {
                        String::new()
                    }
                },
                deletion: false,
                external: false,
                entries: get_entries(props)?,
            }),
        )),
        Statement::ClassDef(ident) => Some((
            if let Statement::Ident(i) = ident.statement {
                i
            } else {
                panic!()
            },
            Entry::Class(Class {
                parent: String::new(),
                deletion: false,
                external: true,
                entries: Vec::new(),
            }),
        )),
        Statement::Property {
            ident,
            value,
            expand,
        } => Some((
            if let Statement::Ident(i) = ident.statement {
                i
            } else {
                panic!()
            },
            get_value(value.statement, expand)?,
        )),
        Statement::Config(inner) => Some((String::new(), Entry::Invisible(get_entries(inner)?))),
        // Ignore
        _ => {
            panic!("Not ready for {:#?}", node);
        }
    })
}

pub fn get_value(statement: Statement, expand: bool) -> Result<Entry, ArmaConfigError> {
    Ok(match statement {
        Statement::Integer(val) => Entry::Int(val),
        Statement::Float(val) => Entry::Float(val),
        Statement::Str(val) => Entry::Str(val),
        Statement::Array(val) => Entry::Array(Array {
            expand,
            elements: get_array(val)?,
        }),
        _ => {
            return Err(ArmaConfigError::InvalidProperty(format!(
                "Invalid property type `{:?}`",
                statement
            )))
        }
    })
}

pub fn get_array(nodes: Vec<Node>) -> Result<Vec<ArrayElement>, ArmaConfigError> {
    let mut elements = Vec::new();
    for n in nodes {
        let expand = if let Statement::Property { expand: e, .. } = n.statement {
            e
        } else {
            false
        };
        elements.push(get_value(n.statement, expand)?.into());
    }
    Ok(elements)
}
