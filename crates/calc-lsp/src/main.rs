mod completion_provider;
mod configuration;
mod diagnostics_provider;
mod document_input_adapter;
mod document_store;
mod formatting_provider;
mod result_adapter;
mod result_comment;
mod server;

#[tokio::main]
async fn main() {
    server::run().await;
}
