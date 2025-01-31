use std::env::args;

use basm_ls::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    if args().any(|s| matches!(s.as_str(), "--version")) {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }
    // # NOTE: need to find other ways to do logging
    let file_appender =
        tracing_appender::rolling::never("/home/yahya/project_files/basm/basm-ls/", "basm.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(non_blocking)
        .init();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
    tracing::info!("server stopping");
}
