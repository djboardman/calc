use crate::{
    BinaryOp, CalcError, CalcErrorKind, Expr, InternalQualifiedName, Span, Statement,
    StringInterner, Token, TokenKind, UnaryOp, lex,
};

pub fn parse(source: &str, interner: &mut StringInterner) -> Result<Statement, CalcError> {
    Parser::new(lex(source, interner)?).parse_statement()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse_statement(&mut self) -> Result<Statement, CalcError> {
        if self.is_section_header() {
            let token = self.advance();
            let TokenKind::Ident(name) = token.kind else {
                unreachable!("section header starts with an identifier");
            };
            self.advance();
            self.expect_eof()?;
            return Ok(Statement::SectionHeader {
                name,
                name_span: token.span,
            });
        }

        if self.is_invalid_section_header() {
            return Err(CalcError::new(
                CalcErrorKind::InvalidSectionHeader,
                self.current_span(),
            ));
        }

        if self.is_assignment() {
            let token = self.advance();
            let TokenKind::Ident(name) = token.kind else {
                unreachable!("assignment starts with an identifier");
            };
            self.advance();
            let value = self.parse_expression()?;
            self.expect_eof()?;
            return Ok(Statement::Assignment {
                name,
                name_span: token.span,
                value,
            });
        }

        let expr = self.parse_expression()?;
        self.expect_eof()?;
        Ok(Statement::Expr(expr))
    }

    fn parse_expression(&mut self) -> Result<Expr, CalcError> {
        self.parse_addition()
    }

    fn parse_addition(&mut self) -> Result<Expr, CalcError> {
        let mut expr = self.parse_multiplication()?;

        loop {
            let op = match self.current().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => break,
            };
            let op_span = self.advance().span;
            let right = self.parse_multiplication()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                op_span,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, CalcError> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.current().kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                _ => break,
            };
            let op_span = self.advance().span;
            let right = self.parse_unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                op_span,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, CalcError> {
        if matches!(self.current().kind, TokenKind::Minus) {
            let op_span = self.advance().span;
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                op_span,
                expr: Box::new(expr),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, CalcError> {
        match &self.current().kind {
            TokenKind::Number(value) => {
                let value = *value;
                let span = self.advance().span;
                Ok(Expr::Number { value, span })
            }
            TokenKind::Currency(currency) => {
                let currency = *currency;
                let span = self.advance().span;
                Ok(Expr::Currency { currency, span })
            }
            TokenKind::Money {
                currency,
                minor_units,
            } => {
                let currency = *currency;
                let minor_units = *minor_units;
                let span = self.advance().span;
                Ok(Expr::Money {
                    currency,
                    minor_units,
                    span,
                })
            }
            TokenKind::Text(text) => {
                let text = *text;
                let span = self.advance().span;
                Ok(Expr::Text { text, span })
            }
            TokenKind::Boolean(value) => {
                let value = *value;
                let span = self.advance().span;
                Ok(Expr::Boolean { value, span })
            }
            TokenKind::Ident(_) => {
                let (name, span) = self.parse_qualified_name()?;
                Ok(Expr::Variable { name, span })
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_right_paren()?;
                Ok(expr)
            }
            TokenKind::LeftBracket => self.parse_list(),
            _ => {
                let span = self.current_span();
                Err(CalcError::new(CalcErrorKind::ExpectedExpression, span))
            }
        }
    }

    fn parse_list(&mut self) -> Result<Expr, CalcError> {
        let start = self.advance().span.start;
        let mut values = Vec::new();

        if matches!(self.current().kind, TokenKind::RightBracket) {
            return Err(CalcError::new(
                CalcErrorKind::ExpectedExpression,
                self.current_span(),
            ));
        }

        loop {
            values.push(self.parse_expression()?);

            if !matches!(self.current().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
            if matches!(self.current().kind, TokenKind::RightBracket) {
                break;
            }
        }

        let end = self.expect_right_bracket()?;
        Ok(Expr::List {
            values,
            span: Span::new(start, end),
        })
    }

    fn expect_right_paren(&mut self) -> Result<(), CalcError> {
        if matches!(self.current().kind, TokenKind::RightParen) {
            self.advance();
            Ok(())
        } else {
            Err(CalcError::new(
                CalcErrorKind::ExpectedToken,
                self.current_span(),
            ))
        }
    }

    fn expect_right_bracket(&mut self) -> Result<usize, CalcError> {
        if matches!(self.current().kind, TokenKind::RightBracket) {
            Ok(self.advance().span.end)
        } else {
            Err(CalcError::new(
                CalcErrorKind::ExpectedToken,
                self.current_span(),
            ))
        }
    }

    fn expect_eof(&self) -> Result<(), CalcError> {
        if matches!(self.current().kind, TokenKind::Eof) {
            Ok(())
        } else {
            Err(CalcError::new(
                CalcErrorKind::UnexpectedToken,
                self.current_span(),
            ))
        }
    }

    fn is_assignment(&self) -> bool {
        matches!(self.current().kind, TokenKind::Ident(_))
            && self
                .tokens
                .get(self.index + 1)
                .is_some_and(|token| matches!(token.kind, TokenKind::Equal))
    }

    fn is_section_header(&self) -> bool {
        matches!(self.current().kind, TokenKind::Ident(_))
            && self
                .tokens
                .get(self.index + 1)
                .is_some_and(|token| matches!(token.kind, TokenKind::Colon))
    }

    fn is_invalid_section_header(&self) -> bool {
        let mut saw_dot = false;

        for token in &self.tokens[self.index..] {
            match token.kind {
                TokenKind::Dot => saw_dot = true,
                TokenKind::Colon => return saw_dot,
                TokenKind::Eof
                | TokenKind::Equal
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::LeftParen
                | TokenKind::RightParen
                | TokenKind::LeftBracket
                | TokenKind::RightBracket
                | TokenKind::Comma
                | TokenKind::Number(_)
                | TokenKind::Currency(_)
                | TokenKind::Money { .. }
                | TokenKind::Text(_)
                | TokenKind::Boolean(_) => return false,
                TokenKind::Ident(_) => {}
            }
        }

        false
    }

    fn parse_qualified_name(&mut self) -> Result<(InternalQualifiedName, Span), CalcError> {
        let first = self.advance();
        let TokenKind::Ident(name) = first.kind else {
            unreachable!("qualified name starts with an identifier");
        };
        let mut parts = vec![name];
        let start = first.span.start;
        let mut end = first.span.end;

        while matches!(self.current().kind, TokenKind::Dot) {
            self.advance();
            let token = self.advance();
            let TokenKind::Ident(name) = token.kind else {
                return Err(CalcError::new(CalcErrorKind::ExpectedToken, token.span));
            };
            end = token.span.end;
            parts.push(name);
        }

        Ok((InternalQualifiedName::new(parts), Span::new(start, end)))
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn current_span(&self) -> Span {
        Span::new(self.current().span.start, self.current().span.end)
    }

    fn advance(&mut self) -> Token {
        let token = std::mem::replace(
            &mut self.tokens[self.index],
            Token::new(TokenKind::Eof, Span::new(0, 0)),
        );
        self.index += 1;
        token
    }
}
