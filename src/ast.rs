//! Abstract Syntax Tree for C subset.

use std::fmt;

/// Root of the program - list of function definitions.
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    /// Variable declaration: int x;
    Decl { name: String },
    /// Declaration with init: int x = 5;
    DeclInit { name: String, init: Box<Expr> },
    /// Expression statement: x = 5;
    Expr(Expr),
    /// return expr;
    Return(Option<Expr>),
    /// if (cond) { ... } else { ... }
    If {
        cond: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    /// while (cond) { ... }
    While { cond: Expr, body: Vec<Statement> },
    /// for (init; cond; step) { ... }
    For {
        init: Option<Box<Statement>>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    Ident(String),
    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation
    UnaryOp { op: UnaryOp, operand: Box<Expr> },
    /// Assignment: x = expr
    Assign { name: String, value: Box<Expr> },
    /// Function call: f(a, b)
    Call { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Le => write!(f, "<="),
            BinOp::Gt => write!(f, ">"),
            BinOp::Ge => write!(f, ">="),
            BinOp::AndAnd => write!(f, "&&"),
            BinOp::OrOr => write!(f, "||"),
        }
    }
}
