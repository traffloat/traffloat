use std::collections::BTreeMap;
use std::rc::Rc;

use crate::util::ReifiedPromise;
use traffloat::config;
use traffloat::shape::Texture;

/// A resource that caches the [`ImageBitmap`][web_sys::ImageBitmap] requests.
#[derive(Default)]
pub struct ImageStore {
    images: BTreeMap<String, Rc<MaybeBitmap>>,
}

impl ImageStore {
    /// Fetches the bitmap for a [`Texture`].
    pub fn fetch(&mut self, url: &str) -> Rc<MaybeBitmap> {
        let rc = self
            .images
            .entry(url.to_string())
            .or_insert_with(|| Rc::new(create_bitmap(url)));
        Rc::clone(rc)
    }
}

fn create_bitmap(url: &str) -> MaybeBitmap {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/bitmap.js")]
    extern "C" {
        fn create_bitmap(url: &str) -> JsValue;
    }

    let promise = create_bitmap(url);
    let promise = ReifiedPromise::new(promise, ());
    MaybeBitmap { promise }
}

/// A struct that wraps a possibly loaded bitmap.
pub struct MaybeBitmap {
    promise: ReifiedPromise<web_sys::ImageBitmap>,
}

impl MaybeBitmap {
    /// Resolves the [`ImageBitmap`][web_sys::ImageBitmap] if it has been loaded.
    pub fn resolve(&self) -> Option<&web_sys::ImageBitmap> {
        self.promise
            .resolved_or_null()
            .expect("Promise result is not an ImageBitmap")
    }
}
