use std::collections::BTreeMap;
use std::rc::Rc;

use crate::util::ReifiedPromise;
use traffloat::config;
use traffloat::shape::Texture;

/// A resource that caches the [`ImageBitmap`][web_sys::ImageBitmap] requests.
#[derive(Default)]
pub struct ImageStore {
    images: BTreeMap<config::Id<Texture>, Rc<MaybeBitmap>>,
}

impl ImageStore {
    /// Fetches the bitmap for a [`Texture`].
    pub fn fetch(&mut self, id: config::Id<Texture>, texture: &Texture) -> Rc<MaybeBitmap> {
        let rc = self
            .images
            .entry(id)
            .or_insert_with(|| Rc::new(create_bitmap(texture)));
        Rc::clone(rc)
    }
}

fn create_bitmap(texture: &Texture) -> MaybeBitmap {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/bitmap.js")]
    extern "C" {
        fn create_bitmap(url: &str) -> JsValue;
    }

    let promise = create_bitmap(texture.url());
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

impl super::Image for MaybeBitmap {
    fn as_bitmap(&self) -> Option<&web_sys::ImageBitmap> {
        self.resolve()
    }
}
