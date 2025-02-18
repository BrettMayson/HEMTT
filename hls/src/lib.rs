use files::FileCache;
use sqf::SqfAnalyzer;
use tokio::net::TcpStream;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tracing::{debug, info, Level};

use crate::diag_manager::DiagManager;
use crate::sqf_project::SqfCache;
use crate::workspace::EditorWorkspaces;

mod color;
mod config;
mod diag_manager;
mod files;
mod positions;
pub mod sqf;
mod sqf_project;
mod workspace;

pub const LEGEND_TYPE: &[SemanticTokenType] = &[SemanticTokenType::FUNCTION];

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
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("sqf".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.into(),
                                    token_modifiers: vec![],
                                },
                                range: Some(false),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                color_provider: Some(ColorProviderCapability::Options(
                    StaticTextDocumentColorProviderOptions {
                        document_selector: Some(vec![
                            DocumentFilter {
                                language: Some("sqf".to_string()),
                                scheme: Some("file".to_string()),
                                pattern: None,
                            },
                            DocumentFilter {
                                language: Some("arma-config".to_string()),
                                scheme: Some("file".to_string()),
                                pattern: None,
                            },
                        ]),
                        id: None,
                    },
                )),
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

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        debug!("did_change_watched_files");
        for x in params.changes {
            if x.uri.path().contains(".toml") {
                config::did_save(x.uri.clone(), Some(self.client.clone())).await;
            }
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("did_open: {:?}", params.text_document.uri);
        SqfCache::cache(params.text_document.uri.clone()).await;
        let document = TextDocumentItem {
            uri: params.text_document.uri,
            text: TextInformation::Full(&params.text_document.text),
            version: Some(params.text_document.version),
        };
        FileCache::get().on_change(&document).await;
        SqfAnalyzer::get().on_change(&document).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        debug!("did_change: {:?}", params.text_document.uri);
        let document = TextDocumentItem {
            text: TextInformation::Changes(params.content_changes),
            uri: params.text_document.uri,
            version: Some(params.text_document.version),
        };
        FileCache::get().on_change(&document).await;
        SqfAnalyzer::get().on_change(&document).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("did_save: {:?}", params.text_document.uri);
        SqfCache::cache(params.text_document.uri.clone()).await;
        config::did_save(params.text_document.uri.clone(), None).await;
        if let Some(text) = params.text {
            let document = TextDocumentItem {
                uri: params.text_document.uri,
                text: TextInformation::Full(&text),
                version: None,
            };
            FileCache::get().on_change(&document).await;
            SqfAnalyzer::get().on_change(&document).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        debug!("did_close: {:?}", params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(SqfAnalyzer::get()
            .hover(
                params.text_document_position_params.text_document.uri,
                params.text_document_position_params.position,
            )
            .await)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        Ok(SqfAnalyzer::get()
            .get_tokens(&params.text_document.uri)
            .await
            .map(|tokens| {
                debug!("sending tokens: {}", tokens.len());
                SemanticTokensResult::Tokens(SemanticTokens {
                    data: tokens,
                    ..Default::default()
                })
            }))
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        color::info(params.text_document.uri).await
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        color::presentation(params).await
    }
}

#[allow(dead_code)]
pub struct TextDocumentItem<'a> {
    uri: Url,
    text: TextInformation<'a>,
    version: Option<i32>,
}

pub enum TextInformation<'a> {
    Full(&'a str),
    Changes(Vec<TextDocumentContentChangeEvent>),
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::DEBUG)
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(server());
}

async fn server() {
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
