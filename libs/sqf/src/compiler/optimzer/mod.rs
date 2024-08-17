/// Optimizes sqf by evaulating expressions when possible and looking for arrays that can be consumed
///
/// ToDo: Any command that "consumes" an array could be upgraded
/// e.g. x = y vectorAdd [0,0,1];
///
use crate::{BinaryCommand, Expression, Statement, Statements, UnaryCommand};
use std::ops::Range;
use tracing::{trace, warn};

impl Statements {
    pub fn optimize(mut self) -> Statements {
        self.content = self.content.into_iter().map(|s| s.optimise()).collect();
        return self;
    }
}

impl Statement {
    pub fn optimise(self) -> Statement {
        match self {
            Self::AssignGlobal(left, expression, right) => {
                return Self::AssignGlobal(left, expression.optimize(), right);
            }
            Self::AssignLocal(left, expression, right) => {
                return Self::AssignLocal(left, expression.optimize(), right);
            }
            Self::Expression(expression, right) => {
                return Self::Expression(expression.optimize(), right);
            }
        }
    }
}

impl Expression {
    fn optimize(self) -> Expression {
        match &self {
            Expression::Code(code) => {
                return Expression::Code(code.clone().optimize());
            }
            Expression::Array(array_old, range) => {
                return Expression::Array(
                    array_old.iter().map(|e| e.clone().optimize()).collect(),
                    range.clone(),
                );
            }
            Expression::UnaryCommand(op_type, right, range) => {
                let right_o = right.clone().optimize();
                match op_type {
                    UnaryCommand::Minus => {
                        fn op(r: &f32) -> f32 {
                            -r
                        }
                        if let Some(eval) = self.op_uni_float(op_type, range, &right_o, op) {
                            return eval;
                        }
                    }
                    UnaryCommand::Named(op_name) => match op_name.to_lowercase().as_str() {
                        "tolower" | "toloweransi" => {
                            fn op(r: &String) -> String {
                                r.to_ascii_lowercase()
                            }
                            if let Some(eval) = self.op_uni_string(op_type, range, &right_o, op) {
                                return eval;
                            }
                        }
                        "toupper" | "toupperansi" => {
                            fn op(r: &String) -> String {
                                r.to_ascii_uppercase()
                            }
                            if let Some(eval) = self.op_uni_string(op_type, range, &right_o, op) {
                                return eval;
                            }
                        }
                        "sqrt" => {
                            fn op(r: &f32) -> f32 {
                                r.sqrt()
                            }
                            if let Some(eval) = self.op_uni_float(op_type, range, &right_o, op) {
                                return eval;
                            }
                        }
                        "params" => {
                            if let Expression::Array(a_array, a_range) = &right_o {
                                if a_array.iter().all(|e| e.is_safe_param()) {
                                    trace!(
                                        "optimizing [U:{}] ({}) => ConsumeableArray",
                                        op_type.as_str(),
                                        self.source()
                                    );
                                    return Expression::UnaryCommand(
                                        UnaryCommand::Named(op_name.clone()),
                                        Box::new(Expression::ConsumeableArray(
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
                return Expression::UnaryCommand(op_type.clone(), Box::new(right_o), range.clone());
            }
            Expression::BinaryCommand(op_type, left, right, range) => {
                let left_o = left.clone().optimize();
                let right_o = right.clone().optimize();
                match op_type {
                    BinaryCommand::Named(op_name) => match op_name.to_lowercase().as_str() {
                        "params" => {
                            if let Expression::Array(a_array, a_range) = &right_o {
                                if a_array.iter().all(|e| e.is_safe_param()) {
                                    trace!(
                                        "optimizing [B:{}] ({}) => ConsumeableArray",
                                        op_type.as_str(),
                                        self.source()
                                    );
                                    return Expression::BinaryCommand(
                                        BinaryCommand::Named(op_name.clone()),
                                        Box::new(left_o),
                                        Box::new(Expression::ConsumeableArray(
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
                            fn op(l: &f32, r: &f32) -> f32 {
                                l + r
                            }
                            if let Some(eval) =
                                self.op_bin_float(op_type, range, &left_o, &right_o, op)
                            {
                                return eval;
                            }
                        }
                        {
                            fn op(l: &String, r: &String) -> String {
                                format!("{}{}", l, r)
                            }
                            if let Some(eval) =
                                self.op_bin_string(op_type, range, &left_o, &right_o, op)
                            {
                                return eval;
                            }
                        }
                    }
                    BinaryCommand::Sub => {
                        fn op(l: &f32, r: &f32) -> f32 {
                            l - r
                        }
                        if let Some(eval) = self.op_bin_float(op_type, range, &left_o, &right_o, op)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Mul => {
                        fn op(l: &f32, r: &f32) -> f32 {
                            l * r
                        }
                        if let Some(eval) = self.op_bin_float(op_type, range, &left_o, &right_o, op)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Div => {
                        fn op(l: &f32, r: &f32) -> f32 {
                            l / r
                        }
                        if let Some(eval) = self.op_bin_float(op_type, range, &left_o, &right_o, op)
                        {
                            return eval;
                        }
                    }
                    BinaryCommand::Rem | BinaryCommand::Mod => {
                        fn op(l: &f32, r: &f32) -> f32 {
                            l % r
                        }
                        if let Some(eval) = self.op_bin_float(op_type, range, &left_o, &right_o, op)
                        {
                            return eval;
                        }
                    }
                    _ => {}
                }
                return Expression::BinaryCommand(
                    op_type.clone(),
                    Box::new(left_o),
                    Box::new(right_o),
                    range.clone(),
                );
            }
            _ => {
                return self;
            }
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
    fn is_safe_param(&self) -> bool {
        match self {
            Expression::Array(array, _) => {
                if let Some(param_default) = array.get(1) {
                    if param_default.is_array() {
                        return false;
                    }
                }
            }
            _ => {}
        }
        return true; // every other check (for constness) will be handled by the serializer
    }

    // Boilerplate for uniary and binary ops
    fn op_uni_string(
        &self,
        op_type: &UnaryCommand,
        range: &Range<usize>,
        right: &Expression,
        op: fn(&String) -> String,
    ) -> Option<Expression> {
        if let Expression::String(right_string, _, ref right_wrapper) = right {
            if right_string.is_ascii() {
                let new_string = op(&right_string.to_string());
                trace!(
                    "optimizing [U:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(),
                    new_string
                );
                return Some(Expression::String(
                    new_string.into(),
                    range.clone(),
                    right_wrapper.clone(),
                ));
            } else {
                warn!(
                    "Skipping Optimization because unicode [U:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(),
                    right_string.to_string()
                );
            }
        }
        return None;
    }
    fn op_uni_float(
        &self,
        op_type: &UnaryCommand,
        range: &Range<usize>,
        right: &Expression,
        op: fn(&f32) -> f32,
    ) -> Option<Expression> {
        if let Expression::Number(crate::Scalar(right_number), _) = right {
            let new_number = op(right_number);
            if new_number.is_finite() {
                trace!(
                    "optimizing [U:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(),
                    new_number
                );
                return Some(Expression::Number(crate::Scalar(new_number), range.clone()));
            } else {
                warn!(
                    "Skipping Optimization because NaN [U:{}] ({}) => {}",
                    op_type.as_str(),
                    self.source(),
                    new_number
                );
            }
        }
        return None;
    }
    fn op_bin_string(
        &self,
        op_type: &BinaryCommand,
        range: &Range<usize>,
        left: &Expression,
        right: &Expression,
        op: fn(&String, &String) -> String,
    ) -> Option<Expression> {
        if let Expression::String(left_string, _, ref _left_wrapper) = left {
            if let Expression::String(right_string, _, ref right_wrapper) = right {
                if right_string.is_ascii() && left_string.is_ascii() {
                    let new_string = op(&left_string.to_string(), &right_string.to_string());
                    trace!(
                        "optimizing [B:{}] ({}) => {}",
                        op_type.as_str(),
                        self.source(),
                        new_string
                    );
                    return Some(Expression::String(
                        new_string.into(),
                        range.clone(),
                        right_wrapper.clone(),
                    ));
                } else {
                    warn!(
                        "Skipping Optimization because unicode [B:{}] ({}) => {}",
                        op_type.as_str(),
                        self.source(),
                        right_string.to_string()
                    );
                }
            }
        }
        return None;
    }
    fn op_bin_float(
        &self,
        op_type: &BinaryCommand,
        range: &Range<usize>,
        left: &Expression,
        right: &Expression,
        op: fn(&f32, &f32) -> f32,
    ) -> Option<Expression> {
        if let Expression::Number(crate::Scalar(left_number), _) = left {
            if let Expression::Number(crate::Scalar(right_number), _) = right {
                let new_number = op(left_number, right_number);
                if new_number.is_finite() {
                    trace!(
                        "optimizing [B:{}] ({}) => {}",
                        op_type.as_str(),
                        self.source(),
                        new_number
                    );
                    return Some(Expression::Number(crate::Scalar(new_number), range.clone()));
                } else {
                    warn!(
                        "Skipping Optimization because NaN [B:{}] ({}) => {}",
                        op_type.as_str(),
                        self.source(),
                        new_number
                    );
                }
            }
        }
        return None;
    }
}
