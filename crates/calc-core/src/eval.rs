use std::collections::HashSet;

use crate::{
    BinaryOp, CalcError, CalcErrorKind, Environment, Expr, InternalQualifiedName, Span, Statement,
    Symbol, UnaryOp, Value,
};

pub fn evaluate_statement(
    statement: &Statement,
    env: &mut Environment,
    scope: &[Symbol],
) -> Result<Value, CalcError> {
    match statement {
        Statement::Expr(expr) => evaluate_expr(expr, env, scope),
        Statement::Assignment { name, value, .. } => {
            let value = evaluate_expr(value, env, scope)?;
            env.set(qualified_name(scope, *name), Value::number(value.number));
            Ok(value)
        }
        Statement::SectionHeader { name_span, .. } => Err(CalcError::new(
            CalcErrorKind::InvalidSectionHeader,
            Span::new(name_span.start, name_span.end),
        )),
    }
}

fn evaluate_expr(expr: &Expr, env: &Environment, scope: &[Symbol]) -> Result<Value, CalcError> {
    match expr {
        Expr::Number { value, .. } => Ok(Value::number(*value)),
        Expr::Variable { name, span } => {
            let Some(name) = env.resolve(name, scope) else {
                return Err(CalcError::new(
                    CalcErrorKind::UndefinedVariable,
                    Span::new(span.start, span.end),
                ));
            };
            match env.get(&name) {
                Some(value) => Ok(Value::number(value.number)),
                None => Err(CalcError::new(
                    CalcErrorKind::UndefinedVariable,
                    Span::new(span.start, span.end),
                )),
            }
        }
        Expr::Unary { op, expr, .. } => {
            let value = evaluate_expr(expr, env, scope)?;
            match op {
                UnaryOp::Negate => Ok(Value::number(-value.number)),
            }
        }
        Expr::Binary {
            left,
            op,
            op_span,
            right,
        } => {
            let left = evaluate_expr(left, env, scope)?;
            let right = evaluate_expr(right, env, scope)?;

            match op {
                BinaryOp::Add => Ok(Value::number(left.number + right.number)),
                BinaryOp::Subtract => Ok(Value::number(left.number - right.number)),
                BinaryOp::Multiply => Ok(Value::number(left.number * right.number)),
                BinaryOp::Divide if right.number == 0.0 => Err(CalcError::new(
                    CalcErrorKind::DivisionByZero,
                    Span::new(op_span.start, op_span.end),
                )),
                BinaryOp::Divide => Ok(Value::number(left.number / right.number)),
            }
        }
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
        Expr::Number { .. } => {}
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
