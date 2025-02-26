use config::ConfigAnalyzer;
use files::FileCache;
use preprocessor::PreprocessorAnalyzer;
use serde_json::Value;
use sqf::SqfAnalyzer;
use tokio::net::TcpStream;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tracing::{Level, debug, info};

use crate::diag_manager::DiagManager;
use crate::workspace::EditorWorkspaces;

mod color;
mod config;
mod diag_manager;
mod files;
mod positions;
mod preprocessor;
pub mod sqf;
mod workspace;

#[derive(Clone, clap::Args)]
pub struct Command {
    port: u16,
}

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
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string()]),
                    retrigger_characters: Some(vec![",".to_string(), ")".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                definition_provider: Some(OneOf::Left(true)),
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
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["hemtt/processed".to_string()],
                    ..Default::default()
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
            EditorWorkspaces::get().initialize(folders, self.client.clone());
        }
        info!("initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        debug!("did_change_workspace_folders: {:?}", params);
        EditorWorkspaces::get().changed(params, self.client.clone());
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("did_change_configuration");
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        debug!("did_change_watched_files");
        for x in params.changes {
            if x.uri.path().contains(".toml") {
                ConfigAnalyzer::get()
                    .on_save(x.uri.clone(), self.client.clone())
                    .await;
                SqfAnalyzer::get()
                    .on_save(x.uri.clone(), self.client.clone())
                    .await;
            }
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("did_open: {:?}", params.text_document.uri);
        let document = TextDocumentItem {
            uri: params.text_document.uri.clone(),
            text: TextInformation::Full(&params.text_document.text),
            version: Some(params.text_document.version),
        };
        FileCache::get().on_change(&document).await;
        ConfigAnalyzer::get()
            .on_open(params.text_document.uri.clone(), self.client.clone())
            .await;
        SqfAnalyzer::get()
            .on_open(params.text_document.uri, self.client.clone())
            .await;
        SqfAnalyzer::get().on_change(&document).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
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
        ConfigAnalyzer::get()
            .on_save(params.text_document.uri.clone(), self.client.clone())
            .await;
        SqfAnalyzer::get()
            .on_save(params.text_document.uri.clone(), self.client.clone())
            .await;
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
        FileCache::get().on_close(&params.text_document.uri).await;
        SqfAnalyzer::get().on_close(&params.text_document.uri).await;
        PreprocessorAnalyzer::get()
            .on_close(&params.text_document.uri)
            .await;
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

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        Ok(PreprocessorAnalyzer::get().signature_help(&params).await)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(PreprocessorAnalyzer::get().goto_definition(&params).await)
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

impl Backend {
    async fn processed(&self, params: ProviderParams) -> Result<Option<Value>> {
        let Some(res) = PreprocessorAnalyzer::get().get_processed(params.url).await else {
            return Ok(None);
        };
        Ok(Some(serde_json::to_value(res).unwrap()))
    }

    async fn compiled(&self, params: ProviderParams) -> Result<Option<Value>> {
        let Some(res) = SqfAnalyzer::get().get_compiled(params.url).await else {
            return Ok(None);
        };
        Ok(Some(serde_json::to_value(res).unwrap()))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ProviderParams {
    url: Url,
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::DEBUG)
        .init();

    server().await;
}

async fn server() {
    // second argument is the port
    let port = std::env::args().nth(1).expect("port is required");

    let stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    info!("connected to server");

    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::build(|client| Backend { client })
        .custom_method("hemtt/processed", Backend::processed)
        .custom_method("hemtt/compiled", Backend::compiled)
        .finish();
    Server::new(read, write, socket).serve(service).await;
}
