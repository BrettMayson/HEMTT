use tokio::net::TcpStream;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tracing::{debug, info, Level};

use crate::diag_manager::DiagManager;
use crate::sqf::SqfCache;
use crate::workspace::EditorWorkspaces;

mod config;
mod diag_manager;
mod positions;
mod sqf;
mod workspace;

#[derive(Debug)]
struct Backend {
    client: Client,
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
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
        info!("initializing");
        DiagManager::init(self.client.clone());
        if let Some(folders) = self.client.workspace_folders().await.unwrap() {
            EditorWorkspaces::get().initialize(folders);
        }
        info!("initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        debug!("did_change_workspace_folders: {:?}", params);
        EditorWorkspaces::get().changed(params);
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("did_change_configuration");
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        debug!("did_change_watched_files");
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("did_open: {:?}", params);
        SqfCache::cache(params.text_document.uri).await;
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        debug!("did_change");
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("did_save");
        SqfCache::cache(params.text_document.uri.clone()).await;
        config::did_save(params.text_document.uri).await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        debug!("did_close");
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(SqfCache::get().hover(
            params.text_document_position_params.text_document.uri,
            params.text_document_position_params.position,
        ))
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::DEBUG)
        .init();

    // first argument is the port
    let port = std::env::args().nth(1).unwrap_or("9632".to_string());

    let stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    info!("connected to server");

    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(read, write, socket).serve(service).await;
}
