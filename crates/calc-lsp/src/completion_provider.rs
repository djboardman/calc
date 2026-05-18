use tower_lsp::lsp_types::{CompletionResponse, Position, Url};

use crate::{document_store::DocumentStore, result_adapter};

pub(crate) fn completions(
    documents: &DocumentStore,
    uri: &Url,
    position: Position,
) -> Option<CompletionResponse> {
    let document = &documents.get(uri)?.evaluation;
    let line = document.lines.get(position.line as usize)?;
    let items = result_adapter::completion_item(document, line)
        .into_iter()
        .collect();

    Some(CompletionResponse::Array(items))
}
