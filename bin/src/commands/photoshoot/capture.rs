use std::sync::{Arc, Mutex};

use image::{DynamicImage, GenericImageView as _};
use scap::capturer::Capturer;

pub struct Capture {
    latest_frame: Arc<Mutex<Option<scap::frame::Frame>>>,
}

impl Capture {
    pub fn new() -> Result<Self, String> {
        if !scap::is_supported() {
            return Err("Platform not supported".into());
        }

        if !scap::has_permission() && !scap::request_permission() {
            return Err("Permission denied".into());
        }

        // let targets = scap::get_all_targets();

        let options = scap::capturer::Options {
            fps: 4,
            target: None, // None captures the primary display
            output_type: scap::frame::FrameType::BGRAFrame,
            ..Default::default()
        };

        let mut capturer = Capturer::build(options).map_err(|e| e.to_string())?;
        capturer.start_capture();

        let latest_frame = Arc::new(Mutex::new(None));
        let latest_frame_clone = latest_frame.clone();

        std::thread::spawn(move || {
            loop {
                if let Ok(frame) = capturer.get_next_frame() {
                    let mut latest = latest_frame_clone
                        .lock()
                        .expect("Failed to lock latest_frame");
                    *latest = Some(frame);
                }
            }
        });

        Ok(Self { latest_frame })
    }

    /// Detect non-black pixels to find the actual window bounds
    fn find_window_bounds(image: &DynamicImage) -> (u32, u32, u32, u32) {
        let (width, height) = image.dimensions();
        let mut min_x = width;
        let mut max_x = 0;
        let mut min_y = height;
        let mut max_y = 0;

        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                // Check if pixel is not pure black (RGB != 0,0,0)
                // Account for alpha channel by checking RGB components
                if pixel[0] > 0 || pixel[1] > 0 || pixel[2] > 0 {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }

        if min_x <= max_x && min_y <= max_y {
            (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
        } else {
            // No non-black pixels found, return original image dimensions
            (0, 0, width, height)
        }
    }

    pub fn screenshot(&self) -> Option<DynamicImage> {
        let frame = self
            .latest_frame
            .lock()
            .expect("Failed to lock latest_frame")
            .clone()?;
        match &frame {
            scap::frame::Frame::YUVFrame { .. } => {
                println!("Captured YUV Frame");
            }
            scap::frame::Frame::RGB { .. } => {
                println!("Captured RGB Frame");
            }
            scap::frame::Frame::RGBx { .. } => {
                println!("Captured RGBx Frame");
            }
            scap::frame::Frame::XBGR { .. } => {
                println!("Captured XBGR Frame");
            }
            scap::frame::Frame::BGRx { .. } => {
                println!("Captured BGRx Frame");
            }
            scap::frame::Frame::BGR0 { .. } => {
                println!("Captured BGR0 Frame");
            }
            scap::frame::Frame::BGRA { .. } => {
                println!("Captured BGRA Frame");
            }
        }
        #[allow(clippy::cast_sign_loss)]
        if let scap::frame::Frame::BGRx(frame_data) = frame {
            let rgb = scap::frame::convert_bgra_to_rgb(frame_data.data);
            println!("Converted frame to RGB with size: {}", rgb.len());
            let image = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                frame_data.width as u32,
                frame_data.height as u32,
                rgb,
            )
            .expect("Failed to create image buffer")
            .into();

            // Crop to actual window bounds to remove black padding on Wayland
            let (x, y, width, height) = Self::find_window_bounds(&image);
            Some(
                image::imageops::crop_imm(&image, x, y, width, height)
                    .to_image()
                    .into(),
            )
        } else {
            println!("Frame is not in BGRx format");
            None
        }
    }
}
