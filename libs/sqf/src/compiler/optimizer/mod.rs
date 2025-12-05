//! Optimizes sqf by evaulating expressions when possible and looking for arrays that can be consumed
//! `ToDo`: what commands consume arrays
//!
use crate::{BinaryCommand, Expression, Statement, Statements, UnaryCommand};
use std::ops::Range;
#[allow(unused_imports)]
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
                        "vectoradd" | "vectordiff" | "vectorcrossproduct" | "vectordotproduct" => {
                            if let Some(consumable) = right_o.get_consumable_array(true, op_name) {
                                right_o = consumable;
                            }
                            if let Some(consumable) = left_o.get_consumable_array(true, op_name) {
                                left_o = consumable;
                            }
                        }
                        "call" => {
                            if matches!(&left_o, Self::Variable(name, _) if name == "_this") {
                                return Self::UnaryCommand(
                                    UnaryCommand::Named("call".into()),
                                    Box::new(right_o),
                                    range.clone(),
                                );
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
                            #[cfg(debug_assertions)]
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
            Self::NularCommand(command, ..) => command.is_constant(),
            Self::Array(array, ..) => array.iter().all(Self::is_constant), // true on empty
            Self::ConsumeableArray(..) => {
                unreachable!("should not be reachable");
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
        if let Self::Array(array, _) = self
            && let Some(param_default) = array.get(1)
            && param_default.is_array()
        {
            return false;
        }
        true
    }

    /// Trys to get a consumable array from an existing array if it can be made a constant
    #[must_use]
    #[allow(unused_variables)]
    fn get_consumable_array(&self, direct: bool, op: &str) -> Option<Self> {
        if let Self::Array(array, range) = &self {
            if !self.is_constant() {
                #[cfg(debug_assertions)]
                trace!("not constant {op}");
                return None;
            }
            if array.is_empty() {
                #[cfg(debug_assertions)]
                trace!("pointless to optimize {op}");
                return None;
            }
            if direct {
                #[cfg(debug_assertions)]
                trace!("optimizing [{op}]'s arg => ConsumeableArray");
                Some(Self::ConsumeableArray(array.clone(), range.clone()))
            } else {
                #[cfg(debug_assertions)]
                trace!("optimizing [{op}]'s arg => +ConsumeableArray (copy)");
                // make a copy of the array so the original cannot be modified
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

    /// Boilerplate for unary string operations
    #[must_use]
    fn op_uni_string(
        &self,
        op_type: &UnaryCommand,
        range: &Range<usize>,
        right: &Self,
        op: fn(&str) -> String,
    ) -> Option<Self> {
        if let Self::String(right_string, _, right_wrapper) = right {
            if right_string.is_ascii() {
                let new_string = op(right_string.as_ref());
                #[cfg(debug_assertions)]
                trace!(
                    "optimizing [U:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(false),
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
                self.source(false),
                right_string.to_string()
            );
        }
        None
    }

    /// Boilerplate for unary float operations
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
                #[cfg(debug_assertions)]
                trace!(
                    "optimizing [U:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(false),
                    new_number
                );
                return Some(Self::Number(crate::Scalar(new_number), range.clone()));
            }
            warn!(
                "Skipping Optimization because NaN [U:{}] ({}) => {}",
                op_type.as_str(),
                self.source(false),
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
        if let Self::String(left_string, _, _left_wrapper) = left
            && let Self::String(right_string, _, right_wrapper) = right
        {
            if right_string.is_ascii() && left_string.is_ascii() {
                let new_string = op(left_string.as_ref(), right_string.as_ref());
                #[cfg(debug_assertions)]
                trace!(
                    "optimizing [B:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(false),
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
                self.source(false),
                right_string.to_string()
            );
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
        let Self::Number(crate::Scalar(right_number), _) = right else {
            return None;
        };
        match left {
            Self::Number(crate::Scalar(left_number), _) => {
                let new_number = op(*left_number, *right_number);
                if new_number.is_finite() {
                    #[cfg(debug_assertions)]
                    trace!(
                        "optimizing [B:{}] ({}) => {}",
                        op_type.as_str(),
                        self.source(false),
                        new_number
                    );
                    return Some(Self::Number(crate::Scalar(new_number), range.clone()));
                }
                warn!(
                    "Skipping Optimization because NaN [B:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(false),
                    new_number
                );
            }
            Self::BinaryCommand(left_op_type, left_op_lhs, left_op_rhs, _) => {
                // reverse chain: (X / 2) * 5  ->  X / (2 / 5)  ->  X / 0.4
                let Self::Number(crate::Scalar(_), _) = **left_op_rhs else {
                    return None;
                };
                let new_op = Self::op_bin_float_chainable_op(left_op_type, op_type)?;
                let result = Self::BinaryCommand(
                    new_op.clone(),
                    left_op_rhs.clone(),
                    Box::new(right.clone()),
                    range.clone(),
                )
                .optimize();
                if let Self::Number(crate::Scalar(ref new_number), _) = result
                    && new_number.is_finite()
                {
                    #[cfg(debug_assertions)]
                    trace!(
                        "optimizing pair ([B:{}], [B:{}]) ({}) => {}",
                        op_type.as_str(),
                        left_op_type.as_str(),
                        self.source(false),
                        new_number
                    );
                    return Some(Self::BinaryCommand(
                        left_op_type.clone(),
                        left_op_lhs.clone(),
                        Box::new(result),
                        range.clone(),
                    ));
                }
                warn!(
                    "Skipping Optimization on float chain [B:{}] ({})",
                    new_op.as_str(),
                    self.source(false)
                );
            }
            _ => {}
        }
        None
    }

    #[must_use]
    /// gets the re-ordered math operation
    const fn op_bin_float_chainable_op(
        left_op: &BinaryCommand,
        right_op: &BinaryCommand,
    ) -> Option<BinaryCommand> {
        match left_op {
            BinaryCommand::Mul => match right_op {
                BinaryCommand::Mul => Some(BinaryCommand::Mul),
                BinaryCommand::Div => Some(BinaryCommand::Div),
                _ => None,
            },
            BinaryCommand::Div => match right_op {
                BinaryCommand::Mul => Some(BinaryCommand::Div),
                BinaryCommand::Div => Some(BinaryCommand::Mul),
                _ => None,
            },
            BinaryCommand::Add => match right_op {
                BinaryCommand::Add => Some(BinaryCommand::Add),
                BinaryCommand::Sub => Some(BinaryCommand::Sub),
                _ => None,
            },
            BinaryCommand::Sub => match right_op {
                BinaryCommand::Add => Some(BinaryCommand::Sub),
                BinaryCommand::Sub => Some(BinaryCommand::Add),
                _ => None,
            },
            _ => None,
        }
    }
}
