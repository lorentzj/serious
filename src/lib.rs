//! Serious is a simple language to evaluate concise mathematical expressions.
//!
//! - The numerical type is f64 (infinities and NaNs raise errors).
//! - Variables are identified by characters within `[A-Za-z]`.
//! - Multiplication is implicit if an operator is omitted, unless the RHS is a constant.
//!     - `3x` means `3*x`.
//!     - `x3` is a parse error.
//! - All operations are infix binary, except for the unary minus.
//!     - `-x + 2` means `0 - x + 2`.
//!     - `2 + -x` is a parse error.
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
//! let result = interpret("x + y + 3.98", &context).unwrap();
//! assert_eq!(result, x + y + 3.98);
//!
//! let result = interpret("3(-2x + 1)(x - 1)", &context).unwrap();
//! assert_eq!(result, 3.*(-2.*x + 1.)*(x - 1.));
//!
//! let result = interpret("4x^3 - 12x^(0.3x - y/(50 - x))", &context).unwrap();
//! assert_eq!(result, 4.*x.powf(3.) - 12.*x.powf(0.3*x - y/(50. - x)));
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