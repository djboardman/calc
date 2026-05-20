use crate::{InternalQualifiedName, Span, Symbol};

#[derive(Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Negate,
}

#[derive(Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number {
        value: f64,
        span: Span,
    },
    Variable {
        name: InternalQualifiedName,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        op_span: Span,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        op_span: Span,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Expr(Expr),
    Assignment {
        name: Symbol,
        name_span: Span,
        value: Expr,
    },
    SectionHeader {
        name: Symbol,
        name_span: Span,
    },
}
