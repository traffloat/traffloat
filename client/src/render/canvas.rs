#![allow(clippy::unwrap_used)]

use std::convert::TryInto;
use std::iter;

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};

use crate::config;
use crate::models;
use common::shape::{self, Shape};
use common::types::*;
use traffloat_client_model::*;

macro_rules! create_programs {
    ($gl:expr; $($name:ident)*) => {{
        let gl = $gl;
        $(
            let $name = {
                let program = gl.create_program().unwrap();
                let vert = create_shader(
                    &gl,
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                        "/shaders/", stringify!($name), ".vert")),
                    WebGlRenderingContext::VERTEX_SHADER,
                );
                let frag = create_shader(
                    &gl,
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                        "/shaders/", stringify!($name), ".frag")),
                    WebGlRenderingContext::FRAGMENT_SHADER,
                );
                (program, vert, frag)
            };
        )*
        $(
            let value = gl.get_shader_parameter(&$name.1, WebGlRenderingContext::COMPILE_STATUS);
            if !value.is_truthy() {
                let log = gl.get_shader_info_log(&$name.1);
                panic!("Error compiling {}.vert: {}",
                    stringify!($name), log.unwrap_or(String::new()));
            }

            let value = gl.get_shader_parameter(&$name.2, WebGlRenderingContext::COMPILE_STATUS);
            if !value.is_truthy() {
                let log = gl.get_shader_info_log(&$name.2);
                panic!("Error compiling {}.frag: {}",
                    stringify!($name), log.unwrap_or(String::new()));
            }
        )*
        $(
            gl.attach_shader(&$name.0, &$name.1);
            gl.attach_shader(&$name.0, &$name.2);
        )*
        $(
            gl.link_program(&$name.0);
        )*
        $(
            gl.get_program_parameter(&$name.0, WebGlRenderingContext::LINK_STATUS);
        )*
        $(
            if !value.is_truthy() {
                let log = gl.get_program_info_log(&$name.0);
                panic!("Error linking {}.frag: {}",
                    stringify!($name), log.unwrap_or(String::new()));
            }
        )*
        ($($name.0,)*)
    }}
}

pub struct Canvas {
    gl: WebGlRenderingContext,
    object_program: WebGlProgram,
    star_program: WebGlProgram,
    noise_buf: Model,

    pub render_requested: bool,
    cube_buf: Model,
    // tetra_buf: Model,
    // sphere_buf: Model,
}

impl Canvas {
    pub fn new(gl: WebGlRenderingContext, noise_seed: u64) -> Self {
        let (object_program, star_program) = create_programs!(&gl;
            object
            star
        );

        let noise_buf = Model::new(&gl, create_stars(noise_seed));
        let cube_buf = Model::new(&gl, models::CUBE);

        Self {
            gl,
            object_program,
            star_program,
            noise_buf,
            cube_buf,

            render_requested: false,
        }
    }

    pub fn render_bg(&self, matrix: Matrix) {
        self.gl.clear_color(0., 0., 0., 1.);
        self.gl.clear_depth(1.);
        self.gl.enable(WebGlRenderingContext::DEPTH_TEST);
        self.gl.depth_func(WebGlRenderingContext::LEQUAL);
        self.gl.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        self.noise_buf.apply(&self.gl, &self.star_program);
        self.gl.use_program(Some(&self.star_program));
        set_uniform_matrix(&self.gl, &self.star_program, "u_projection", matrix);
        self.noise_buf.draw(&self.gl);
    }

    pub fn render_shape(&self, camera_matrix: Matrix, shape: Shape) {
        let buf = match shape.unit {
            shape::Unit::Cube => &self.cube_buf,
            _ => unimplemented!(),
        };
        buf.apply(&self.gl, &self.object_program);
        self.gl.use_program(Some(&self.object_program));
        set_uniform_matrix(&self.gl, &self.star_program, "u_projection", camera_matrix);
        set_uniform_matrix(&self.gl, &self.star_program, "u_object", camera_matrix);
        buf.draw(&self.gl);
    }
}

fn create_shader(gl: &WebGlRenderingContext, code: &'static str, ty: u32) -> WebGlShader {
    let shader = gl.create_shader(ty).unwrap();
    gl.shader_source(&shader, &code);
    gl.compile_shader(&shader);

    shader
}

struct Model {
    positions: WebGlBuffer,
    colors: WebGlBuffer,
    indices: WebGlBuffer,
    len: i32,
}

impl Model {
    fn new(gl: &WebGlRenderingContext, mesh: impl AbstractMesh) -> Self {
        let positions = gl.create_buffer().unwrap();
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&positions));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::from(mesh.vertices()),
            WebGlRenderingContext::STATIC_DRAW,
        );

        let colors = gl.create_buffer().unwrap();
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&colors));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::from(mesh.colors()),
            WebGlRenderingContext::STATIC_DRAW,
        );

        let indices = gl.create_buffer().unwrap();
        gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&indices));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &js_sys::Int16Array::from(unsafe {
                traffloat_client_model::transmute_slice::<u16, i16>(mesh.faces())
            }),
            WebGlRenderingContext::STATIC_DRAW,
        );

        Self {
            positions,
            colors,
            indices,
            len: mesh.vertices().len() as i32,
        }
    }

    fn apply(&self, gl: &WebGlRenderingContext, program: &WebGlProgram) {
        {
            gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.positions));
            let location = gl.get_attrib_location(program, "a_vertex_pos");
            if location == -1 {
                panic!("Shader attribute a_vertex_pos cannot be enabled");
            }
            let location = location as u32;

            gl.vertex_attrib_pointer_with_i32(
                location,
                3,
                WebGlRenderingContext::FLOAT,
                false,
                0,
                0,
            );
            gl.enable_vertex_attrib_array(location);
        }

        {
            gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.colors));
            let location = gl.get_attrib_location(program, "a_vertex_color");
            if location == -1 {
                panic!("Shader attribute a_vertex_color cannot be enabled");
            }
            let location = location as u32;

            gl.vertex_attrib_pointer_with_i32(
                location,
                4,
                WebGlRenderingContext::FLOAT,
                false,
                0,
                0,
            );
            gl.enable_vertex_attrib_array(location);
        }

        gl.bind_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.indices),
        );
    }

    fn draw(&self, gl: &WebGlRenderingContext) {
        gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            self.len / 3,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );
    }
}

fn set_uniform_matrix(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    name: &str,
    matrix: Matrix,
) {
    let uniform = gl.get_uniform_location(program, name);
    gl.uniform_matrix4fv_with_f32_array(uniform.as_ref(), false, matrix.as_slice());
}

fn create_stars(seed: u64) -> impl AbstractMesh {
    use rand::prelude::*;

    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    let mut colors = Vec::new();

    let mut pos_rng = rand_xoshiro::SplitMix64::seed_from_u64(seed);
    let mut size_rng = rand_xoshiro::SplitMix64::seed_from_u64(seed);
    let mut color_rng = rand_xoshiro::SplitMix64::seed_from_u64(seed);
    for sample in rand_distr::UnitSphere
        .sample_iter(&mut pos_rng)
        .take(config::NOISE_SIZE)
    {
        let sample: [f32; 3] = sample; // type coercion
        let vector = Vector::from_column_slice(&sample);
        let mut a = vector.cross(&Vector::new(1., 0., 0.));
        let mut b = vector.cross(&a);

        let size_root = size_rng.gen::<f32>();
        let mut size = 1. - size_root * size_root;
        size *= config::NOISE_SCALE;

        a *= size;
        b *= size;

        let i = vertices.len().try_into().expect("NOISE_SIZE is too large");
        vertices.push(Vertex(
            (vector - a)
                .as_slice()
                .try_into()
                .expect("Vector3 -> [f32; 3]"),
        ));
        vertices.push(Vertex(
            (vector - b)
                .as_slice()
                .try_into()
                .expect("Vector3 -> [f32; 3]"),
        ));
        vertices.push(Vertex(
            (vector + b)
                .as_slice()
                .try_into()
                .expect("Vector3 -> [f32; 3]"),
        ));
        vertices.push(Vertex(
            (vector + a)
                .as_slice()
                .try_into()
                .expect("Vector3 -> [f32; 3]"),
        ));
        colors.extend(iter::repeat(Color(color_rng.gen())).take(4));

        faces.push(Face([i, i + 1, i + 2]));
        faces.push(Face([i + 3, i + 1, i + 2]));
    }

    log::debug!("vertices.len() = {:?}", vertices.len());

    DynamicMesh {
        vertices,
        faces,
        colors,
    }
}