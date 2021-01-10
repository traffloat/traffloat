use web_sys::{HtmlCanvasElement, WebGlRenderingContext};
use yew::prelude::*;
use yew::services::keyboard::{KeyListenerHandle, KeyboardService};
use yew::services::render::{RenderService, RenderTask};
use yew::services::resize::{ResizeService, ResizeTask, WindowDimensions};

use super::WebSocket;
use crate::render::{KeyAction, Render};

pub struct Game {
    link: ComponentLink<Self>,
    props: Properties,
    _resize_task: ResizeTask,
    render_task: Option<RenderTask>,
    key_handles: Vec<KeyListenerHandle>,
    render: Option<Render>,
    dim: WindowDimensions,
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
        self.render
            .as_mut()
            .expect("run_render called without initializing render context")
            .ren();
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
        Self {
            link,
            props,
            _resize_task: resize_task,
            key_handles: Vec::new(),
            render_task: None,
            render: None,
            dim: WindowDimensions::get_dimensions(&web_sys::window().unwrap()),
        }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
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
            Message::KeyDown(key) => {
                if let Some(render) = self.render.as_mut() {
                    let code = key.code();
                    if let Some(action) = KeyAction::from_code(code.as_str()) {
                        render.set_key(action, true);
                    }
                }
                false
            }
            Message::KeyUp(key) => {
                if let Some(render) = self.render.as_mut() {
                    let code = key.code();
                    if let Some(action) = KeyAction::from_code(code.as_str()) {
                        render.set_key(action, false);
                    }
                }
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
        self.render = Some(Render::new(gl, (self.dim.width, self.dim.height)));

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
}

#[derive(Clone, Debug, Properties)]
pub struct Properties {
    pub ws: WebSocket,
}
