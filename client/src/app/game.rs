use std::time::Duration;

use specs::WorldExt;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};
use yew::prelude::*;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::services::keyboard::{KeyListenerHandle, KeyboardService};
use yew::services::render::{RenderService, RenderTask};
use yew::services::resize::{ResizeService, ResizeTask, WindowDimensions};

use super::WebSocket;
use crate::keymap::{Action, ActionEvent};
use crate::render::{Camera, Canvas};
use common::types::{Clock, Time};

pub struct Game {
    link: ComponentLink<Self>,
    props: Properties,
    _resize_task: ResizeTask,
    _dispatch_task: IntervalTask,
    render_task: Option<RenderTask>,
    key_handles: Vec<KeyListenerHandle>,
    dim: WindowDimensions,
    setup: (specs::World, specs::Dispatcher<'static, 'static>),
}

impl Game {
    fn document() -> web_sys::Document {
        let window = web_sys::window().unwrap();
        window.document().unwrap()
    }
    fn canvas() -> (HtmlCanvasElement, WebGlRenderingContext) {
        use wasm_bindgen::JsCast;

        let document = Self::document();
        let elem = document.get_element_by_id("game_canvas").unwrap();
        let elem = elem.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let gl = elem
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        (elem, gl)
    }

    fn run_render(&mut self) {
        let canvas = self.setup.0.get_mut::<Canvas>();
        if let Some(canvas) = canvas {
            canvas.render_requested = true;
        }
    }

    fn schedule_render(&mut self) {
        let task = RenderService::request_animation_frame(self.link.callback(Message::Render));
        self.render_task = Some(task);
    }
}

impl Component for Game {
    type Message = Message;
    type Properties = Properties;

    fn create(props: Properties, link: ComponentLink<Self>) -> Self {
        let resize_task = ResizeService::new().register(link.callback(Message::WindowResize));
        let dispatch_task =
            IntervalService::spawn(Duration::from_millis(10), link.callback(Message::Dispatch));
        Self {
            link,
            props,
            _resize_task: resize_task,
            _dispatch_task: dispatch_task,
            key_handles: Vec::new(),
            render_task: None,
            dim: WindowDimensions::get_dimensions(&web_sys::window().unwrap()),
            setup: crate::setup_specs(),
        }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
        fn update_key(world: &mut specs::World, key: KeyboardEvent, down: bool) -> ShouldRender {
            let action = Action::from_code(key.code().as_str());
            if let Some(action) = action {
                let chan: &mut shrev::EventChannel<ActionEvent> = world
                    .get_mut()
                    .expect("EventChannel<ActionEvent> not initialized");
                chan.single_write(ActionEvent {
                    action,
                    active: down,
                });
            }
            false
        }

        match msg {
            Message::WindowResize(dim) => {
                self.dim = dim;
                true
            }
            Message::Render(_time) => {
                self.run_render();
                self.schedule_render();
                false
            }
            Message::KeyDown(key) => update_key(&mut self.setup.0, key, true),
            Message::KeyUp(key) => update_key(&mut self.setup.0, key, false),
            Message::Dispatch(()) => {
                let (world, dispatcher) = &mut self.setup;
                let clock: &mut Clock = world.get_mut().expect("Clock was initialized at setup");
                clock.inc_time(Time(1));
                dispatcher.dispatch(world);
                world.maintain();
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <canvas id="game_canvas"
                width=self.dim.width height=self.dim.height
                style="width: 100vw; height: 100vh; display: block;"
                />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        let (_, gl) = Self::canvas();
        let canvas = Canvas::new(gl, self.props.server_seed());
        match self.setup.0.get_mut::<Canvas>() {
            Some(ptr) => *ptr = canvas,
            None => self.setup.0.insert::<Canvas>(canvas),
        }
        let camera = self
            .setup
            .0
            .get_mut::<Camera>()
            .expect("Camera should be initialized");

        #[allow(clippy::cast_precision_loss)]
        {
            camera.aspect = (self.dim.width as f32) / (self.dim.height as f32);
        }

        let body = Self::document().body().unwrap();

        self.key_handles.push(KeyboardService::register_key_down(
            &body,
            self.link.callback(Message::KeyDown),
        ));
        self.key_handles.push(KeyboardService::register_key_up(
            &body,
            self.link.callback(Message::KeyUp),
        ));

        if first_render {
            self.schedule_render();
        }
    }
}

pub enum Message {
    WindowResize(WindowDimensions),
    Render(f64),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    Dispatch(()),
}

#[derive(Clone, Debug, Properties)]
pub struct Properties {
    pub addr: String,
    pub port: u16,
    pub ws: WebSocket,
}

impl Properties {
    fn server_seed(&self) -> u64 {
        use crc64fast::Digest;
        let mut c = Digest::new();
        c.write(self.addr.as_bytes());
        c.write(&self.port.to_le_bytes());
        c.sum64()
    }
}
