use calc_core::Span;
use tower_lsp::lsp_types::{Diagnostic, Position, Range, Url};

use crate::{document_store::DocumentStore, result_adapter};

pub(crate) fn diagnostics(documents: &DocumentStore, uri: &Url) -> Vec<Diagnostic> {
    let Some(document) = documents.get(uri) else {
        return Vec::new();
    };

    document
        .evaluation
        .lines
        .iter()
        .filter_map(result_adapter::diagnostic)
        .collect()
}

pub(crate) fn range(line: usize, span: &Span) -> Range {
    Range {
        start: Position {
            line: line as u32,
            character: span.start as u32,
        },
        end: Position {
            line: line as u32,
            character: span.end as u32,
        },
    }
}
