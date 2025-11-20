use serde::Serialize;
use tracing::error;
use url::Url;

use crate::Backend;

#[derive(Debug, Serialize)]
pub struct WssInfo {
    pub path: String,
    pub channels: u16,
    #[serde(rename = "sampleRate")]
    pub sample_rate: u32,
    pub compression: String,
}

pub fn convert(url: &Url, to: &str, out: Option<String>) -> Result<WssInfo, String> {
    let path = url
        .to_file_path()
        .map_err(|()| "Only file URLs are supported".to_string())?;
    let output = out.map_or_else(|| path.with_extension(to), std::path::PathBuf::from);
    let file = fs_err::File::open(&path).map_err(|_| "File not found".to_string())?;
    let mut wss = match path
        .extension()
        .expect("Must have extension for command")
        .to_str()
        .expect("Extension must be valid")
    {
        "wss" => hemtt_wss::Wss::read(file),
        "wav" => hemtt_wss::Wss::from_wav(file),
        "ogg" => hemtt_wss::Wss::from_ogg(file),
        "mp3" => hemtt_wss::Wss::from_mp3(file),
        _ => {
            error!("Unsupported file type");
            return Err("Unsupported file type".to_string());
        }
    }
    .map_err(|e| format!("Error reading file: {e}"))?;

    let data = match to {
        "wss" => {
            let mut buffer = Vec::new();
            wss.set_compression(hemtt_wss::Compression::Nibble);
            wss.write(&mut buffer)
                .map_err(|e| format!("Error writing file: {e}"))?;
            Ok(buffer)
        }
        "wav" => wss.to_wav(),
        "ogg" => wss.to_ogg(),
        _ => {
            return Err("Unsupported file type to convert to".to_string());
        }
    }
    .map_err(|e| format!("Error converting file: {e}"))?;
    let mut out_file =
        fs_err::File::create(&output).map_err(|_| "Error creating file".to_string())?;
    std::io::Write::write_all(&mut out_file, &data)
        .map_err(|_| "Error writing file".to_string())?;
    Ok(WssInfo {
        path: output.to_string_lossy().to_string(),
        channels: wss.channels(),
        sample_rate: wss.sample_rate(),
        compression: wss.compression().as_str().to_string(),
    })
}

impl Backend {
    #[expect(clippy::unused_async, reason = "required by callsite")]
    pub async fn audio_convert(
        &self,
        params: ConvertParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        match convert(&params.url, &params.to, params.out) {
            Ok(res) => Ok(Some(
                serde_json::to_value(res).expect("Serialization failed"),
            )),
            Err(e) => {
                error!("Error converting audio: {e}");
                Ok(None)
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ConvertParams {
    url: Url,
    to: String,
    out: Option<String>,
}
