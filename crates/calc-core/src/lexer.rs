use crate::{CalcError, CalcErrorKind, Span, StringInterner, Token, TokenKind};

pub fn lex(source: &str, interner: &mut StringInterner) -> Result<Vec<Token>, CalcError> {
    let mut lexer = Lexer {
        source,
        interner,
        index: 0,
    };
    lexer.lex()
}

struct Lexer<'a, 'interner> {
    source: &'a str,
    interner: &'interner mut StringInterner,
    index: usize,
}

impl Lexer<'_, '_> {
    fn lex(&mut self) -> Result<Vec<Token>, CalcError> {
        let mut tokens = Vec::new();

        while let Some((start, ch)) = self.current_char() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance_char();
                }
                '0'..='9' | '.' => tokens.push(self.number()?),
                'a'..='z' | 'A'..='Z' | '_' => tokens.push(self.ident()),
                '+' => tokens.push(self.single(TokenKind::Plus)),
                '-' => tokens.push(self.single(TokenKind::Minus)),
                '*' => tokens.push(self.single(TokenKind::Star)),
                '/' => tokens.push(self.single(TokenKind::Slash)),
                '=' => tokens.push(self.single(TokenKind::Equal)),
                '(' => tokens.push(self.single(TokenKind::LeftParen)),
                ')' => tokens.push(self.single(TokenKind::RightParen)),
                _ => {
                    let end = start + ch.len_utf8();
                    return Err(CalcError::new(
                        CalcErrorKind::UnexpectedCharacter,
                        Span::new(start, end),
                    ));
                }
            }
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.index, self.index),
        ));
        Ok(tokens)
    }

    fn number(&mut self) -> Result<Token, CalcError> {
        let start = self.index;
        let mut seen_dot = false;

        while let Some((_, ch)) = self.current_char() {
            match ch {
                '0'..='9' => {
                    self.advance_char();
                }
                '.' if !seen_dot => {
                    seen_dot = true;
                    self.advance_char();
                }
                _ => break,
            }
        }

        let span = Span::new(start, self.index);
        let text = span.source(self.source);
        match text.parse::<f64>() {
            Ok(value) => Ok(Token::new(TokenKind::Number(value), span)),
            Err(_) => Err(CalcError::new(CalcErrorKind::InvalidNumber, span)),
        }
    }

    fn ident(&mut self) -> Token {
        let start = self.index;

        while let Some((_, ch)) = self.current_char() {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                    self.advance_char();
                }
                _ => break,
            }
        }

        let span = Span::new(start, self.index);
        Token::new(
            TokenKind::Ident(self.interner.intern(span.source(self.source))),
            span,
        )
    }

    fn single(&mut self, kind: TokenKind) -> Token {
        let start = self.index;
        let (_, ch) = self
            .current_char()
            .expect("single token requires a character");
        self.advance_char();
        Token::new(kind, Span::new(start, start + ch.len_utf8()))
    }

    fn current_char(&self) -> Option<(usize, char)> {
        self.source[self.index..]
            .char_indices()
            .next()
            .map(|(offset, ch)| (self.index + offset, ch))
    }

    fn advance_char(&mut self) {
        if let Some((_, ch)) = self.current_char() {
            self.index += ch.len_utf8();
        }
    }
}
