use std::str::FromStr;

use config::ConfigAnalyzer;
use files::FileCache;
use preprocessor::PreprocessorAnalyzer;
use serde_json::Value;
use sqf::SqfAnalyzer;
use tokio::net::TcpStream;
use tower_lsp::jsonrpc::Result;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[allow(clippy::wildcard_imports)]
use tower_lsp::lsp_types::*;

use tracing::{Level, debug, info};

use crate::diag_manager::DiagManager;
use crate::workspace::EditorWorkspaces;

mod audio;
mod color;
mod completion;
mod config;
mod diag_manager;
mod files;
mod p3d;
mod paa;
mod positions;
mod preprocessor;
mod rpt;
mod sqf;
mod workspace;

#[derive(Clone, clap::Args)]
pub struct Command {
    port: u16,
}

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let did_filter = FileOperationRegistrationOptions {
            filters: vec![FileOperationFilter {
                scheme: Some(String::from("file")),
                pattern: FileOperationPattern {
                    glob: "**/*.*".to_string(),
                    matches: Some(FileOperationPatternKind::File),
                    options: None,
                },
            }],
        };
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
                    file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                        did_create: Some(did_filter.clone()),
                        did_rename: Some(did_filter.clone()),
                        did_delete: Some(did_filter),
                        ..Default::default()
                    }),
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string()]),
                    retrigger_characters: Some(vec![",".to_string(), ")".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "(".to_string(),
                        ",".to_string(),
                        "\\".to_string(),
                    ]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                    completion_item: None,
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
        if let Some(folders) = self
            .client
            .workspace_folders()
            .await
            .expect("Failed to get workspace folders")
        {
            EditorWorkspaces::get().initialize(folders, &self.client);
        }
        info!("initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        debug!("did_change_workspace_folders: {:?}", params);
        EditorWorkspaces::get().changed(params, &self.client);
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("did_change_configuration");
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        for file in params.files {
            ConfigAnalyzer::get()
                .on_save(
                    Url::from_str(&file.uri).expect("Failed to parse URL"),
                    self.client.clone(),
                )
                .await;
            SqfAnalyzer::get()
                .on_save(
                    Url::from_str(&file.uri).expect("Failed to parse URL"),
                    self.client.clone(),
                )
                .await;
        }
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        for file in params.files {
            ConfigAnalyzer::get()
                .on_save(
                    Url::from_str(&file.uri).expect("Failed to parse URL"),
                    self.client.clone(),
                )
                .await;
            SqfAnalyzer::get()
                .on_save(
                    Url::from_str(&file.uri).expect("Failed to parse URL"),
                    self.client.clone(),
                )
                .await;
        }
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        for file in params.files {
            ConfigAnalyzer::get()
                .on_save(
                    Url::from_str(&file.old_uri).expect("Failed to parse URL"),
                    self.client.clone(),
                )
                .await;
            SqfAnalyzer::get()
                .on_save(
                    Url::from_str(&file.old_uri).expect("Failed to parse URL"),
                    self.client.clone(),
                )
                .await;
            ConfigAnalyzer::get()
                .on_save(
                    Url::from_str(&file.new_uri).expect("Failed to parse URL"),
                    self.client.clone(),
                )
                .await;
            SqfAnalyzer::get()
                .on_save(
                    Url::from_str(&file.new_uri).expect("Failed to parse URL"),
                    self.client.clone(),
                )
                .await;
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = TextDocumentItem {
            uri: params.text_document.uri.clone(),
            text: TextInformation::Full(&params.text_document.text),
            version: Some(params.text_document.version),
        };
        FileCache::get().on_change(&document);
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
        FileCache::get().on_change(&document);
        SqfAnalyzer::get().on_change(&document).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
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
            FileCache::get().on_change(&document);
            SqfAnalyzer::get().on_change(&document).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        FileCache::get().on_close(&params.text_document.uri);
        SqfAnalyzer::get().on_close(&params.text_document.uri);
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
        Ok(color::info(&params.text_document.uri))
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        Ok(color::presentation(&params))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let (_, ext) = uri
            .path()
            .rsplit_once('.')
            .unwrap_or_else(|| (uri.path(), ""));
        let ext = ext.to_lowercase();
        if ["sqf", "ext", "cpp", "hpp", "inc"].contains(&ext.as_str()) {
            return completion::completion(params.text_document_position, params.context).await;
        }
        Ok(None)
    }
}

impl Backend {
    async fn processed(&self, params: ProviderParams) -> Result<Option<Value>> {
        let Some(res) = PreprocessorAnalyzer::get().get_processed(params.url).await else {
            return Ok(None);
        };
        Ok(Some(
            serde_json::to_value(res).expect("Failed to serialize processed result"),
        ))
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

    let stream = TcpStream::connect(format!("127.0.0.1:{port}"))
        .await
        .expect("Failed to connect to server");

    info!("connected to server");

    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::build(|client| Backend { client })
        .custom_method("hemtt/audio/convert", Backend::audio_convert)
        .custom_method("hemtt/p3d/json", Backend::p3d_json)
        .custom_method("hemtt/paa/json", Backend::paa_json)
        .custom_method("hemtt/paa/p3d", Backend::paa_p3d)
        .custom_method("hemtt/paa/convert", Backend::paa_convert)
        .custom_method("hemtt/processed", Backend::processed)
        .custom_method("hemtt/sqf/compiled", Backend::sqf_compiled)
        // .custom_method("hemtt/rpt/locate", Backend::locate_rpt)
        .finish();
    Server::new(read, write, socket).serve(service).await;
}
