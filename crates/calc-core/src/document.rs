use std::collections::HashSet;

use crate::{
    CalcError, Environment, Expr, Statement, StringInterner, Symbol, Value, evaluate_statement,
    parse,
};

#[derive(Debug)]
pub struct DocumentEvaluation {
    pub lines: Vec<LineEvaluation>,
    interner: StringInterner,
    states: Vec<LineState>,
}

impl DocumentEvaluation {
    pub fn symbol_text(&self, symbol: Symbol) -> &str {
        self.interner.get(symbol)
    }
}

#[derive(Debug)]
pub struct LineEvaluation {
    pub line: usize,
    pub result: Result<Option<Value>, CalcError>,
    pub defines: Option<Symbol>,
}

#[derive(Debug)]
struct LineState {
    source: String,
    depends_on: HashSet<Symbol>,
}

struct LineEntry {
    evaluation: LineEvaluation,
    state: LineState,
}

pub fn evaluate_new_document(source: &str) -> DocumentEvaluation {
    let mut document = DocumentEvaluation {
        lines: Vec::new(),
        interner: StringInterner::new(),
        states: Vec::new(),
    };
    let mut env = Environment::new();

    for (line, text) in source.split('\n').enumerate() {
        let entry = evaluate_line(line, text, &mut document.interner, &mut env);
        apply_line_to_environment(&entry.evaluation, &mut env);
        document.lines.push(entry.evaluation);
        document.states.push(entry.state);
    }

    document
}

pub fn evaluate_edited_document(previous: DocumentEvaluation, source: &str) -> DocumentEvaluation {
    let new_lines: Vec<&str> = source.split('\n').collect();
    let Some(change) = changed_line_range(&previous.states, &new_lines) else {
        return previous;
    };

    let DocumentEvaluation {
        lines,
        interner,
        states,
    } = previous;
    let mut old_entries = lines
        .into_iter()
        .zip(states)
        .map(|(evaluation, state)| Some(LineEntry { evaluation, state }))
        .collect::<Vec<_>>();
    let mut document = DocumentEvaluation {
        lines: Vec::new(),
        interner,
        states: Vec::new(),
    };
    let mut env = Environment::new();
    let mut changed_variables = changed_variables(&old_entries, &change);

    for (line, text) in new_lines.iter().enumerate() {
        let in_changed_range = line >= change.start && line < change.new_end;
        let old_index = old_line_index(line, &change);
        let old_entry =
            old_index.and_then(|index| old_entries.get_mut(index).and_then(Option::take));
        let depends_on_changed = old_entry
            .as_ref()
            .is_some_and(|entry| depends_on_any(&entry.state, &changed_variables));

        let entry = if in_changed_range || depends_on_changed {
            let entry = evaluate_line(line, text, &mut document.interner, &mut env);
            if let Some(symbol) = entry.evaluation.defines {
                changed_variables.insert(symbol);
            }
            entry
        } else if let Some(old_entry) = old_entry {
            reuse_line(old_entry, line)
        } else {
            evaluate_line(line, text, &mut document.interner, &mut env)
        };

        apply_line_to_environment(&entry.evaluation, &mut env);
        document.lines.push(entry.evaluation);
        document.states.push(entry.state);
    }

    document
}

#[derive(Debug)]
struct ChangeRange {
    start: usize,
    old_end: usize,
    new_end: usize,
}

fn evaluate_line(
    line: usize,
    source: &str,
    interner: &mut StringInterner,
    env: &mut Environment,
) -> LineEntry {
    if source.trim().is_empty() {
        return LineEntry {
            evaluation: LineEvaluation {
                line,
                result: Ok(None),
                defines: None,
            },
            state: LineState {
                source: source.to_string(),
                depends_on: HashSet::new(),
            },
        };
    }

    match parse(source, interner) {
        Ok(statement) => {
            let defines = statement_definition(&statement);
            let depends_on = statement_dependencies(&statement);
            let result = evaluate_statement(&statement, env).map(Some);

            LineEntry {
                evaluation: LineEvaluation {
                    line,
                    result,
                    defines,
                },
                state: LineState {
                    source: source.to_string(),
                    depends_on,
                },
            }
        }
        Err(error) => LineEntry {
            evaluation: LineEvaluation {
                line,
                result: Err(error),
                defines: None,
            },
            state: LineState {
                source: source.to_string(),
                depends_on: HashSet::new(),
            },
        },
    }
}

fn apply_line_to_environment(line: &LineEvaluation, env: &mut Environment) {
    if let (Some(symbol), Ok(Some(value))) = (line.defines, &line.result) {
        env.set(symbol, Value::number(value.number));
    }
}

fn changed_line_range(old_states: &[LineState], new_lines: &[&str]) -> Option<ChangeRange> {
    let shared_len = old_states.len().min(new_lines.len());
    let mut first = 0;

    while first < shared_len && old_states[first].source == new_lines[first] {
        first += 1;
    }

    if first == old_states.len() && first == new_lines.len() {
        return None;
    }

    let mut old_end = old_states.len();
    let mut new_end = new_lines.len();

    while old_end > first
        && new_end > first
        && old_states[old_end - 1].source == new_lines[new_end - 1]
    {
        old_end -= 1;
        new_end -= 1;
    }

    Some(ChangeRange {
        start: first,
        old_end,
        new_end,
    })
}

fn old_line_index(new_line: usize, change: &ChangeRange) -> Option<usize> {
    if new_line < change.start {
        Some(new_line)
    } else if new_line >= change.new_end {
        Some(new_line + change.old_end - change.new_end)
    } else {
        None
    }
}

fn reuse_line(mut entry: LineEntry, new_line_number: usize) -> LineEntry {
    entry.evaluation.line = new_line_number;
    entry
}

fn changed_variables(old_entries: &[Option<LineEntry>], change: &ChangeRange) -> HashSet<Symbol> {
    let mut symbols = HashSet::new();

    for line in change.start..change.old_end {
        if let Some(Some(entry)) = old_entries.get(line)
            && let Some(symbol) = entry.evaluation.defines
        {
            symbols.insert(symbol);
        }
    }

    symbols
}

fn depends_on_any(state: &LineState, symbols: &HashSet<Symbol>) -> bool {
    state
        .depends_on
        .iter()
        .any(|symbol| symbols.contains(symbol))
}

fn statement_definition(statement: &Statement) -> Option<Symbol> {
    match statement {
        Statement::Expr(_) => None,
        Statement::Assignment { name, .. } => Some(*name),
    }
}

fn statement_dependencies(statement: &Statement) -> HashSet<Symbol> {
    let mut dependencies = HashSet::new();

    match statement {
        Statement::Expr(expr) => expr_dependencies(expr, &mut dependencies),
        Statement::Assignment { value, .. } => expr_dependencies(value, &mut dependencies),
    }

    dependencies
}

fn expr_dependencies(expr: &Expr, dependencies: &mut HashSet<Symbol>) {
    match expr {
        Expr::Number { .. } => {}
        Expr::Variable { name, .. } => {
            dependencies.insert(*name);
        }
        Expr::Unary { expr, .. } => expr_dependencies(expr, dependencies),
        Expr::Binary { left, right, .. } => {
            expr_dependencies(left, dependencies);
            expr_dependencies(right, dependencies);
        }
    }
}
