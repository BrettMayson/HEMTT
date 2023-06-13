use image::Rgba;
use std::path::Path;

use crate::error::Error;

pub struct Photoshoot {}

impl Photoshoot {
    pub fn image(name: &str, from: &Path, to: &Path) -> Result<(), Error> {
        let path = from.join(format!("{name}.png"));
        let mut new = image::open(path)?.into_rgba8();
        let crop = 612;
        let new = image::imageops::crop(&mut new, (1024 - crop) / 2, 768 - crop, crop, crop);
        let mut new = image::imageops::resize(
            &new.to_image(),
            512,
            512,
            image::imageops::FilterType::Nearest,
        );
        for pixel in new.pixels_mut() {
            if is_background(pixel) {
                pixel.0[0] = 0;
                pixel.0[1] = 0;
                pixel.0[2] = 0;
                pixel.0[3] = 0;
                continue;
            }
        }
        new.save(to.join(format!("{name}.png")))?;
        Ok(())
    }
}

fn is_background(pixel: &mut Rgba<u8>) -> bool {
    (pixel.0[0] >= 253 && pixel.0[1] <= 200 && pixel.0[2] >= 253)
        || (pixel.0[0] == 0 && pixel.0[1] >= 30 && pixel.0[2] == 0)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Photoshoot;

    #[test]
    fn uniforms() {
        let from = PathBuf::from("tests/photoshoot");
        let to = PathBuf::from("tests/photoshoot/out");
        println!("{}", std::env::current_dir().unwrap().display());
        Photoshoot::image("U_B_CombatUniform_mcam", &from, &to).unwrap();
        Photoshoot::image("U_C_Journalist", &from, &to).unwrap();
        Photoshoot::image("U_C_Man_casual_6_F", &from, &to).unwrap();
        Photoshoot::image("U_B_ParadeUniform_01_US_F", &from, &to).unwrap();
    }
}
