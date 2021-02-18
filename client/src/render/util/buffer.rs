use std::convert::TryInto;

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext};

pub struct FloatBuffer {
    buffer: WebGlBuffer,
    component_size: u32,
}

impl FloatBuffer {
    pub fn create(gl: &WebGlRenderingContext, data: &[f32], component_size: u32) -> Self {
        let buffer = gl.create_buffer().expect("Failed to allocate WebGL buffer");
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

        let array = js_sys::Float32Array::from(data);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &array,
            WebGlRenderingContext::STATIC_DRAW,
        );

        Self {
            buffer,
            component_size,
        }
    }

    pub fn apply(&self, gl: &WebGlRenderingContext, program: &WebGlProgram, attr: &str) {
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.buffer));
        let location = gl.get_attrib_location(program, attr);
        assert!(location >= 0, "Failed to enable shader attribute {}", attr);
        let location = location as u32;

        gl.vertex_attrib_pointer_with_i32(
            location,
            self.component_size as i32,
            WebGlRenderingContext::FLOAT,
            false, // normalization is noop on floats
            0,     // no stride; contiguous floats
            0,     // zero offset; start from beginning
        );
        gl.enable_vertex_attrib_array(location);
    }
}

pub struct IndexBuffer {
    buffer: WebGlBuffer,
    component_size: i32,
    len: i32,
}

impl IndexBuffer {
    pub fn create(gl: &WebGlRenderingContext, data: &[u16], component_size: i32) -> Self {
        let buffer = gl.create_buffer().expect("Failed to allocate WebGL buffer");
        gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));

        let array = js_sys::Uint16Array::from(data);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &array,
            WebGlRenderingContext::STATIC_DRAW,
        );

        Self {
            buffer,
            component_size,
            len: data.len().try_into().expect("Buffer is too large"),
        }
    }

    pub fn draw(&self, gl: &WebGlRenderingContext) {
        gl.bind_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.buffer),
        );
        gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            self.len,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );
    }
}
