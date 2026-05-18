use calc_core::{DocumentEvaluation, LineEvaluation, Value};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity};

use crate::diagnostics_provider;

pub(crate) fn completion_item(
    document: &DocumentEvaluation,
    line: &LineEvaluation,
) -> Option<CompletionItem> {
    let value = line.result.as_ref().ok().and_then(Option::as_ref)?;

    Some(CompletionItem {
        label: completion_label(document, line, value),
        kind: Some(CompletionItemKind::VALUE),
        insert_text: None,
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

fn completion_label(document: &DocumentEvaluation, line: &LineEvaluation, value: &Value) -> String {
    match line.defines {
        Some(symbol) => format!("{} = {}", document.symbol_text(symbol), value.number),
        None => value.number.to_string(),
    }
}
