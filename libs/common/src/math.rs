//! Math utilities

use std::{collections::HashMap, str::FromStr};

#[must_use]
/// Evaluates a mathematical expression
pub fn eval(expression: &str) -> Option<f64> {
    evaluate_postfix(&shunting_yard(expression)?)
}

fn shunting_yard(expression: &str) -> Option<Vec<Token>> {
    let mut output_queue: Vec<Token> = Vec::new();
    let mut operator_stack: Vec<Token> = Vec::new();
    let operators: HashMap<char, (u8, Associativity)> = [
        ('+', (1, Associativity::Left)),
        ('-', (1, Associativity::Left)),
        ('*', (2, Associativity::Left)),
        ('/', (2, Associativity::Left)),
        ('^', (3, Associativity::Right)),
        ('%', (2, Associativity::Left)),
    ]
    .iter()
    .copied()
    .collect();

    let tokens = tokenize(expression).ok()?;

    for token in tokens {
        match token {
            Token::Number(_) => output_queue.push(token),
            Token::Operator(op) => {
                while let Some(Token::Operator(top_op)) = operator_stack.last() {
                    if let Some((precedence, associativity)) = operators.get(&op)
                        && let Some((top_precedence, _top_associativity)) = operators.get(top_op)
                        && ((associativity == &Associativity::Left && precedence <= top_precedence)
                            || (associativity == &Associativity::Right
                                && precedence < top_precedence))
                    {
                        output_queue.push(operator_stack.pop()?);
                        continue;
                    }
                    break;
                }
                operator_stack.push(token);
            }
            Token::UnaryMinus => {
                operator_stack.push(token);
            }
            Token::LeftParenthesis => operator_stack.push(token),
            Token::RightParenthesis => {
                let mut hit_left = false;
                while let Some(top_token) = operator_stack.pop() {
                    if top_token == Token::LeftParenthesis {
                        hit_left = true;
                        break;
                    }
                    output_queue.push(top_token);
                }
                if !hit_left {
                    return None;
                }
            }
        }
    }

    while let Some(operator) = operator_stack.pop() {
        output_queue.push(operator);
    }

    Some(output_queue)
}

fn evaluate_postfix(tokens: &[Token]) -> Option<f64> {
    let mut stack: Vec<f64> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(num) => stack.push(*num),
            Token::Operator(op) => {
                let right = stack.pop()?;
                let left = stack.pop()?;
                let result = match op {
                    '+' => left + right,
                    '-' => left - right,
                    '*' => left * right,
                    '/' => left / right,
                    '^' => left.powf(right),
                    '%' => left % right,
                    _ => return None,
                };
                stack.push(result);
            }
            Token::UnaryMinus => {
                let operand = stack.pop()?;
                stack.push(-operand);
            }
            _ => return None,
        }
    }

    stack.pop()
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Token {
    Number(f64),
    Operator(char),
    UnaryMinus,
    LeftParenthesis,
    RightParenthesis,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Associativity {
    Left,
    Right,
}

fn tokenize(expression: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut current_number = String::new();

    for c in expression.chars() {
        match c {
            '0'..='9' | '.' => current_number.push(c),
            _ => {
                if !current_number.is_empty() {
                    if current_number == "-" {
                        tokens.push(Token::UnaryMinus);
                    } else {
                        tokens.push(Token::Number(
                            current_number
                                .parse()
                                .map_err(|e: <f64 as FromStr>::Err| e.to_string())?,
                        ));
                    }
                    current_number.clear();
                }
                match c {
                    '+' | '*' | '/' | '^' | '%' => tokens.push(Token::Operator(c)),
                    '(' => tokens.push(Token::LeftParenthesis),
                    ')' => tokens.push(Token::RightParenthesis),
                    '-' => {
                        if matches!(
                            tokens.last(),
                            Some(Token::Operator(_) | Token::LeftParenthesis) | None
                        ) {
                            current_number.push(c);
                        } else {
                            tokens.push(Token::Operator(c));
                        }
                    }
                    ' ' => (),
                    _ => return Err(format!("Invalid operator: {c}")),
                }
            }
        }
    }

    if !current_number.is_empty() {
        if current_number == "-" {
            tokens.push(Token::UnaryMinus);
        } else {
            tokens.push(Token::Number(
                current_number
                    .parse()
                    .map_err(|e: <f64 as FromStr>::Err| e.to_string())?,
            ));
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn eval_addition() {
        assert_eq!(super::eval("1+1"), Some(2.0));
        assert_eq!(super::eval("1 + 1"), Some(2.0));
        assert_eq!(super::eval("-1 + 1"), Some(0.0));
    }

    #[test]
    pub fn eval_subtraction() {
        assert_eq!(super::eval("1-1"), Some(0.0));
        assert_eq!(super::eval("1 - 1"), Some(0.0));
    }

    #[test]
    pub fn eval_multiplication() {
        assert_eq!(super::eval("2*2"), Some(4.0));
        assert_eq!(super::eval("2 * 2"), Some(4.0));
        assert_eq!(super::eval("-0.01 * 0.5"), Some(-0.005));
    }

    #[test]
    pub fn eval_division() {
        assert_eq!(super::eval("4/2"), Some(2.0));
        assert_eq!(super::eval("4 / 2"), Some(2.0));
    }

    #[test]
    pub fn eval_exponentiation() {
        assert_eq!(super::eval("2^2"), Some(4.0));
        assert_eq!(super::eval("2 ^ 2"), Some(4.0));
    }

    #[test]
    pub fn eval_modulo() {
        assert_eq!(super::eval("5%2"), Some(1.0));
        assert_eq!(super::eval("5 % 2"), Some(1.0));
    }

    #[test]
    pub fn eval_parentheses() {
        assert_eq!(super::eval("(1+1)*2"), Some(4.0));
        assert_eq!(super::eval("(1+1) * 2"), Some(4.0));
        assert_eq!(super::eval("( 1 + 1 ) * 2"), Some(4.0));
        assert_eq!(super::eval("1+(1*2)"), Some(3.0));
        assert_eq!(super::eval("1 + (1 * 2)"), Some(3.0));
        assert_eq!(super::eval("1 + ( 1 * 2 )"), Some(3.0));
    }

    #[test]
    pub fn eval_negative() {
        assert_eq!(super::eval("1+-1"), Some(0.0));
        assert_eq!(super::eval("1 + -1"), Some(0.0));
    }

    #[test]
    pub fn eval_decimal() {
        assert_eq!(super::eval("0.5"), Some(0.5));
    }

    #[test]
    pub fn eval_invalid() {
        assert_eq!(super::eval("1+"), None);
        assert_eq!(super::eval("1 +"), None);
    }

    #[test]
    pub fn eval_invalid_parentheses() {
        assert_eq!(super::eval("(1+1"), None);
        assert_eq!(super::eval("(1 + 1"), None);
        assert_eq!(super::eval("1+1)"), None);
        assert_eq!(super::eval("1 + 1)"), None);
    }

    #[test]
    pub fn eval_invalid_operator() {
        assert_eq!(super::eval("1&1"), None);
        assert_eq!(super::eval("1 & 1"), None);
    }

    #[test]
    fn basic() {
        assert_eq!(super::eval("1 + 1"), Some(2.0));
        assert_eq!(super::eval("1 + 1 * 2"), Some(3.0));
        assert_eq!(super::eval("1 + 1 * 2 / 2"), Some(2.0));
        assert_eq!(super::eval("1 - 1"), Some(0.0));
        assert_eq!(super::eval("1 - 1 * 2"), Some(-1.0));
        assert_eq!(super::eval("1 - 1 * 2 / 2"), Some(0.0));
    }

    #[test]
    fn parens() {
        assert_eq!(super::eval("(1 + 1) * 2"), Some(4.0));
        assert_eq!(super::eval("1 + (1 * 2)"), Some(3.0));
        assert_eq!(super::eval("1 + (1 * 2) / 2"), Some(2.0));
        assert_eq!(super::eval("1 - (1 + 1)"), Some(-1.0));
        assert_eq!(super::eval("1 - (1 * 2)"), Some(-1.0));
        assert_eq!(super::eval("1 - (1 * 2) / 2"), Some(0.0));
    }

    #[test]
    fn negation() {
        assert_eq!(super::eval("1 + -1"), Some(0.0));
        assert_eq!(super::eval("1 + -1 * 2"), Some(-1.0));
        assert_eq!(super::eval("1 + -1 * 2 / 2"), Some(0.0));
        assert_eq!(super::eval("1 - -1"), Some(2.0));
    }

    #[test]
    fn minus() {
        assert_eq!(super::eval("-2"), Some(-2.0));
        assert_eq!(super::eval("-(2)"), Some(-2.0));
        assert_eq!(super::eval("-(-2)"), Some(2.0));
        assert_eq!(super::eval("-(1 + 1)"), Some(-2.0));
        assert_eq!(super::eval("-(-(1 + 1))"), Some(2.0));
        assert_eq!(super::eval("2 * -(3 + 1)"), Some(-8.0));
    }
}
