use tracing::error;
use url::Url;

use crate::Backend;

pub fn json(url: &Url) -> Result<serde_json::Value, String> {
    let path = url
        .to_file_path()
        .map_err(|()| "Only file URLs are supported".to_string())?;
    let mut file = fs_err::File::open(&path).map_err(|_| "File not found".to_string())?;
    let p3d = hemtt_p3d::P3D::read(&mut file).map_err(|e| format!("{e:?}"))?;
    serde_json::to_value(&p3d).map_err(|e| format!("{e:?}"))
}

impl Backend {
    #[expect(clippy::unused_async, reason = "required by callsite")]
    pub async fn p3d_json(
        &self,
        params: JsonParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        match json(&params.url) {
            Ok(res) => Ok(Some(
                serde_json::to_value(res).expect("Serialization failed"),
            )),
            Err(e) => {
                error!("Error converting p3d to json: {}", e);
                Ok(None)
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonParams {
    url: Url,
}
