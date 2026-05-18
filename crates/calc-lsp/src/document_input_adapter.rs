use calc_core::{evaluate_edited_document, evaluate_new_document};
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::document_store::{DocumentState, DocumentStore};

pub(crate) fn open_document(store: &mut DocumentStore, uri: Url, source: String) {
    let evaluation = evaluate_new_document(&source);
    store.open(uri, source, evaluation);
}

pub(crate) fn change_document(
    store: &mut DocumentStore,
    uri: &Url,
    changes: Vec<TextDocumentContentChangeEvent>,
) {
    let Some(state) = store.get_mut(uri) else {
        return;
    };

    for change in changes {
        apply_change(state, change);
    }
}

fn apply_change(state: &mut DocumentState, change: TextDocumentContentChangeEvent) {
    state.source = change.text;
    let previous = std::mem::replace(&mut state.evaluation, evaluate_new_document(""));
    state.evaluation = evaluate_edited_document(previous, &state.source);
}
