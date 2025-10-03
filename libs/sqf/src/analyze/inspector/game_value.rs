//! Game Values and mapping them from commands

use arma3_wiki::model::{Arg, Call, Param, Value};
use indexmap::IndexSet;
use tracing::{trace, warn};

use crate::{Expression, parser::database::Database};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GameValue {
    Anything,
    // Assignment, // as in z = call {x=1}???
    Array(Option<Vec<Vec<GameValue>>>, Option<ArrayType>),
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
    StructuredText,
    SwitchType,
    Task,
    TeamMember,
    WhileType,
    WithType,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ArrayType {
    Color,
    // Pos2d,
    PosAGL,
    // PosAGLS,
    PosASL,
    PosASLW,
    PosATL,
    /// from object's center `getPosWorld`
    PosWorld,
    PosRelative,
}

impl GameValue {
    #[must_use]
    /// Gets cmd return types based on input types
    pub fn from_cmd(
        expression: &Expression,
        lhs_set: Option<&IndexSet<Self>>,
        rhs_set: Option<&IndexSet<Self>>,
        database: &Database,
    ) -> IndexSet<Self> {
        let mut return_types = IndexSet::new();
        let cmd_name = expression.command_name().expect("has a name");
        let Some(command) = database.wiki().commands().get(cmd_name) else {
            println!("cmd {cmd_name} not in db?");
            return IndexSet::from([Self::Anything]);
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
            let game_value = Self::from_wiki_value(value);
            // println!("match syntax {syntax:?}");
            return_types.insert(game_value);
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
        set: &IndexSet<Self>,
        arg: &Arg,
        params: &[Param],
    ) -> bool {
        match arg {
            Arg::Item(name) => {
                let Some(param) = params.iter().find(|p| p.name() == name) else {
                    /// Varadic cmds which will be missing wiki param matches (only effects debug logging)
                    const WIKI_CMDS_IGNORE_MISSING_PARAM: &[&str] = &[
                        "addResources",
                        "createTask",
                        "ctRemoveRows",
                        "format",
                        "formatText",
                        "getGraphValues",
                        "inAreaArray",
                        "inAreaArrayIndexes",
                        "insert",
                        "kbReact",
                        "kbTell",
                        "lineIntersects",
                        "lineIntersectsObjs",
                        "lineIntersectsSurfaces",
                        "lineIntersectsWith",
                        "param",
                        "params",
                        "ppEffectCreate",
                        "set3DENAttributes",
                        "set3DENMissionAttributes",
                        "setAttributes",
                        "setGroupId",
                        "setGroupIdGlobal",
                        "setPiPEffect",
                        "textLogFormat",
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
                set.iter().any(|s| {
                    match s {
                        Self::Anything | Self::Array(None, _) => {
                            // println!("array (any/generic) pass");
                            true
                        }
                        Self::Array(Some(gv_array), _) => {
                            /// Varadic cmds which need special handling for arg arrays
                            const WIKI_CMDS_IGNORE_ARGS: &[&str] = &[
                                "createHashMapFromArray",
                                "insert",
                                "set3DENAttributes",
                                "set3DENMissionAttributes",
                            ];
                            if WIKI_CMDS_IGNORE_ARGS.contains(&cmd_name) {
                                return true;
                            }
                            // note: some syntaxes take more than others
                            for (index, arg) in arg_array.iter().enumerate() {
                                let possible = if index < gv_array.len() {
                                    gv_array[index].iter().cloned().collect()
                                } else {
                                    IndexSet::new()
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
    pub fn match_set_to_value(set: &IndexSet<Self>, right_wiki: &Value, optional: bool) -> bool {
        // println!("Checking {set:?} against {right_wiki:?} [O:{optional}]");
        if optional && (set.is_empty() || set.contains(&Self::Nothing)) {
            return true;
        }
        let right = Self::from_wiki_value(right_wiki);
        set.iter().any(|gv| Self::match_values(gv, &right))
    }

    #[must_use]
    /// matches values are compatible (Anything will always match)
    pub fn match_values(left: &Self, right: &Self) -> bool {
        if matches!(left, Self::Anything) || matches!(right, Self::Anything) {
            return true;
        }
        if let (Self::Array(_, Some(lpos)), Self::Array(_, Some(rpos))) = (left, right)
            && lpos != rpos
        {
            // ToDo: Handle matching array types better eg: AGLS vs AGL
            // false-positive:
            /*
               private _dropPos = _target modelToWorld [0.4, 0.75, 0]; //offset someone unconscious isn't lying over it
               _dropPos set [2, ((getPosASL _target) select 2)];
               _holder setPosASL _dropPos;
            */
            // println!("array mismatch {lpos:?}!={rpos:?}");
            // return false;
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
            | Value::Position // position is often too generic to match?
            | Value::Waypoint => Self::Array(None, None),
            // Value::Position3dAGLS => Self::Array(None, Some(ArrayType::PosAGLS)),
            Value::Position3dAGLS | Value::Position3dAGL => Self::Array(None, Some(ArrayType::PosAGL)), // merge
            Value::Position3dASL => Self::Array(None, Some(ArrayType::PosASL)),
            Value::Position3DASLW => Self::Array(None, Some(ArrayType::PosASLW)),
            Value::Position3dATL => Self::Array(None, Some(ArrayType::PosATL)),
            Value::Boolean => Self::Boolean(None),
            Value::Code => Self::Code(None),
            Value::Config => Self::Config,
            Value::Control => Self::Control,
            Value::DiaryRecord => Self::DiaryRecord,
            Value::Display => Self::Display,
            Value::ForType => Self::ForType(None),
            Value::Group => Self::Group,
            Value::HashMapUnknown => Self::HashMap,
            Value::IfType => Self::IfType,
            Value::Location => Self::Location,
            Value::Namespace => Self::Namespace,
            Value::Nothing => Self::Nothing,
            Value::Number => Self::Number(None),
            Value::Object => Self::Object,
            Value::ScriptHandle => Self::ScriptHandle,
            Value::Side => Self::Side,
            Value::String => Self::String(None),
            Value::SwitchType => Self::SwitchType,
            Value::StructuredText => Self::StructuredText,
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
            Self::Array(_, _) => Self::Array(None, None),
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
            Self::Array(gv_array_option, position_option) => {
                let str_len = gv_array_option
                    .clone()
                    .map_or_else(|| "GENERIC".to_string(), |l| format!("len {}", l.len()));
                let str_pos = position_option
                    .clone()
                    .map_or_else(String::new, |p| format!(":{p:?}"));
                format!("ArrayExp({str_len}{str_pos})")
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
