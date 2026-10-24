use url::Url;

use crate::{Backend, workspace::EditorWorkspaces};
impl Backend {
    #[expect(
        clippy::unused_async,
        clippy::unused_async_trait_impl,
        reason = "required by callsite"
    )]
    pub async fn path_copy(
        &self,
        params: JsonParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        Ok(get_path(&params.url)
            .map(|res| serde_json::to_value(res).expect("Serialization failed")))
    }
}

pub fn get_path(url: &Url) -> Option<String> {
    tracing::trace!("get_path called with URL path: {}", url.path());
    let workspace = EditorWorkspaces::get().guess_workspace(url)?;
    let source = workspace.join_url(url).ok()?;
    let path = source.as_virtual_str().replace('/', "\\");
    tracing::trace!("get_path returning: {}", path);
    Some(path)
}
#[derive(Debug, serde::Deserialize)]
pub struct JsonParams {
    url: Url,
}
