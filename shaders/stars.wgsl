var<uniform> camera_trans: mat3x3<f32>;

struct Vertex {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;
    vertex_output.clip_position = vec4((camera_trans * vertex.pos).xyz, 1.);
}

@fragment
fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(vertex_output.color, 1.0);
}
