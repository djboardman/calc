use crate::{Span, Symbol};

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Number(f64),
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
