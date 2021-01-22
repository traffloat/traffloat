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
