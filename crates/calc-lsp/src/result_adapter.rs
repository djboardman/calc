use calc_core::{DocumentEvaluation, LineEvaluation};
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity, InlayHint, InlayHintKind,
    InlayHintLabel, Position,
};

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

pub(crate) fn inlay_hint(
    source: &str,
    _document: &DocumentEvaluation,
    line: &LineEvaluation,
) -> Option<InlayHint> {
    let value = line.result.as_ref().ok().and_then(Option::as_ref)?;
    let line_text = source.split('\n').nth(line.line)?;

    Some(InlayHint {
        position: Position {
            line: line.line as u32,
            character: line_text.len() as u32,
        },
        label: InlayHintLabel::String(format!("= {}", value.number)),
        kind: Some(InlayHintKind::TYPE),
        text_edits: None,
        tooltip: None,
        padding_left: Some(true),
        padding_right: Some(false),
        data: None,
    })
}
