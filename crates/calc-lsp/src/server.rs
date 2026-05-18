use std::sync::Arc;

use tokio::sync::Mutex;
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    },
};

use crate::{
    completion_provider, configuration::Configuration, diagnostics_provider,
    document_input_adapter, document_store::DocumentStore,
};

pub(crate) async fn run() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(CalcLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}

struct CalcLanguageServer {
    client: Client,
    _configuration: Configuration,
    documents: Arc<Mutex<DocumentStore>>,
}

impl CalcLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            _configuration: Configuration,
            documents: Arc::new(Mutex::new(DocumentStore::default())),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CalcLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                ..ServerCapabilities::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _params: tower_lsp::lsp_types::InitializedParams) {}

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let source = params.text_document.text;
        let diagnostics = {
            let mut documents = self.documents.lock().await;
            document_input_adapter::open_document(&mut documents, uri.clone(), source);
            diagnostics_provider::diagnostics(&documents, &uri)
        };

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let diagnostics = {
            let mut documents = self.documents.lock().await;
            document_input_adapter::change_document(&mut documents, &uri, params.content_changes);
            diagnostics_provider::diagnostics(&documents, &uri)
        };

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let mut documents = self.documents.lock().await;
            documents.close(&uri);
        }

        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let documents = self.documents.lock().await;
        Ok(completion_provider::completions(
            &documents,
            &params.text_document_position.text_document.uri,
            params.text_document_position.position,
        ))
    }
}
