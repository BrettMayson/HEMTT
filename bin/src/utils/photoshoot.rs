use image::{ImageBuffer, Rgb, Rgba};
use std::path::Path;

use crate::error::Error;

pub struct Photoshoot {}

impl Photoshoot {
    /// Processes a weapon screenshot
    ///
    /// # Errors
    /// [`Error::Image`] if the image could not be loaded
    pub fn weapon(name: &str, from: &Path) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Error> {
        let path = from.join(format!("{name}.png"));
        let mut new = image::open(path)?.into_rgba8();
        let crop = 612;
        let new = image::imageops::crop(&mut new, (1280 - crop) / 2, 720 - crop, crop, crop);
        let mut new = image::imageops::resize(
            &new.to_image(),
            512,
            512,
            image::imageops::FilterType::Nearest,
        );
        for pixel in new.pixels_mut() {
            if is_background(*pixel) {
                pixel.0[0] = 0;
                pixel.0[1] = 0;
                pixel.0[2] = 0;
                pixel.0[3] = 0;
                continue;
            }
            Self::gamma_rgba(pixel);
        }
        Ok(new)
    }

    /// Processes an editor preview screenshot
    ///
    /// # Errors
    /// [`Error::Image`] if the image could not be loaded
    pub fn preview(path: &Path) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Error> {
        let new = image::open(path)?.into_rgb8();
        let mut new = image::imageops::resize(&new, 455, 256, image::imageops::FilterType::Nearest);
        for pixel in new.pixels_mut() {
            Self::gamma_rgb(pixel);
        }
        Ok(new)
    }

    // adjust gamma because Arma blasts the hell out of it
    fn gamma_rgba(pixel: &mut Rgba<u8>) {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        for i in 0..3 {
            pixel.0[i] = ((f32::from(pixel.0[i]) / 255.0).powf(1.8) * 255.0_f32) as u8;
        }
    }

    fn gamma_rgb(pixel: &mut Rgb<u8>) {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        for i in 0..3 {
            pixel.0[i] = ((f32::from(pixel.0[i]) / 255.0).powf(2.2) * 255.0_f32) as u8;
        }
    }
}

const fn is_background(pixel: Rgba<u8>) -> bool {
    (pixel.0[0] >= 240 && pixel.0[1] <= 200 && pixel.0[2] >= 240)
        || (pixel.0[0] == 0 && pixel.0[1] >= 30 && pixel.0[2] == 0)
}
