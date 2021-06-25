use std::cell;
use std::collections::BTreeMap;
use std::mem;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::{ImageBitmap, WebGlTexture, WebGlRenderingContext};

use traffloat::shape;
use crate::util::ReifiedPromise;

#[wasm_bindgen(module = "/js/bitmap.js")]
extern "C" {
    fn load_textures(url: &str) -> JsValue;

    fn get_bitmap(value: &JsValue) -> ImageBitmap;
    fn get_index(value: &JsValue) -> String;
}


pub struct Pool {
    map: cell::RefCell<BTreeMap<String, Rc<Atlas>>>,
    dummy: Rc<WebGlTexture>,
}

impl Pool {
    /// Initializes the texture pool and prepares a default dummy texture.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let texture = gl.create_texture().expect("Failed to create WebGL texture");
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));
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
        .expect("Failed to initialize to WebGL texture");

        Self {
            map: Default::default(),
            dummy: Rc::new(texture),
        }
    }

    fn load(&self, url: &str) -> Rc<Atlas> {
        let mut map = self.map.borrow_mut();
        let rc = map.entry(url.to_string())
            .or_insert_with(|| Rc::new(Atlas::load(url)));
        Rc::clone(rc)
    }

    /// Retrieves a sprite for the given texture, or returns the dummy texture
    pub fn sprite(&self, texture: &shape::Texture, gl: &WebGlRenderingContext) -> (Rc<WebGlTexture>, ShapeSprites) {
        let atlas = self.load(texture.url());
        atlas.get(texture.name(), gl).unwrap_or_else(|| {
            (
                Rc::clone(&self.dummy),
                ShapeSprites::Cube(DUMMY_CUBE_SPRITES)
            )
        })
    }
}

/// An atlas, which is a single image containing many small sprites.
pub struct Atlas(cell::RefCell<AtlasEnum>);

impl Atlas {
    /// Loads an atlas from the given URL.
    pub fn load(url: &str) -> Self {
        let promise = load_textures(url);
        let promise = ReifiedPromise::<JsValue>::new(promise, ());
        Self(cell::RefCell::new(AtlasEnum::Pending(promise)))
    }

    /// Gets information about a sprite if available.
    pub fn get(&self, name: &str, gl: &WebGlRenderingContext) -> Option<(Rc<WebGlTexture>, ShapeSprites)> {
        let mut ae = self.0.borrow_mut();
        ae.update(gl);
        if let AtlasEnum::Ready { index, texture } = &*ae {
            let sprites = index.sprites(name)?;
            Some((Rc::clone(&texture), sprites))
        } else {
            None
        }
    }
}

enum AtlasEnum {
    Pending(ReifiedPromise<JsValue>),
    Ready {
        index: Index,
        texture: Rc<WebGlTexture>,
    },
}

impl AtlasEnum {
    /// Ensures that the variant of the enum reflects the underlying [`ReifiedPromise`].
    fn update(&mut self, gl: &WebGlRenderingContext) {
        if let Self::Pending(promise) = self {
            let resolve = promise.resolved_or_null().expect("Failed resolving atlas");
            if let Some(value) = resolve {
                let (index, bitmap) = decompose_value(value);

                let texture = gl.create_texture().expect("Failed to create WebGL texture");
                gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));
                gl.tex_image_2d_with_u32_and_u32_and_image_bitmap(
                    WebGlRenderingContext::TEXTURE_2D,    // target,
                    0,                                    // level
                    WebGlRenderingContext::RGBA as i32,   // internalformat
                    WebGlRenderingContext::RGBA,          // format
                    WebGlRenderingContext::UNSIGNED_BYTE, // type
                    &bitmap,                                   // pixels
                )
                .expect("Failed to assign WebGL texture");
                gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);

                *self = Self::Ready {
                    index,
                    texture: Rc::new(texture),
                };
            }
        }
    }
}

/// Decompose the [`load_textures`] response into `index` and `bitmap` fields and parse the index.
fn decompose_value(value: &JsValue) -> (Index, ImageBitmap) {
    let bitmap = get_bitmap(value);
    let index_json = get_index(value);
    let index: Index = serde_json::from_str(&index_json).expect("Failed parsing texture index");
    (index, bitmap)
}

/// The loaded index of an atlas.
#[derive(serde::Deserialize)]
pub struct Index {
    #[serde(flatten)]
    items: BTreeMap<String, ShapeSprites>,
}

impl Index {
    /// Returns the information of a sprite in this atlas.
    pub fn sprites(&self, name: &str) -> Option<ShapeSprites> {
        todo!()
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeSprites {
    /// Cube
    Cube(CubeSprites),
}

#[derive(serde::Deserialize, getset::CopyGetters, Debug, Clone, Copy)]
pub struct CubeSprites {
    #[getset(get_copy = "pub")]
    xp: RectSprite,
    #[getset(get_copy = "pub")]
    xn: RectSprite,
    #[getset(get_copy = "pub")]
    yp: RectSprite,
    #[getset(get_copy = "pub")]
    yn: RectSprite,
    #[getset(get_copy = "pub")]
    zp: RectSprite,
    #[getset(get_copy = "pub")]
    zn: RectSprite,
}

/// A rectangular sprite.
#[derive(serde::Deserialize, getset::CopyGetters, Debug, Clone, Copy)]
pub struct RectSprite {
    /// Starting X-coordinate of the rectangle.
    #[getset(get_copy = "pub")]
    x: u32,
    /// Starting Y-coordinate of the rectangle.
    #[getset(get_copy = "pub")]
    y: u32,
    /// Width of the rectangle.
    #[getset(get_copy = "pub")]
    width: u32,
    /// Height of the rectangle.
    #[getset(get_copy = "pub")]
    height: u32,
}

const DUMMY_RECT_SPRITE: RectSprite = RectSprite {
    x: 0,
    y: 0,
    width: 1,
    height: 1,
};

const DUMMY_CUBE_SPRITES: CubeSprites = CubeSprites {
    xp: DUMMY_RECT_SPRITE,
    xn: DUMMY_RECT_SPRITE,
    yp: DUMMY_RECT_SPRITE,
    yn: DUMMY_RECT_SPRITE,
    zp: DUMMY_RECT_SPRITE,
    zn: DUMMY_RECT_SPRITE,
};
