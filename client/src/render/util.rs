use web_sys::{WebGlProgram, WebGlRenderingContext};

pub fn create_shader(
    gl: &WebGlRenderingContext,
    prog: &WebGlProgram,
    file: &str,
    code: &str,
    shader_ty: u32,
) {
    let shader = gl
        .create_shader(shader_ty)
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
