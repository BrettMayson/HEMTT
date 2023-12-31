#[cfg(feature = "compiler")]
pub mod compiler;
#[cfg(feature = "parser")]
pub mod parser;

pub mod analyze;
mod error;
mod misc;

use std::ops::Range;

pub use self::error::Error;

use arma3_wiki::model::Version;
#[doc(no_inline)]
pub use float_ord::FloatOrd as Scalar;
use parser::database::Database;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Statements {
    content: Vec<Statement>,
    /// The source code string of this section of code.
    /// This isn't required to actually be anything significant, but will be displayed in-game if a script error occurs.
    source: String,
}

impl Statements {
    #[must_use]
    pub fn content(&self) -> &[Statement] {
        &self.content
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    /// Adds a source string to this code chunk.
    pub fn with_source(self, source: String) -> Self {
        Self {
            content: self.content,
            source,
        }
    }

    #[must_use]
    pub fn span(&self) -> Range<usize> {
        let start = self
            .content
            .first()
            .map(|s| s.span().start)
            .unwrap_or_default();
        let end = self
            .content
            .last()
            .map(|s| s.span().end)
            .unwrap_or_default();
        start..end
    }

    #[must_use]
    /// Gets the highest version required by any command in this code chunk.
    pub fn required_version(&self, database: &Database) -> (String, Version, Range<usize>) {
        // TODO can probably replace String with Rc<str>
        fn extract_expression(
            expression: &Expression,
            database: &Database,
        ) -> Option<(String, Version, Range<usize>)> {
            match expression {
                Expression::NularCommand(command, span) => Some((
                    command.as_str().to_string(),
                    *database.command_version(command.as_str())?,
                    span.clone(),
                )),
                Expression::UnaryCommand(command, children, span) => {
                    let command_version = database.command_version(command.as_str())?;
                    let left = extract_expression(children, database)?;
                    let left_version = database.command_version(&left.0)?;
                    if command_version > left_version {
                        Some((command.as_str().to_string(), *command_version, span.clone()))
                    } else {
                        Some((left.0, left.1, left.2))
                    }
                }
                Expression::BinaryCommand(command, left, right, span) => {
                    let command_version = database.command_version(command.as_str())?;
                    let left = extract_expression(left, database)?;
                    let left_version = database.command_version(&left.0)?;
                    let right = extract_expression(right, database)?;
                    let right_version = database.command_version(&right.0)?;
                    if command_version > left_version && command_version > right_version {
                        Some((command.as_str().to_string(), *command_version, span.clone()))
                    } else if left_version > right_version {
                        Some((left.0, left.1, left.2))
                    } else {
                        Some((right.0, right.1, right.2))
                    }
                }
                Expression::Code(statements) => {
                    let (command, version, span) = statements.required_version(database);
                    Some((command.as_str().to_string(), version, span))
                }
                _ => None,
            }
        }
        let mut version = Version::new(0, 0);
        let mut span = 0..0;
        let mut command = String::new();
        for statement in &self.content {
            if let Some((used_command, command_version, command_span)) = match statement {
                Statement::AssignGlobal(_, expression, _)
                | Statement::AssignLocal(_, expression, _)
                | Statement::Expression(expression, _) => extract_expression(expression, database),
            } {
                if command_version > version {
                    command = used_command.to_string();
                    version = command_version;
                    span = command_span.clone();
                }
            }
        }
        (command, version, span)
    }
}

impl From<Vec<Statement>> for Statements {
    fn from(content: Vec<Statement>) -> Self {
        Self {
            content,
            source: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    AssignGlobal(String, Expression, Range<usize>),
    AssignLocal(String, Expression, Range<usize>),
    Expression(Expression, Range<usize>),
}

impl Statement {
    #[must_use]
    pub fn walk_statements(&self) -> Vec<&Self> {
        match self {
            Self::AssignGlobal(_, expression, _)
            | Self::AssignLocal(_, expression, _)
            | Self::Expression(expression, _) => vec![self]
                .into_iter()
                .chain(expression.walk_statements())
                .collect(),
        }
    }

    #[must_use]
    pub fn walk_expressions(&self) -> Vec<&Expression> {
        match self {
            Self::AssignGlobal(_, expression, _)
            | Self::AssignLocal(_, expression, _)
            | Self::Expression(expression, _) => expression.walk_expressions(),
        }
    }

    #[must_use]
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::AssignGlobal(_, _, span)
            | Self::AssignLocal(_, _, span)
            | Self::Expression(_, span) => span.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Code(Statements),
    String(String, Range<usize>),
    Number(Scalar<f32>, Range<usize>),
    Boolean(bool, Range<usize>),
    Array(Vec<Self>, Range<usize>),
    NularCommand(NularCommand, Range<usize>),
    UnaryCommand(UnaryCommand, Box<Self>, Range<usize>),
    BinaryCommand(BinaryCommand, Box<Self>, Box<Self>, Range<usize>),
    Variable(String, Range<usize>),
}

impl Expression {
    #[must_use]
    pub fn source(&self) -> String {
        match self {
            Self::Code(code) => code.source().to_string(),
            Self::String(string, _) => format!("\"{string}\""),
            Self::Number(number, _) => number.0.to_string(),
            Self::Boolean(boolean, _) => boolean.to_string(),
            Self::Array(array, _) => {
                let mut out = String::new();
                out.push('[');
                for (i, element) in array.iter().enumerate() {
                    if i != 0 {
                        out.push(',');
                    }
                    out.push_str(element.source().as_str());
                }
                out.push(']');
                out
            }
            Self::NularCommand(command, _) => command.as_str().to_string(),
            Self::UnaryCommand(command, child, _) => {
                format!("{} {}", command.as_str(), child.source())
            }
            Self::BinaryCommand(command, left, right, _) => {
                format!("{} {} {}", left.source(), command.as_str(), right.source())
            }
            Self::Variable(variable, _) => variable.to_string(),
        }
    }

    #[must_use]
    pub fn walk_statements(&self) -> Vec<&Statement> {
        let mut root = vec![];
        match self {
            Self::Code(code) => {
                for statement in code.content() {
                    root.extend(statement.walk_statements());
                }
            }
            Self::UnaryCommand(_, child, _) => {
                root.extend(child.walk_statements());
            }
            Self::BinaryCommand(_, left, right, _) => {
                root.extend(left.walk_statements());
                root.extend(right.walk_statements());
            }
            Self::Array(array, _) => {
                for element in array {
                    root.extend(element.walk_statements());
                }
            }
            _ => {}
        }
        root
    }

    #[must_use]
    pub fn walk_expressions(&self) -> Vec<&Self> {
        let mut root = vec![self];
        match self {
            Self::Code(code) => {
                for statement in code.content() {
                    root.extend(statement.walk_expressions());
                }
            }
            Self::UnaryCommand(_, child, _) => {
                root.extend(child.walk_expressions());
            }
            Self::BinaryCommand(_, left, right, _) => {
                root.extend(left.walk_expressions());
                root.extend(right.walk_expressions());
            }
            Self::Array(array, _) => {
                for element in array {
                    root.extend(element.walk_expressions());
                }
            }
            _ => {}
        }
        root
    }

    #[must_use]
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Code(code) => code.span(),
            Self::Array(_, span) => span.start - 1..span.end,
            Self::String(_, span)
            | Self::Number(_, span)
            | Self::Boolean(_, span)
            | Self::NularCommand(_, span)
            | Self::UnaryCommand(_, _, span)
            | Self::BinaryCommand(_, _, _, span)
            | Self::Variable(_, span) => span.clone(),
        }
    }

    #[must_use]
    pub fn full_span(&self) -> Range<usize> {
        match self {
            Self::Code(code) => code.span(),
            Self::Array(_, _) => self.span(),
            Self::String(_, span)
            | Self::Number(_, span)
            | Self::Boolean(_, span)
            | Self::NularCommand(_, span)
            | Self::Variable(_, span) => span.clone(),
            Self::UnaryCommand(_, child, span) => span.start..child.full_span().end,
            Self::BinaryCommand(_, left, right, _) => left.full_span().start..right.full_span().end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NularCommand {
    pub name: String,
}

impl NularCommand {
    #[must_use]
    pub fn is_constant(&self) -> bool {
        crate::parser::database::is_constant_command(&self.name)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryCommand {
    /// A named command.
    /// Non-alphanumeric commands (such as `==` or `!`) should not go here.
    Named(String),
    Plus,
    Minus,
    Not,
}

impl UnaryCommand {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Named(name) => name,
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Not => "!",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryCommand {
    /// A named command.
    /// Non-alphanumeric commands (such as `==` or `!`) or commands with special precedence should not go here.
    Named(String),
    Or,
    And,
    Eq,
    NotEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    ConfigPath,
    Associate,
    Else,
    Add,
    Sub,
    Max,
    Min,
    Mul,
    Div,
    Rem,
    Mod,
    Atan2,
    Exp,
    Select,
}

impl BinaryCommand {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Named(name) => name,
            Self::Or => "||",
            Self::And => "&&",
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::ConfigPath => ">>",
            Self::GreaterEq => ">=",
            Self::LessEq => "<=",
            Self::Greater => ">",
            Self::Less => "<",
            Self::Else => "else",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Max => "max",
            Self::Min => "min",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
            Self::Mod => "mod",
            Self::Atan2 => "atan2",
            Self::Exp => "^",
            Self::Associate => ":",
            Self::Select => "#",
        }
    }
}
