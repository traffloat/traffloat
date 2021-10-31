use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use arcstr::ArcStr;
use derive_new::new;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ImageBitmap, WebGlRenderingContext, WebGlTexture};

use super::util::UniformLocation;
use crate::util::ReifiedPromise;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, new)]
pub struct AtlasId {
    // micro optimization: put sprite_id before variant_name for faster comparison.
    spritesheet_id: u32,
    variant_name:   ArcStr,
}

pub struct Pool {
    context_path: String,
    dummy:        Rc<Texture>,
    map:          RefCell<BTreeMap<AtlasId, Atlas>>,
}

impl Pool {
    pub fn new(gl: &WebGlRenderingContext, context_path: String) -> Self {
        let dummy = {
            let texture = gl.create_texture().expect("Failed to create WebGL texture");
            gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

            // Buffers a single 00 byte as the image content,
            // i.e. a one-pixel transparent image.
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGlRenderingContext::TEXTURE_2D,    // target
                0,                                    // level
                WebGlRenderingContext::ALPHA as i32,  // internalFormat
                1,                                    // width
                1,                                    // height
                0,                                    // border
                WebGlRenderingContext::ALPHA,         // format
                WebGlRenderingContext::UNSIGNED_BYTE, // type
                Some(b"\0"),
            )
            .expect("Failed to initialize dummy WebGL texture");

            Rc::new(Texture { texture })
        };

        Self { context_path, dummy, map: RefCell::default() }
    }

    pub fn resolve(&self, gl: &WebGlRenderingContext, id: &AtlasId) -> Rc<Texture> {
        let mut map = self.map.borrow_mut();
        let atlas = map.entry(id.clone()).or_insert_with(|| {
            let url = format!(
                "{}/assets/{}/{:08x}.png",
                &self.context_path, &id.variant_name, id.spritesheet_id
            );
            let promise = ReifiedPromise::new(load_textures(&url), ());
            Atlas { fsm: RefCell::new(AtlasFsm::Loading(promise)) }
        });

        atlas.check_ready(gl);

        match &*atlas.fsm.borrow() {
            AtlasFsm::Loading(_) => Rc::clone(&self.dummy),
            AtlasFsm::Ready(texture) => Rc::clone(&texture),
        }
    }
}

struct Atlas {
    fsm: RefCell<AtlasFsm>,
}

impl Atlas {
    fn check_ready(&self, gl: &WebGlRenderingContext) {
        let mut fsm = self.fsm.borrow_mut();

        if let AtlasFsm::Loading(promise) = &*fsm {
            let resolve = promise.resolved_or_null().expect("Error retrieving texture");

            if let Some(bitmap) = resolve {
                let bitmap: &ImageBitmap =
                    bitmap.dyn_ref().expect("Promise did not return a bitmap");

                let texture = gl.create_texture().expect("Failed to prepare WebGL texture");
                gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

                gl.tex_image_2d_with_u32_and_u32_and_image_bitmap(
                    WebGlRenderingContext::TEXTURE_2D,    // target,
                    0,                                    // level
                    WebGlRenderingContext::RGBA as i32,   // internalformat
                    WebGlRenderingContext::RGBA,          // format
                    WebGlRenderingContext::UNSIGNED_BYTE, // type
                    bitmap,                               // pixels
                )
                .expect("Failed to initialize WebGL texture");
                gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);

                *fsm = AtlasFsm::Ready(Rc::new(Texture { texture }));
            }
        }
    }
}

enum AtlasFsm {
    Loading(ReifiedPromise<JsValue>),
    Ready(Rc<Texture>),
}

pub struct Texture {
    texture: WebGlTexture,
}

impl Texture {
    pub fn apply(&self, gl: &WebGlRenderingContext, tex_unif: &UniformLocation<i32>) {
        gl.active_texture(WebGlRenderingContext::TEXTURE0);
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&self.texture));
        tex_unif.assign(gl, 0);
    }
}

#[wasm_bindgen(module = "/js/bitmap.js")]
extern "C" {
    fn load_textures(url: &str) -> JsValue;
}
