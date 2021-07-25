use super::parser::{parse, Expression, ExpressionData, Operation};
use super::error::{Error, ErrorType};

/// A hashmap from identifiers to values which can be applied to an expression using [serious::interpret](interpret).
pub type Context = std::collections::HashMap<char, f64>;

/// Creates a [serious::Context](Context) which can be applied to an expression using [serious::interpret](interpret).
/// 
/// Each `id` (char) will bound to its corresponding `val` (f64).
///
/// ```
/// use serious::{interpreter::Context, create_context};
///
/// let mut test_context = Context::new();
///
/// assert_eq!(create_context!{}, test_context);
///
/// test_context.insert('a', 4.);
/// assert_eq!(create_context!{'a' => 4.}, test_context);
/// 
/// test_context.insert('b', 5.);
/// assert_eq!(create_context!{'a' => 4., 'b' => 5.}, test_context);
/// ```
#[macro_export]
macro_rules! create_context {
    ($($id:expr => $val:expr),*$(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        use std::collections::HashMap;
        let iter = IntoIterator::into_iter([$(($id, $val),)*]);
        HashMap::<char, f64>::from(Iterator::collect(iter))
    }};
}

fn op_representation(op: Operation) -> char {
    match op {
        Operation::Exponentiate => '^',
        Operation::Multiply => '*',
        Operation::Divide => '/',
        Operation::Add => '+',
        Operation::Subtract => '-'
    }
}

/// Evaluates a pre-parsed Serious expression.
pub fn interpret_tree(tree: Expression, context: &Context) -> Result<f64, Error> {
    match tree.data {
        ExpressionData::Constant(val) => Ok(val),
        ExpressionData::Op(lhs, op, rhs) => {
            let (lhs, rhs) = (interpret_tree(*lhs, context)?, interpret_tree(*rhs, context)?);
            let result = match op {
                Operation::Add => lhs + rhs,
                Operation::Subtract => lhs - rhs,
                Operation::Multiply => lhs * rhs,
                Operation::Divide => {
                    if rhs == 0. {
                        return Err(Error::new(
                            ErrorType::UndefinedOperation,
                            "division by zero is undefined".to_string(),
                            tree.start,
                            tree.end
                        ))
                    } else {
                        lhs/rhs
                    }
                }
                Operation::Exponentiate => {
                    if lhs == 0. && rhs == 0. {
                        f64::NAN
                    } else {
                        lhs.powf(rhs)
                    }
                }
            };

            if result.is_infinite() {
                Err(Error::new(
                    ErrorType::Overflow,
                    format!("({}) {} ({}) overflowed f64", lhs, op_representation(op), rhs),
                    tree.start,
                    tree.end
                ))
            } else if result.is_nan() {
                Err(Error::new(
                    ErrorType::UndefinedOperation,
                    format!("({}) {} ({}) is undefined", lhs, op_representation(op), rhs),
                    tree.start,
                    tree.end
                ))
            } else {
                Ok(result)
            }
        }

        ExpressionData::Identifier(name) => {
            match context.get(&name) {
                Some(val) => Ok(*val),
                None => Err(Error::new(
                    ErrorType::UnboundIdentifier,
                    format!("identifier '{}' is not bound", name),
                    tree.start,
                    tree.end
                ))
            }
        }
    }
}

/// Evaluates a Serious expression.
pub fn interpret(text: &str, bound_vars: &Context) -> Result<f64, Error> {
    interpret_tree(parse(text)?, bound_vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal() {
        let val = interpret("10.3", &create_context!{}).unwrap();
        assert_eq!(val, 10.3);
    }

    #[test]
    fn err_from_parse() {
        let err = interpret("(1*(2+3)", &create_context!{}).unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::BadParse,
            "failed to match paren".to_string(),
            0,
            1
        ));
    }

    #[test]
    fn unbound_var() {
        let context = create_context!{'x' => 3.};

        let err = interpret("3 + xy", &context).unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::UnboundIdentifier,
            "identifier 'y' is not bound".to_string(),
            5,
            6
        ));
    }

    #[test]
    fn div_zero_1() {
        let err = interpret("10/0", &create_context!{}).unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::UndefinedOperation,
            "division by zero is undefined".to_string(),
            0,
            4
        ));
    }

    #[test]
    fn div_zero_2() {
        let err = interpret("2^(56 / (2 - 2)) * 3", &create_context!{}).unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::UndefinedOperation,
            "division by zero is undefined".to_string(),
            2,
            16
        ));
    }

    #[test]
    fn simple_add() {
        let val = interpret("1 + 2 + 3 + 4.8", &create_context!{}).unwrap();
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

    #[test]
    fn bad_pow() {
        let err = interpret("4 + (1 - 2)^0.5", &create_context!{}).unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::UndefinedOperation,
            "(-1) ^ (0.5) is undefined".to_string(),
            4,
            15
        ));
    }

    #[test]
    fn eval_to_infinity() {
        let err = interpret("3 + (9 + 1)^999", &create_context!{}).unwrap_err();
        assert_eq!(err, Error::new(
            ErrorType::Overflow,
            "(10) ^ (999) overflowed f64".to_string(),
            4,
            15
        ));
    }
}