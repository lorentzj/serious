use super::error::{Error, ErrorType};
pub use super::lexer::Operation;
use super::lexer::{lex, Token, TokenType};

/// The semantic content of an expression.
#[derive(Debug, PartialEq)]
pub enum ExpressionData {
    /// An binary operation.
    /// In the case of unary minus, the first expression will be a zero-width 0.
    Op(Box<Expression>, Operation, Box<Expression>),
    /// A literal constant.
    Constant(f64),
    /// A named identifier.
    Identifier(char)
}

/// The output of a successful parse; contains sub-expressions in a tree structure.
#[derive(Debug, PartialEq)]
pub struct Expression {
    /// The semantic content of the expression.
    pub data: ExpressionData,
    /// The start of the expression in the original text.
    pub start: usize,
    /// The end of the expression in the original text.
    pub end: usize
}

impl Expression {
    /// Create an expression for a constant literal.
    pub fn new_const(val: f64, start: usize, end: usize) -> Expression {
        let data = ExpressionData::Constant(val);
        Expression { data, start, end }
    }

    /// Create an expression for an identifier.
    pub fn new_id(name: char, start: usize, end: usize) -> Expression {
        let data = ExpressionData::Identifier(name);
        Expression { data, start, end }
    }

    /// Create an expression given an operation and its two operands.
    /// The expression bounds will be recalcalated as the start of the left-hand side and the end of the right-hand side.
    pub fn new_op(lhs: Expression, op: Operation, rhs: Expression) -> Expression {
        let start = lhs.start;
        let end = rhs.end;
        let data = ExpressionData::Op(Box::new(lhs), op, Box::new(rhs));
        Expression { data, start, end }
    }

    /// Re-assign the `start` and `end` positions of an expression.
    /// This is useful for including the parentheses around a sub-expression.
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

fn parse_tokens(tokens: &[Token], start: usize, expect_close_paren: bool) -> Result<(Expression, usize), Error> {
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
            if start + 1 == tokens.len() {
                return Err(Error::new(
                    ErrorType::BadParse,
                    "failed to match paren".to_string(),
                    tokens[start].start,
                    tokens[start].end
                ));
            }
            let (inner_expr, end_paren) = parse_tokens(tokens, start + 1, true)?;
            if end_paren == tokens.len() {
                return Err(Error::new(
                    ErrorType::BadParse,
                    "failed to match paren".to_string(),
                    tokens[start].start,
                    tokens[start].end
                ));
            }
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

    while i < tokens.len() {
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
            },
            TokenType::CloseParen => {
                if expect_close_paren {
                    break;
                } else {
                    return Err(Error::new(
                        ErrorType::BadParse,
                        "expected expression".to_string(),
                        tokens[i].start,
                        tokens[i].end
                    ))    
                }
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
                let old_i = i;
                let (inner_expr, end_paren) = parse_tokens(tokens, old_i + 1, true)?;
                i = end_paren + 1;
                if end_paren == tokens.len() {
                    return Err(Error::new(
                        ErrorType::BadParse,
                        "failed to match paren".to_string(),
                        tokens[old_i].start,
                        tokens[old_i].end
                    ))
                }
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

        let at_end = if i == tokens.len() {
            true
        } else if expect_close_paren {
            tokens[i].token_type == TokenType::CloseParen
        } else {
            false
        };

        while let Some((curr_op, curr_rhs)) = stack.pop() {
            if let Some((prev_op, prev_rhs)) = stack.pop() {
                let prev_precedence_wins = precedence(&prev_op) < precedence(&curr_op);
                if prev_precedence_wins && !at_end {
                    stack.push((prev_op, prev_rhs));
                    stack.push((curr_op, curr_rhs));
                    break;
                } else if let Some((prev_prev_op, prev_prev_rhs)) = stack.pop() {
                    if prev_precedence_wins {
                        stack.push((prev_prev_op, prev_prev_rhs));
                        stack.push((prev_op, Expression::new_op(prev_rhs, curr_op, curr_rhs)));
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

    Ok((curr_lhs, i))
}

/// Parses a Serious expression into an abstract syntax tree.
pub fn parse(text: &str) -> Result<Expression, Error> {
    let tokens = lex(text)?;
    Ok(parse_tokens(&tokens, 0, false)?.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let err = parse("").unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "expected token".to_string(),
            0,
            1
        ));
    }

    #[test]
    fn err_from_lex() {
        let err = parse("2*0.2.3").unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "invalid float literal".to_string(),
            2,
            7
        ));
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
        let tree = parse("(x+y)(2^(20z))^2").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_id('x', 1, 2),
                Operation::Add,
                Expression::new_id('y', 3, 4)
            ).with_bounds(0, 5),
            Operation::Multiply,
            Expression::new_op(
                Expression::new_op(
                    Expression::new_const(2., 6, 7),
                    Operation::Exponentiate,
                    Expression::new_op(
                        Expression::new_const(20., 9, 11),
                        Operation::Multiply,
                        Expression::new_id('z', 11, 12)
                    ).with_bounds(8, 13)
                ).with_bounds(5, 14),
                Operation::Exponentiate,
                Expression::new_const(2., 15, 16)
            )
        ));
    }

    #[test]
    fn extra_open_paren() {
        let err = parse("(1*(2+3)").unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "failed to match paren".to_string(),
            0,
            1
        ));
    }

    #[test]
    fn extra_close_paren() {
        let err = parse("3 + ((4*1)^2+(2+3))) + 6").unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "expected expression".to_string(),
            19,
            20
        ));
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
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "constant on RHS of implicit multiplication".to_string(),
            1,
            2
        ));
    }

    #[test]
    fn empty_paren() {
        let err = parse("x()").unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "expected expression".to_string(),
            2,
            3
        ));
    }

    #[test]
    fn just_paren() {
        let err = parse("(").unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "failed to match paren".to_string(),
            0,
            1
        ));
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

    #[test]
    fn complex_implicit_mult_parens() {
        let tree = parse("4z(9x^2 + 30)").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_const(4., 0, 1),
                Operation::Multiply,
                Expression::new_id('z', 1, 2)
            ),
            Operation::Multiply,
            Expression::new_op(
                Expression::new_op(
                    Expression::new_const(9., 3, 4),
                    Operation::Multiply,
                    Expression::new_op(
                        Expression::new_id('x', 4, 5),
                        Operation::Exponentiate,
                        Expression::new_const(2., 6, 7)
                    )
                ),
                Operation::Add,
                Expression::new_const(30., 10, 12),
            ).with_bounds(2, 13),
        ));
    }

    #[test]
    fn factored_quadratic() {
        let tree = parse("(2a+5)(a-4)").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_op(
                    Expression::new_const(2., 1, 2),
                    Operation::Multiply,
                    Expression::new_id('a', 2, 3)
                ),
                Operation::Add,
                Expression::new_const(5., 4, 5)
            ).with_bounds(0, 6),
            Operation::Multiply,
            Expression::new_op(
                Expression::new_id('a', 7, 8),
                Operation::Subtract,
                Expression::new_const(4., 9, 10),
            ).with_bounds(6, 11)
        ));
    }

    #[test]
    fn factored_quartic() {
        let tree = parse("(a + 2)(a - 4)(a^20 + 8) + 4").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_op(
                    Expression::new_op(
                        Expression::new_id('a', 1, 2),
                        Operation::Add,
                        Expression::new_const(2., 5, 6)
                    ).with_bounds(0, 7),
                    Operation::Multiply,
                    Expression::new_op(
                        Expression::new_id('a', 8, 9),
                        Operation::Subtract,
                        Expression::new_const(4., 12, 13),
                    ).with_bounds(7, 14)                
                ),
                Operation::Multiply,
                Expression::new_op(
                    Expression::new_op(
                        Expression::new_id('a', 15, 16),
                        Operation::Exponentiate,
                        Expression::new_const(20., 17, 19)
                    ),
                    Operation::Add,
                    Expression::new_const(8., 22, 23)
                ).with_bounds(14, 24)
            ),
            Operation::Add,
            Expression::new_const(4., 27, 28)
        ));
    }

    #[test]
    fn unary_minus() {
        let tree = parse("-2x").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_const(0., 0, 0),
            Operation::Subtract,
            Expression::new_op(
                Expression::new_const(2., 1, 2),
                Operation::Multiply,
                Expression::new_id('x', 2, 3)
            )
        ));
    }

    #[test]
    fn order_of_ops_unary_minus() {
        let tree = parse("-2x^2 - 3").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_const(0., 0, 0),
                Operation::Subtract,
                Expression::new_op(
                    Expression::new_const(2., 1, 2),
                    Operation::Multiply,
                    Expression::new_op(
                        Expression::new_id('x', 2, 3),
                        Operation::Exponentiate,
                        Expression::new_const(2., 4, 5),
                    )
                )
            ),
            Operation::Subtract,
            Expression::new_const(3., 8, 9)
        ));
    }

    #[test]
    fn unary_minus_error() {
        let err = parse("3*-2x").unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "expected expression; wrap in parens for unary minus".to_string(),
            2,
            3
        ));
    }

    #[test]
    fn unary_minus_nested() {
        let tree = parse("3*(-2xy)").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_const(3., 0, 1),
            Operation::Multiply,
            Expression::new_op(
                Expression::new_const(0., 3, 3),
                Operation::Subtract,
                Expression::new_op(
                    Expression::new_op(
                        Expression::new_const(2., 4, 5),
                        Operation::Multiply,
                        Expression::new_id('x', 5, 6)
                    ),
                    Operation::Multiply,
                    Expression::new_id('y', 6, 7)
                )
            ).with_bounds(2, 8)
        ));
    }

    #[test]
    fn complex_polynomial_1() {
        let tree = parse("1 + 3x^2y^3 + 6").unwrap();
        assert_eq!(tree, Expression::new_op(
            Expression::new_op(
                Expression::new_const(1., 0, 1),
                Operation::Add,
                Expression::new_op(
                    Expression::new_op(
                        Expression::new_const(3., 4, 5),
                        Operation::Multiply,
                        Expression::new_op(
                            Expression::new_id('x', 5, 6),
                            Operation::Exponentiate,
                            Expression::new_const(2., 7, 8)
                        ),
                    ),
                    Operation::Multiply,
                    Expression::new_op(
                        Expression::new_id('y', 8, 9),
                        Operation::Exponentiate,
                        Expression::new_const(3., 10, 11)
                    ),
                ),
            ),
            Operation::Add,
            Expression::new_const(6., 14, 15)
        ));
    }
}