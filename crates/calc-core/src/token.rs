use crate::{Span, Symbol};

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Number(f64),
    Currency(Symbol),
    Money { currency: Symbol, minor_units: i64 },
    Text(Symbol),
    Boolean(bool),
    Ident(Symbol),
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    Colon,
    Dot,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Comma,
    Eof,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
