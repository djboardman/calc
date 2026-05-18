use crate::{
    BinaryOp, CalcError, CalcErrorKind, Environment, Expr, Span, Statement, UnaryOp, Value,
};

pub fn evaluate_statement(
    statement: &Statement,
    env: &mut Environment,
) -> Result<Value, CalcError> {
    match statement {
        Statement::Expr(expr) => evaluate_expr(expr, env),
        Statement::Assignment { name, value, .. } => {
            let value = evaluate_expr(value, env)?;
            env.set(*name, Value::number(value.number));
            Ok(value)
        }
    }
}

fn evaluate_expr(expr: &Expr, env: &Environment) -> Result<Value, CalcError> {
    match expr {
        Expr::Number { value, .. } => Ok(Value::number(*value)),
        Expr::Variable { name, span } => match env.get(*name) {
            Some(value) => Ok(Value::number(value.number)),
            None => Err(CalcError::new(
                CalcErrorKind::UndefinedVariable,
                Span::new(span.start, span.end),
            )),
        },
        Expr::Unary { op, expr, .. } => {
            let value = evaluate_expr(expr, env)?;
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
            let left = evaluate_expr(left, env)?;
            let right = evaluate_expr(right, env)?;

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
