use std::path::PathBuf;

use image::GenericImageView;

use crate::Error;

#[derive(clap::Args)]
/// Convert images to and from PAA format
pub struct PaaConvertArgs {
    /// Source file (PAA or image)
    src: String,
    /// Destination file (PAA or image)
    dest: String,
}

/// Execute the convert command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &PaaConvertArgs) -> Result<(), Error> {
    let from = PathBuf::from(&args.src);
    let output = PathBuf::from(&args.dest);
    if output.exists() {
        error!("Output file already exists");
        return Ok(());
    }
    if ["paa", "pac"].contains(
        &from
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
    ) {
        let paa = hemtt_paa::Paa::read(std::fs::File::open(from)?)?;
        if let Err(e) = paa.maps()[0].0.get_image().save(output) {
            error!("Failed to save image: {}", e);
        } else {
            info!("PAA converted");
        }
    } else {
        let image = image::open(from)?;
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
        })?;
        let mut file = std::fs::File::create(output)?;
        paa.write(&mut file)?;
        info!("Image converted to PAA");
    }
    Ok(())
}
