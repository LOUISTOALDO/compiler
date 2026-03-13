//! Parser - Recursive descent parser building AST from tokens.

use crate::ast::*;
use crate::lexer::Token;
use anyhow::{anyhow, Result};
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut functions = Vec::new();
        while self.peek().is_some() {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function> {
        self.expect(Token::Int)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        loop {
            if self.matches(Token::RParen) {
                break;
            }
            if !params.is_empty() {
                self.expect(Token::Comma)?;
            }
            self.expect(Token::Int)?;
            params.push(self.expect_ident()?);
        }
        self.expect(Token::LBrace)?;
        let mut body = Vec::new();
        while !self.matches(Token::RBrace) {
            body.push(self.parse_statement()?);
        }
        Ok(Function { name, params, body })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        // int x; or int x = expr;
        if self.matches(Token::Int) {
            let name = self.expect_ident()?;
            if self.matches(Token::Eq) {
                let init = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                return Ok(Statement::DeclInit {
                    name,
                    init: Box::new(init),
                });
            }
            self.expect(Token::Semicolon)?;
            return Ok(Statement::Decl { name });
        }

        if self.matches(Token::Return) {
            let expr = if self.matches(Token::Semicolon) {
                None
            } else {
                let e = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Some(e)
            };
            return Ok(Statement::Return(expr));
        }

        if self.matches(Token::If) {
            self.expect(Token::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(Token::RParen)?;
            self.expect(Token::LBrace)?;
            let mut then_body = Vec::new();
            while !self.matches(Token::RBrace) {
                then_body.push(self.parse_statement()?);
            }
            let else_body = if self.matches(Token::Else) {
                self.expect(Token::LBrace)?;
                let mut body = Vec::new();
                while !self.matches(Token::RBrace) {
                    body.push(self.parse_statement()?);
                }
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

        if self.matches(Token::While) {
            self.expect(Token::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(Token::RParen)?;
            self.expect(Token::LBrace)?;
            let mut body = Vec::new();
            while !self.matches(Token::RBrace) {
                body.push(self.parse_statement()?);
            }
            return Ok(Statement::While { cond, body });
        }

        if self.matches(Token::For) {
            self.expect(Token::LParen)?;
            let init = if self.matches(Token::Semicolon) {
                None
            } else if self.peek() == Some(&Token::Int) {
                Some(Box::new(self.parse_statement()?))
            } else {
                let e = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Some(Box::new(Statement::Expr(e)))
            };
            let cond = if self.peek() == Some(&Token::Semicolon) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect(Token::Semicolon)?;
            let step = if self.peek() == Some(&Token::RParen) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect(Token::RParen)?;
            self.expect(Token::LBrace)?;
            let mut body = Vec::new();
            while !self.matches(Token::RBrace) {
                body.push(self.parse_statement()?);
            }
            return Ok(Statement::For {
                init,
                cond,
                step,
                body,
            });
        }

        // Expression statement
        let expr = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        Ok(Statement::Expr(expr))
    }

    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    /// || (lowest precedence)
    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.matches(Token::OrOr) {
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
        let mut left = self.parse_equality()?;
        while self.matches(Token::AndAnd) {
            let right = self.parse_equality()?;
            left = Expr::BinOp {
                op: BinOp::AndAnd,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_relational()?;
        loop {
            let op = if self.matches(Token::EqEq) {
                BinOp::Eq
            } else if self.matches(Token::Ne) {
                BinOp::Ne
            } else {
                break;
            };
            let right = self.parse_relational()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_relational(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive()?;
        loop {
            let op = if self.matches(Token::Lt) {
                BinOp::Lt
            } else if self.matches(Token::Le) {
                BinOp::Le
            } else if self.matches(Token::Gt) {
                BinOp::Gt
            } else if self.matches(Token::Ge) {
                BinOp::Ge
            } else {
                break;
            };
            let right = self.parse_additive()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = if self.matches(Token::Plus) {
                BinOp::Add
            } else if self.matches(Token::Minus) {
                BinOp::Sub
            } else {
                break;
            };
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = if self.matches(Token::Star) {
                BinOp::Mul
            } else if self.matches(Token::Slash) {
                BinOp::Div
            } else if self.matches(Token::Percent) {
                BinOp::Mod
            } else {
                break;
            };
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
        if self.matches(Token::Minus) {
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            });
        }
        if self.matches(Token::Bang) {
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        if let Some(Token::Number(n)) = self.next_if(|t| matches!(t, Token::Number(_))) {
            return Ok(Expr::Number(n));
        }

        if let Some(Token::Ident(name)) = self.next_if(|t| matches!(t, Token::Ident(_))) {
            if self.matches(Token::LParen) {
                // Function call
                let mut args = Vec::new();
                loop {
                    if self.matches(Token::RParen) {
                        break;
                    }
                    if !args.is_empty() {
                        self.expect(Token::Comma)?;
                    }
                    args.push(self.parse_expr()?);
                }
                return Ok(Expr::Call { name, args });
            }
            if self.matches(Token::Eq) {
                let value = self.parse_expr()?;
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Ok(Expr::Ident(name));
        }

        if self.matches(Token::LParen) {
            let expr = self.parse_expr()?;
            self.expect(Token::RParen)?;
            return Ok(expr);
        }

        Err(anyhow!("Unexpected token: {:?}", self.peek()))
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn matches(&mut self, expected: Token) -> bool {
        if self.peek() == Some(&expected) {
            self.next();
            true
        } else {
            false
        }
    }

    fn next_if(&mut self, pred: impl FnOnce(&Token) -> bool) -> Option<Token> {
        if self.peek().map_or(false, pred) {
            self.next()
        } else {
            None
        }
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        match self.next() {
            Some(t) if t == expected => Ok(()),
            Some(t) => Err(anyhow!("Expected {:?}, got {:?}", expected, t)),
            None => Err(anyhow!("Expected {:?}, got EOF", expected)),
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        match self.next() {
            Some(Token::Ident(s)) => Ok(s),
            Some(t) => Err(anyhow!("Expected identifier, got {:?}", t)),
            None => Err(anyhow!("Expected identifier, got EOF")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn test_parse_number() {
        let tokens = lex("5").unwrap();
        let mut p = Parser::new(tokens);
        let expr = p.parse_expr().unwrap();
        assert!(matches!(expr, Expr::Number(5)));
    }

    #[test]
    fn test_parse_call_empty() {
        let tokens = lex("fib()").unwrap();
        let mut p = Parser::new(tokens);
        let expr = p.parse_expr().unwrap();
        match &expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "fib");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Call, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_call() {
        let tokens = lex("int main() { return fib(5); }").unwrap();
        let mut p = Parser::new(tokens);
        let program = p.parse().unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
    }

    #[test]
    fn test_parse_call_from_file() {
        let source = std::fs::read_to_string("examples/just_call.c").unwrap();
        let tokens = lex(&source).unwrap();
        let mut p = Parser::new(tokens);
        let program = p.parse().unwrap();
        assert_eq!(program.functions.len(), 1);
    }
}
