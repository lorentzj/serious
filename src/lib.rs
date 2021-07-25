//! Serious is a simple language to evaluate concise mathematical expressions.
//!
//! - The numerical type is [`f64`] (infinities and NaNs raise errors).
//! - Variables are identified by characters within `[A-Za-z]`.
//! - Multiplication is implicit if an operator is omitted, unless the RHS is a constant.
//! - All operations are infix binary, except for the unary minus.
//! - Operations are left-associative unless overridden by parentheses or precedence rules:
//!
//! | Operator | Meaning                                                | Precedence
//! | -------- | ------------------------------------------------------ | ----------
//! | `^`      | [Exponentiate](crate::parser::Operation::Exponentiate) | 2
//! | `*`      | [Multiply](crate::parser::Operation::Multiply)         | 1
//! | `/`      | [Divide](crate::parser::Operation::Divide)             | 1
//! | `+`      | [Add](crate::parser::Operation::Add)                   | 0
//! | `-`      | [Subtract](crate::parser::Operation::Subtract)         | 0
//!
//! # Example Usage:
//! ```
//! use serious::{create_context, interpreter::interpret};
//!
//! // create a context of bound variables
//! let (x, y) = (12.34, 9999.);
//! let context = create_context!{'x' => x, 'y' => y};
//!
//! // evaluate an expression, given the context
//! let result = interpret("y^2(-2x^3 + 1)/5.2", &context).unwrap();
//! assert_eq!(result, y.powf(2.)*(-2.*x.powf(3.) + 1.)/5.2);
//! ```

/// Converts input text into tokens for parsing (used in [parser](crate::parser)).
mod lexer;

/// Converts text into an [`Expression`](crate::parser::Expression) (an abstract syntax tree).
pub mod parser;

/// Evaluates an [`Expression`](crate::parser::Expression), given a [`Context`](crate::interpreter::Context) of bound identifiers.
pub mod interpreter;

/// Defines the type for failed [`Result`]s of [`parse`](crate::parser::parse), [`interpret`](crate::interpreter::interpret), and [`interpret_tree`](crate::interpreter::interpret_tree).
pub mod error;