use std::f32::consts::PI;

use anyhow::{Context, Result};

use crate::{input, vec, BoxContext, PickTarget, Server, State};

struct RenderLoop<S: Server> {
    server:   S,
    gl:       three_d::Context,
    pipeline: three_d::ForwardPipeline,
    camera:   three_d::Camera,
    state:    State,
    ctrl:     input::Control,
    axes:     Option<three_d::Axes>,
    loaded:   bool,
}

impl<S: Server> RenderLoop<S> {
    pub fn new(server: S, config: Config, window: &three_d::Window) -> Result<Self> {
        let gl = window.gl().context("initializing graphics context")?;

        let pipeline = three_d::ForwardPipeline::new(&gl).context("creating rendering pipeline")?;

        let camera = three_d::Camera::new_perspective(
            &gl,
            window.viewport().context("fetching window viewport")?,
            three_d::vec3(0.0, 0.0, -5.0),
            three_d::vec3(0.0, 0.0, 0.0),
            three_d::vec3(0.0, -1.0, 0.0),
            config.fov,
            0.1,
            config.zfar,
        )
        .context("creating camera")?;

        let state = State::default();

        let ctrl = input::Control::new(
            config.linear_speed,
            config.rotate_speed,
            config.linear_sensitivity,
            config.rotate_sensitivity,
        );

        let axes = Some(three_d::Axes::new(&gl, 0.05, 1.0).context("creating axes mesh")?);

        Ok(Self { server, gl, pipeline, camera, state, ctrl, axes, loaded: false })
    }

    pub fn main_loop(mut self, window: three_d::Window) -> Result<()> {
        window
            .render_loop(move |frame_input| match self.pass(frame_input) {
                Ok(output) => output,
                Err(err) => panic!("Error executing render loop: {:?}", err),
            })
            .context("executing render loop")
    }

    fn pass(&mut self, mut frame_input: three_d::FrameInput) -> Result<three_d::FrameOutput> {
        while let Some(event) = self.server.receive().context("receiving event from simulation")? {
            self.state
                .handle_event(event, &self.gl, &mut self.server)
                .context("handling simulation event")?;
        }

        let mut redraw = frame_input.first_frame;
        redraw |= self.camera.set_viewport(frame_input.viewport).context("updating viewport")?;

        if !self.loaded {
            self.loaded = true;
            redraw = true;
        }

        redraw |= self
            .ctrl
            .handle_events(&mut self.camera, &mut frame_input.events)
            .context("handling camera input")?;

        if !redraw {
            redraw = frame_input
                .events
                .iter()
                .any(|event| matches!(event, three_d::Event::MouseMotion { .. }));
        }

        if redraw {
            let sunlight = three_d::DirectionalLight::new(
                &self.gl,
                1.,
                three_d::Color::WHITE,
                &vec(self.state.sun.source_direction(&self.state.clock)),
            )
            .context("loading sunlight")?;

            for node in self.state.nodes.values_mut() {
                node.check_loading(
                    &self.state.std_meshes,
                    &self.state.texture_pool,
                    &self.gl,
                    &self.server,
                )
                .context("loading objects")?;
            }

            fn node_edge_objects(
                state: &State,
            ) -> impl Iterator<Item = (PickTarget, &dyn three_d::Object)> {
                state
                    .nodes
                    .iter()
                    .flat_map(|(&k, object)| {
                        object.models().iter().map(move |object| {
                            (PickTarget::Node(k), object as &dyn three_d::Object)
                        })
                    })
                    .chain(state.edges.iter().flat_map(|(&k, edge)| {
                        edge.objects().iter().map(move |object| {
                            (PickTarget::Edge(k), object as &dyn three_d::Object)
                        })
                    }))
            }

            let picked = input::handle_pick(
                &self.gl,
                &self.camera,
                &frame_input,
                node_edge_objects(&self.state),
            )
            .context("detecting cursor target")?;
            self.state.set_picked(&self.gl, picked).context("Updating pick target")?;

            let objects: Vec<_> = node_edge_objects(&self.state)
                .map(|(_, object)| object)
                .chain(self.axes.as_ref().map(|axes| axes as &dyn three_d::Object))
                .collect();

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
                )?;
                Ok(())
            })
            .context("flushing objects to screen")?;
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
            zfar:               100.0,
            linear_speed:       10.0,
            rotate_speed:       three_d::degrees(30.).into(),
            linear_sensitivity: 0.7,
            rotate_sensitivity: PI / 1000.,
        }
    }
}

pub fn run<S: Server>(server: S, config: Config) -> Result<()> {
    let window = three_d::Window::new(three_d::WindowSettings {
        title: String::from("Traffloat"),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .context("creating window")?;

    let render_loop =
        RenderLoop::new(server, config, &window).context("initializing render loop states")?;
    render_loop.main_loop(window)
}
