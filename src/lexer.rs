//! Lexer - Tokenizes C source code into a stream of tokens.

use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip r"//[^\n]*")]
// Block comments: /* ... */ (greedy - use // for single-line)
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
pub enum Token {
    // Keywords
    #[token("int")]
    Int,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,

    // Literals
    #[regex(r"\d+", |lex| lex.slice().parse().ok())]
    Number(i32),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[token("==")]
    EqEq,
    #[token("!=")]
    Ne,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,

    #[token("=")]
    Eq,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Ident(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Lexes source code into tokens. Stops on first error.
pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    while let Some(tok) = lexer.next() {
        match tok {
            Ok(t) => tokens.push(t),
            Err(()) => return Err(format!("Invalid token at position {}", lexer.span().start)),
        }
    }
    Ok(tokens)
}

/// Lexer iterator for LALRPOP: yields `(start, Token, end)` for each token.
/// Yields `None` at EOF. Errors are represented as invalid tokens (lex returns Err).
pub struct LalrpopLexer<'input> {
    lexer: logos::Lexer<'input, Token>,
}

impl<'input> LalrpopLexer<'input> {
    pub fn new(source: &'input str) -> Self {
        Self {
            lexer: Token::lexer(source),
        }
    }
}

#[derive(Debug)]
pub struct LexicalError;

impl Iterator for LalrpopLexer<'_> {
    type Item = Result<(usize, Token, usize), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next() {
            Some(Ok(tok)) => {
                let span = self.lexer.span();
                Some(Ok((span.start, tok, span.end)))
            }
            Some(Err(_)) => Some(Err(LexicalError)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_numbers() {
        let tokens = lex("42 0 123").unwrap();
        assert_eq!(tokens, vec![Token::Number(42), Token::Number(0), Token::Number(123)]);
    }

    #[test]
    fn test_lex_fib5() {
        let tokens = lex("fib(5)").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], Token::Ident(_)));
        assert!(matches!(tokens[1], Token::LParen));
        assert!(matches!(tokens[2], Token::Number(5)));
        assert!(matches!(tokens[3], Token::RParen));
    }

    #[test]
    fn test_lex_keywords() {
        let tokens = lex("int return if else while").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Int, Token::Return, Token::If, Token::Else, Token::While]
        );
    }
}
