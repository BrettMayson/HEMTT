mod compiled;
mod hover;
mod lints;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::{addons::DefinedFunctions, reporting::Token};
use tower_lsp::Client;
use tracing::warn;
use url::Url;

use crate::{
    TextDocumentItem,
    files::FileCache,
    workspace::{EditorWorkspace, EditorWorkspaces},
};

#[derive(Clone)]
pub struct SqfAnalyzer {
    tokens: Arc<DashMap<Url, Vec<Arc<Token>>>>,
    databases: Arc<DashMap<EditorWorkspace, Arc<Database>>>,
    pub(crate) functions_defined: Arc<DashMap<String, HashMap<String, DefinedFunctions>>>,
}

impl SqfAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<SqfAnalyzer> = LazyLock::new(|| SqfAnalyzer {
            tokens: Arc::new(DashMap::new()),
            databases: Arc::new(DashMap::new()),
            functions_defined: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub async fn on_change(&self, document: &TextDocumentItem<'_>) {
        if !std::path::Path::new(document.uri.path())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqf"))
        {
            return;
        }
        let Some(workspace) = EditorWorkspaces::get()
            .guess_workspace_retry(&document.uri)
            .await
        else {
            warn!("Failed to find workspace for {:?}", document.uri);
            return;
        };
        let source = match workspace.join_url(&document.uri) {
            Ok(source) => source,
            Err(e) => {
                warn!("Failed to join url {:?}: {}", document.uri, e);
                return;
            }
        };
        let text = FileCache::get().text(&document.uri).unwrap_or_default();
        let Ok(tokens) = hemtt_preprocessor::parse::str(&text, &source) else {
            warn!("Failed to parse file");
            return;
        };
        self.tokens.insert(document.uri.clone(), tokens);
    }

    pub fn workspace_added(&self, workspace: &EditorWorkspace, client: Client) {
        self.check_lints(workspace, client);
    }

    pub async fn on_open(&self, url: Url, client: Client) {
        self.partial_recheck_lints(url, client).await;
    }

    pub async fn on_save(&self, url: Url, client: Client) {
        self.partial_recheck_lints(url, client).await;
    }

    pub fn on_close(&self, url: &Url) {
        self.tokens.remove(url);
    }

    /// The command database for a workspace, including its custom commands.
    ///
    /// Falling back to [`Database::a3`] would drop the workspace's custom
    /// commands, and the parser treats an unknown identifier as a variable, so
    /// every use of one would produce a false diagnostic. Reporting nothing is
    /// better than reporting wrong.
    fn get_database(&self, workspace: &EditorWorkspace) -> Result<Arc<Database>, hemtt_sqf::Error> {
        if let Some(database) = self.databases.get(workspace) {
            return Ok(database.clone());
        }
        let database = Arc::new(Database::a3_with_workspace(workspace.root(), false)?);
        self.databases
            .insert(workspace.clone(), Arc::clone(&database));
        Ok(database)
    }
}

#[cfg(test)]
mod tests {
    use super::SqfAnalyzer;
    use crate::workspace::tests::fixture;

    #[test]
    fn database_is_cached_per_workspace() {
        let analyzer = SqfAnalyzer::get();
        let workspace = fixture("project");
        let first = analyzer.get_database(&workspace).expect("builds");
        let second = analyzer.get_database(&workspace).expect("builds");
        assert!(std::sync::Arc::ptr_eq(&first, &second));
    }

    /// A folder with no project has no custom commands to load.
    #[test]
    fn database_without_project() {
        SqfAnalyzer::get()
            .get_database(&fixture("plain"))
            .expect("builds");
    }

    /// A custom command that cannot be read used to fall back to the stock
    /// database, which drops every custom command the project defines and so
    /// reports uses of them as variables. Nothing must be linted instead.
    #[test]
    fn database_with_unreadable_custom_command() {
        let Err(error) = SqfAnalyzer::get().get_database(&fixture("badcommands")) else {
            panic!("bad.txt is not valid utf-8, the database should not build");
        };
        assert!(
            matches!(error, hemtt_sqf::Error::CustomCommandIo(_)),
            "{error:?}"
        );
    }
}
