//! Game Values and mapping them from commands

use std::collections::HashSet;

use arma3_wiki::model::{Arg, Call, Param, Value};
use tracing::{trace, warn};

use crate::{parser::database::Database, Expression};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GameValue {
    Anything,
    // Assignment, // as in z = call {x=1}???
    Array(Option<Vec<Vec<GameValue>>>),
    Boolean(Option<Expression>),
    Code(Option<Expression>),
    Config,
    Control,
    DiaryRecord,
    Display,
    ForType(Option<Vec<Expression>>),
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
    /// Gets cmd return types based on input types
    pub fn from_cmd(
        expression: &Expression,
        lhs_set: Option<&HashSet<Self>>,
        rhs_set: Option<&HashSet<Self>>,
        database: &Database,
    ) -> HashSet<Self> {
        let mut return_types = HashSet::new();
        let cmd_name = expression.command_name().expect("has a name");
        let Some(command) = database.wiki().commands().get(cmd_name) else {
            println!("cmd {cmd_name} not in db?");
            return HashSet::from([Self::Anything]);
        };

        for syntax in command.syntax() {
            // println!("syntax {:?}", syntax.call());
            match syntax.call() {
                Call::Nular => {
                    if !matches!(expression, Expression::NularCommand(..)) {
                        continue;
                    }
                }
                Call::Unary(rhs_arg) => {
                    if !matches!(expression, Expression::UnaryCommand(..))
                        || !Self::match_set_to_arg(
                            cmd_name,
                            rhs_set.expect("unary rhs"),
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
                            cmd_name,
                            lhs_set.expect("binary lhs"),
                            lhs_arg,
                            syntax.params(),
                        )
                        || !Self::match_set_to_arg(
                            cmd_name,
                            rhs_set.expect("binary rhs"),
                            rhs_arg,
                            syntax.params(),
                        )
                    {
                        continue;
                    }
                }
            }
            let value = &syntax.ret().0;
            // println!("match syntax {syntax:?}");
            return_types.insert(Self::from_wiki_value(value));
        }
        // println!("lhs_set {lhs_set:?}");
        // println!("rhs_set {rhs_set:?}");
        // println!(
        //     "cmd [{}] = {}:{:?}",
        //     cmd_name,
        //     return_types.len(),
        //     return_types
        // );
        return_types
    }

    #[must_use]
    pub fn match_set_to_arg(
        cmd_name: &str,
        set: &HashSet<Self>,
        arg: &Arg,
        params: &[Param],
    ) -> bool {
        match arg {
            Arg::Item(name) => {
                let Some(param) = params.iter().find(|p| p.name() == name) else {
                    /// Varadic cmds which will be missing wiki param matches
                    const WIKI_CMDS_IGNORE_MISSING_PARAM: &[&str] = &[
                        "format",
                        "formatText",
                        "param",
                        "params",
                        "setGroupId",
                        "setGroupIdGlobal",
                        "set3DENMissionAttributes",
                        "setPiPEffect",
                        "ppEffectCreate",
                        "inAreaArray",
                    ];
                    if !WIKI_CMDS_IGNORE_MISSING_PARAM.contains(&cmd_name) {
                        // warn!("cmd {cmd_name} - param {name} not found");
                    }
                    return true;
                };
                // println!(
                //     "[arg {name}] typ: {:?}, opt: {:?}",
                //     param.typ(),
                //     param.optional()
                // );
                Self::match_set_to_value(set, param.typ(), param.optional())
            }
            Arg::Array(arg_array) => {
                const WIKI_CMDS_IGNORE_ARGS: &[&str] = &["createHashMapFromArray"];
                if WIKI_CMDS_IGNORE_ARGS.contains(&cmd_name) {
                    return true;
                }

                set.iter().any(|s| {
                    match s {
                        Self::Anything | Self::Array(None) => {
                            // println!("array (any/generic) pass");
                            true
                        }
                        Self::Array(Some(gv_array)) => {
                            // println!("array (gv: {}) expected (arg: {})", gv_array.len(), arg_array.len());
                            // note: some syntaxes take more than others
                            for (index, arg) in arg_array.iter().enumerate() {
                                let possible = if index < gv_array.len() {
                                    gv_array[index].iter().cloned().collect()
                                } else {
                                    HashSet::new()
                                };
                                if !Self::match_set_to_arg(cmd_name, &possible, arg, params) {
                                    return false;
                                }
                            }
                            true
                        }
                        _ => false,
                    }
                })
            }
        }
    }

    #[must_use]
    pub fn match_set_to_value(set: &HashSet<Self>, right_wiki: &Value, optional: bool) -> bool {
        // println!("Checking {:?} against {:?} [O:{optional}]", set, right_wiki);
        if optional && (set.is_empty() || set.contains(&Self::Nothing)) {
            return true;
        }
        let right = Self::from_wiki_value(right_wiki);
        set.iter().any(|gv| Self::match_values(gv, &right))
    }

    #[must_use]
    /// matches values are compatible (Anything will always match)
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
            Value::Anything | Value::EdenEntity => Self::Anything,
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
    /// Gets a generic version of a type
    pub fn make_generic(&self) -> Self {
        match self {
            Self::Array(_) => Self::Array(None),
            Self::Boolean(_) => Self::Boolean(None),
            Self::Code(_) => Self::Code(None),
            Self::ForType(_) => Self::ForType(None),
            Self::Number(_) => Self::Number(None),
            Self::String(_) => Self::String(None),
            _ => self.clone(),
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
            Self::ForType(for_args_array) => {
                if for_args_array.is_some() {
                    format!("ForType(var {for_args_array:?})")
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
            Self::Array(gv_array_option) =>
            {
                #[allow(clippy::option_if_let_else)]
                if let Some(gv_array) = gv_array_option {
                    format!("ArrayExp(len {})", gv_array.len())
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
