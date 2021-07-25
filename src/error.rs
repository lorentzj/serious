/// Categories for the errors from the parser or interpreter.
#[derive(Debug, PartialEq)]
pub enum ErrorType {
    /// Returned by [`parse`](crate::parser::parse)/[`interpret`](crate::interpreter::interpret) at unexpected tokens or unmatched parentheses.
    BadParse,
    /// Returned by [`interpret`](crate::interpreter::interpret)/[`interpret_tree`](crate::interpreter::interpret_tree) if an [`Identifier`](crate::parser::ExpressionData::Identifier) is not bound in the [`Context`](crate::interpreter::Context).
    UnboundIdentifier,
    /// Returned by [`interpret`](crate::interpreter::interpret)/[`interpret_tree`](crate::interpreter::interpret_tree) if an [`Operation`](crate::parser::Operation) returns NaN or a division by 0 is attempted.
    UndefinedOperation,
    /// Returned by [`parse`](crate::parser::parse)/[`interpret`](crate::interpreter::interpret) if a literal constant is too large to fit in an [`f64`] or by [`interpret`](crate::interpreter::interpret)/[`interpret_tree`](crate::interpreter::interpret_tree) if an operation returns an infinity.
    Overflow
}

/// Defines the type for expressions that fail to parse or evaluate.
/// The `start` and `end` of the `Error` is the text span of the expression where the parser or interpreter failed.
///
/// ```
/// use serious::{create_context, interpreter::interpret, error::Error, error::ErrorType};
///
/// let err = interpret("6 + 4.3/(25 - 5^2)", &create_context!{}).unwrap_err();
///
/// assert_eq!(err, Error::new(
///     ErrorType::UndefinedOperation,
///     "4.3/0 is undefined".to_string(),
///     4,
///     18
/// ));
/// ```
#[derive(Debug, PartialEq)]
pub struct Error {
    pub error_type: ErrorType,
    pub message: String,
    pub start: usize,
    pub end: usize
}

impl Error {
    pub fn new(error_type: ErrorType, message: String, start: usize, end: usize) -> Error {
        Error { error_type, message, start, end }
    }
}