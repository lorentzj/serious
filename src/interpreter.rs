use super::parser::parse;
use super::parser::Expression;
use super::parser::ExpressionData;
use super::parser::Operation;
use super::error::Error;
use super::error::ErrorType;

use std::collections::HashMap;

pub fn interpret_tree(tree: Expression, context: &HashMap<char, f64>) -> Result<f64, Error> {
    match tree.data {
        ExpressionData::Constant(val) => Ok(val),
        ExpressionData::Op(lhs, op, rhs) => {
            let (lhs, rhs) = (interpret_tree(*lhs, context)?, interpret_tree(*rhs, context)?);
            match op {
                Operation::Add => Ok(lhs + rhs),
                Operation::Subtract => Ok(lhs - rhs),
                Operation::Multiply => Ok(lhs * rhs),
                Operation::Divide => {
                    if rhs == 0. {
                        Err(Error::new(
                            ErrorType::UndefinedOperation,
                            "division by zero".to_string(),
                            tree.start,
                            tree.end
                        ))
                    } else {
                        Ok(lhs / rhs)
                    }
                }
                Operation::Exponentiate => {
                    if lhs == 0. && rhs == 0. {
                        Err(Error::new(
                            ErrorType::UndefinedOperation,
                            "0^0 is undetermined".to_string(),
                            tree.start,
                            tree.end
                        ))
                    } else {
                        Ok(lhs.powf(rhs))
                    }
                }
            }
        }
        ExpressionData::Identifier(name) => {
            match context.get(&name) {
                Some(val) => Ok(*val),
                None => Err(Error::new(
                    ErrorType::UnboundIdentifier,
                    format!("identifier {} is not bound", name),
                    tree.start,
                    tree.end
                ))
            }
        }
    }
}

pub fn interpret(text: &str, bound_vars: &HashMap<char, f64>) -> Result<f64, Error> {
    interpret_tree(parse(text)?, bound_vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal() {
        let val = interpret("10.3", &HashMap::new()).unwrap();
        assert_eq!(val, 10.3);
    }

    #[test]
    fn unbound_var() {
        let mut context: HashMap<char, f64> = HashMap::new();

        context.insert('x', 3.);

        let err = interpret("3 + xy", &context).unwrap_err();
        assert_eq!(err.message, "identifier y is not bound");
        assert_eq!(err.start, 5);
        assert_eq!(err.end, 6);
    }

    #[test]
    fn div_zero_1() {
        let err = interpret("10/0", &HashMap::new()).unwrap_err();
        assert_eq!(err.message, "division by zero");
        assert_eq!(err.start, 0);
        assert_eq!(err.end, 4);
    }

    #[test]
    fn div_zero_2() {
        let err = interpret("2^(56 / (2 - 2)) * 3", &HashMap::new()).unwrap_err();
        assert_eq!(err.message, "division by zero");
        assert_eq!(err.start, 2);
        assert_eq!(err.end, 16);
    }

    #[test]
    fn simple_add() {
        let val = interpret("1 + 2 + 3 + 4.8", &HashMap::new()).unwrap();
        assert_eq!(val, 10.8);
    }

    #[test]
    fn pythagoras() {
        let mut context: HashMap<char, f64> = HashMap::new();

        context.insert('x', 3.);
        context.insert('y', 4.);

        let val = interpret("(x^2 + y^2)^0.5", &context).unwrap();
        assert_eq!(val, 5.);
    }

    #[test]
    fn quadratic() {
        let mut context: HashMap<char, f64> = HashMap::new();

        context.insert('x', 4.);

        let val = interpret("-2x^2 + 3x - 5", &context).unwrap();
        assert_eq!(val, -25.);
    }
}