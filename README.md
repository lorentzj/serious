# Serious
A simple language for mathematical expressions.

 - The numerical type is `f64` (infinities and NaNs yield errors).
 - Variables are identified by characters within `[A-Za-z]`.
 - Multiplication is implicit where an operator is omitted.
 - All operations are infix binary, except for the unary minus.
 - Operations are left-associative unless overridden by parentheses or precedence rules.

 ## Full Documentation:
 https://lorentzj.github.io/serious/doc/serious/

 ## Example Usage:
 ```rust
 use serious::{create_context, interpreter::interpret};

 // create a context of bound variables
 let (x, y) = (12.34, 9999.);
 let context = create_context!{'x' => x, 'y' => y};

 // evaluate an expression, given the context
 let result = interpret("34.2x + y^2(-2x^3 + 1)/5.2", &context).unwrap();
 assert_eq!(result, 34.2*x + y.powf(2.)*(-2.*x.powf(3.) + 1.)/5.2);
 ```