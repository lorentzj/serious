#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: usize
}

impl ParseError {
    pub fn new(message: String, position: usize) -> ParseError {
        ParseError { message, position }
    }
}