use serde_json::Value;
use tokio::net::TcpStream;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use config::ConfigServer;
use tracing::{debug, info, Level};

mod config;
mod positions;

#[derive(Debug)]
struct Backend {
    client: Client,
    config: ConfigServer,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("initialized");
        self.config.initialize().await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        debug!("did_change_workspace_folders: {:?}", params);
        self.config.did_change_workspace_folders(params).await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("did_change_configuration");
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        debug!("did_change_watched_files");
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        debug!("execute_command");
        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("did_open: {:?}", params);
        let uri_string = params.text_document.uri.to_string();
        if uri_string.ends_with(".hpp") || uri_string.ends_with("config.cpp") {
            self.config.did_open(params).await;
        }
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        debug!("did_change");
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("did_save");
        self.config.did_save(params).await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        debug!("did_close");
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        debug!("completion");
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::DEBUG)
        .init();

    let stream = TcpStream::connect("127.0.0.1:9632").await.unwrap();

    info!("connected to server");

    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::new(|client| Backend {
        config: ConfigServer::new(client.clone()),
        client,
    });
    Server::new(read, write, socket).serve(service).await;
}
