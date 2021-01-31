use std::collections::BTreeMap;

use crate::util::ReifiedPromise;
use traffloat::shape::Texture;
use traffloat::types::Id;

#[derive(Default)]
pub struct ImageStore {
    images: BTreeMap<Id<Texture>, MaybeBitmap>,
}

impl ImageStore {
    pub fn fetch(&mut self, id: Id<Texture>, texture: &Texture) -> &MaybeBitmap {
        self.images
            .entry(id)
            .or_insert_with(|| create_bitmap(texture))
    }
}

fn create_bitmap(texture: &Texture) -> MaybeBitmap {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    #[wasm_bindgen(module = "/js/bitmap.js")]
    extern "C" {
        fn create_bitmap(url: &str) -> JsValue;
    }

    let promise = create_bitmap(texture.url.as_str());
    let promise = ReifiedPromise::new(promise, ());
    MaybeBitmap { promise }
}

pub struct MaybeBitmap {
    promise: ReifiedPromise<web_sys::ImageBitmap>,
}

impl MaybeBitmap {
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
