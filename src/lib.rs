//! Serious is a simple language for mathematical expressions.
//! 
//! - The numerical type is an f64
//! - PEMDAS operations and unary minus are supported by `()`, `^`, `*`, `/`, `+`, `-`
//! - Multiplication is implicit if an operator is omitted, unless the RHS is a constant.
//!
//! Example Usage:
//! ```
//! use serious::interpreter::interpret;
//! use std::collections::HashMap;
//! 
//! // create a hashmap of bound identifiers (each 1 char)
//! let mut context: HashMap<char, f64> = HashMap::new();
//! let (x, y) = (2., 3.);
//! context.insert('x', x);
//! context.insert('y', y);
//!
//! let result = interpret("x + y + 3", &context).unwrap();
//! assert_eq!(result, x + y + 3.);

//! let result = interpret("3(2x + 1)", &context).unwrap();
//! assert_eq!(result, 3.*(2.*x + 1.));
//! ```

/// This module defines the type for expressions that fail to evaluate.
pub mod error;

/// This module converts input text into tokens for parsing.
pub mod lexer;

/// This module converts tokens into an abstract syntax tree.
pub mod parser;

/// This module evaluates an abstract syntax tree into a value, given a context of bound identifiers.
pub mod interpreter;