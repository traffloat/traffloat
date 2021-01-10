#![allow(clippy::unwrap_used)]

use std::f32::consts::PI;

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext as GL};

use crate::models;
use camera::Camera;

type Matrix = nalgebra::Matrix4<f32>;
type Vector = nalgebra::Vector3<f32>;

mod camera;

macro_rules! shader {
    ($gl:expr, $prog:expr, $file:expr, $ty:ident) => {
        let code = include_str!(concat!("../../shaders/", $file));
        let shader = $gl.create_shader(GL::$ty).unwrap();
        $gl.shader_source(&shader, &code);
        $gl.compile_shader(&shader);
        $gl.attach_shader(&$prog, &shader);

        let value = $gl.get_shader_parameter(&shader, GL::COMPILE_STATUS);
        if !value.is_truthy() {
            let log = $gl.get_shader_info_log(&shader);
            panic!("Error linking {}: {}", $file, log.unwrap_or(String::new()));
        }
    };
}

macro_rules! prog {
    ($gl:expr, $file:expr) => {{
        let prog = $gl.create_program().unwrap();
        shader!($gl, prog, concat!($file, ".vert"), VERTEX_SHADER);
        shader!($gl, prog, concat!($file, ".frag"), FRAGMENT_SHADER);

        // TODO https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#compile_shaders_and_link_programs_in_parallel
        $gl.link_program(&prog);

        let value = $gl.get_program_parameter(&prog, GL::LINK_STATUS);
        if !value.is_truthy() {
            let log = $gl.get_program_info_log(&prog);
            panic!("Error linking {}: {}", $file, log.unwrap_or(String::new())); }

        prog
    }};
}

struct Buffer {
    buffer: WebGlBuffer,
    name: &'static str,
    ty: u32,
    gl_ty: u32,
    comp_size: u32,
    len: u32,
}

impl Buffer {
    fn new(
        gl: &GL,
        ty: u32,
        gl_ty: u32,
        src: &js_sys::Object,
        name: &'static str,
        comp_size: u32,
        len: u32,
    ) -> Self {
        let buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(ty, Some(&buffer));
        gl.buffer_data_with_array_buffer_view(ty, src, GL::STATIC_DRAW);
        Self {
            buffer,
            name,
            ty,
            gl_ty,
            comp_size,
            len,
        }
    }

    fn apply(&self, gl: &GL, program: &WebGlProgram) {
        gl.bind_buffer(self.ty, Some(&self.buffer));
        if self.ty == GL::ARRAY_BUFFER {
            let loc = gl.get_attrib_location(program, self.name) as u32;
            if (loc as i32) == -1 {
                panic!("Shader attribute {} cannot be enabled", self.name);
            }
            gl.vertex_attrib_pointer_with_i32(loc, self.comp_size as i32, self.gl_ty, false, 0, 0);
            gl.enable_vertex_attrib_array(loc);
        }
    }

    fn draw(&self, gl: &GL) {
        gl.draw_elements_with_i32(GL::TRIANGLES, self.len as i32, self.gl_ty, 0);
    }
}

macro_rules! buffer {
    ($gl:expr, $name:literal $ty:ident $gl_ty:ident $array:ident, $src:expr, $comp_size:expr) => {{
        Buffer::new(
            $gl,
            GL::$ty,
            GL::$gl_ty,
            &js_sys::$array::from($src),
            $name,
            $comp_size,
            $src.len() as u32,
        )
    }};
}

pub struct Render {
    gl: GL,
    aspect: (i32, i32),
    tf: WebGlProgram,
    sphere_vertices: Buffer,
    sphere_faces: Buffer,
    camera: Camera,
    frames: u64,
}

impl Render {
    pub fn new(gl: GL, aspect: (i32, i32)) -> Self {
        let tf = prog!(gl, "tf");

        let sphere_vertices = buffer!(&gl, "a_vertex_pos" ARRAY_BUFFER FLOAT Float32Array, models::CUBE.vertices(), 3);
        let sphere_faces = buffer!(&gl, "a_vertex_pos" ELEMENT_ARRAY_BUFFER UNSIGNED_SHORT Int16Array, models::CUBE.faces(), 3);

        Self {
            gl,
            aspect,
            tf,
            sphere_vertices,
            sphere_faces,
            camera: Camera {
                aspect: (aspect.0 as f32) / (aspect.1 as f32),
                ..Camera::default()
            },
            frames: 0,
        }
    }

    pub fn ren(&mut self) {
        self.frames += 1;

        self.gl.clear_color(0., 0., 0., 1.);
        self.gl.clear_depth(1.);
        self.gl.enable(GL::DEPTH_TEST);
        self.gl.depth_func(GL::LEQUAL);
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        // self.camera.yaw = (self.frames % 200) as f32 / 100.0 * std::f32::consts::PI;
        let pitch = (self.frames as i64 % 400 - 200) as f32 / 200.0 * std::f32::consts::PI;
        self.camera.zoom = Vector::new(1., 1., 1.) * 0.1;

        let rot = Matrix::from_euler_angles(0., pitch, PI / 4.);
        self.draw_sphere(Matrix::new_translation(&Vector::new(0., 0., -5.)) * rot);
    }

    fn draw_sphere(&mut self, matrix: Matrix) {
        self.sphere_vertices.apply(&self.gl, &self.tf);
        self.sphere_faces.apply(&self.gl, &self.tf);
        self.gl.use_program(Some(&self.tf));
        self.set_uniform_matrix(&self.tf, "u_object", matrix);
        self.set_uniform_matrix(&self.tf, "u_projection", self.camera.inv_transform());
        self.sphere_faces.draw(&self.gl);
    }

    fn set_uniform_matrix(&self, program: &WebGlProgram, name: &str, matrix: Matrix) {
        let unif = self.gl.get_uniform_location(&program, name);
        self.gl
            .uniform_matrix4fv_with_f32_array(unif.as_ref(), false, matrix.as_slice());
    }
}
