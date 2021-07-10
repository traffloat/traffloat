use std::convert::TryInto;

use js_sys::Float32Array;
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext};

/// A buffer of float values to be passed to a WebGL program.
pub struct FloatBuffer {
    buffer: WebGlBuffer,
    component_size: u32,
}

/// Usage pattern of a buffer.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
pub enum BufferUsage {
    /// The contents are intended to be specified once by the application,
    /// and used many times as the source for WebGL drawing and image specification commands.
    WriteOnceReadMany,
    /// The contents are intended to be respecified repeatedly by the application,
    /// and used many times as the source for WebGL drawing and image specification commands.
    WriteManyReadMany,
    /// The contents are intended to be specified once by the application,
    /// and used at most a few times as the source for WebGL drawing and image specification commands.
    WriteOnceReadFew,
}

impl BufferUsage {
    /// The WebGL constant for the buffer usage.
    pub fn as_gl_usage(self) -> u32 {
        match self {
            Self::WriteOnceReadMany => WebGlRenderingContext::STATIC_DRAW,
            Self::WriteManyReadMany => WebGlRenderingContext::DYNAMIC_DRAW,
            Self::WriteOnceReadFew => WebGlRenderingContext::STREAM_DRAW,
        }
    }
}

impl FloatBuffer {
    /// Creates a float buffer.
    pub fn create(
        gl: &WebGlRenderingContext,
        data: &[f32],
        component_size: u32,
        usage: BufferUsage,
    ) -> Self {
        let buffer = gl.create_buffer().expect("Failed to allocate WebGL buffer");
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

        let array = Float32Array::from(data);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &array,
            usage.as_gl_usage(),
        );

        Self {
            buffer,
            component_size,
        }
    }

    /// Modifies the contents of a float buffer.
    ///
    /// Buffers on which this method is used should use [`BufferUsage::WriteManyReadMany`] when
    /// created.
    pub fn update(&self, gl: &WebGlRenderingContext, data: &[f32]) {
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.buffer));
        let array = Float32Array::from(data);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &array,
            WebGlRenderingContext::DYNAMIC_DRAW,
        );
    }

    /// Apply the buffer at the given attribute location.
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

/// A buffer of index values to be passed to a WebGL program.
pub struct IndexBuffer {
    buffer: WebGlBuffer,
    len: i32,
}

impl IndexBuffer {
    /// Creates an index buffer.
    pub fn create(gl: &WebGlRenderingContext, data: &[u16]) -> Self {
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
            len: data.len().try_into().expect("Buffer is too large"),
        }
    }

    /// Draws on a WebGL context using the indices in this buffer.
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
