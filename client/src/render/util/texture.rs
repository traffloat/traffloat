use std::cell;
use std::rc::Rc;

use web_sys::{WebGlRenderingContext, WebGlTexture, ImageBitmap, WebGlUniformLocation};

use traffloat::config;
use crate::render::MaybeBitmap;

/// A 2D WebGL texture
pub struct Texture {
    cell: cell::RefCell<MaybeTexture>,
}

enum MaybeTexture {
    Pending(PendingTexture),
    Loaded(LoadedTexture),
}

struct PendingTexture {
    bitmap: Rc<MaybeBitmap>,
    texture: Option<WebGlTexture>,
}

struct LoadedTexture {
    texture: WebGlTexture,
}

impl Texture {
    /// Creates a 2D WebGL texture
    pub fn create(gl: &WebGlRenderingContext, bitmap: Rc<MaybeBitmap>) -> Self {
        let texture = gl.create_texture().expect("Failed to create WebGL texture");
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGlRenderingContext::TEXTURE_2D, // target
            0, // level
            WebGlRenderingContext::ALPHA as i32, // internalFormat
            1, // width
            1, // height
            0, // border
            WebGlRenderingContext::ALPHA, // format
            WebGlRenderingContext::UNSIGNED_BYTE, // type
            Some(b"\0"),
        ).expect("Failed to initialize to WebGL texture");

        Self {
            cell: cell::RefCell::new(MaybeTexture::Pending(PendingTexture {
                bitmap,
                texture: Some(texture),
            }))
        }
    }

    /// Retrieves a reference to the WebGlTexture object.
    ///
    /// The returned reference must be dropped before calling this method again.
    pub fn texture(&self, gl: &WebGlRenderingContext) -> cell::Ref<'_, WebGlTexture> {
        fn init_texture(gl: &WebGlRenderingContext, ib: &ImageBitmap, texture: &WebGlTexture) {
            gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(texture));
            gl.tex_image_2d_with_u32_and_u32_and_image_bitmap(
                WebGlRenderingContext::TEXTURE_2D, // target,
                0, // level
                WebGlRenderingContext::RGBA as i32, // internalformat
                WebGlRenderingContext::RGBA, // format
                WebGlRenderingContext::UNSIGNED_BYTE, // type
                ib, // pixels
            ).expect("Failed to assign WebGL texture");
            gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);
        }

        {
            let mut cell = self.cell.borrow_mut();
            if let MaybeTexture::Pending(pt) = &mut *cell {
                if let Some(ib) = pt.bitmap.resolve() {
                    let texture = pt.texture.take().expect("Dropped texture");
                    init_texture(gl, ib, &texture);
                    *cell = MaybeTexture::Loaded(LoadedTexture { texture });
                }
            }
        }

        cell::Ref::map(self.cell.borrow(), |mt| match mt {
            MaybeTexture::Pending(texture) => texture.texture.as_ref().expect("Dropped texture"),
            MaybeTexture::Loaded(texture) => &texture.texture,
        })
    }

    pub fn apply(&self, gl: &WebGlRenderingContext, uniform_location: &WebGlUniformLocation) {
        gl.active_texture(WebGlRenderingContext::TEXTURE0);
        {
            let texture = self.texture(gl);
            gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&*texture));
        }
        gl.uniform1i(Some(uniform_location), 0);
    }
}
