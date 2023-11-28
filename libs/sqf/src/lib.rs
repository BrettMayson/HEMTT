#[cfg(feature = "compiler")]
pub mod compiler;
#[cfg(feature = "parser")]
pub mod parser;

pub mod analyze;
mod error;
mod misc;

use std::ops::Range;

pub use self::error::Error;

use a3_wiki::model::Version;
#[doc(no_inline)]
pub use float_ord::FloatOrd as Scalar;
use parser::database::Database;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Statements {
    pub content: Vec<Statement>,
    /// The source code string of this section of code.
    /// This isn't required to actually be anything significant, but will be displayed in-game if a script error occurs.
    pub source: String,
}

impl Statements {
    #[must_use]
    /// Adds a source string to this code chunk.
    pub fn with_source(self, source: String) -> Self {
        Self {
            content: self.content,
            source,
        }
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
                | Statement::Expression(expression) => extract_expression(expression, database),
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
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Code(Statements),
    String(String),
    Number(Scalar<f32>),
    Boolean(bool),
    Array(Vec<Self>, Range<usize>),
    NularCommand(NularCommand, Range<usize>),
    UnaryCommand(UnaryCommand, Box<Self>, Range<usize>),
    BinaryCommand(BinaryCommand, Box<Self>, Box<Self>, Range<usize>),
    Variable(String, Range<usize>),
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
