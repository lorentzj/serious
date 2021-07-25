#[derive(Debug, PartialEq)]
pub enum ErrorType {
    BadParse,
    UnboundIdentifier,
    UndefinedOperation,
    Overflow
}

#[derive(Debug)]
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