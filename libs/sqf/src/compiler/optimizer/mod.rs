//! Optimizes sqf by evaulating expressions when possible and looking for arrays that can be consumed
//! `ToDo`: what commands consume arrays
//! `ToDo`: reduce logging when stable
//!
use crate::{BinaryCommand, Expression, Statement, Statements, UnaryCommand};
use std::ops::Range;
use tracing::{trace, warn};

impl Statements {
    /// optimize Statements
    #[must_use]
    pub fn optimize(mut self) -> Self {
        self.content = self.content.into_iter().map(Statement::optimize).collect();
        self
    }
}

impl Statement {
    /// optimize Statement
    #[must_use]
    pub fn optimize(self) -> Self {
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
    /// optimize Expression
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
                let mut right_o = right.clone().optimize(); // Optimized RHS
                match op_type {
                    UnaryCommand::Minus => {
                        if let Some(eval) =
                            self.op_uni_float(op_type, range, &right_o, std::ops::Neg::neg)
                        {
                            return eval;
                        }
                    }
                    UnaryCommand::Named(op_name) => match op_name.to_lowercase().as_str() {
                        "tolower" | "toloweransi" => {
                            if let Some(eval) = self.op_uni_string(
                                op_type,
                                range,
                                &right_o,
                                str::to_ascii_lowercase,
                            ) {
                                return eval;
                            }
                        }
                        "toupper" | "toupperansi" => {
                            if let Some(eval) = self.op_uni_string(
                                op_type,
                                range,
                                &right_o,
                                str::to_ascii_uppercase,
                            ) {
                                return eval;
                            }
                        }
                        "sqrt" => {
                            if let Some(eval) =
                                self.op_uni_float(op_type, range, &right_o, f32::sqrt)
                            {
                                return eval;
                            }
                        }
                        // could return part of the rhs's default value
                        "params" => {
                            if let Self::Array(r_array, _) = &right_o {
                                let direct = r_array.iter().all(Self::is_not_array_default_value);
                                if let Some(consumable) =
                                    right_o.get_consumable_array(direct, op_name)
                                {
                                    right_o = consumable;
                                }
                            }
                        }
                        // could return part of the rhs's default value
                        "param" => {
                            let direct = right_o.is_not_array_default_value();
                            if let Some(consumable) = right_o.get_consumable_array(direct, op_name)
                            {
                                right_o = consumable;
                            }
                        }
                        // commands that fully consume arrays and leave no crumbs
                        "positioncameratoworld" | "random" => {
                            if let Some(consumable) = right_o.get_consumable_array(true, op_name) {
                                right_o = consumable;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
                Self::UnaryCommand(op_type.clone(), Box::new(right_o), range.clone())
            }
            Self::BinaryCommand(op_type, left, right, range) => {
                let mut left_o = left.clone().optimize(); // Optimized LHS
                let mut right_o = right.clone().optimize(); // Optimized RHS
                match op_type {
                    BinaryCommand::Named(op_name) => match op_name.to_lowercase().as_str() {
                        // could return part of the rhs's default value
                        "params" => {
                            if let Self::Array(r_array, _) = &right_o {
                                let direct = r_array.iter().all(Self::is_not_array_default_value);
                                if let Some(consumable) =
                                    right_o.get_consumable_array(direct, op_name)
                                {
                                    right_o = consumable;
                                }
                            }
                        }
                        // could return part of the rhs's default value (all use 2nd arg as default value)
                        "param" | "getvariable" | "setvariable" | "getordefault" => {
                            let direct = right_o.is_not_array_default_value();
                            if let Some(consumable) = right_o.get_consumable_array(direct, op_name)
                            {
                                right_o = consumable;
                            }
                        }
                        // commands that fully consume arrays and leave no crumbs
                        "vectoradd" | "vectordiff" => {
                            if let Some(consumable) = right_o.get_consumable_array(true, op_name) {
                                right_o = consumable;
                            }
                            if let Some(consumable) = left_o.get_consumable_array(true, op_name) {
                                left_o = consumable;
                            }
                        }
                        _ => {}
                    },
                    BinaryCommand::Add => {
                        if let Some(eval) =
                            self.op_bin_float(op_type, range, &left_o, &right_o, std::ops::Add::add)
                        {
                            return eval;
                        }
                        if let Some(eval) =
                            self.op_bin_string(op_type, range, &left_o, &right_o, |l, r| {
                                format!("{l}{r}")
                            })
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Sub => {
                        if let Some(eval) =
                            self.op_bin_float(op_type, range, &left_o, &right_o, std::ops::Sub::sub)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Mul => {
                        if let Some(eval) =
                            self.op_bin_float(op_type, range, &left_o, &right_o, std::ops::Mul::mul)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Div => {
                        if let Some(eval) =
                            self.op_bin_float(op_type, range, &left_o, &right_o, std::ops::Div::div)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Rem | BinaryCommand::Mod => {
                        if let Some(eval) =
                            self.op_bin_float(op_type, range, &left_o, &right_o, std::ops::Rem::rem)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Else => {
                        if let (Self::Code(_), Self::Code(_)) = (&left_o, &right_o) {
                            trace!("optimizing [B:{}] => ConsumeableArray", op_type.as_str());
                            return Self::ConsumeableArray(vec![left_o, right_o], range.clone());
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

    /// Is the expression fully constant and something that can be pushed
    #[must_use]
    fn is_constant(&self) -> bool {
        match self {
            Self::Code(..) | Self::String(..) | Self::Number(..) | Self::Boolean(..) => true,
            Self::NularCommand(ref command, ..) => command.is_constant(),
            Self::Array(ref array, ..) => array.iter().all(Self::is_constant), // true on empty
            Self::ConsumeableArray(..) => {
                panic!("should not be reachable");
            }
            _ => false,
        }
    }

    /// Checks if the expresion is not an array that has an array type in index 1
    /// This is the default-value for both param(s) and getVariable
    /// Prevents the following error
    /// ```sqf
    /// sqfc = {
    ///    params [["_a", []]];
    ///     x = _a;
    /// };
    /// call sqfc;
    /// x pushBack 5;
    /// call sqfc // x is now [5] - the const has been modified
    /// ```
    #[must_use]
    fn is_not_array_default_value(&self) -> bool {
        if let Self::Array(array, _) = self {
            if let Some(param_default) = array.get(1) {
                if param_default.is_array() {
                    return false;
                }
            }
        }
        true
    }

    /// Trys to get a consumable array from an existing array if it can be made a constant
    #[must_use]
    fn get_consumable_array(&self, direct: bool, op: &String) -> Option<Self> {
        if let Self::Array(array, range) = &self {
            if !self.is_constant() {
                // println!("debug: not const {op}");
                return None;
            }
            if array.is_empty() {
                println!("debug: pointless to optimize {op}");
                return None;
            }
            if direct {
                trace!("optimizing [{op}]'s arg => ConsumeableArray");
                Some(Self::ConsumeableArray(array.clone(), range.clone()))
            } else {
                // make a copy of the array so the original cannot be modified
                trace!("optimizing [{op}]'s arg => +ConsumeableArray (copy)");
                Some(Self::UnaryCommand(
                    UnaryCommand::Plus,
                    Box::new(Self::ConsumeableArray(array.clone(), range.clone())),
                    range.clone(),
                ))
            }
        } else {
            None
        }
    }

    /// Boilerplate for uniary string operations
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

    /// Boilerplate for uniary float operations
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

    /// Boilerplate for binary string operations
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
                    let new_string = op(left_string.as_ref(), right_string.as_ref());
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

    /// Boilerplate for binary float operations
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
