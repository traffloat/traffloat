//! Node rendering

use safety::Safety;
use traffloat::appearance;
use traffloat::space::{Matrix, Vector};
use typed_builder::TypedBuilder;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use crate::render::scene::mesh;
use crate::render::texture;
use crate::render::util::{create_program, AttrLocation, UniformLocation};

/// Stores the setup data for node rendering.
pub struct Program {
    prog:         WebGlProgram,
    cube:         mesh::PreparedMesh,
    cylinder:     mesh::PreparedIndexedMesh,
    a_pos:        AttrLocation,
    a_normal:     AttrLocation,
    a_tex_pos:    AttrLocation,
    a_tex_offset: AttrLocation,
    u_proj:       UniformLocation<Matrix>,
    u_sun:        UniformLocation<Vector>,
    u_filter:     UniformLocation<Vector>,
    u_inv_gain:   UniformLocation<f32>,
    u_tex:        UniformLocation<i32>,
    u_tex_dim:    UniformLocation<[f32; 2]>,
}

impl Program {
    /// Initializes node canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let prog = create_program(gl, glsl!("node"));

        let cube = mesh::CUBE.prepare(gl);
        let cylinder = mesh::FUSED_CYLINDER.prepare(gl);

        let a_pos = AttrLocation::new(gl, &prog, "a_pos");
        let a_normal = AttrLocation::new(gl, &prog, "a_normal");
        let a_tex_pos = AttrLocation::new(gl, &prog, "a_tex_pos");
        let a_tex_offset = AttrLocation::new(gl, &prog, "a_tex_offset");
        let u_proj = UniformLocation::new(gl, &prog, "u_proj");
        let u_sun = UniformLocation::new(gl, &prog, "u_sun");
        let u_filter = UniformLocation::new(gl, &prog, "u_filter");
        let u_inv_gain = UniformLocation::new(gl, &prog, "u_inv_gain");
        let u_tex = UniformLocation::new_optional(gl, &prog, "u_tex");
        let u_tex_dim = UniformLocation::new_optional(gl, &prog, "u_tex_dim");

        Self {
            prog,
            cube,
            cylinder,
            a_pos,
            a_normal,
            a_tex_pos,
            a_tex_offset,
            u_proj,
            u_sun,
            u_filter,
            u_inv_gain,
            u_tex,
            u_tex_dim,
        }
    }

    /// Draws a node on the canvas.
    ///
    /// The projection matrix transforms unit model coordinates to projection coordinates directly.
    pub fn draw(
        &self,
        DrawArgs { gl, proj, sun, filter, selected, texture, shape_unit }: DrawArgs<'_>,
    ) {
        use mesh::AbstractPreparedMesh;

        gl.use_program(Some(&self.prog));
        self.u_proj.assign(gl, proj);
        self.u_sun.assign(gl, sun);
        self.u_filter.assign(gl, filter);
        self.u_inv_gain.assign(gl, if selected { 0.5f32 } else { 1f32 });

        // The dynamic dispatch here is roughly equialent to
        // an enum matching on the unit type and should not impact performance.
        let mesh: &dyn AbstractPreparedMesh = match shape_unit {
            appearance::Unit::Cube => &self.cube,
            appearance::Unit::Cylinder => &self.cylinder,
            _ => todo!(),
        };
        self.a_pos.assign(gl, mesh.positions());
        self.a_normal.assign(gl, mesh.normals());
        self.a_tex_pos.assign(gl, mesh.tex_pos());
        texture.apply(
            mesh.tex_sprite_number(),
            mesh.tex_offset(),
            self.a_tex_offset,
            &self.u_tex,
            &self.u_tex_dim,
            gl,
        );

        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MAG_FILTER,
            WebGlRenderingContext::NEAREST.homosign(),
        );
        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MIN_FILTER,
            WebGlRenderingContext::NEAREST_MIPMAP_NEAREST.homosign(),
        );
        mesh.draw(gl);
    }
}

/// Arguments for [`Program::draw`]
#[derive(TypedBuilder)]
pub struct DrawArgs<'t> {
    /// The WebGL context.
    gl:         &'t WebGlRenderingContext,
    /// The projection matrix transforming unit model coordinates to projection coordinates
    /// directly.
    proj:       Matrix,
    /// The world direction of the sun.
    sun:        Vector,
    /// The RGB color filter on the node.
    filter:     Vector,
    /// Whether this node is selected.
    selected:   bool,
    /// The spritesheet for the shape.
    texture:    &'t texture::PreparedTexture,
    /// The shape to draw.
    shape_unit: appearance::Unit,
}
