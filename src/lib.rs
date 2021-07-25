//! Serious is a simple language to evaluate concise mathematical expressions.
//!
//! - The numerical type is [f64](https://doc.rust-lang.org/std/primitive.f64.html) (infinities and NaNs raise errors).
//! - Variables are identified by characters within `[A-Za-z]`.
//! - Multiplication is implicit if an operator is omitted, unless the RHS is a constant.
//!     - `3x` means `3*x`.
//!     - `x3` is a parse error.
//! - All operations are infix binary, except for the unary minus.
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
//! use serious::{create_context, interpret};
//!
//! // create a context of bound variables
//! let (x, y) = (12.34, 9999.);
//! let context = create_context!{'x' => x, 'y' => y};
//!
//! // evaluate an expression, given the context
//! let result = interpret("3(-2x + 1)^2(x - 1)", &context).unwrap();
//! assert_eq!(result, 3.*(-2.*x + 1.).powf(2.)*(x - 1.));
//! ```

/// Defines the type for expressions that fail to evaluate.
pub mod error;

/// Converts input text into tokens for parsing.
mod lexer;

/// Converts tokens into an abstract syntax tree.
pub mod parser;

/// Evaluates an abstract syntax, given a context of bound identifiers.
pub mod interpreter;

pub use parser::parse;

pub use interpreter::Context;
pub use interpreter::interpret;
pub use interpreter::interpret_tree;

pub use error::Error;
pub use error::ErrorType;