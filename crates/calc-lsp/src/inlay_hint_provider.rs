use tower_lsp::lsp_types::{InlayHint, InlayHintParams, Url};

use crate::{document_store::DocumentStore, result_adapter};

pub(crate) fn inlay_hints(
    documents: &DocumentStore,
    uri: &Url,
    params: &InlayHintParams,
) -> Option<Vec<InlayHint>> {
    let document = documents.get(uri)?;
    let start = params.range.start.line as usize;
    let end = params.range.end.line as usize;

    Some(
        document
            .evaluation
            .lines
            .iter()
            .filter(|line| line.line >= start && line.line <= end)
            .filter_map(|line| {
                result_adapter::inlay_hint(&document.source, &document.evaluation, line)
            })
            .collect(),
    )
}
