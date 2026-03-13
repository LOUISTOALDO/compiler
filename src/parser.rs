//! Recursive descent parser for C subset.
//! Uses lexer tokens - avoids LALR ambiguity with assignment in expressions.

use crate::ast::*;
use crate::lexer::{lex, Token};
use anyhow::{anyhow, Result};
use std::iter::Peekable;
use std::vec::IntoIter;

type TokenStream = Peekable<IntoIter<Token>>;

pub fn parse(source: &str) -> Result<Program> {
    let tokens = lex(source).map_err(|e| anyhow!("Lex error: {}", e))?;
    let mut p = Parser::new(tokens);
    p.parse_program()
}

struct Parser {
    tokens: TokenStream,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn parse_program(&mut self) -> Result<Program> {
        let mut functions = Vec::new();
        while self.peek().is_some() {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function> {
        self.expect_token(Token::Int)?;
        let name = self.expect_ident()?;
        self.expect_token(Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect_token(Token::RParen)?;
        self.expect_token(Token::LBrace)?;
        let mut body = Vec::new();
        while !self.is_token(Token::RBrace) {
            body.push(self.parse_statement()?);
        }
        self.expect_token(Token::RBrace)?;
        Ok(Function { name, params, body })
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>> {
        let mut params = Vec::new();
        if self.is_token(Token::Int) {
            self.next();
            params.push(self.expect_ident()?);
            while self.is_token(Token::Comma) {
                self.next();
                self.expect_token(Token::Int)?;
                params.push(self.expect_ident()?);
            }
        }
        Ok(params)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        if self.is_token(Token::Int) {
            self.next();
            let name = self.expect_ident()?;
            if self.is_token(Token::Eq) {
                self.next();
                let init = self.parse_expr()?;
                self.expect_token(Token::Semicolon)?;
                return Ok(Statement::DeclInit {
                    name,
                    init: Box::new(init),
                });
            }
            self.expect_token(Token::Semicolon)?;
            return Ok(Statement::Decl { name });
        }
        if self.is_token(Token::Return) {
            self.next();
            if self.is_token(Token::Semicolon) {
                self.next();
                return Ok(Statement::Return(None));
            }
            let expr = self.parse_expr()?;
            self.expect_token(Token::Semicolon)?;
            return Ok(Statement::Return(Some(expr)));
        }
        if self.is_token(Token::If) {
            self.next();
            self.expect_token(Token::LParen)?;
            let cond = self.parse_expr()?;
            self.expect_token(Token::RParen)?;
            self.expect_token(Token::LBrace)?;
            let mut then_body = Vec::new();
            while !self.is_token(Token::RBrace) {
                then_body.push(self.parse_statement()?);
            }
            self.expect_token(Token::RBrace)?;
            let else_body = if self.is_token(Token::Else) {
                self.next();
                self.expect_token(Token::LBrace)?;
                let mut body = Vec::new();
                while !self.is_token(Token::RBrace) {
                    body.push(self.parse_statement()?);
                }
                self.expect_token(Token::RBrace)?;
                Some(body)
            } else {
                None
            };
            return Ok(Statement::If {
                cond,
                then_body,
                else_body,
            });
        }
        if self.is_token(Token::While) {
            self.next();
            self.expect_token(Token::LParen)?;
            let cond = self.parse_expr()?;
            self.expect_token(Token::RParen)?;
            self.expect_token(Token::LBrace)?;
            let mut body = Vec::new();
            while !self.is_token(Token::RBrace) {
                body.push(self.parse_statement()?);
            }
            self.expect_token(Token::RBrace)?;
            return Ok(Statement::While { cond, body });
        }
        if self.is_token(Token::For) {
            self.next();
            self.expect_token(Token::LParen)?;
            let init = self.parse_for_init()?;
            let cond = self.parse_for_cond()?;
            let step = self.parse_for_step()?;
            self.expect_token(Token::LBrace)?;
            let mut body = Vec::new();
            while !self.is_token(Token::RBrace) {
                body.push(self.parse_statement()?);
            }
            self.expect_token(Token::RBrace)?;
            return Ok(Statement::For {
                init,
                cond,
                step,
                body,
            });
        }
        let expr = self.parse_expr()?;
        self.expect_token(Token::Semicolon)?;
        Ok(Statement::Expr(expr))
    }

    fn parse_for_init(&mut self) -> Result<Option<Box<Statement>>> {
        if self.is_token(Token::Semicolon) {
            self.next();
            return Ok(None);
        }
        if self.is_token(Token::Int) {
            self.next();
            let name = self.expect_ident()?;
            if self.is_token(Token::Eq) {
                self.next();
                let init = self.parse_expr()?;
                self.expect_token(Token::Semicolon)?;
                return Ok(Some(Box::new(Statement::DeclInit {
                    name,
                    init: Box::new(init),
                })));
            }
            self.expect_token(Token::Semicolon)?;
            return Ok(Some(Box::new(Statement::Decl { name })));
        }
        let expr = self.parse_expr()?;
        self.expect_token(Token::Semicolon)?;
        Ok(Some(Box::new(Statement::Expr(expr))))
    }

    fn parse_for_cond(&mut self) -> Result<Option<Expr>> {
        if self.is_token(Token::Semicolon) {
            self.next();
            return Ok(None);
        }
        let expr = self.parse_expr()?;
        self.expect_token(Token::Semicolon)?;
        Ok(Some(expr))
    }

    fn parse_for_step(&mut self) -> Result<Option<Expr>> {
        if self.is_token(Token::RParen) {
            return Ok(None);
        }
        let expr = self.parse_expr()?;
        self.expect_token(Token::RParen)?;
        Ok(Some(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.is_token(Token::OrOr) {
            self.next();
            let right = self.parse_and()?;
            left = Expr::BinOp {
                op: BinOp::OrOr,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_eq()?;
        while self.is_token(Token::AndAnd) {
            self.next();
            let right = self.parse_eq()?;
            left = Expr::BinOp {
                op: BinOp::AndAnd,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_eq(&mut self) -> Result<Expr> {
        let mut left = self.parse_rel()?;
        loop {
            if self.is_token(Token::EqEq) {
                self.next();
                let right = self.parse_rel()?;
                left = Expr::BinOp {
                    op: BinOp::Eq,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.is_token(Token::Ne) {
                self.next();
                let right = self.parse_rel()?;
                left = Expr::BinOp {
                    op: BinOp::Ne,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_rel(&mut self) -> Result<Expr> {
        let mut left = self.parse_add()?;
        loop {
            let op = if self.is_token(Token::Lt) {
                self.next();
                Some(BinOp::Lt)
            } else if self.is_token(Token::Le) {
                self.next();
                Some(BinOp::Le)
            } else if self.is_token(Token::Gt) {
                self.next();
                Some(BinOp::Gt)
            } else if self.is_token(Token::Ge) {
                self.next();
                Some(BinOp::Ge)
            } else {
                None
            };
            if let Some(op) = op {
                let right = self.parse_add()?;
                left = Expr::BinOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Expr> {
        let mut left = self.parse_mul()?;
        while let Some(op) = self.peek().and_then(|t| match t {
            Token::Plus => Some(BinOp::Add),
            Token::Minus => Some(BinOp::Sub),
            _ => None,
        }) {
            self.next();
            let right = self.parse_mul()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        while let Some(op) = self.peek().and_then(|t| match t {
            Token::Star => Some(BinOp::Mul),
            Token::Slash => Some(BinOp::Div),
            Token::Percent => Some(BinOp::Mod),
            _ => None,
        }) {
            self.next();
            let right = self.parse_unary()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.is_token(Token::Minus) {
            self.next();
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            });
        }
        if self.is_token(Token::Bang) {
            self.next();
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Ident(name)) => {
                if self.is_token(Token::LParen) {
                    self.next();
                    let args = self.parse_arg_list()?;
                    self.expect_token(Token::RParen)?;
                    Ok(Expr::Call { name, args })
                } else if self.is_token(Token::Eq) {
                    self.next();
                    let value = self.parse_expr()?;
                    Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect_token(Token::RParen)?;
                Ok(expr)
            }
            other => Err(anyhow!(
                "Unexpected token {:?}, expected expression",
                other
            )),
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        if !self.is_token(Token::RParen) {
            args.push(self.parse_expr()?);
            while self.is_token(Token::Comma) {
                self.next();
                args.push(self.parse_expr()?);
            }
        }
        Ok(args)
    }

    fn is_token(&mut self, t: Token) -> bool {
        match (&t, self.peek()) {
            (Token::Int, Some(Token::Int)) => true,
            (Token::Return, Some(Token::Return)) => true,
            (Token::If, Some(Token::If)) => true,
            (Token::Else, Some(Token::Else)) => true,
            (Token::While, Some(Token::While)) => true,
            (Token::For, Some(Token::For)) => true,
            (Token::LParen, Some(Token::LParen)) => true,
            (Token::RParen, Some(Token::RParen)) => true,
            (Token::LBrace, Some(Token::LBrace)) => true,
            (Token::RBrace, Some(Token::RBrace)) => true,
            (Token::Semicolon, Some(Token::Semicolon)) => true,
            (Token::Comma, Some(Token::Comma)) => true,
            (Token::Eq, Some(Token::Eq)) => true,
            (Token::EqEq, Some(Token::EqEq)) => true,
            (Token::Ne, Some(Token::Ne)) => true,
            (Token::Lt, Some(Token::Lt)) => true,
            (Token::Le, Some(Token::Le)) => true,
            (Token::Gt, Some(Token::Gt)) => true,
            (Token::Ge, Some(Token::Ge)) => true,
            (Token::Plus, Some(Token::Plus)) => true,
            (Token::Minus, Some(Token::Minus)) => true,
            (Token::Star, Some(Token::Star)) => true,
            (Token::Slash, Some(Token::Slash)) => true,
            (Token::Percent, Some(Token::Percent)) => true,
            (Token::AndAnd, Some(Token::AndAnd)) => true,
            (Token::OrOr, Some(Token::OrOr)) => true,
            (Token::Bang, Some(Token::Bang)) => true,
            _ => false,
        }
    }

    fn expect_token(&mut self, want: Token) -> Result<()> {
        let got = self.next();
        match (&want, &got) {
            (Token::Int, Some(Token::Int))
            | (Token::Return, Some(Token::Return))
            | (Token::If, Some(Token::If))
            | (Token::Else, Some(Token::Else))
            | (Token::While, Some(Token::While))
            | (Token::For, Some(Token::For))
            | (Token::LParen, Some(Token::LParen))
            | (Token::RParen, Some(Token::RParen))
            | (Token::LBrace, Some(Token::LBrace))
            | (Token::RBrace, Some(Token::RBrace))
            | (Token::Semicolon, Some(Token::Semicolon))
            | (Token::Comma, Some(Token::Comma))
            | (Token::Eq, Some(Token::Eq))
            | (Token::EqEq, Some(Token::EqEq))
            | (Token::Ne, Some(Token::Ne))
            | (Token::Lt, Some(Token::Lt))
            | (Token::Le, Some(Token::Le))
            | (Token::Gt, Some(Token::Gt))
            | (Token::Ge, Some(Token::Ge))
            | (Token::Plus, Some(Token::Plus))
            | (Token::Minus, Some(Token::Minus))
            | (Token::Star, Some(Token::Star))
            | (Token::Slash, Some(Token::Slash))
            | (Token::Percent, Some(Token::Percent))
            | (Token::AndAnd, Some(Token::AndAnd))
            | (Token::OrOr, Some(Token::OrOr))
            | (Token::Bang, Some(Token::Bang)) => Ok(()),
            _ => Err(anyhow!("Expected {:?}, got {:?}", want, got)),
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        match self.next() {
            Some(Token::Ident(s)) => Ok(s),
            other => Err(anyhow!("Expected identifier, got {:?}", other)),
        }
    }
}
