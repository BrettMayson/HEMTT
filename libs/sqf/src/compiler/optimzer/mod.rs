/// Optimizes sqf by evaulating expressions when possible and looking for arrays that can be consumed
///
/// `ToDo`: Any command that "consumes" an array could be upgraded
/// e.g. x = y vectorAdd [0,0,1];
///
use crate::{BinaryCommand, Expression, Statement, Statements, UnaryCommand};
use std::ops::Range;
use tracing::{trace, warn};

impl Statements {
    #[must_use]
    pub fn optimize(mut self) -> Self {
        self.content = self.content.into_iter().map(Statement::optimise).collect();
        self
    }
}

impl Statement {
    #[must_use]
    pub fn optimise(self) -> Self {
        match self {
            Self::AssignGlobal(left, expression, right) => {
                Self::AssignGlobal(left, expression.optimize(), right)
            }
            Self::AssignLocal(left, expression, right) => {
                Self::AssignLocal(left, expression.optimize(), right)
            }
            Self::Expression(expression, right) => Self::Expression(expression.optimize(), right),
        }
    }
}

impl Expression {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    fn optimize(self) -> Self {
        match &self {
            Self::Code(code) => Self::Code(code.clone().optimize()),
            Self::Array(array_old, range) => {
                let array_new = array_old.iter().map(|e| e.clone().optimize()).collect();
                Self::Array(array_new, range.clone())
            }
            Self::UnaryCommand(op_type, right, range) => {
                let right_o = right.clone().optimize();
                match op_type {
                    UnaryCommand::Minus => {
                        fn op(r: f32) -> f32 {
                            -r
                        }
                        if let Some(eval) = self.op_uni_float(op_type, range, &right_o, op) {
                            return eval;
                        }
                    }
                    UnaryCommand::Named(op_name) => match op_name.to_lowercase().as_str() {
                        "tolower" | "toloweransi" => {
                            fn op(r: &str) -> String {
                                r.to_ascii_lowercase()
                            }
                            if let Some(eval) = self.op_uni_string(op_type, range, &right_o, op) {
                                return eval;
                            }
                        }
                        "toupper" | "toupperansi" => {
                            fn op(r: &str) -> String {
                                r.to_ascii_uppercase()
                            }
                            if let Some(eval) = self.op_uni_string(op_type, range, &right_o, op) {
                                return eval;
                            }
                        }
                        "sqrt" => {
                            fn op(r: f32) -> f32 {
                                r.sqrt()
                            }
                            if let Some(eval) = self.op_uni_float(op_type, range, &right_o, op) {
                                return eval;
                            }
                        }
                        "params" => {
                            if let Self::Array(a_array, a_range) = &right_o {
                                if a_array.iter().all(Self::is_safe_param) {
                                    trace!(
                                        "optimizing [U:{}] ({}) => ConsumeableArray",
                                        op_type.as_str(),
                                        self.source()
                                    );
                                    return Self::UnaryCommand(
                                        UnaryCommand::Named(op_name.clone()),
                                        Box::new(Self::ConsumeableArray(
                                            a_array.clone(),
                                            a_range.clone(),
                                        )),
                                        range.clone(),
                                    );
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
                Self::UnaryCommand(op_type.clone(), Box::new(right_o), range.clone())
            }
            Self::BinaryCommand(op_type, left, right, range) => {
                let left_o = left.clone().optimize();
                let right_o = right.clone().optimize();
                match op_type {
                    #[allow(clippy::single_match)]
                    BinaryCommand::Named(op_name) => match op_name.to_lowercase().as_str() {
                        "params" => {
                            if let Self::Array(a_array, a_range) = &right_o {
                                if a_array.iter().all(Self::is_safe_param) {
                                    trace!(
                                        "optimizing [B:{}] ({}) => ConsumeableArray",
                                        op_type.as_str(),
                                        self.source()
                                    );
                                    return Self::BinaryCommand(
                                        BinaryCommand::Named(op_name.clone()),
                                        Box::new(left_o),
                                        Box::new(Self::ConsumeableArray(
                                            a_array.clone(),
                                            a_range.clone(),
                                        )),
                                        range.clone(),
                                    );
                                }
                            }
                        }
                        _ => {}
                    },
                    BinaryCommand::Add => {
                        {
                            fn op(l: f32, r: f32) -> f32 {
                                l + r
                            }
                            if let Some(eval) =
                                self.op_bin_float(op_type, range, &left_o, &right_o, op)
                            {
                                return eval;
                            }
                        }
                        {
                            fn op(l: &str, r: &str) -> String {
                                format!("{l}{r}")
                            }
                            if let Some(eval) =
                                self.op_bin_string(op_type, range, &left_o, &right_o, op)
                            {
                                return eval;
                            }
                        }
                    }
                    BinaryCommand::Sub => {
                        fn op(l: f32, r: f32) -> f32 {
                            l - r
                        }
                        if let Some(eval) = self.op_bin_float(op_type, range, &left_o, &right_o, op)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Mul => {
                        fn op(l: f32, r: f32) -> f32 {
                            l * r
                        }
                        if let Some(eval) = self.op_bin_float(op_type, range, &left_o, &right_o, op)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Div => {
                        fn op(l: f32, r: f32) -> f32 {
                            l / r
                        }
                        if let Some(eval) = self.op_bin_float(op_type, range, &left_o, &right_o, op)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Rem | BinaryCommand::Mod => {
                        fn op(l: f32, r: f32) -> f32 {
                            l % r
                        }
                        if let Some(eval) = self.op_bin_float(op_type, range, &left_o, &right_o, op)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Else => {
                        if let Self::Code(_) = left_o {
                            if let Self::Code(_) = right_o {
                                return Self::ConsumeableArray(
                                    vec![left_o, right_o],
                                    range.clone(),
                                );
                            }
                        }
                    }
                    _ => {}
                }
                Self::BinaryCommand(
                    op_type.clone(),
                    Box::new(left_o),
                    Box::new(right_o),
                    range.clone(),
                )
            }
            _ => self,
        }
    }

    /*
    Don't present a consumable array that could be modified: Check if param will return an array as a default value
    sqfc = {
        params [["_a", []]];
        x = _a;
    };
    call sqfc;
    x pushBack 5;
    call sqfc
    x is now [5] - the const has been modified
    */
    #[must_use]
    fn is_safe_param(&self) -> bool {
        #[allow(clippy::single_match)]
        match self {
            Self::Array(array, _) => {
                if let Some(param_default) = array.get(1) {
                    if param_default.is_array() {
                        return false;
                    }
                }
            }
            _ => {}
        }
        true // every other check (for constness) will be handled by the serializer
    }

    // Boilerplate for uniary and binary ops
    #[must_use]
    fn op_uni_string(
        &self,
        op_type: &UnaryCommand,
        range: &Range<usize>,
        right: &Self,
        op: fn(&str) -> String,
    ) -> Option<Self> {
        if let Self::String(right_string, _, ref right_wrapper) = right {
            if right_string.is_ascii() {
                let new_string = op(right_string.as_ref());
                trace!(
                    "optimizing [U:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(),
                    new_string
                );
                return Some(Self::String(
                    new_string.into(),
                    range.clone(),
                    right_wrapper.clone(),
                ));
            }
            warn!(
                "Skipping Optimization because unicode [U:{}] ({}) => {}",
                op_type.as_str(),
                self.source(),
                right_string.to_string()
            );
        }
        None
    }
    #[must_use]
    fn op_uni_float(
        &self,
        op_type: &UnaryCommand,
        range: &Range<usize>,
        right: &Self,
        op: fn(f32) -> f32,
    ) -> Option<Self> {
        if let Self::Number(crate::Scalar(right_number), _) = right {
            let new_number = op(*right_number);
            if new_number.is_finite() {
                trace!(
                    "optimizing [U:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(),
                    new_number
                );
                return Some(Self::Number(crate::Scalar(new_number), range.clone()));
            }
            warn!(
                "Skipping Optimization because NaN [U:{}] ({}) => {}",
                op_type.as_str(),
                self.source(),
                new_number
            );
        }
        None
    }
    #[must_use]
    fn op_bin_string(
        &self,
        op_type: &BinaryCommand,
        range: &Range<usize>,
        left: &Self,
        right: &Self,
        op: fn(&str, &str) -> String,
    ) -> Option<Self> {
        if let Self::String(left_string, _, ref _left_wrapper) = left {
            if let Self::String(right_string, _, ref right_wrapper) = right {
                if right_string.is_ascii() && left_string.is_ascii() {
                    let new_string = op(left_string.as_ref(), left_string.as_ref());
                    trace!(
                        "optimizing [B:{}] ({}) => {}",
                        op_type.as_str(),
                        self.source(),
                        new_string
                    );
                    return Some(Self::String(
                        new_string.into(),
                        range.clone(),
                        right_wrapper.clone(),
                    ));
                }
                warn!(
                    "Skipping Optimization because unicode [B:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(),
                    right_string.to_string()
                );
            }
        }
        None
    }
    #[must_use]
    fn op_bin_float(
        &self,
        op_type: &BinaryCommand,
        range: &Range<usize>,
        left: &Self,
        right: &Self,
        op: fn(f32, f32) -> f32,
    ) -> Option<Self> {
        if let Self::Number(crate::Scalar(left_number), _) = left {
            if let Self::Number(crate::Scalar(right_number), _) = right {
                let new_number = op(*left_number, *right_number);
                if new_number.is_finite() {
                    trace!(
                        "optimizing [B:{}] ({}) => {}",
                        op_type.as_str(),
                        self.source(),
                        new_number
                    );
                    return Some(Self::Number(crate::Scalar(new_number), range.clone()));
                }
                warn!(
                    "Skipping Optimization because NaN [B:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(),
                    new_number
                );
            }
        }
        None
    }
}
