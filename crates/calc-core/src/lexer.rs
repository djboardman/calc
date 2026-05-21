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
                '#' => break,
                '"' => tokens.push(self.text()?),
                '£' => tokens.push(self.symbol_currency("GBP")?),
                '$' => tokens.push(self.symbol_currency("USD")?),
                '€' => tokens.push(self.symbol_currency("EUR")?),
                '0'..='9' => tokens.push(self.number()?),
                '.' if self.next_char_is_digit() => tokens.push(self.number()?),
                '.' => tokens.push(self.single(TokenKind::Dot)),
                'a'..='z' | 'A'..='Z' | '_' => tokens.push(self.ident_or_keyword()?),
                '+' => tokens.push(self.single(TokenKind::Plus)),
                '-' => tokens.push(self.single(TokenKind::Minus)),
                '*' => tokens.push(self.single(TokenKind::Star)),
                '/' => tokens.push(self.single(TokenKind::Slash)),
                '=' => tokens.push(self.single(TokenKind::Equal)),
                ':' => tokens.push(self.single(TokenKind::Colon)),
                '(' => tokens.push(self.single(TokenKind::LeftParen)),
                ')' => tokens.push(self.single(TokenKind::RightParen)),
                '[' => tokens.push(self.single(TokenKind::LeftBracket)),
                ']' => tokens.push(self.single(TokenKind::RightBracket)),
                ',' => tokens.push(self.single(TokenKind::Comma)),
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

    fn ident_or_keyword(&mut self) -> Result<Token, CalcError> {
        let start = self.index;

        while let Some((_, ch)) = self.current_char() {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                    let prefix = &self.source[start..self.index];
                    if prefix.len() == 3 && is_iso_4217_currency(prefix) && ch.is_ascii_digit() {
                        break;
                    }
                    self.advance_char();
                }
                _ => break,
            }
        }

        let span = Span::new(start, self.index);
        let text = span.source(self.source);

        if text == "true" {
            return Ok(Token::new(TokenKind::Boolean(true), span));
        }
        if text == "false" {
            return Ok(Token::new(TokenKind::Boolean(false), span));
        }
        if let Some(currency) = money_currency_prefix(text)
            && self.current_starts_number()
        {
            let amount = self.money_amount()?;
            return Ok(Token::new(
                TokenKind::Money {
                    currency: self.interner.intern(currency),
                    minor_units: amount.minor_units,
                },
                Span::new(start, amount.end),
            ));
        }
        if text.len() == 3 && is_iso_4217_currency(text) {
            return Ok(Token::new(
                TokenKind::Currency(self.interner.intern(text)),
                span,
            ));
        }

        Ok(Token::new(
            TokenKind::Ident(self.interner.intern(text)),
            span,
        ))
    }

    fn symbol_currency(&mut self, currency: &str) -> Result<Token, CalcError> {
        let start = self.index;
        self.advance_char();

        if self.current_starts_number() {
            let amount = self.money_amount()?;
            return Ok(Token::new(
                TokenKind::Money {
                    currency: self.interner.intern(currency),
                    minor_units: amount.minor_units,
                },
                Span::new(start, amount.end),
            ));
        }

        Ok(Token::new(
            TokenKind::Currency(self.interner.intern(currency)),
            Span::new(start, self.index),
        ))
    }

    fn text(&mut self) -> Result<Token, CalcError> {
        let start = self.index;
        self.advance_char();
        let content_start = self.index;

        while let Some((_, ch)) = self.current_char() {
            if ch == '"' {
                let content = &self.source[content_start..self.index];
                self.advance_char();
                return Ok(Token::new(
                    TokenKind::Text(self.interner.intern(content)),
                    Span::new(start, self.index),
                ));
            }
            self.advance_char();
        }

        Err(CalcError::new(
            CalcErrorKind::UnexpectedCharacter,
            Span::new(start, self.index),
        ))
    }

    fn money_amount(&mut self) -> Result<MoneyAmount, CalcError> {
        let number = self.number()?;
        let text = number.span.source(self.source);

        Ok(MoneyAmount {
            minor_units: parse_money_minor_units(text)
                .ok_or_else(|| CalcError::new(CalcErrorKind::InvalidNumber, number.span))?,
            end: self.index,
        })
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

    fn next_char_is_digit(&self) -> bool {
        let mut chars = self.source[self.index..].chars();
        chars.next();
        matches!(chars.next(), Some('0'..='9'))
    }

    fn current_starts_number(&self) -> bool {
        matches!(self.current_char(), Some((_, '0'..='9')))
            || matches!(self.current_char(), Some((_, '.'))) && self.next_char_is_digit()
    }
}

struct MoneyAmount {
    minor_units: i64,
    end: usize,
}

fn money_currency_prefix(text: &str) -> Option<&str> {
    if text.len() == 3 && is_iso_4217_currency(text) {
        Some(text)
    } else {
        None
    }
}

fn parse_money_minor_units(text: &str) -> Option<i64> {
    let value = text.parse::<f64>().ok()?;
    Some((value * 100.0).round() as i64)
}

fn is_iso_4217_currency(text: &str) -> bool {
    ISO_4217_CODES.binary_search(&text).is_ok()
}

const ISO_4217_CODES: &[&str] = &[
    "AED", "AFN", "ALL", "AMD", "ANG", "AOA", "ARS", "AUD", "AWG", "AZN", "BAM", "BBD", "BDT",
    "BGN", "BHD", "BIF", "BMD", "BND", "BOB", "BOV", "BRL", "BSD", "BTN", "BWP", "BYN", "BZD",
    "CAD", "CDF", "CHE", "CHF", "CHW", "CLF", "CLP", "CNY", "COP", "COU", "CRC", "CUP", "CVE",
    "CZK", "DJF", "DKK", "DOP", "DZD", "EGP", "ERN", "ETB", "EUR", "FJD", "FKP", "GBP", "GEL",
    "GHS", "GIP", "GMD", "GNF", "GTQ", "GYD", "HKD", "HNL", "HTG", "HUF", "IDR", "ILS", "INR",
    "IQD", "IRR", "ISK", "JMD", "JOD", "JPY", "KES", "KGS", "KHR", "KMF", "KPW", "KRW", "KWD",
    "KYD", "KZT", "LAK", "LBP", "LKR", "LRD", "LSL", "LYD", "MAD", "MDL", "MGA", "MKD", "MMK",
    "MNT", "MOP", "MRU", "MUR", "MVR", "MWK", "MXN", "MXV", "MYR", "MZN", "NAD", "NGN", "NIO",
    "NOK", "NPR", "NZD", "OMR", "PAB", "PEN", "PGK", "PHP", "PKR", "PLN", "PYG", "QAR", "RON",
    "RSD", "RUB", "RWF", "SAR", "SBD", "SCR", "SDG", "SEK", "SGD", "SHP", "SLE", "SOS", "SRD",
    "SSP", "STN", "SVC", "SYP", "SZL", "THB", "TJS", "TMT", "TND", "TOP", "TRY", "TTD", "TWD",
    "TZS", "UAH", "UGX", "USD", "USN", "UYI", "UYU", "UYW", "UZS", "VED", "VES", "VND", "VUV",
    "WST", "XAF", "XAG", "XAU", "XBA", "XBB", "XBC", "XBD", "XCD", "XCG", "XDR", "XOF", "XPD",
    "XPF", "XPT", "XSU", "XTS", "XUA", "XXX", "YER", "ZAR", "ZMW", "ZWG",
];
