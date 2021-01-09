#![allow(clippy::unwrap_used)]

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext as GL};

pub struct Render {
    gl: GL,
    test: WebGlProgram,
    test_buf: WebGlBuffer,
}

impl Render {
    pub fn new(gl: GL) -> Self {
        macro_rules! shader {
            ($gl:expr, $prog:expr, $file:expr, $ty:ident) => {
                let code = include_str!(concat!("../shaders/", $file));
                let shader = $gl.create_shader(GL::$ty).unwrap();
                $gl.shader_source(&shader, &code);
                $gl.compile_shader(&shader);
                $gl.attach_shader(&$prog, &shader);

                let value = gl.get_shader_parameter(&shader, GL::COMPILE_STATUS);
                if !value.is_truthy() {
                    let log = gl.get_shader_info_log(&shader);
                    panic!("Error linking {}: {}", $file, log.unwrap_or(String::new()));
                }
            };
        }

        macro_rules! prog {
            ($gl:expr, $file:expr) => {{
                let prog = $gl.create_program().unwrap();
                shader!($gl, prog, concat!($file, ".vert"), VERTEX_SHADER);
                shader!($gl, prog, concat!($file, ".frag"), FRAGMENT_SHADER);
                $gl.link_program(&prog);

                let value = gl.get_program_parameter(&prog, GL::LINK_STATUS);
                if !value.is_truthy() {
                    let log = gl.get_program_info_log(&prog);
                    panic!("Error linking {}: {}", $file, log.unwrap_or(String::new()));
                }

                prog
            }};
        }

        let test = prog!(gl, "test");
        let test_buf = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&test_buf));
        let verts = js_sys::Float32Array::from(
            &[
                1.0f32, 2.0f32, 3.0f32, 2.0f32, 1.0f32, 3.0f32, 2.0f32, 3.0f32, 1.0f32, 3.0f32,
                2.0f32, 1.0f32,
            ][..],
        );
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);
        Self { gl, test, test_buf }
    }

    pub fn ren(&self) {
        self.gl.clear_color(0.8, 0.5, 0.2, 0.3);
        self.gl.clear(GL::COLOR_BUFFER_BIT);

        self.gl.use_program(Some(&self.test));

        let a_pos = self.gl.get_attrib_location(&self.test, "a_pos") as u32;
        self.gl
            .vertex_attrib_pointer_with_f64(a_pos, 3, GL::FLOAT, false, 0, 0.);
        self.gl.enable_vertex_attrib_array(a_pos);
        self.gl.draw_arrays(GL::TRIANGLES, 0, 4);
    }
}
