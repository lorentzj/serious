//! Serious is a simple language to evaluate concise mathematical expressions.
//! 
//! - The numerical type is f64.
//! - 1-char identifiers are bound when evaluating an expression.
//! - Multiplication is implicit if an operator is omitted, unless the RHS is a constant (e.g. `3x`, not `x3`).
//! - All operations are infix binary, except for the unary minus (`-x` is interpreted as `0 - x`).
//! - Operations are left-associative unless overridden by parentheses or precedence rules:
//!
//! | Operator | Meaning      | Precedence
//! | -------- | ------------ | ----------
//! | `^`      | Exponentiate | 2
//! | `*`      | Multipy      | 1
//! | `/`      | Divide       | 1
//! | `+`      | Add          | 0   
//! | `-`      | Subtract     | 0
//!
//! Example Usage:
//! ```
//! use serious::create_context;
//! use serious::interpret;
//! 
//! // create a context of bound variables
//! let (x, y) = (12.34, 9999.);
//! let context = create_context!{'x' => x, 'y' => y};
//!
//! // evaluate an expression, given a context
//! let result = interpret("x + y + 3.98", &context).unwrap();
//! assert_eq!(result, x + y + 3.98);
//!
//! let result = interpret("3(-2x + 1)(x - 1)", &context).unwrap();
//! assert_eq!(result, 3.*(-2.*x + 1.)*(x - 1.));
//!
//! let result = interpret("4x^(-2) + 12x^(0.3xy) - (56.78/x)^13.2", &context).unwrap();
//! assert_eq!(result, 4.*x.powf(-2.) + 12.*x.powf(0.3*x*y) - (56.78/x).powf(13.2));
//! ```

/// This module defines the type for expressions that fail to evaluate.
pub mod error;

/// This module converts input text into tokens for parsing.
pub mod lexer;

/// This module converts tokens into an abstract syntax tree.
pub mod parser;

/// This module evaluates an abstract syntax tree into a value, given a context of bound identifiers.
pub mod interpreter;

pub use parser::parse;
pub use interpreter::interpret;