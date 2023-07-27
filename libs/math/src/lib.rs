use std::collections::HashMap;

pub fn eval(expression: &str) -> Option<f64> {
    evaluate_postfix(&shunting_yard(expression))
}

fn shunting_yard(expression: &str) -> Vec<Token> {
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
    .cloned()
    .collect();

    let tokens = tokenize(expression);

    for token in tokens {
        match token {
            Token::Number(_) => output_queue.push(token),
            Token::Operator(op) => {
                while let Some(Token::Operator(top_op)) = operator_stack.last() {
                    if let Some((precedence, associativity)) = operators.get(&op) {
                        if let Some((top_precedence, _top_associativity)) = operators.get(top_op) {
                            if (associativity == &Associativity::Left
                                && precedence <= top_precedence)
                                || (associativity == &Associativity::Right
                                    && precedence < top_precedence)
                            {
                                output_queue.push(operator_stack.pop().unwrap());
                                continue;
                            }
                        }
                    }
                    break;
                }
                operator_stack.push(token);
            }
            Token::LeftParenthesis => operator_stack.push(token),
            Token::RightParenthesis => {
                while let Some(top_token) = operator_stack.pop() {
                    if let Token::LeftParenthesis = top_token {
                        break;
                    }
                    output_queue.push(top_token);
                }
            }
        }
    }

    while let Some(operator) = operator_stack.pop() {
        output_queue.push(operator);
    }

    output_queue
}

fn evaluate_postfix(tokens: &[Token]) -> Option<f64> {
    let mut stack: Vec<f64> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(num) => stack.push(*num),
            Token::Operator(op) => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
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
            _ => return None,
        }
    }

    stack.pop()
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Token {
    Number(f64),
    Operator(char),
    LeftParenthesis,
    RightParenthesis,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Associativity {
    Left,
    Right,
}

fn tokenize(expression: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut current_number = String::new();

    for c in expression.chars() {
        match c {
            '0'..='9' | '.' => current_number.push(c),
            _ => {
                if !current_number.is_empty() {
                    tokens.push(Token::Number(current_number.parse().unwrap()));
                    current_number.clear();
                }
                match c {
                    '+' | '*' | '/' | '^' | '%' => tokens.push(Token::Operator(c)),
                    '(' => tokens.push(Token::LeftParenthesis),
                    ')' => tokens.push(Token::RightParenthesis),
                    '-' => {
                        if matches!(
                            tokens.last(),
                            Some(Token::Operator(_) | Token::LeftParenthesis)
                        ) {
                            current_number.push(c);
                        } else {
                            tokens.push(Token::Operator(c));
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    if !current_number.is_empty() {
        tokens.push(Token::Number(current_number.parse().unwrap()));
    }

    tokens
}
