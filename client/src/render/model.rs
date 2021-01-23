use std::sync::atomic::{AtomicUsize, Ordering};

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext};

use common::texture;
use common::types::*;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Component)]
#[storage(storage::BTreeStorage)]
pub struct LoadedBuffers {
    pub buffers: texture::Buffers,
    pub id: usize,
}

impl LoadedBuffers {
    pub fn new(model: texture::Model) -> Option<Result<LoadedBuffers, &'static str>> {
        let buffers = match model.to_buffers() {
            Ok(buffers) => buffers,
            Err(err) => return Some(Err(err)),
        };
        let buffers = LoadedBuffers {
            buffers,
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        };

        Some(Ok(buffers))
    }
}

pub struct LoadedModel {
    positions: WebGlBuffer,
    normals: WebGlBuffer,
    colors: WebGlBuffer,
    shininesses: WebGlBuffer,
    reflectances: WebGlBuffer,
    faces: WebGlBuffer,
    len: i32,
}

#[allow(clippy::unwrap_used)]
impl LoadedModel {
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(gl: &WebGlRenderingContext, buffers: LoadedBuffers) -> Self {
        fn new_f32_buffer(gl: &WebGlRenderingContext, data: &[f32]) -> WebGlBuffer {
            let buf = gl.create_buffer().unwrap();
            gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buf));
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &js_sys::Float32Array::from(data),
                WebGlRenderingContext::STATIC_DRAW,
            );
            buf
        }
        fn new_u16_buffer(gl: &WebGlRenderingContext, data: &[u16]) -> WebGlBuffer {
            let buf = gl.create_buffer().unwrap();
            gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buf));
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                &js_sys::Uint16Array::from(data),
                WebGlRenderingContext::STATIC_DRAW,
            );
            buf
        }

        let positions = new_f32_buffer(gl, &buffers.buffers.positions);
        let normals = new_f32_buffer(gl, &buffers.buffers.normals);
        let colors = new_f32_buffer(gl, &buffers.buffers.colors);
        let shininesses = new_f32_buffer(gl, &buffers.buffers.shininesses);
        let reflectances = new_f32_buffer(gl, &buffers.buffers.reflectances);
        let faces = new_u16_buffer(gl, &buffers.buffers.faces);

        let len = (buffers.buffers.faces.len() / 3) as i32;

        Self {
            positions,
            normals,
            colors,
            shininesses,
            reflectances,
            faces,
            len,
        }
    }

    pub fn draw(&self, gl: &WebGlRenderingContext, program: &WebGlProgram) {
        fn bind_floats(
            gl: &WebGlRenderingContext,
            program: &WebGlProgram,
            buf: &WebGlBuffer,
            name: &str,
            size: i32,
        ) {
            gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buf));
            let location = gl.get_attrib_location(program, name);
            assert!(location >= 0, "Shader attribute {} cannot be enabled", name);
            let location = location as u32;

            gl.vertex_attrib_pointer_with_i32(
                location,
                size,
                WebGlRenderingContext::FLOAT,
                false,
                0,
                0,
            );
            gl.enable_vertex_attrib_array(location);
        }

        bind_floats(gl, program, &self.positions, "a_vertex_pos", 3);
        bind_floats(gl, program, &self.normals, "a_vertex_normal", 3);
        bind_floats(gl, program, &self.colors, "a_vertex_color", 3);
        bind_floats(gl, program, &self.shininesses, "a_vertex_shininess", 1);
        bind_floats(gl, program, &self.reflectances, "a_vertex_reflect", 3);

        gl.bind_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.faces),
        );

        gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            self.len,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );
    }
}

/*
unsafe fn transmute_slice<T, U>(data: &[T]) -> &[U] {
    let size = data.len();
    let ptr = data.as_ptr();
    std::slice::from_raw_parts(ptr as *const U, size)
}
*/
