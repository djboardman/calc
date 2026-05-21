use std::collections::HashSet;

use crate::{
    BinaryOp, CalcError, CalcErrorKind, Environment, Expr, InternalQualifiedName, Span, Statement,
    StringInterner, Symbol, UnaryOp, Value,
};

pub fn evaluate_statement(
    statement: &Statement,
    env: &mut Environment,
    scope: &[Symbol],
    interner: &StringInterner,
) -> Result<Value, CalcError> {
    match statement {
        Statement::Expr(expr) => evaluate_expr(expr, env, scope, interner),
        Statement::Assignment { name, value, .. } => {
            let value = evaluate_expr(value, env, scope, interner)?;
            env.set(qualified_name(scope, *name), value.clone());
            Ok(value)
        }
        Statement::SectionHeader { name_span, .. } => Err(CalcError::new(
            CalcErrorKind::InvalidSectionHeader,
            Span::new(name_span.start, name_span.end),
        )),
    }
}

fn evaluate_expr(
    expr: &Expr,
    env: &Environment,
    scope: &[Symbol],
    interner: &StringInterner,
) -> Result<Value, CalcError> {
    match expr {
        Expr::Number { value, .. } => Ok(Value::number(*value)),
        Expr::Currency { currency, .. } => Ok(Value::currency(interner.get(*currency))),
        Expr::Money {
            currency,
            minor_units,
            ..
        } => Ok(Value::money(interner.get(*currency), *minor_units)),
        Expr::Text { text, .. } => Ok(Value::text(interner.get(*text))),
        Expr::Boolean { value, .. } => Ok(Value::boolean(*value)),
        Expr::List { values, span } => {
            let mut evaluated = Vec::new();
            let mut value_type = None;

            for expr in values {
                let value = evaluate_expr(expr, env, scope, interner)?;
                let item_type = value.type_key();
                if value_type
                    .as_ref()
                    .is_some_and(|value_type| value_type != &item_type)
                {
                    return Err(CalcError::new(
                        CalcErrorKind::MixedListTypes,
                        Span::new(span.start, span.end),
                    ));
                }
                value_type = Some(item_type);
                evaluated.push(value);
            }

            Ok(Value::list(evaluated))
        }
        Expr::Variable { name, span } => {
            let Some(name) = env.resolve(name, scope) else {
                return Err(CalcError::new(
                    CalcErrorKind::UndefinedVariable,
                    Span::new(span.start, span.end),
                ));
            };
            match env.get(&name) {
                Some(value) => Ok(value.clone()),
                None => Err(CalcError::new(
                    CalcErrorKind::UndefinedVariable,
                    Span::new(span.start, span.end),
                )),
            }
        }
        Expr::Unary { op, expr, .. } => {
            let value = evaluate_expr(expr, env, scope, interner)?;
            match op {
                UnaryOp::Negate => negate(value, expr),
            }
        }
        Expr::Binary {
            left,
            op,
            op_span,
            right,
        } => {
            let left = evaluate_expr(left, env, scope, interner)?;
            let right = evaluate_expr(right, env, scope, interner)?;

            evaluate_binary(left, op, Span::new(op_span.start, op_span.end), right)
        }
    }
}

fn negate(value: Value, expr: &Expr) -> Result<Value, CalcError> {
    match value {
        Value::Number(number) => Ok(Value::number(-number)),
        Value::Money {
            currency,
            minor_units,
        } => Ok(Value::money(currency, -minor_units)),
        _ => Err(CalcError::new(
            CalcErrorKind::UnsupportedTypeOperation,
            expr_span(expr),
        )),
    }
}

fn evaluate_binary(
    left: Value,
    op: &BinaryOp,
    op_span: Span,
    right: Value,
) -> Result<Value, CalcError> {
    match (left, op, right) {
        (Value::Number(left), BinaryOp::Add, Value::Number(right)) => {
            Ok(Value::number(left + right))
        }
        (Value::Number(left), BinaryOp::Subtract, Value::Number(right)) => {
            Ok(Value::number(left - right))
        }
        (Value::Number(left), BinaryOp::Multiply, Value::Number(right)) => {
            Ok(Value::number(left * right))
        }
        (Value::Number(_), BinaryOp::Divide, Value::Number(0.0)) => {
            Err(CalcError::new(CalcErrorKind::DivisionByZero, op_span))
        }
        (Value::Number(left), BinaryOp::Divide, Value::Number(right)) => {
            Ok(Value::number(left / right))
        }
        (
            Value::Money {
                currency: left_currency,
                minor_units: left,
            },
            BinaryOp::Add,
            Value::Money {
                currency: right_currency,
                minor_units: right,
            },
        ) if left_currency == right_currency => Ok(Value::money(left_currency, left + right)),
        (
            Value::Money {
                currency: left_currency,
                minor_units: left,
            },
            BinaryOp::Subtract,
            Value::Money {
                currency: right_currency,
                minor_units: right,
            },
        ) if left_currency == right_currency => Ok(Value::money(left_currency, left - right)),
        (Value::Text(left), BinaryOp::Add, Value::Text(right)) => {
            Ok(Value::text(format!("{left}{right}")))
        }
        (
            Value::Money {
                currency,
                minor_units,
            },
            BinaryOp::Multiply,
            Value::Number(number),
        ) => Ok(Value::money(
            currency,
            ((minor_units as f64) * number).round() as i64,
        )),
        (
            Value::Money {
                currency: _,
                minor_units: _,
            },
            BinaryOp::Divide,
            Value::Number(0.0),
        ) => Err(CalcError::new(CalcErrorKind::DivisionByZero, op_span)),
        (
            Value::Money {
                currency,
                minor_units,
            },
            BinaryOp::Divide,
            Value::Number(number),
        ) => Ok(Value::money(
            currency,
            ((minor_units as f64) / number).round() as i64,
        )),
        _ => Err(CalcError::new(
            CalcErrorKind::UnsupportedTypeOperation,
            op_span,
        )),
    }
}

fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Number { span, .. }
        | Expr::Currency { span, .. }
        | Expr::Money { span, .. }
        | Expr::Text { span, .. }
        | Expr::Boolean { span, .. }
        | Expr::List { span, .. }
        | Expr::Variable { span, .. } => Span::new(span.start, span.end),
        Expr::Unary { op_span, expr, .. } => Span::new(op_span.start, expr_span(expr).end),
        Expr::Binary { left, right, .. } => Span::new(expr_span(left).start, expr_span(right).end),
    }
}

pub(crate) fn qualified_name(scope: &[Symbol], name: Symbol) -> InternalQualifiedName {
    let mut parts = scope.to_vec();
    parts.push(name);
    InternalQualifiedName::new(parts)
}

pub(crate) fn resolve_expr_dependencies(
    expr: &Expr,
    env: &Environment,
    scope: &[Symbol],
    dependencies: &mut HashSet<InternalQualifiedName>,
) {
    match expr {
        Expr::Number { .. }
        | Expr::Currency { .. }
        | Expr::Money { .. }
        | Expr::Text { .. }
        | Expr::Boolean { .. } => {}
        Expr::List { values, .. } => {
            for value in values {
                resolve_expr_dependencies(value, env, scope, dependencies);
            }
        }
        Expr::Variable { name, .. } => {
            if let Some(name) = env.resolve(name, scope) {
                dependencies.insert(name);
            }
        }
        Expr::Unary { expr, .. } => resolve_expr_dependencies(expr, env, scope, dependencies),
        Expr::Binary { left, right, .. } => {
            resolve_expr_dependencies(left, env, scope, dependencies);
            resolve_expr_dependencies(right, env, scope, dependencies);
        }
    }
}
