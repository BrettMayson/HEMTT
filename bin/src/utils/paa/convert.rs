use std::path::PathBuf;

use crate::Error;

#[derive(clap::Args)]
/// Convert images to and from PAA format
pub struct PaaConvertArgs {
    /// Source file (PAA or image)
    src: String,
    /// Destination file (PAA or image)
    dest: String,
}

fn invalid_data(message: impl Into<String>) -> Error {
    Error::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    ))
}

/// Execute the convert command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &PaaConvertArgs) -> Result<(), Error> {
    let from = PathBuf::from(&args.src);
    let output = PathBuf::from(&args.dest);
    if output.exists() {
        return Err(Error::OutputExists(args.dest.clone()));
    }
    if ["paa", "pac"].contains(
        &from
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
    ) {
        let paa = hemtt_paa::Paa::read(fs_err::File::open(from)?)?;
        let Some(map) = paa.maps().first() else {
            return Err(invalid_data("PAA has no maps"));
        };
        let image = map.0.get_image().map_err(invalid_data)?;
        image.save(output)?;
        info!("PAA converted");
    } else {
        let image = image::open(from)?;
        let format = hemtt_paa::PaXType::for_dynamic(&image);
        let continue_conversion = confirm_format(format, &args.src)?;
        if !continue_conversion {
            return Ok(());
        }
        let paa = hemtt_paa::Paa::from_dynamic(&image, format)?;
        let mut file = fs_err::File::create(output)?;
        paa.write(&mut file)?;
        info!("Image converted to {} PAA", format.to_string());
    }
    Ok(())
}

/// Confirm whether to use the selected format.
///
/// # Errors
/// Returns an error if the confirmation prompt fails.
pub fn confirm_format(format: hemtt_paa::PaXType, path: &str) -> Result<bool, Error> {
    if format == hemtt_paa::PaXType::ARGB8 {
        warn!("Using ARGB8 format for non-power-of-two image: {}", path);
        warn!("This is likely unintended and will not work as a standard texture.");
        if !dialoguer::Confirm::new()
            .with_prompt("Continue?")
            .default(false)
            .interact()?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{PaaConvertArgs, execute};
    use crate::Error;

    #[test]
    fn existing_output_is_an_error() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        image::RgbaImage::new(4, 4)
            .save("texture.png")
            .expect("write png");
        fs_err::write("texture_co.paa", "keep").expect("write output");

        let result = execute(&PaaConvertArgs {
            src: "texture.png".to_string(),
            dest: "texture_co.paa".to_string(),
        });
        assert!(
            matches!(&result, Err(Error::OutputExists(path)) if path == "texture_co.paa"),
            "expected an output exists error, got {result:?}"
        );
        assert_eq!(
            fs_err::read_to_string("texture_co.paa").expect("read"),
            "keep"
        );
    }
}
