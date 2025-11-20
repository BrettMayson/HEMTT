use std::path::PathBuf;

use image::GenericImageView;
use tracing::error;
use url::Url;

use crate::{Backend, workspace::EditorWorkspaces};

pub fn json(url: &Url) -> Result<serde_json::Value, String> {
    let path = url
        .to_file_path()
        .map_err(|()| "Only file URLs are supported".to_string())?;
    let mut file = std::fs::File::open(&path).map_err(|_| "File not found".to_string())?;
    let value: serde_json::Value = serde_json::from_str(
        hemtt_paa::Paa::read(&mut file)
            .map_err(|e| format!("{e:?}"))?
            .json()?
            .as_str(),
    )
    .map_err(|e| format!("{e:?}"))?;
    Ok(value)
}

pub fn convert(url: &Url, to: &str, out: Option<String>) -> Result<PathBuf, String> {
    let path = url
        .to_file_path()
        .map_err(|()| "Only file URLs are supported".to_string())?;
    let output = out.map_or_else(|| path.with_extension(to), std::path::PathBuf::from);
    if ["paa", "pac"].contains(
        &path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
    ) {
        let paa = hemtt_paa::Paa::read(std::fs::File::open(path).map_err(|e| format!("{e:?}"))?)
            .map_err(|e| format!("{e:?}"))?;
        if let Err(e) = paa.maps()[0].0.get_image().save(&output) {
            error!("Failed to save image: {}", e);
            return Err(format!("Failed to save image: {e}"));
        }
    } else {
        let image = image::open(path).map_err(|e| format!("{e:?}"))?;
        let paa = hemtt_paa::Paa::from_dynamic(&image, {
            let (width, height) = image.dimensions();
            if !height.is_power_of_two() || !width.is_power_of_two() {
                hemtt_paa::PaXType::ARGB8
            } else {
                let has_transparency = image.pixels().any(|p| p.2[3] < 255);
                if has_transparency {
                    hemtt_paa::PaXType::DXT5
                } else {
                    hemtt_paa::PaXType::DXT1
                }
            }
        })
        .map_err(|e| format!("{e:?}"))?;
        let mut file = std::fs::File::create(&output)
            .map_err(|e| format!("Failed to create output file: {e}"))?;
        paa.write(&mut file).map_err(|e| format!("{e:?}"))?;
    }
    Ok(output)
}

impl Backend {
    #[expect(clippy::unused_async, reason = "required by callsite")]
    pub async fn paa_json(
        &self,
        params: JsonParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        match json(&params.url) {
            Ok(res) => Ok(Some(
                serde_json::to_value(res).expect("Serialization failed"),
            )),
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
        let source = if let Ok(Some(source)) = workspace.root().locate_with_pdrive(&params.texture)
        {
            source
        } else {
            let texture = format!("\\{}", &params.texture);
            tracing::debug!("Locating {:?}", &texture);
            if let Ok(Some(source)) = workspace.root().locate_with_pdrive(&texture) {
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
        let Ok(paa) = hemtt_paa::Paa::read(&mut source).map_err(|e| format!("{e:?}")) else {
            tracing::warn!("Failed to read paa {:?}", params.url);
            return Ok(None);
        };
        Ok(Some(
            serde_json::to_value(paa.maps().first().expect("No maps found").0.json())
                .expect("Serialization failed"),
        ))
    }

    #[expect(clippy::unused_async, reason = "required by callsite")]
    pub async fn paa_convert(
        &self,
        params: ConvertParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        match convert(&params.url, &params.to, params.out) {
            Ok(res) => Ok(Some(
                serde_json::to_value(res).expect("Serialization failed"),
            )),
            Err(e) => {
                error!("Error converting image: {e}");
                Ok(None)
            }
        }
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

#[derive(Debug, serde::Deserialize)]
pub struct ConvertParams {
    url: Url,
    to: String,
    out: Option<String>,
}
