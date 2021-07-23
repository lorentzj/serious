pub use super::error::ParseError;
pub use super::lexer::Operation;
use super::lexer::Token;
use super::lexer::TokenType;
use super::lexer::lex;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Op(Box<Expression>, Operation, Box<Expression>),
    Constant(f64),
    Identifier(char),
}

fn precedence(operation: &Operation) -> i32 {
    match operation {
        Operation::Add => 0,
        Operation::Subtract => 0,
        Operation::Multiply => 1,
        Operation::Divide => 1,
        Operation::Exponentiate => 2
    }
}

fn match_paren(tokens: &[Token], start: usize) -> Result<usize, ParseError> {
    let mut i = start;
    let mut level = 1;
    while i < tokens.len() {
        match tokens[i].token_type {
            TokenType::OpenParen => {
                level += 1;
            }
            TokenType::CloseParen => {
                level -= 1;
                match level {
                    0 => {
                        return Ok(i);
                    }
                    level if level < 0 => {
                        return Err(ParseError::new("expected expression".to_string(), i));
                    }
                    _ => ()
                }
            }
            _ => ()
        }
        i += 1;
    }
    Err(ParseError::new("failed to match paren".to_string(), start - 1))
}

fn parse_tokens(tokens: &[Token], start: usize, end: usize) -> Result<Expression, ParseError> {
    let mut stack: Vec<(Operation, Expression)> = vec![];
    let (mut curr_lhs, mut i) = match tokens[start].token_type {
        TokenType::Constant(val) => {
            (Expression::Constant(val), start + 1)
        }
        TokenType::Identifier(name) => {
            (Expression::Identifier(name), start + 1)
        }
        TokenType::Op(Operation::Subtract) => {
            (Expression::Constant(0.), start)
        }
        TokenType::OpenParen => {
            let end_paren = match_paren(tokens, start + 1)?;
            (parse_tokens(tokens, start + 1, end_paren - 1)?, end_paren + 1)
        }
        _ => {
            return Err(ParseError::new("expected expression".to_string(), tokens[start].start));
        }
    };

    while i < end {
        let curr_op = match tokens[i].token_type {
            TokenType::Op(op) => {
                i += 1;
                op
            }
            TokenType::Identifier(_) | TokenType::OpenParen => Operation::Multiply,
            TokenType::Constant(_) => {
                return Err(ParseError::new("constant on RHS of implicit multiplication".to_string(), tokens[i].start));
            }
            _ => {
                return Err(ParseError::new("expected operation".to_string(), tokens[i].start));
            }
        };

        if i == tokens.len() {
            return Err(ParseError::new("expected expression".to_string(), tokens[i - 1].end));
        }

        let curr_rhs = match tokens[i].token_type {
            TokenType::Op(Operation::Subtract) => {
                return Err(ParseError::new("expected expression; wrap in parens for unary minus".to_string(), tokens[i].start));
            }
            TokenType::Op(_) => {
                return Err(ParseError::new("expected expression".to_string(), tokens[i].start));
            }
            TokenType::Identifier(name) => {
                i += 1;
                Expression::Identifier(name)
            }
            TokenType::Constant(val) => {
                i += 1;
                Expression::Constant(val)
            }
            TokenType::OpenParen => {
                let end_paren = match_paren(tokens, i + 1)?;
                let old_i = i;
                i = end_paren + 1;
                parse_tokens(tokens, old_i + 1, end_paren - 1)?
            }
            TokenType::CloseParen => {
                return Err(ParseError::new("expected expression".to_string(), tokens[i].start));
            }
        };

        stack.push((curr_op, curr_rhs));

        while let Some((curr_op, curr_rhs)) = stack.pop() {
            if let Some((prev_op, prev_rhs)) = stack.pop() {
                let prev_precedence_wins = precedence(&prev_op) < precedence(&curr_op);
                let not_at_end = i < end;
                if prev_precedence_wins && not_at_end {
                    stack.push((prev_op, prev_rhs));
                    stack.push((curr_op, curr_rhs));
                    break;
                } else if let Some((prev_prev_op, prev_prev_rhs)) = stack.pop() {
                    if prev_precedence_wins {
                        stack.push((prev_prev_op, prev_prev_rhs));
                        stack.push((prev_op, Expression::Op(
                            Box::new(prev_rhs),
                            curr_op,
                            Box::new(curr_rhs)
                        )));
                    } else {
                        stack.push((prev_prev_op, Expression::Op(
                            Box::new(prev_prev_rhs),
                            prev_op,
                            Box::new(prev_rhs)
                        )));
                        stack.push((curr_op, curr_rhs));
                    }
                } else if prev_precedence_wins {
                        stack.push((prev_op, Expression::Op(
                            Box::new(prev_rhs),
                            curr_op,
                            Box::new(curr_rhs)
                        )));
                } else {
                    curr_lhs = Expression::Op(
                        Box::new(curr_lhs),
                        prev_op,
                        Box::new(prev_rhs),
                    );
                    stack.push((curr_op, curr_rhs));    
                }
            } else {
                stack.push((curr_op, curr_rhs));
                break;
            }
        }
    }

    if let Some((last_op, last_rhs)) = stack.pop() {
        curr_lhs = Expression::Op(
            Box::new(curr_lhs),
            last_op,
            Box::new(last_rhs),
        );
    }

    Ok(curr_lhs)
}

pub fn parse(text: &str) -> Result<Expression, ParseError> {
    let tokens = lex(text)?;
    parse_tokens(&tokens, 0, tokens.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let err = parse("").unwrap_err();
        assert_eq!(err.message, "expected token");
        assert_eq!(err.position, 0);
    }

    #[test]
    fn err_from_lex() {
        let err = parse("2*0.2.3").unwrap_err();
        assert_eq!(err.message, "invalid float literal");
        assert_eq!(err.position, 2);
    }

    #[test]
    fn simple_constant() {
        let tree = parse("100").unwrap();
        assert_eq!(tree, Expression::Constant(100.));
    }

    #[test]
    fn simple_add() {
        let tree = parse("1+2").unwrap();
            assert_eq!(tree, Expression::Op(
                Box::new(Expression::Constant(1.)),
                Operation::Add,
                Box::new(Expression::Constant(2.))
            ));
    }

    #[test]
    fn left_associate() {
        let tree = parse("1+2-3").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Constant(1.)),
                Operation::Add,
                Box::new(Expression::Constant(2.))
            )),
            Operation::Subtract,
            Box::new(Expression::Constant(3.))
        ));
    }

    #[test]
    fn order_of_ops_add_div() {
        let tree = parse("A+2/3").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Identifier('A')),
            Operation::Add,
            Box::new(Expression::Op(
                Box::new(Expression::Constant(2.)),
                Operation::Divide,
                Box::new(Expression::Constant(3.))
            ))
        ));
    }

    #[test]
    fn order_of_ops_mult_exp() {
        let tree = parse("1*x^3").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Constant(1.)),
            Operation::Multiply,
            Box::new(Expression::Op(
                Box::new(Expression::Identifier('x')),
                Operation::Exponentiate,
                Box::new(Expression::Constant(3.))
            ))
        ));
    }

    #[test]
    fn order_of_ops_complex() {
        let tree = parse("2*y^4+x*3^2").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Constant(2.)),
                Operation::Multiply,
                Box::new(Expression::Op(
                    Box::new(Expression::Identifier('y')),
                    Operation::Exponentiate,
                    Box::new(Expression::Constant(4.))
                )),
            )),
            Operation::Add,
            Box::new(Expression::Op(
                Box::new(Expression::Identifier('x')),
                Operation::Multiply,
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(3.)),
                    Operation::Exponentiate,
                    Box::new(Expression::Constant(2.))
                ))
            ))
        ));
    }

    #[test]
    fn order_of_ops_add_exp() {
        let tree = parse("1+2^3").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Constant(1.)),
            Operation::Add,
            Box::new(Expression::Op(
                Box::new(Expression::Constant(2.)),
                Operation::Exponentiate,
                Box::new(Expression::Constant(3.))
            ))
        ));
    }

    #[test]
    fn order_of_ops_multilevel() {
        let tree = parse("1 + xy^2").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Constant(1.)),
            Operation::Add,
            Box::new(Expression::Op(
                Box::new(Expression::Identifier('x')),
                Operation::Multiply,
                Box::new(Expression::Op(
                    Box::new(Expression::Identifier('y')),
                    Operation::Exponentiate,
                    Box::new(Expression::Constant(2.))
                ))
            ))
        ));
    }

    #[test]
    fn order_of_ops_repeated_exp() {
        let tree = parse("1 + xy^2^3").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Constant(1.)),
            Operation::Add,
            Box::new(Expression::Op(
                Box::new(Expression::Identifier('x')),
                Operation::Multiply,
                Box::new(Expression::Op(
                    Box::new(Expression::Op(
                        Box::new(Expression::Identifier('y')),
                        Operation::Exponentiate,
                        Box::new(Expression::Constant(2.))
                    )),
                    Operation::Exponentiate,
                    Box::new(Expression::Constant(3.))
                ))
            ))
        ));
    }

    #[test]
    fn simple_parens() {
        let tree = parse("1*(2+3)").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Constant(1.)),
            Operation::Multiply,
            Box::new(Expression::Op(
                Box::new(Expression::Constant(2.)),
                Operation::Add,
                Box::new(Expression::Constant(3.))
            ))
        ));
    }

    #[test]
    fn nested_parens() {
        let tree = parse("(4+y)*(1*(20^z))").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Constant(4.)),
                Operation::Add,
                Box::new(Expression::Identifier('y'))
            )),
            Operation::Multiply,
            Box::new(Expression::Op(
                Box::new(Expression::Constant(1.)),
                Operation::Multiply,
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(20.)),
                    Operation::Exponentiate,
                    Box::new(Expression::Identifier('z'))
                ))
            ))
        ));
    }

    #[test]
    fn extra_open_paren() {
        let err = parse("(1*(2+3)").unwrap_err();
        assert_eq!(err.message, "failed to match paren");
        assert_eq!(err.position, 0);
    }

    #[test]
    fn extra_close_paren() {
        let err = parse("4*1+(2+3))").unwrap_err();
        assert_eq!(err.message, "expected operation");
        assert_eq!(err.position, 9);
    }

    #[test]
    fn implicit_mult_identifier() {
        let tree = parse("2+4x+7").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Constant(2.)),
                Operation::Add,
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(4.)),
                    Operation::Multiply,
                    Box::new(Expression::Identifier('x'))
                )),
            )),
            Operation::Add,
            Box::new(Expression::Constant(7.)),
        ));
    }

    #[test]
    fn attempt_const_implicit_mult() {
        let err = parse("x3").unwrap_err();
        assert_eq!(err.message, "constant on RHS of implicit multiplication");
        assert_eq!(err.position, 1);
    }

    #[test]
    fn empty_paren() {
        let err = parse("x()").unwrap_err();
        assert_eq!(err.message, "expected expression");
        assert_eq!(err.position, 2);
    }

    #[test]
    fn just_paren() {
        let err = parse("(").unwrap_err();
        assert_eq!(err.message, "failed to match paren");
        assert_eq!(err.position, 0);
    }

    #[test]
    fn complex_implicit_mult_identifier() {
        let tree = parse("4x^2 + 2xy").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Constant(4.)),
                Operation::Multiply,
                Box::new(Expression::Op(
                    Box::new(Expression::Identifier('x')),
                    Operation::Exponentiate,
                    Box::new(Expression::Constant(2.))
                )),
            )),
            Operation::Add,
            Box::new(Expression::Op(
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(2.)),
                    Operation::Multiply,
                    Box::new(Expression::Identifier('x'))
                )),
                Operation::Multiply,
                Box::new(Expression::Identifier('y'))
            )),
        ));
    }

    #[test]
    fn complex_implicit_mult_parens() {
        let tree = parse("4z(9x^2 + 3)").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Constant(4.)),
                Operation::Multiply,
                Box::new(Expression::Identifier('z')),
            )),
            Operation::Multiply,
            Box::new(Expression::Op(
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(9.)),
                    Operation::Multiply,
                    Box::new(Expression::Op(
                        Box::new(Expression::Identifier('x')),
                        Operation::Exponentiate,
                        Box::new(Expression::Constant(2.))
                    )),
                )),
                Operation::Add,
                Box::new(Expression::Constant(3.))
            ))
        ));
    }

    #[test]
    fn factored_quadratic() {
        let tree = parse("(2a+5)(a-4)").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(2.)),
                    Operation::Multiply,
                    Box::new(Expression::Identifier('a')),
                )),
                Operation::Add,
                Box::new(Expression::Constant(5.))
            )),
            Operation::Multiply,
            Box::new(Expression::Op(
                Box::new(Expression::Identifier('a')),
                Operation::Subtract,
                Box::new(Expression::Constant(4.))
            ))
        ));
    }

    #[test]
    fn factored_quartic() {
        let tree = parse("(a + 2)(a - 4)(a^2 + 8)").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Op(
                    Box::new(Expression::Identifier('a')),
                    Operation::Add,
                    Box::new(Expression::Constant(2.))
                )),
                Operation::Multiply,
                Box::new(Expression::Op(
                    Box::new(Expression::Identifier('a')),
                    Operation::Subtract,
                    Box::new(Expression::Constant(4.))
                ))
            )),
            Operation::Multiply,
            Box::new(Expression::Op(
                Box::new(Expression::Op(
                    Box::new(Expression::Identifier('a')),
                    Operation::Exponentiate,
                    Box::new(Expression::Constant(2.)),
                )),
                Operation::Add,
                Box::new(Expression::Constant(8.))
            ))
        ));
    }

    #[test]
    fn unary_minus() {
        let tree = parse("-2x").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Constant(0.)),
            Operation::Subtract,
            Box::new(Expression::Op(
                Box::new(Expression::Constant(2.)),
                Operation::Multiply,
                Box::new(Expression::Identifier('x'))
            ))
        ));
    }

    #[test]
    fn order_of_ops_unary_minus() {
        let tree = parse("-2x^2 - 3").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Constant(0.)),
                Operation::Subtract,
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(2.)),
                    Operation::Multiply,
                    Box::new(Expression::Op(
                        Box::new(Expression::Identifier('x')),
                        Operation::Exponentiate,
                        Box::new(Expression::Constant(2.)),
                    ))
                ))
            )),
            Operation::Subtract,
            Box::new(Expression::Constant(3.))
        ));
    }

    #[test]
    fn unary_minus_error() {
        let err = parse("3*-2x").unwrap_err();
        assert_eq!(err.message, "expected expression; wrap in parens for unary minus");
        assert_eq!(err.position, 2);
    }

    #[test]
    fn unary_minus_nested() {
        let tree = parse("3*(-2xy)").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Constant(3.)),
            Operation::Multiply,
            Box::new(Expression::Op(
                Box::new(Expression::Constant(0.)),
                Operation::Subtract,
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(2.)),
                    Operation::Multiply,
                    Box::new(Expression::Identifier('x'))
                ))
            ))
        ));
    }

    #[test]
    fn complex_polynomial_1() {
        let tree = parse("1 + 3x^2y^3 + 6").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Constant(1.)),
                Operation::Add,
                Box::new(Expression::Op(
                    Box::new(Expression::Op(
                        Box::new(Expression::Constant(3.)),
                        Operation::Multiply,
                        Box::new(Expression::Op(
                            Box::new(Expression::Identifier('x')),
                            Operation::Exponentiate,
                            Box::new(Expression::Constant(2.))
                        )),
                    )),
                    Operation::Multiply,
                    Box::new(Expression::Op(
                        Box::new(Expression::Identifier('y')),
                        Operation::Exponentiate,
                        Box::new(Expression::Constant(3.))
                    )),
                )),
            )),
            Operation::Add,
            Box::new(Expression::Constant(6.))
        ));
    }

    #[test]
    fn complex_polynomial_2() {
        let tree = parse("3a^4b^3 + c^2d").unwrap();
        assert_eq!(tree, Expression::Op(
            Box::new(Expression::Op(
                Box::new(Expression::Op(
                    Box::new(Expression::Constant(3.)),
                    Operation::Multiply,
                    Box::new(Expression::Op(
                        Box::new(Expression::Identifier('a')),
                        Operation::Exponentiate,
                        Box::new(Expression::Constant(4.))
                    )),
                )),
                Operation::Multiply,
                Box::new(Expression::Op(
                    Box::new(Expression::Identifier('b')),
                    Operation::Exponentiate,
                    Box::new(Expression::Constant(3.))
                )),
            )),
            Operation::Add,
            Box::new(Expression::Op(
                Box::new(Expression::Op(
                    Box::new(Expression::Identifier('c')),
                    Operation::Exponentiate,
                    Box::new(Expression::Constant(2.))
                )),
                Operation::Multiply,
                Box::new(Expression::Identifier('d'))
            )),
        ));
    }
}