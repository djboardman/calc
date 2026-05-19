use tower_lsp::lsp_types::{CompletionResponse, Position, Url};

use crate::{document_store::DocumentStore, result_adapter};

pub(crate) fn completions(
    documents: &DocumentStore,
    uri: &Url,
    position: Position,
) -> Option<CompletionResponse> {
    let document = &documents.get(uri)?.evaluation;
    let items = document
        .lines
        .iter()
        .take(position.line as usize)
        .filter_map(|line| result_adapter::variable_completion_item(document, line))
        .collect();

    Some(CompletionResponse::Array(items))
}
