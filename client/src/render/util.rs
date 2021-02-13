use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlUniformLocation};

pub type GlMatrix = nalgebra::Matrix4<f32>;
pub type GlVector = nalgebra::Vector3<f32>;

pub fn create_shader(
    gl: &WebGlRenderingContext,
    prog: &WebGlProgram,
    file: &str,
    code: &str,
    shader_type: u32,
) {
    let shader = gl
        .create_shader(shader_type)
        .expect("Failed to initialize WebGL shader");
    gl.shader_source(&shader, &code);
    gl.compile_shader(&shader);
    gl.attach_shader(&prog, &shader);

    let value = gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS);
    if !value.is_truthy() {
        let log = gl.get_shader_info_log(&shader);
        panic!("Error linking {}: {}", file, log.unwrap_or_default());
    }
}

pub fn create_program(
    gl: &WebGlRenderingContext,
    vert_file: &str,
    vert_code: &str,
    frag_file: &str,
    frag_code: &str,
) -> WebGlProgram {
    let prog = gl
        .create_program()
        .expect("Failed to initialize WebGL program");
    create_shader(
        gl,
        &prog,
        vert_file,
        vert_code,
        WebGlRenderingContext::VERTEX_SHADER,
    );
    create_shader(
        gl,
        &prog,
        frag_file,
        frag_code,
        WebGlRenderingContext::FRAGMENT_SHADER,
    );

    // TODO https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#compile_shaders_and_link_programs_in_parallel
    gl.link_program(&prog);

    let value = gl.get_program_parameter(&prog, WebGlRenderingContext::LINK_STATUS);
    if !value.is_truthy() {
        let log = gl.get_program_info_log(&prog);
        panic!(
            "Error linking {}/{}: {}",
            vert_file,
            frag_file,
            log.unwrap_or_default()
        );
    }

    prog
}

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
            len: data.len() as i32,
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

pub trait WebglExt {
    fn canvas(&self) -> &WebGlRenderingContext;

    fn set_uniform(&self, program: &WebGlProgram, name: &str, uniform: impl Uniform) {
        let gl = self.canvas();
        let location = gl.get_uniform_location(program, name);
        uniform.apply(location, gl);
    }
}

impl WebglExt for WebGlRenderingContext {
    fn canvas(&self) -> &Self {
        self
    }
}

pub trait Uniform {
    fn apply(&self, location: Option<WebGlUniformLocation>, gl: &WebGlRenderingContext);
}

macro_rules! impl_uniform {
    ($unif:ident, $vec:ident, {$($extra:tt)*}) => {
        impl Uniform for nalgebra::$vec<f32> {
            fn apply(&self, location: Option<WebGlUniformLocation>, gl: &WebGlRenderingContext) {
                gl.$unif(location.as_ref(), $($extra)* self.as_slice());
            }
        }
    }
}

impl_uniform!(uniform2fv_with_f32_array, Vector2, {});
impl_uniform!(uniform3fv_with_f32_array, Vector3, {});
impl_uniform!(uniform4fv_with_f32_array, Vector4, {});

impl_uniform!(uniform_matrix2fv_with_f32_array, Matrix2, {false, });
impl_uniform!(uniform_matrix3fv_with_f32_array, Matrix3, {false, });
impl_uniform!(uniform_matrix4fv_with_f32_array, Matrix4, {false, });

impl Uniform for f32 {
    fn apply(&self, location: Option<WebGlUniformLocation>, gl: &WebGlRenderingContext) {
        gl.uniform1f(location.as_ref(), *self);
    }
}
