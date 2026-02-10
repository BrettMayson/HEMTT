use std::io::Cursor;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct FromPaaResult {
    data: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
}

#[wasm_bindgen]
impl FromPaaResult {
    #[wasm_bindgen(constructor)]
    pub fn new(s: &Uint8Array) -> Self {
        let bytes = s.to_vec();
        let paa = crate::Paa::read(Cursor::new(bytes)).expect("Failed to read PAA");
        let mut buffer = Cursor::new(Vec::new());
        paa.maps()[0]
            .0
            .get_image()
            .write_to(&mut buffer, image::ImageFormat::Png)
            .expect("Failed to write PNG");

        Self {
            data: std::rc::Rc::new(std::cell::RefCell::from(buffer.into_inner())),
        }
    }

    pub fn data_ptr(&self) -> *const u8 {
        self.data.clone().borrow().as_ptr()
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn data_len(&self) -> u32 {
        self.data.clone().borrow().len() as u32
    }
}

#[cfg(feature = "generate")]
use image::GenericImageView;

#[cfg(feature = "generate")]
#[wasm_bindgen]
pub struct ToPaaResult {
    data: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
    format: String,
}

#[cfg(feature = "generate")]
#[wasm_bindgen]
impl ToPaaResult {
    #[wasm_bindgen(constructor)]
    pub fn new(s: &Uint8Array) -> Self {
        let bytes = s.to_vec();
        let img = image::load_from_memory(&bytes).expect("Failed to load image from memory");
        let (width, height) = img.dimensions();
        let format = if !height.is_power_of_two() || !width.is_power_of_two() {
            crate::PaXType::ARGB8
        } else {
            let has_transparency = img.pixels().any(|p| p.2[3] < 255);
            if has_transparency {
                crate::PaXType::DXT5
            } else {
                crate::PaXType::DXT1
            }
        };
        let paa = crate::Paa::from_dynamic(&img, format).expect("Failed to create PAA from image");
        let mut buffer = Cursor::new(Vec::new());
        paa.write(&mut buffer).expect("Failed to write PAA");
        Self {
            data: std::rc::Rc::new(std::cell::RefCell::from(buffer.into_inner())),
            format: format.to_string(),
        }
    }

    pub fn data_ptr(&self) -> *const u8 {
        self.data.clone().borrow().as_ptr()
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn data_len(&self) -> u32 {
        self.data.clone().borrow().len() as u32
    }

    pub fn format(&self) -> String {
        self.format.clone()
    }
}
