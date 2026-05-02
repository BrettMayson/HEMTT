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
            Token::Function(_) | Token::UnaryMinus => {
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
                // If there's a function on top of the stack, pop it to output
                if let Some(Token::Function(_)) = operator_stack.last() {
                    output_queue.push(operator_stack.pop()?);
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
            Token::Function(name) => {
                let operand = stack.pop()?;
                let result = match name.as_str() {
                    "sin" => operand.sin(),
                    "cos" => operand.cos(),
                    "tan" | "tg" => operand.tan(),
                    "asin" => operand.asin(),
                    "acos" => operand.acos(),
                    "atan" | "atg" => operand.atan(),
                    "rad" => operand.to_radians(),
                    "deg" => operand.to_degrees(),
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

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Operator(char),
    Function(String),
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
    let mut current_identifier = String::new();
    let chars: Vec<char> = expression.chars().collect();
    let functions = [
        "acos", "asin", "atan", "atg", "cos", "deg", "rad", "sin", "tan", "tg",
    ];

    for (i, &c) in chars.iter().enumerate() {
        match c {
            '0'..='9' | '.' => {
                if !current_identifier.is_empty() {
                    // We were reading an identifier, so finish it first
                    let identifier = current_identifier.clone();
                    current_identifier.clear();
                    if identifier == "pi" {
                        tokens.push(Token::Number(std::f64::consts::PI));
                    } else if functions.contains(&identifier.as_str()) {
                        tokens.push(Token::Function(identifier));
                    } else {
                        return Err(format!("Unknown identifier: {identifier}"));
                    }
                }
                current_number.push(c);
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                if !current_number.is_empty() {
                    tokens.push(Token::Number(
                        current_number
                            .parse()
                            .map_err(|e: <f64 as FromStr>::Err| e.to_string())?,
                    ));
                    current_number.clear();
                }
                current_identifier.push(c);
            }
            _ => {
                if !current_number.is_empty() {
                    tokens.push(Token::Number(
                        current_number
                            .parse()
                            .map_err(|e: <f64 as FromStr>::Err| e.to_string())?,
                    ));
                    current_number.clear();
                }
                if !current_identifier.is_empty() {
                    let identifier = current_identifier.clone();
                    current_identifier.clear();
                    if identifier == "pi" {
                        tokens.push(Token::Number(std::f64::consts::PI));
                    } else if functions.contains(&identifier.as_str()) {
                        tokens.push(Token::Function(identifier));
                    } else {
                        return Err(format!("Unknown identifier: {identifier}"));
                    }
                }

                match c {
                    '+' | '*' | '/' | '^' | '%' => tokens.push(Token::Operator(c)),
                    '(' => tokens.push(Token::LeftParenthesis),
                    ')' => tokens.push(Token::RightParenthesis),
                    '-' => {
                        let is_unary_context = matches!(
                            tokens.last(),
                            Some(Token::Operator(_) | Token::LeftParenthesis | Token::Function(_))
                                | None
                        );

                        if is_unary_context {
                            let next_char = chars.get(i + 1);
                            if matches!(next_char, Some('0'..='9' | '.')) {
                                current_number.push(c);
                            } else {
                                tokens.push(Token::UnaryMinus);
                            }
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
        tokens.push(Token::Number(
            current_number
                .parse()
                .map_err(|e: <f64 as FromStr>::Err| e.to_string())?,
        ));
    }
    if !current_identifier.is_empty() {
        let identifier = current_identifier.clone();
        if identifier == "pi" {
            tokens.push(Token::Number(std::f64::consts::PI));
        } else if functions.contains(&identifier.as_str()) {
            tokens.push(Token::Function(identifier));
        } else {
            return Err(format!("Unknown identifier: {identifier}"));
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
        assert_eq!(super::eval("1 + 2 + 3"), Some(6.0));
    }

    #[test]
    pub fn eval_subtraction() {
        assert_eq!(super::eval("1-1"), Some(0.0));
        assert_eq!(super::eval("1 - 1"), Some(0.0));
        assert_eq!(super::eval("1 - -1"), Some(2.0));
        assert_eq!(super::eval("1 - 2 - 3"), Some(-4.0));
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
        assert_eq!(super::eval("(-1)"), Some(-1.0));
        assert_eq!(super::eval("2--1"), Some(3.0));
        assert_eq!(super::eval("(-1) + 2"), Some(1.0));
    }

    #[test]
    fn constants() {
        assert_eq!(super::eval("pi"), Some(std::f64::consts::PI));
        assert_eq!(super::eval("pi + 1"), Some(std::f64::consts::PI + 1.0));
        assert_eq!(super::eval("2 * pi"), Some(2.0 * std::f64::consts::PI));
    }

    #[test]
    fn trigonometric() {
        // sin(0) = 0
        assert_eq!(super::eval("sin(0)"), Some(0.0));
        // sin(pi/2) = 1
        let sin_pi_half = super::eval("sin(pi / 2)").expect("Failed to evaluate sin(pi / 2)");
        assert!((sin_pi_half - 1.0).abs() < 1e-10);

        // cos(0) = 1
        assert_eq!(super::eval("cos(0)"), Some(1.0));
        // cos(pi) = -1
        let cos_pi = super::eval("cos(pi)").expect("Failed to evaluate cos(pi)");
        assert!((cos_pi - (-1.0)).abs() < 1e-10);

        // tan(0) = 0
        assert_eq!(super::eval("tan(0)"), Some(0.0));

        // tg is alias for tan
        assert_eq!(super::eval("tg(0)"), Some(0.0));
    }

    #[test]
    fn inverse_trigonometric() {
        // asin(0) = 0
        assert_eq!(super::eval("asin(0)"), Some(0.0));
        // asin(1) = pi/2
        let asin_one = super::eval("asin(1)").expect("Failed to evaluate asin(1)");
        assert!((asin_one - std::f64::consts::PI / 2.0).abs() < 1e-10);

        // acos(1) = 0
        assert_eq!(super::eval("acos(1)"), Some(0.0));
        // acos(0) = pi/2
        let acos_zero = super::eval("acos(0)").expect("Failed to evaluate acos(0)");
        assert!((acos_zero - std::f64::consts::PI / 2.0).abs() < 1e-10);

        // atan(0) = 0
        assert_eq!(super::eval("atan(0)"), Some(0.0));

        // atg is alias for atan
        assert_eq!(super::eval("atg(0)"), Some(0.0));
    }

    #[test]
    fn angle_conversion() {
        // rad(180) = pi
        let rad_180 = super::eval("rad(180)").expect("Failed to evaluate rad(180)");
        assert!((rad_180 - std::f64::consts::PI).abs() < 1e-10);

        // deg(pi) = 180
        let deg_pi = super::eval("deg(pi)").expect("Failed to evaluate deg(pi)");
        assert!((deg_pi - 180.0).abs() < 1e-10);
    }

    #[test]
    fn complex_expressions() {
        // sin(pi / 2) + cos(0)
        let expr =
            super::eval("sin(pi / 2) + cos(0)").expect("Failed to evaluate sin(pi / 2) + cos(0)");
        assert!((expr - 2.0).abs() < 1e-10);

        // 2 * sin(pi / 6) = 1 (sin(30°) = 0.5)
        let expr = super::eval("2 * sin(rad(30))").expect("Failed to evaluate 2 * sin(rad(30))");
        assert!((expr - 1.0).abs() < 1e-10);
    }
}
