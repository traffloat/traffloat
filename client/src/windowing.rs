use std::error::Error;
use std::f32::consts::PI;

use crate::{input, vec, Server, State};

struct RenderLoop<S: Server> {
    server:   S,
    gl:       three_d::Context,
    pipeline: three_d::ForwardPipeline,
    camera:   three_d::Camera,
    state:    State,
    ctrl:     input::Control,
    loaded:   bool,
}

impl<S: Server> RenderLoop<S> {
    pub fn new(
        server: S,
        config: Config,
        window: &three_d::Window,
    ) -> Result<Self, Box<dyn Error>> {
        let gl = window.gl()?;

        let pipeline = three_d::ForwardPipeline::new(&gl)?;

        let camera = three_d::Camera::new_perspective(
            &gl,
            window.viewport()?,
            three_d::vec3(0.0, 0.0, -5.0),
            three_d::vec3(0.0, 0.0, 0.0),
            three_d::vec3(0.0, -1.0, 0.0),
            config.fov,
            0.1,
            config.zfar,
        )?;

        let state = State::default();

        let ctrl = input::Control::new(
            config.linear_speed,
            config.rotate_speed,
            config.linear_sensitivity,
            config.rotate_sensitivity,
        );

        Ok(Self { server, gl, pipeline, camera, state, ctrl, loaded: false })
    }

    pub fn main_loop(mut self, window: three_d::Window) -> Result<(), Box<dyn Error>> {
        window.render_loop(move |frame_input| self.pass(frame_input).unwrap())
    }

    fn pass(
        &mut self,
        mut frame_input: three_d::FrameInput,
    ) -> Result<three_d::FrameOutput, Box<dyn Error>> {
        while let Some(event) = self.server.receive()? {
            self.state.handle_event(event, &self.gl);
        }

        let mut redraw = frame_input.first_frame;
        redraw |= self.camera.set_viewport(frame_input.viewport)?;

        if !self.loaded {
            self.loaded = true;
            redraw = true;
        }

        redraw |= self.ctrl.handle_events(&mut self.camera, &mut frame_input.events)?;

        if redraw {
            let sunlight = three_d::DirectionalLight::new(
                &self.gl,
                1.,
                three_d::Color::WHITE,
                &vec(self.state.sun.source_direction(&self.state.clock)),
            )?;

            let node_objects = self.state.nodes.values().map(|node| node.object());
            let edge_objects = self.state.edges.values().map(|edge| edge.object());
            let objects: Vec<&dyn three_d::Object> = node_objects.chain(edge_objects).collect();

            three_d::Screen::write(&self.gl, three_d::ClearState::default(), || {
                self.pipeline.render_pass(
                    &self.camera,
                    &objects,
                    &three_d::Lights {
                        ambient:        Some(three_d::AmbientLight::default()),
                        directional:    vec![sunlight],
                        spot:           Vec::new(),
                        point:          Vec::new(),
                        lighting_model: three_d::LightingModel::Blinn,
                    },
                )
            })?;
        }

        Ok(three_d::FrameOutput {
            swap_buffers: redraw,
            wait_next_event: false,
            ..Default::default()
        })
    }
}

pub struct Config {
    pub fov:                three_d::Radians,
    pub zfar:               f32,
    pub linear_speed:       f32,
    pub rotate_speed:       three_d::Radians,
    pub linear_sensitivity: f32,
    pub rotate_sensitivity: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fov:                three_d::degrees(60.0).into(),
            zfar:               1000.0,
            linear_speed:       1.0,
            rotate_speed:       three_d::degrees(30.).into(),
            linear_sensitivity: 1.0,
            rotate_sensitivity: PI / 100.,
        }
    }
}

pub fn run<S: Server>(server: S, config: Config) -> Result<(), Box<dyn Error>> {
    let window = three_d::Window::new(three_d::WindowSettings {
        title: String::from("Traffloat"),
        max_size: Some((1280, 720)),
        ..Default::default()
    })?;

    let render_loop = RenderLoop::new(server, config, &window)?;
    render_loop.main_loop(window)
}
