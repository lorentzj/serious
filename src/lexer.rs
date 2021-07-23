use std::iter::FromIterator;
use super::error::Error;
use super::error::ErrorType;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiate
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    OpenParen,
    CloseParen,
    Op(Operation),
    Constant(f64),
    Identifier(char),
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize
}

impl Token {
    fn new(token_type: TokenType, start: usize, end: usize) -> Token {
        Token { token_type, start, end }
    }
}

#[derive(Debug)]
struct LexerState {
    index: usize,
    curr_number: Vec<char>,
    tokens: Vec<Token>
}

type IntermediateLexerState = Result<LexerState, Error>;

impl LexerState {
    fn new() -> LexerState {
        LexerState {
            index: 0,
            curr_number: vec![],
            tokens: Vec::new()
        }
    }

    fn parse_current_number(mut self) -> IntermediateLexerState {
        if !self.curr_number.is_empty() {
            let n_len = self.curr_number.len();
            let parsed_number = String::from_iter(self.curr_number).parse::<f64>();
            match parsed_number {
                Ok(n)  => {
                    if n == f64::INFINITY {
                        return Err(Error::new(
                            ErrorType::BadParse,
                            "number too large to fit in f64".to_string(),
                            self.index - n_len,
                            self.index
                        ))
                    }
                    self.tokens.push(Token::new(TokenType::Constant(n), self.index - n_len, self.index));
                }
                Err(msg) => return Err(Error::new(
                    ErrorType::BadParse,
                    msg.to_string(),
                    self.index - n_len,
                    self.index
                ))
            }
            self.curr_number = vec![];
        }

        Ok(self)
    }

    fn finalize(state: IntermediateLexerState) -> Result<Vec<Token>, Error> {
        let state = state?.parse_current_number()?;
        Ok(state.tokens)
    }
}

fn consume_char(state: IntermediateLexerState, (i, next): (usize, char)) -> IntermediateLexerState {
    let mut state = state?;
    match next {
        '0'..='9' | '.' => {
            state.curr_number.push(next);
        },
        _ => {
            state = state.parse_current_number()?;
            match next {
                '('       => state.tokens.push(Token::new(TokenType::OpenParen, i, i + 1)),
                ')'       => state.tokens.push(Token::new(TokenType::CloseParen, i, i + 1)),
                '+'       => state.tokens.push(Token::new(TokenType::Op(Operation::Add), i, i + 1)),
                '-'       => state.tokens.push(Token::new(TokenType::Op(Operation::Subtract), i, i + 1)),
                '*'       => state.tokens.push(Token::new(TokenType::Op(Operation::Multiply), i, i + 1)),
                '/'       => state.tokens.push(Token::new(TokenType::Op(Operation::Divide), i, i + 1)),
                '^'       => state.tokens.push(Token::new(TokenType::Op(Operation::Exponentiate), i, i + 1)),
                'A'..='z' => state.tokens.push(Token::new(TokenType::Identifier(next), i, i + 1)),
                ' '       => (),
                _         => return Err(Error::new(
                    ErrorType::BadParse,
                    format!("invalid character '{}'", next),
                    i,
                    i + 1
                ))
            }
        }
    }
    state.index += 1;
    Ok(state)
}

pub fn lex(text: &str) -> Result<Vec<Token>, Error> {
    if text.is_empty() {
        return Err(Error::new(
            ErrorType::BadParse,
            "expected token".to_string(),
            0,
            1
        ))
    }
    let chars = text.chars().enumerate();
    let state = chars.fold(Ok(LexerState::new()), consume_char);
    LexerState::finalize(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let err = lex("").unwrap_err();
        assert_eq!(err.message, "expected token");
        assert_eq!(err.start, 0);
        assert_eq!(err.end, 1);
    }

    #[test]
    fn too_large() {
        let mut too_big = f64::MAX.to_string();
        too_big.push_str("0");

        let err = lex(too_big.as_str()).unwrap_err();
        assert_eq!(err.message, "number too large to fit in f64");
        assert_eq!(err.start, 0);
        assert_eq!(err.end, 310);
    }

    #[test]
    fn invalid_float_extra_decimal() {
        let err = lex("0.2.3").unwrap_err();
        assert_eq!(err.message, "invalid float literal");
        assert_eq!(err.start, 0);
        assert_eq!(err.end, 5);
    }

    #[test]
    fn invalid_float_only_decimal() {
        let err = lex("abc.").unwrap_err();
        assert_eq!(err.message, "invalid float literal");
        assert_eq!(err.start, 3);
        assert_eq!(err.end, 4);
    }

    #[test]
    fn simple_mult() {
        let tokens = lex("4*0.23").unwrap();
        assert_eq!(tokens, vec![
            Token::new(TokenType::Constant(4.), 0, 1),
            Token::new(TokenType::Op(Operation::Multiply), 1, 2),
            Token::new(TokenType::Constant(0.23), 2, 6)
        ]);
    }

    #[test]
    fn simple_add() {
        let tokens = lex("0+45").unwrap();
        assert_eq!(tokens, vec![
            Token::new(TokenType::Constant(0.), 0, 1),
            Token::new(TokenType::Op(Operation::Add), 1, 2),
            Token::new(TokenType::Constant(45.), 2, 4)
        ]);
    }

    #[test]
    fn with_spaces() {
        let tokens = lex("5+ 4 * 3     * 9").unwrap();
        assert_eq!(tokens, vec![
            Token::new(TokenType::Constant(5.), 0, 1),
            Token::new(TokenType::Op(Operation::Add), 1, 2),
            Token::new(TokenType::Constant(4.), 3, 4),
            Token::new(TokenType::Op(Operation::Multiply), 5, 6),
            Token::new(TokenType::Constant(3.), 7, 8),
            Token::new(TokenType::Op(Operation::Multiply), 13, 14),
            Token::new(TokenType::Constant(9.), 15, 16)
        ]);
    }

    #[test]
    fn parens() {
        let tokens = lex("0+(7*5)+(6*(7+8+90))").unwrap();
        assert_eq!(tokens, vec![
            Token::new(TokenType::Constant(0.), 0, 1),
            Token::new(TokenType::Op(Operation::Add), 1, 2),
            Token::new(TokenType::OpenParen, 2, 3),
            Token::new(TokenType::Constant(7.), 3, 4),
            Token::new(TokenType::Op(Operation::Multiply), 4, 5),
            Token::new(TokenType::Constant(5.), 5, 6),
            Token::new(TokenType::CloseParen, 6, 7),
            Token::new(TokenType::Op(Operation::Add), 7, 8),
            Token::new(TokenType::OpenParen, 8, 9),
            Token::new(TokenType::Constant(6.), 9, 10),
            Token::new(TokenType::Op(Operation::Multiply), 10, 11),
            Token::new(TokenType::OpenParen, 11, 12),
            Token::new(TokenType::Constant(7.), 12, 13),
            Token::new(TokenType::Op(Operation::Add), 13, 14),
            Token::new(TokenType::Constant(8.), 14, 15),
            Token::new(TokenType::Op(Operation::Add), 15, 16),
            Token::new(TokenType::Constant(90.), 16, 18),
            Token::new(TokenType::CloseParen, 18, 19),
            Token::new(TokenType::CloseParen, 19, 20)
        ]);
    }

    #[test]
    fn identifier() {
        let tokens = lex("8y(4X + 7.3)").unwrap();
        assert_eq!(tokens, vec![
            Token::new(TokenType::Constant(8.), 0, 1),
            Token::new(TokenType::Identifier('y'), 1, 2),
            Token::new(TokenType::OpenParen, 2, 3),
            Token::new(TokenType::Constant(4.), 3, 4),
            Token::new(TokenType::Identifier('X'), 4, 5),
            Token::new(TokenType::Op(Operation::Add), 6, 7),
            Token::new(TokenType::Constant(7.3), 8, 11),
            Token::new(TokenType::CloseParen, 11, 12)
        ]);
    }
}