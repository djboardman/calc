use calc_core::{DocumentEvaluation, LineEvaluation};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity};

use crate::diagnostics_provider;

pub(crate) fn variable_completion_item(
    document: &DocumentEvaluation,
    line: &LineEvaluation,
) -> Option<CompletionItem> {
    line.result.as_ref().ok()?;
    let symbol = line.defines?;
    let variable = document.symbol_text(symbol);

    Some(CompletionItem {
        label: variable.to_string(),
        kind: Some(CompletionItemKind::VARIABLE),
        insert_text: Some(variable.to_string()),
        ..CompletionItem::default()
    })
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
