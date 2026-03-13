//! C Compiler - Lexer, Parser, and x86-64 Code Generator
//!
//! Supports a subset of C: int, variables, arithmetic, if/while, functions.

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;

pub use ast::*;
pub use codegen::*;
pub use lexer::*;
pub use parser::*;
