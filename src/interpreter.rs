use super::parser::{parse, Expression, ExpressionData, Operation};
use super::error::{Error, ErrorType};

pub type Context = std::collections::HashMap<char, f64>;

#[macro_export]
macro_rules! create_context {
    ($($id:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        use std::collections::HashMap;
        let iter = IntoIterator::into_iter([$(($id, $v),)*]);
        HashMap::<char, f64>::from(Iterator::collect(iter))
    }};
}

pub fn interpret_tree(tree: Expression, context: &Context) -> Result<f64, Error> {
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

pub fn interpret(text: &str, bound_vars: &Context) -> Result<f64, Error> {
    interpret_tree(parse(text)?, bound_vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal() {
        let val = interpret("10.3", &Context::new()).unwrap();
        assert_eq!(val, 10.3);
    }

    #[test]
    fn unbound_var() {
        let context = create_context!{'x' => 3.};

        let err = interpret("3 + xy", &context).unwrap_err();
        assert_eq!(err.message, "identifier y is not bound");
        assert_eq!(err.start, 5);
        assert_eq!(err.end, 6);
    }

    #[test]
    fn div_zero_1() {
        let err = interpret("10/0", &Context::new()).unwrap_err();
        assert_eq!(err.message, "division by zero");
        assert_eq!(err.start, 0);
        assert_eq!(err.end, 4);
    }

    #[test]
    fn div_zero_2() {
        let err = interpret("2^(56 / (2 - 2)) * 3", &Context::new()).unwrap_err();
        assert_eq!(err.message, "division by zero");
        assert_eq!(err.start, 2);
        assert_eq!(err.end, 16);
    }

    #[test]
    fn simple_add() {
        let val = interpret("1 + 2 + 3 + 4.8", &Context::new()).unwrap();
        assert_eq!(val, 10.8);
    }

    #[test]
    fn pythagoras() {
        let context = create_context!{'x' => 3., 'y' => 4.};

        let val = interpret("(x^2 + y^2)^0.5", &context).unwrap();
        assert_eq!(val, 5.);
    }

    #[test]
    fn quadratic() {
        let context = create_context!{'x' => 4.};

        let val = interpret("-2x^2 + 3x - 5", &context).unwrap();
        assert_eq!(val, -25.);
    }
}