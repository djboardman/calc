use crate::Span;

#[derive(Debug, Eq, PartialEq)]
pub enum CalcErrorKind {
    UnexpectedCharacter,
    InvalidNumber,
    ExpectedExpression,
    ExpectedToken,
    UnexpectedToken,
    UndefinedVariable,
    DivisionByZero,
    InvalidIndentation,
    InvalidSectionHeader,
    UnsupportedTypeOperation,
    MixedListTypes,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CalcError {
    pub kind: CalcErrorKind,
    pub span: Span,
}

impl CalcError {
    pub(crate) fn new(kind: CalcErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}
