use calc_core::{DocumentEvaluation, LineEvaluation, QualifiedName};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use crate::diagnostics_provider;

pub(crate) fn qualified_name_text(document: &DocumentEvaluation, name: &QualifiedName) -> String {
    name.parts
        .iter()
        .map(|symbol| document.symbol_text(*symbol))
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn diagnostic(line: &LineEvaluation) -> Option<Diagnostic> {
    let error = line.result.as_ref().err()?;

    Some(Diagnostic {
        range: diagnostics_provider::range(line.line, &error.span),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("calc".to_string()),
        message: format!("{:?}", error.kind),
        ..Diagnostic::default()
    })
}
