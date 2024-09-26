use std::{collections::HashSet, sync::Arc};

use arma3_wiki::model::{Arg, Call, Param, Value};
use tracing::{trace, warn};

use crate::{parser::database::Database, Expression};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GameValue {
    Anything,
    // Assignment, // as in z = call {x=1}???
    Array(Option<Expression>),
    Boolean(Option<Expression>),
    Code(Option<Expression>),
    Config,
    Control,
    DiaryRecord,
    Display,
    ForType(Option<Expression>),
    Group,
    HashMap,
    IfType,
    Location,
    Namespace,
    Number(Option<Expression>),
    Nothing,
    Object,
    ScriptHandle,
    Side,
    String(Option<Expression>),
    SwitchType,
    Task,
    TeamMember,
    WhileType,
    WithType,
}

impl GameValue {
    #[must_use]
    pub fn from_cmd(
        expression: &Expression,
        lhs_set: Option<&HashSet<Self>>,
        rhs_set: Option<&HashSet<Self>>,
        database: &Arc<Database>,
    ) -> HashSet<Self> {
        let mut return_types = HashSet::new();
        let cmd_name = expression.command_name().expect("has a name");
        let Some(command) = database.wiki().commands().get(cmd_name) else {
            trace!("cmd {cmd_name} not in db?"); //ToDo: this can't find short cmds like &&, ||
            return HashSet::from([Self::Anything]);
        };

        for syntax in command.syntax() {
            match syntax.call() {
                Call::Nular => {
                    if !matches!(expression, Expression::NularCommand(..)) {
                        continue;
                    }
                }
                Call::Unary(rhs_arg) => {
                    if !matches!(expression, Expression::UnaryCommand(..))
                        || !Self::match_set_to_arg(
                            rhs_set.expect("u args"),
                            rhs_arg,
                            syntax.params(),
                        )
                    {
                        continue;
                    }
                }
                Call::Binary(lhs_arg, rhs_arg) => {
                    if !matches!(expression, Expression::BinaryCommand(..))
                        || !Self::match_set_to_arg(
                            lhs_set.expect("b args"),
                            lhs_arg,
                            syntax.params(),
                        )
                        || !Self::match_set_to_arg(
                            rhs_set.expect("b args"),
                            rhs_arg,
                            syntax.params(),
                        )
                    {
                        continue;
                    }
                }
            }
            let value = &syntax.ret().0;
            return_types.insert(Self::from_wiki_value(value));
        }
        // trace!(
        //     "cmd [{}] = {}:{:?}",
        //     cmd_name,
        //     return_types.len(),
        //     return_types
        // );
        return_types
    }

    #[must_use]
    pub fn match_set_to_arg(set: &HashSet<Self>, arg: &Arg, params: &[Param]) -> bool {
        match arg {
            Arg::Item(name) => {
                // trace!("looking for {name} in {params:?}");
                let Some(param) = params.iter().find(|p| p.name() == name) else {
                    warn!("param not found");
                    return true;
                };
                Self::match_set_to_value(set, param.typ())
            }
            Arg::Array(_vec_arg) => {
                // todo: each individual array arg
                Self::match_set_to_value(set, &Value::ArrayUnknown)
            }
        }
    }

    #[must_use]
    pub fn match_set_to_value(set: &HashSet<Self>, right_wiki: &Value) -> bool {
        let right = Self::from_wiki_value(right_wiki);
        set.iter().any(|gv| Self::match_values(gv, &right))
    }

    #[must_use]
    /// matches values are compatible (Anything will always match)
    /// todo: think about how nil and any interact?
    pub fn match_values(left: &Self, right: &Self) -> bool {
        if matches!(left, Self::Anything) {
            return true;
        }
        if matches!(right, Self::Anything) {
            return true;
        }
        std::mem::discriminant(left) == std::mem::discriminant(right)
    }

    #[must_use]
    /// Maps from Wiki:Value to Inspector:GameValue
    pub fn from_wiki_value(value: &Value) -> Self {
        match value {
            Value::Anything => Self::Anything,
            Value::ArrayColor
            | Value::ArrayColorRgb
            | Value::ArrayColorRgba
            | Value::ArrayDate
            | Value::ArraySized { .. }
            | Value::ArrayUnknown
            | Value::ArrayUnsized { .. }
            | Value::Position
            | Value::Waypoint => Self::Array(None),
            Value::Boolean => Self::Boolean(None),
            Value::Code => Self::Code(None),
            Value::Config => Self::Config,
            Value::Control => Self::Control,
            Value::DiaryRecord => Self::DiaryRecord,
            Value::Display => Self::Display,
            Value::ForType => Self::ForType(None),
            Value::IfType => Self::IfType,
            Value::Group => Self::Group,
            Value::Location => Self::Location,
            Value::Namespace => Self::Namespace,
            Value::Nothing => Self::Nothing,
            Value::Number => Self::Number(None),
            Value::Object => Self::Object,
            Value::ScriptHandle => Self::ScriptHandle,
            Value::Side => Self::Side,
            Value::String => Self::String(None),
            Value::SwitchType => Self::SwitchType,
            Value::Task => Self::Task,
            Value::TeamMember => Self::TeamMember,
            Value::WhileType => Self::WhileType,
            Value::WithType => Self::WithType,
            Value::Unknown => {
                trace!("wiki has syntax with [unknown] type");
                Self::Anything
            }
            _ => {
                warn!("wiki type [{value:?}] not matched");
                Self::Anything
            }
        }
    }

    #[must_use]
    /// Get as a string for debugging
    pub fn as_debug(&self) -> String {
        match self {
            // Self::Assignment() => {
            //     format!("Assignment")
            // }
            Self::Anything => "Anything".to_string(),
            Self::ForType(expression) => {
                if let Some(Expression::String(str, _, _)) = expression {
                    format!("ForType(var {str})")
                } else {
                    "ForType(GENERIC)".to_string()
                }
            }
            Self::Number(expression) => {
                if let Some(Expression::Number(num, _)) = expression {
                    format!("Number({num:?})",)
                } else {
                    "Number(GENERIC)".to_string()
                }
            }
            Self::String(expression) => {
                if let Some(Expression::String(str, _, _)) = expression {
                    format!("String({str})")
                } else {
                    "String(GENERIC)".to_string()
                }
            }
            Self::Boolean(expression) => {
                if let Some(Expression::Boolean(bool, _)) = expression {
                    format!("Boolean({bool})")
                } else {
                    "Boolean(GENERIC)".to_string()
                }
            }
            Self::Array(expression) => {
                if let Some(Expression::Array(array, _)) = expression {
                    format!("ArrayExp(len {})", array.len())
                } else {
                    "ArrayExp(GENERIC)".to_string()
                }
            }
            Self::Code(expression) => {
                if let Some(Expression::Code(statements)) = expression {
                    format!("Code(len {})", statements.content().len())
                } else {
                    "Code(GENERIC)".to_string()
                }
            }
            _ => "Other(todo)".to_string(),
        }
    }
}
