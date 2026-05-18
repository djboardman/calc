use std::collections::HashMap;

use calc_core::DocumentEvaluation;
use tower_lsp::lsp_types::Url;

#[derive(Default)]
pub(crate) struct DocumentStore {
    documents: HashMap<Url, DocumentState>,
}

pub(crate) struct DocumentState {
    pub(crate) source: String,
    pub(crate) evaluation: DocumentEvaluation,
}

impl DocumentStore {
    pub(crate) fn open(&mut self, uri: Url, source: String, evaluation: DocumentEvaluation) {
        self.documents
            .insert(uri, DocumentState { source, evaluation });
    }

    pub(crate) fn get(&self, uri: &Url) -> Option<&DocumentState> {
        self.documents.get(uri)
    }

    pub(crate) fn get_mut(&mut self, uri: &Url) -> Option<&mut DocumentState> {
        self.documents.get_mut(uri)
    }

    pub(crate) fn close(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }
}
