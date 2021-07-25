use super::error::{Error, ErrorType};
pub use super::lexer::Operation;
use super::lexer::{lex, Token, TokenType};

#[derive(Debug, PartialEq)]
pub enum ExpressionData {
    Op(Box<Expression>, Operation, Box<Expression>),
    Constant(f64),
    Identifier(char),
}

#[derive(Debug, PartialEq)]
pub struct Expression {
    pub data: ExpressionData,
    pub start: usize,
    pub end: usize
}

impl Expression {
    pub fn new_const(val: f64, start: usize, end: usize) -> Expression {
        let data = ExpressionData::Constant(val);
        Expression { data, start, end }
    }

    pub fn new_id(name: char, start: usize, end: usize) -> Expression {
        let data = ExpressionData::Identifier(name);
        Expression { data, start, end }
    }

    pub fn new_op(lhs: Expression, op: Operation, rhs: Expression) -> Expression {
        let start = lhs.start;
        let end = rhs.end;
        let data = ExpressionData::Op(Box::new(lhs), op, Box::new(rhs));
        Expression { data, start, end }
    }

    pub fn with_bounds(self, start: usize, end: usize) -> Expression {
        Expression { data: self.data, start, end }
    }
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

fn match_paren(tokens: &[Token], start: usize) -> Result<usize, Error> {
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
                        return Err(Error::new(
                            ErrorType::BadParse,
                            "expected expression".to_string(),
                            tokens[i].start,
                            tokens[i].end
                        ))
                    }
                    _ => ()
                }
            }
            _ => ()
        }
        i += 1;
    }

    Err(Error::new(
        ErrorType::BadParse,
        "failed to match paren".to_string(),
        tokens[start - 1].start,
        tokens[start - 1].end
    ))
}

fn parse_tokens(tokens: &[Token], start: usize, end: usize) -> Result<Expression, Error> {
    let mut stack: Vec<(Operation, Expression)> = vec![];
    let (mut curr_lhs, mut i) = match tokens[start].token_type {
        TokenType::Constant(val) => {
            (Expression::new_const(val, tokens[start].start, tokens[start].end), start + 1)
        }
        TokenType::Identifier(name) => {
            (Expression::new_id(name, tokens[start].start, tokens[start].end), start + 1)
        }
        TokenType::Op(Operation::Subtract) => {
            // unary minus implemented as a zero-width 0
            (Expression::new_const(0., tokens[start].start, tokens[start].start), start)
        }
        TokenType::OpenParen => {
            let end_paren = match_paren(tokens, start + 1)?;
            let inner_expr = parse_tokens(tokens, start + 1, end_paren)?;
            (inner_expr.with_bounds(tokens[start].start, tokens[end_paren].end), end_paren + 1)
        }
        _ => {
            return Err(Error::new(
                ErrorType::BadParse,
                "expected expression".to_string(),
                tokens[start].start,
                tokens[start].end
            ))
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
                return Err(Error::new(
                    ErrorType::BadParse,
                    "constant on RHS of implicit multiplication".to_string(),
                    tokens[i].start,
                    tokens[i].end
                ))    
            }
            _ => {
                return Err(Error::new(
                    ErrorType::BadParse,
                    "expected operation".to_string(),
                    tokens[i].start,
                    tokens[i].end
                ))    
            }
        };

        if i == tokens.len() {
            return Err(Error::new(
                ErrorType::BadParse,
                "expected expression".to_string(),
                tokens[i - 1].end,
                tokens[i - 1].end + 1
            ))
        }

        let curr_rhs = match tokens[i].token_type {
            TokenType::Op(Operation::Subtract) => {
                return Err(Error::new(
                    ErrorType::BadParse,
                    "expected expression; wrap in parens for unary minus".to_string(),
                    tokens[i].start,
                    tokens[i].end
                ))        
            }
            TokenType::Op(_) => {
                return Err(Error::new(
                    ErrorType::BadParse,
                    "expected expression".to_string(),
                    tokens[i].start,
                    tokens[i].end
                ))
            }
            TokenType::Identifier(name) => {
                i += 1;
                Expression::new_id(name, tokens[i - 1].start, tokens[i - 1].end)
            }
            TokenType::Constant(val) => {
                i += 1;
                Expression::new_const(val, tokens[i - 1].start, tokens[i - 1].end)
            }
            TokenType::OpenParen => {
                let end_paren = match_paren(tokens, i + 1)?;
                let old_i = i;
                i = end_paren + 1;
                let inner_expr = parse_tokens(tokens, old_i + 1, end_paren)?;
                inner_expr.with_bounds(tokens[old_i].start, tokens[end_paren].end)  
            }
            TokenType::CloseParen => {
                return Err(Error::new(
                    ErrorType::BadParse,
                    "expected expression".to_string(),
                    tokens[i].start,
                    tokens[i].end
                ))
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
                        stack.push((prev_op, Expression::new_op(
                            prev_rhs,
                            curr_op,
                            curr_rhs
                        )));
                    } else {
                        stack.push((prev_prev_op, Expression::new_op(prev_prev_rhs, prev_op, prev_rhs)));
                        stack.push((curr_op, curr_rhs));
                    }
                } else if prev_precedence_wins {
                        stack.push((prev_op, Expression::new_op(prev_rhs, curr_op, curr_rhs)));
                } else {
                    curr_lhs = Expression::new_op(curr_lhs, prev_op, prev_rhs);
                    stack.push((curr_op, curr_rhs));    
                }
            } else {
                stack.push((curr_op, curr_rhs));
                break;
            }
        }
    }

    if let Some((last_op, last_rhs)) = stack.pop() {
        curr_lhs = Expression::new_op(curr_lhs, last_op, last_rhs);
    }

    Ok(curr_lhs)
}

/// Parses a Serious expression into an abstract syntax tree.
pub fn parse(text: &str) -> Result<Expression, Error> {
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
        assert_eq!(err.start, 0);
        assert_eq!(err.end, 1);
    }

    #[test]
    fn err_from_lex() {
        let err = parse("2*0.2.3").unwrap_err();
        assert_eq!(err.message, "invalid float literal");
        assert_eq!(err.start, 2);
        assert_eq!(err.end, 7);
    }

    #[test]
    fn simple_constant() {
        let tree = parse("100").unwrap();
        assert_eq!(tree, Expression::new_const(100., 0, 3));
    }

    #[test]
    fn simple_add() {
        let tree = parse("1+2").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_const(1., 0, 1),
            Operation::Add,
            Expression::new_const(2., 2, 3)
        ));
    }

    #[test]
    fn left_associate() {
        let tree = parse("1+2-3").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_const(1., 0, 1),
                Operation::Add,
                Expression::new_const(2., 2, 3)
            ),
            Operation::Subtract,
            Expression::new_const(3., 4, 5)
        ));
    }

    #[test]
    fn order_of_ops_add_div() {
        let tree = parse("A + 2/3").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_id('A', 0, 1),
            Operation::Add,
            Expression::new_op(
                Expression::new_const(2., 4, 5),
                Operation::Divide,
                Expression::new_const(3., 6, 7)
            )
        ));
    }

    #[test]
    fn order_of_ops_mult_exp() {
        let tree = parse("2x^3").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_const(2., 0, 1),
            Operation::Multiply,
            Expression::new_op(
                Expression::new_id('x', 1, 2),
                Operation::Exponentiate,
                Expression::new_const(3., 3, 4)
            )
        ));
    }

    #[test]
    fn order_of_ops_multilevel_1() {
        let tree = parse("2x^4 + x*3").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_const(2., 0, 1),
                Operation::Multiply,
                Expression::new_op(
                    Expression::new_id('x', 1, 2),
                    Operation::Exponentiate,
                    Expression::new_const(4., 3, 4)
                )
            ),
            Operation::Add,
            Expression::new_op(
                Expression::new_id('x', 7, 8),
                Operation::Multiply,
                Expression::new_const(3., 9, 10)
            )
        ));
    }

    #[test]
    fn order_of_ops_multilevel_2() {
        let tree = parse("1+2*3 * 4^5").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_const(1., 0, 1),
            Operation::Add,
            Expression::new_op(
                Expression::new_op(
                    Expression::new_const(2., 2, 3),
                    Operation::Multiply,
                    Expression::new_const(3., 4, 5)
                ),
                Operation::Multiply,
                Expression::new_op(
                    Expression::new_const(4., 8, 9),
                    Operation::Exponentiate,
                    Expression::new_const(5., 10, 11)
                )
            )
        ));
    }

    #[test]
    fn order_of_ops_multilevel_3() {
        let tree = parse("1 + xy^2^3").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_const(1., 0, 1),
            Operation::Add,
            Expression::new_op(
                Expression::new_id('x', 4, 5),
                Operation::Multiply,
                Expression::new_op(
                    Expression::new_op(
                        Expression::new_id('y', 5, 6),
                        Operation::Exponentiate,
                        Expression::new_const(2., 7, 8)
                    ),
                    Operation::Exponentiate,
                    Expression::new_const(3., 9, 10)
                )
            )
        ));
    }

    #[test]
    fn simple_parens() {
        let tree = parse("2*( x + 0.4 )").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_const(2., 0, 1),
            Operation::Multiply,
            Expression::new_op(
                Expression::new_id('x', 4, 5),
                Operation::Add,
                Expression::new_const(0.4, 8, 11)
            ).with_bounds(2, 13)
        ));
    }

    #[test]
    fn nested_parens() {
        let tree = parse("(x+y)(2^(20z))").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_id('x', 1, 2),
                Operation::Add,
                Expression::new_id('y', 3, 4)
            ).with_bounds(0, 5),
            Operation::Multiply,
            Expression::new_op(
                Expression::new_const(2., 6, 7),
                Operation::Exponentiate,
                Expression::new_op(
                    Expression::new_const(20., 9, 11),
                    Operation::Multiply,
                    Expression::new_id('z', 11, 12)
                ).with_bounds(8, 13)
            ).with_bounds(5, 14)
        ));
    }

    #[test]
    fn extra_open_paren() {
        let err = parse("(1*(2+3)").unwrap_err();
        assert_eq!(err.message, "failed to match paren");
        assert_eq!(err.start, 0);
        assert_eq!(err.end, 1);
    }

    #[test]
    fn extra_close_paren() {
        let err = parse("4*1+(2+3))").unwrap_err();
        assert_eq!(err.message, "expected operation");
        assert_eq!(err.start, 9);
        assert_eq!(err.end, 10);
    }

    #[test]
    fn implicit_mult_identifier() {
        let tree = parse("2+4x+7").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_const(2., 0, 1),
                Operation::Add,
                Expression::new_op(
                    Expression::new_const(4., 2, 3),
                    Operation::Multiply,
                    Expression::new_id('x', 3, 4)
                )
            ),
            Operation::Add,
            Expression::new_const(7., 5, 6)
        ));
    }

    #[test]
    fn attempt_const_implicit_mult() {
        let err = parse("x3").unwrap_err();
        assert_eq!(err.message, "constant on RHS of implicit multiplication");
        assert_eq!(err.start, 1);
        assert_eq!(err.end, 2);
    }

    #[test]
    fn empty_paren() {
        let err = parse("x()").unwrap_err();
        assert_eq!(err.message, "expected expression");
        assert_eq!(err.start, 2);
        assert_eq!(err.end, 3);
    }

    #[test]
    fn just_paren() {
        let err = parse("(").unwrap_err();
        assert_eq!(err.message, "failed to match paren");
        assert_eq!(err.start, 0);
        assert_eq!(err.end, 1);
    }

    #[test]
    fn complex_implicit_mult_identifier() {
        let tree = parse("4x^2 + 2xy").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_const(4., 0, 1),
                Operation::Multiply,
                Expression::new_op(
                    Expression::new_id('x', 1, 2),
                    Operation::Exponentiate,
                    Expression::new_const(2., 3, 4)
                )
            ),
            Operation::Add,
            Expression::new_op(
                Expression::new_op(
                    Expression::new_const(2., 7, 8),
                    Operation::Multiply,
                    Expression::new_id('x', 8, 9)
                ),
                Operation::Multiply,
                Expression::new_id('y', 9, 10),
            ),
        ));
    }

    // #[test]
    // fn complex_implicit_mult_parens() {
    //     let tree = parse("4z(9x^2 + 3)").unwrap();
    //     assert_eq!(tree, Expression::Op(
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Constant(4.)),
    //             Operation::Multiply,
    //             Box::new(Expression::Identifier('z')),
    //         )),
    //         Operation::Multiply,
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Constant(9.)),
    //                 Operation::Multiply,
    //                 Box::new(Expression::Op(
    //                     Box::new(Expression::Identifier('x')),
    //                     Operation::Exponentiate,
    //                     Box::new(Expression::Constant(2.))
    //                 )),
    //             )),
    //             Operation::Add,
    //             Box::new(Expression::Constant(3.))
    //         ))
    //     ));
    // }

    // #[test]
    // fn factored_quadratic() {
    //     let tree = parse("(2a+5)(a-4)").unwrap();
    //     assert_eq!(tree, Expression::Op(
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Constant(2.)),
    //                 Operation::Multiply,
    //                 Box::new(Expression::Identifier('a')),
    //             )),
    //             Operation::Add,
    //             Box::new(Expression::Constant(5.))
    //         )),
    //         Operation::Multiply,
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Identifier('a')),
    //             Operation::Subtract,
    //             Box::new(Expression::Constant(4.))
    //         ))
    //     ));
    // }

    // #[test]
    // fn factored_quartic() {
    //     let tree = parse("(a + 2)(a - 4)(a^2 + 8)").unwrap();
    //     assert_eq!(tree, Expression::Op(
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Identifier('a')),
    //                 Operation::Add,
    //                 Box::new(Expression::Constant(2.))
    //             )),
    //             Operation::Multiply,
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Identifier('a')),
    //                 Operation::Subtract,
    //                 Box::new(Expression::Constant(4.))
    //             ))
    //         )),
    //         Operation::Multiply,
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Identifier('a')),
    //                 Operation::Exponentiate,
    //                 Box::new(Expression::Constant(2.)),
    //             )),
    //             Operation::Add,
    //             Box::new(Expression::Constant(8.))
    //         ))
    //     ));
    // }

    // #[test]
    // fn unary_minus() {
    //     let tree = parse("-2x").unwrap();
    //     assert_eq!(tree, Expression::Op(
    //         Box::new(Expression::Constant(0.)),
    //         Operation::Subtract,
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Constant(2.)),
    //             Operation::Multiply,
    //             Box::new(Expression::Identifier('x'))
    //         ))
    //     ));
    // }

    // #[test]
    // fn order_of_ops_unary_minus() {
    //     let tree = parse("-2x^2 - 3").unwrap();
    //     assert_eq!(tree, Expression::Op(
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Constant(0.)),
    //             Operation::Subtract,
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Constant(2.)),
    //                 Operation::Multiply,
    //                 Box::new(Expression::Op(
    //                     Box::new(Expression::Identifier('x')),
    //                     Operation::Exponentiate,
    //                     Box::new(Expression::Constant(2.)),
    //                 ))
    //             ))
    //         )),
    //         Operation::Subtract,
    //         Box::new(Expression::Constant(3.))
    //     ));
    // }

    // #[test]
    // fn unary_minus_error() {
    //     let err = parse("3*-2x").unwrap_err();
    //     assert_eq!(err.message, "expected expression; wrap in parens for unary minus");
    //     assert_eq!(err.start, 2);
    //     assert_eq!(err.end, 3);
    // }

    // #[test]
    // fn unary_minus_nested() {
    //     let tree = parse("3*(-2xy)").unwrap();
    //     assert_eq!(tree, Expression::Op(
    //         Box::new(Expression::Constant(3.)),
    //         Operation::Multiply,
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Constant(0.)),
    //             Operation::Subtract,
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Constant(2.)),
    //                 Operation::Multiply,
    //                 Box::new(Expression::Identifier('x'))
    //             ))
    //         ))
    //     ));
    // }

    // #[test]
    // fn complex_polynomial_1() {
    //     let tree = parse("1 + 3x^2y^3 + 6").unwrap();
    //     assert_eq!(tree, Expression::Op(
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Constant(1.)),
    //             Operation::Add,
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Op(
    //                     Box::new(Expression::Constant(3.)),
    //                     Operation::Multiply,
    //                     Box::new(Expression::Op(
    //                         Box::new(Expression::Identifier('x')),
    //                         Operation::Exponentiate,
    //                         Box::new(Expression::Constant(2.))
    //                     )),
    //                 )),
    //                 Operation::Multiply,
    //                 Box::new(Expression::Op(
    //                     Box::new(Expression::Identifier('y')),
    //                     Operation::Exponentiate,
    //                     Box::new(Expression::Constant(3.))
    //                 )),
    //             )),
    //         )),
    //         Operation::Add,
    //         Box::new(Expression::Constant(6.))
    //     ));
    // }

    // #[test]
    // fn complex_polynomial_2() {
    //     let tree = parse("3a^4b^3 + c^2d").unwrap();
    //     assert_eq!(tree, Expression::Op(
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Constant(3.)),
    //                 Operation::Multiply,
    //                 Box::new(Expression::Op(
    //                     Box::new(Expression::Identifier('a')),
    //                     Operation::Exponentiate,
    //                     Box::new(Expression::Constant(4.))
    //                 )),
    //             )),
    //             Operation::Multiply,
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Identifier('b')),
    //                 Operation::Exponentiate,
    //                 Box::new(Expression::Constant(3.))
    //             )),
    //         )),
    //         Operation::Add,
    //         Box::new(Expression::Op(
    //             Box::new(Expression::Op(
    //                 Box::new(Expression::Identifier('c')),
    //                 Operation::Exponentiate,
    //                 Box::new(Expression::Constant(2.))
    //             )),
    //             Operation::Multiply,
    //             Box::new(Expression::Identifier('d'))
    //         )),
    //     ));
    // }
}