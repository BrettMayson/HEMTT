use std::path::PathBuf;

use hemtt_wss::Compression;

use crate::Error;

use super::{SupportedFile, guess_file_type};

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
/// Convert between audio formats
pub struct ConvertArgs {
    /// File to convert (wss, wav, ogg, or mp3)
    pub(crate) file: String,
    /// Output file path or new extension (e.g., "wss" or "output.wss")
    pub(crate) output: String,
    /// Compression level for WSS output (0, 4, 8), 8 is recommended
    #[arg(long, short)]
    pub(crate) compression: Option<u8>,
}

/// Convert an audio file
pub fn convert(file: &PathBuf, output: &str, compression: Option<u8>) -> Result<(), Error> {
    let compression = Compression::from_u32(u32::from(compression.unwrap_or(0)))?;
    let mut wss = match guess_file_type(file)? {
        Some(SupportedFile::Wss) => hemtt_wss::Wss::read(std::fs::File::open(file)?)?,
        Some(SupportedFile::Wav) => hemtt_wss::Wss::from_wav(std::fs::File::open(file)?)?,
        Some(SupportedFile::Ogg) => hemtt_wss::Wss::from_ogg(std::fs::File::open(file)?)?,
        Some(SupportedFile::Mp3) => hemtt_wss::Wss::from_mp3(std::fs::File::open(file)?)?,
        _ => {
            println!("Unsupported file type");
            return Ok(());
        }
    };
    let (extension, output) = if output.contains('.') {
        let output = PathBuf::from(output);
        (
            output
                .extension()
                .expect("No extension")
                .to_str()
                .expect("extension is not valid")
                .to_string(),
            output,
        )
    } else {
        let output_file = file.with_extension(output);
        (output.to_string(), output_file)
    };

    let data = match extension.as_str() {
        "wss" => {
            let mut buffer = Vec::new();
            wss.set_compression(compression);
            wss.write(&mut buffer)?;
            buffer
        }
        "wav" => wss.to_wav()?,
        "ogg" => wss.to_ogg()?,
        _ => {
            println!("Unsupported file type to convert to: {extension}");
            return Ok(());
        }
    };
    let _ = std::fs::create_dir_all(
        output
            .parent()
            .expect("Output file has no parent directory"),
    );
    std::io::Write::write_all(&mut std::fs::File::create(&output)?, &data)?;
    println!("Converted to: {}", output.display());
    Ok(())
}
