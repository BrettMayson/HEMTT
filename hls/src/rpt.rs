use hemtt_common::steam::find_app;
use tracing::info;

use crate::Backend;

impl Backend {
    pub async fn locate_rpt(
        &self,
        _:  serde_json::Value,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        info!("Locating Arma 3 installation path via Steam...");
        find_app(107_410)
            .map(|path| {
                info!("Located Arma 3 installation at {}", path.display());
                Some(serde_json::Value::String(path.to_string_lossy().to_string()))
            })
            .ok_or_else(|| {
                info!("Could not locate Arma 3 installation");
                tower_lsp::jsonrpc::Error::invalid_params("Could not locate Arma 3 installation")
            })
    }
}
