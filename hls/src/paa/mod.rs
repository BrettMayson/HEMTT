use tracing::error;
use url::Url;

use crate::{Backend, workspace::EditorWorkspaces};

pub fn json(url: Url) -> Result<serde_json::Value, String> {
    let path = url
        .to_file_path()
        .map_err(|_| "Only file URLs are supported".to_string())?;
    let mut file = std::fs::File::open(&path).map_err(|_| "File not found".to_string())?;
    let value: serde_json::Value = serde_json::from_str(
        hemtt_paa::Paa::read(&mut file)
            .map_err(|e| format!("{:?}", e))?
            .json()?
            .as_str(),
    )
    .map_err(|e| format!("{:?}", e))?;
    Ok(value)
}

impl Backend {
    pub async fn paa_json(
        &self,
        params: JsonParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        match json(params.url) {
            Ok(res) => Ok(Some(serde_json::to_value(res).unwrap())),
            Err(e) => {
                error!("Error converting paa to json: {}", e);
                Ok(None)
            }
        }
    }

    pub async fn paa_p3d(
        &self,
        params: P3dParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        let Some(workspace) = EditorWorkspaces::get()
            .guess_workspace_retry(&params.url)
            .await
        else {
            tracing::warn!("Failed to find workspace for {:?}", &params.url);
            return Ok(None);
        };
        tracing::debug!("Locating {:?}", &params.texture);
        let source = if let Ok(Some(source)) = workspace.root().locate(&params.texture) {
            source
        } else {
            let texture = format!("\\{}", &params.texture);
            tracing::debug!("Locating {:?}", &texture);
            if let Ok(Some(source)) = workspace.root().locate(&texture) {
                source
            } else {
                tracing::warn!("Failed to locate {:?}", &params.texture);
                return Ok(None);
            }
        };
        let Ok(mut source) = source.path.open_file() else {
            tracing::warn!("Failed to open file {:?}", source.path);
            return Ok(None);
        };
        let Ok(paa) = hemtt_paa::Paa::read(&mut source).map_err(|e| format!("{:?}", e)) else {
            tracing::warn!("Failed to read paa {:?}", params.url);
            return Ok(None);
        };
        Ok(Some(
            serde_json::to_value(paa.maps().first().unwrap().json()).unwrap(),
        ))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonParams {
    url: Url,
}

#[derive(Debug, serde::Deserialize)]
pub struct P3dParams {
    url: Url,
    texture: String,
}
