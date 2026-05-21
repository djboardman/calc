use std::collections::HashSet;

use crate::{
    CalcError, CalcErrorKind, Environment, InternalQualifiedName, Span, Statement, StringInterner,
    Symbol, Value, evaluate_statement, parse, qualified_name, resolve_expr_dependencies,
};

#[derive(Debug)]
pub struct DocumentEvaluation {
    pub lines: Vec<LineEvaluation>,
    interner: StringInterner,
    _states: Vec<LineState>,
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
    pub defines: Option<InternalQualifiedName>,
}

#[derive(Debug)]
struct LineState {
    _source: String,
    _depends_on: HashSet<InternalQualifiedName>,
}

struct LineEntry {
    evaluation: LineEvaluation,
    state: LineState,
}

#[derive(Debug)]
struct SectionEntry {
    indent: usize,
    name: Symbol,
}

pub fn evaluate_new_document(source: &str) -> DocumentEvaluation {
    evaluate_document(source, StringInterner::new())
}

pub fn evaluate_edited_document(previous: DocumentEvaluation, source: &str) -> DocumentEvaluation {
    evaluate_document(source, previous.interner)
}

fn evaluate_document(source: &str, interner: StringInterner) -> DocumentEvaluation {
    let mut document = DocumentEvaluation {
        lines: Vec::new(),
        interner,
        _states: Vec::new(),
    };
    let mut env = Environment::new();
    let mut sections = Vec::new();

    for (line, text) in source.split('\n').enumerate() {
        let entry = evaluate_line(line, text, &mut document.interner, &mut env, &mut sections);
        document._states.push(entry.state);
        document.lines.push(entry.evaluation);
    }

    document
}

fn evaluate_line(
    line: usize,
    source: &str,
    interner: &mut StringInterner,
    env: &mut Environment,
    sections: &mut Vec<SectionEntry>,
) -> LineEntry {
    let source_without_comment = source_before_comment(source);
    if source_without_comment.trim().is_empty() {
        return line_entry(line, source, Ok(None), None, HashSet::new());
    }

    let indent = match leading_indent(source) {
        Ok(indent) => indent,
        Err(error) => return line_entry(line, source, Err(error), None, HashSet::new()),
    };
    dedent(sections, indent);
    let scope = section_scope(sections);

    match parse(source, interner) {
        Ok(Statement::SectionHeader { name, .. }) => {
            sections.push(SectionEntry { indent, name });
            line_entry(line, source, Ok(None), None, HashSet::new())
        }
        Ok(statement) => {
            let defines = statement_definition(&statement, &scope);
            let depends_on = statement_dependencies(&statement, env, &scope);
            let result = evaluate_statement(&statement, env, &scope, interner).map(Some);

            line_entry(line, source, result, defines, depends_on)
        }
        Err(error) => line_entry(line, source, Err(error), None, HashSet::new()),
    }
}

fn line_entry(
    line: usize,
    source: &str,
    result: Result<Option<Value>, CalcError>,
    defines: Option<InternalQualifiedName>,
    depends_on: HashSet<InternalQualifiedName>,
) -> LineEntry {
    LineEntry {
        evaluation: LineEvaluation {
            line,
            result,
            defines,
        },
        state: LineState {
            _source: source.to_string(),
            _depends_on: depends_on,
        },
    }
}

fn source_before_comment(source: &str) -> &str {
    let mut in_text = false;

    for (index, ch) in source.char_indices() {
        match ch {
            '"' => in_text = !in_text,
            '#' if !in_text => return &source[..index],
            _ => {}
        }
    }

    source
}

fn leading_indent(source: &str) -> Result<usize, CalcError> {
    let mut indent = 0;

    for (index, ch) in source.char_indices() {
        match ch {
            ' ' => indent += 1,
            '\t' => {
                return Err(CalcError::new(
                    CalcErrorKind::InvalidIndentation,
                    Span::new(index, index + ch.len_utf8()),
                ));
            }
            _ => return Ok(indent),
        }
    }

    Ok(indent)
}

fn dedent(sections: &mut Vec<SectionEntry>, indent: usize) {
    while sections
        .last()
        .is_some_and(|section| indent <= section.indent)
    {
        sections.pop();
    }
}

fn section_scope(sections: &[SectionEntry]) -> Vec<Symbol> {
    sections.iter().map(|section| section.name).collect()
}

fn statement_definition(statement: &Statement, scope: &[Symbol]) -> Option<InternalQualifiedName> {
    match statement {
        Statement::Expr(_) | Statement::SectionHeader { .. } => None,
        Statement::Assignment { name, .. } => Some(qualified_name(scope, *name)),
    }
}

fn statement_dependencies(
    statement: &Statement,
    env: &Environment,
    scope: &[Symbol],
) -> HashSet<InternalQualifiedName> {
    let mut dependencies = HashSet::new();

    match statement {
        Statement::Expr(expr) => resolve_expr_dependencies(expr, env, scope, &mut dependencies),
        Statement::Assignment { value, .. } => {
            resolve_expr_dependencies(value, env, scope, &mut dependencies);
        }
        Statement::SectionHeader { .. } => {}
    }

    dependencies
}
