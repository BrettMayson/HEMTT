use std::io::Cursor;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[wasm_bindgen]
pub struct ImageResult {
    data: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
}

#[wasm_bindgen]
impl ImageResult {
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
