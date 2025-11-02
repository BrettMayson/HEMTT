//! Game Values and mapping them from commands

use std::ops::Range;

use arma3_wiki::model::{Arg, Call, Param, Value};
use indexmap::IndexSet;
use tracing::{trace, warn};

use crate::{Expression, parser::database::Database};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GameValue {
    Anything,
    // Assignment, // as in z = call {x=1}???
    #[allow(clippy::type_complexity)]
    Array(
        Option<Vec<Vec<(GameValue, Range<usize>)>>>,
        Option<ArrayType>,
    ),
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
    ///
    /// The return is (return type, expected lhs, expected rhs)
    pub fn from_cmd(
        expression: &Expression,
        lhs_set: Option<&IndexSet<Self>>,
        rhs_set: Option<&IndexSet<Self>>,
        database: &Database,
    ) -> (IndexSet<Self>, IndexSet<Self>, IndexSet<Self>) {
        let mut return_types = IndexSet::new();
        let cmd_name = expression.command_name().expect("has a name");
        let Some(command) = database.wiki().commands().get(cmd_name) else {
            println!("cmd {cmd_name} not in db?");
            return (
                IndexSet::from([Self::Anything]),
                IndexSet::new(),
                IndexSet::new(),
            );
        };

        let mut expected_lhs = IndexSet::new();
        let mut expected_rhs = IndexSet::new();

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
                        || !{
                            let (is_match, expected) = Self::match_set_to_arg(
                                cmd_name,
                                rhs_set.expect("unary rhs"),
                                rhs_arg,
                                syntax.params(),
                            );
                            expected_rhs.extend(expected);
                            is_match
                        }
                    {
                        continue;
                    }
                }
                Call::Binary(lhs_arg, rhs_arg) => {
                    if !matches!(expression, Expression::BinaryCommand(..))
                        || !{
                            let (is_match, expected) = Self::match_set_to_arg(
                                cmd_name,
                                lhs_set.expect("binary lhs"),
                                lhs_arg,
                                syntax.params(),
                            );
                            expected_lhs.extend(expected);
                            is_match
                        }
                        || !{
                            let (is_match, expected) = Self::match_set_to_arg(
                                cmd_name,
                                rhs_set.expect("binary rhs"),
                                rhs_arg,
                                syntax.params(),
                            );
                            expected_rhs.extend(expected);
                            is_match
                        }
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
        (return_types, expected_lhs, expected_rhs)
    }

    #[must_use]
    pub fn match_set_to_arg(
        cmd_name: &str,
        set: &IndexSet<Self>,
        arg: &Arg,
        params: &[Param],
    ) -> (bool, IndexSet<Self>) {
        match arg {
            Arg::Item(name) => {
                let Some(param) = params.iter().find(|p| p.name() == name) else {
                    /// Varidic cmds which will be missing wiki param matches (only affects debug logging)
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
                    return (true, IndexSet::new());
                };
                // println!(
                //     "[arg {name}] typ: {:?}, opt: {:?}",
                //     param.typ(),
                //     param.optional()
                // );
                let (is_match, expected_gv) =
                    Self::match_set_to_value(set, param.typ(), param.optional());
                let mut expected = IndexSet::new();
                expected.insert(expected_gv);
                (is_match, expected)
            }
            Arg::Array(arg_array) => {
                let mut expected = IndexSet::new();
                let is_match = set.iter().any(|s| {
                    match s {
                        Self::Anything | Self::Array(None, _) => {
                            // println!("{cmd_name}: array (any/generic) pass");
                            true
                        }
                        Self::Array(Some(gv_array), _) => {
                            // println!("{cmd_name}: array (gv: {}) expected (arg: {})", gv_array.len(), arg_array.len());
                            // note: some syntaxes take more than others
                            for (index, arg) in arg_array.iter().enumerate() {
                                let possible = if index < gv_array.len() {
                                    gv_array[index].iter().map(|(gv, _)| gv.clone()).collect()
                                } else {
                                    IndexSet::new()
                                };
                                let (is_match, expected_gv) =
                                    Self::match_set_to_arg(cmd_name, &possible, arg, params);
                                expected.extend(expected_gv);
                                if !is_match {
                                    if let Arg::Array(args) = arg {
                                        // handle edge case on varidic cmds that take arrays (e.g. `createHashMapFromArray`)
                                        if args.iter().any(|a| match a {
                                            Arg::Array(_) => false,
                                            Arg::Item(name) => {
                                                !params.iter().any(|p| p.name() == name) // test if has missing args inside the array
                                            }
                                        }) {
                                            // println!("using special exception for varidic array from {cmd_name}"); // only ~4 cmds need this
                                            continue;
                                        }
                                    }
                                    // println!("array arg {index} no match {arg:?} in {s:?}");
                                    return false;
                                }
                            }
                            true
                        }
                        _ => false,
                    }
                });
                (is_match, expected)
            }
        }
    }

    #[must_use]
    /// checks if a set of `GameValues` matches a wiki Value (with optional flag)
    /// returns the expected `GameValue` type
    pub fn match_set_to_value(
        set: &IndexSet<Self>,
        right_wiki: &Value,
        optional: bool,
    ) -> (bool, Self) {
        // println!("Checking {set:?} against {right_wiki:?} [O:{optional}]");
        let right = Self::from_wiki_value(right_wiki);
        if optional && (set.is_empty() || set.contains(&Self::Nothing)) {
            return (true, right);
        }
        (set.iter().any(|gv| Self::match_values(gv, &right)), right)
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
    /// Returns the common generic type of all array elements.
    ///
    /// If the set is empty or contains any non-array types, returns `Anything`.
    /// If array element types differ, returns `Anything`.
    pub fn get_array_value_type(set: &IndexSet<Self>) -> Self {
        let mut result: Option<Self> = None;
        for gv in set {
            match gv {
                Self::Array(Some(array_outer), _) => {
                    for outer in array_outer {
                        for (inner, _) in outer {
                            if result.is_none() {
                                result = Some(inner.make_generic());
                            } else if let Some(existing) = &result
                                && !Self::match_values(existing, inner)
                            {
                                return Self::Anything;
                            }
                        }
                    }
                }
                _ => return Self::Anything,
            }
        }
        result.unwrap_or(Self::Anything)
    }
}

impl std::fmt::Display for GameValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::Array(_, Some(array_type)) = self {
            return write!(f, "Array<{array_type:?}>");
        }
        write!(
            f,
            "{}",
            match self {
                Self::Anything => "Anything",
                Self::Array(_, _) => "Array",
                Self::Boolean(_) => "Boolean",
                Self::Code(_) => "Code",
                Self::Config => "Config",
                Self::Control => "Control",
                Self::DiaryRecord => "DiaryRecord",
                Self::Display => "Display",
                Self::ForType(_) => "ForType",
                Self::Group => "Group",
                Self::HashMap => "HashMap",
                Self::IfType => "IfType",
                Self::Location => "Location",
                Self::Namespace => "Namespace",
                Self::Number(_) => "Number",
                Self::Nothing => "Nothing",
                Self::Object => "Object",
                Self::ScriptHandle => "ScriptHandle",
                Self::Side => "Side",
                Self::String(_) => "String",
                Self::StructuredText => "StructuredText",
                Self::SwitchType => "SwitchType",
                Self::Task => "Task",
                Self::TeamMember => "TeamMember",
                Self::WhileType => "WhileType",
                Self::WithType => "WithType",
            }
        )
    }
}
