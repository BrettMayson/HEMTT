use tracing::error;
use url::Url;

use crate::Backend;

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
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonParams {
    url: Url,
}
