use std::rc::Rc;
use std::time::Duration;

use yew::prelude::*;
use yew::services::{interval, keyboard as kb_srv, render as render_srv, resize};

use super::GameArgs;
use crate::input;
use crate::render;
use crate::util;
use traffloat::types::{Clock, Time};
use traffloat::SetupEcs;

pub struct Game {
    props: Props,
    link: ComponentLink<Self>,
    legion: traffloat::Legion,
    _resize_task: resize::ResizeTask,
    render_task: render_srv::RenderTask,
    _simulation_task: interval::IntervalTask,
    _keyboard_task: [kb_srv::KeyListenerHandle; 2],
    render_comm: render::Comm,
    canvas_ref: NodeRef,
}

impl Game {
    fn simulate(&mut self) {
        {
            let mut clock = self
                .legion
                .resources
                .get_mut::<Clock>()
                .expect("Clock was uninitialized");
            clock.inc_time(Time(1));
        }

        let time = util::measure(|| self.legion.run());
        self.render_comm.perf.push_exec_us(time);
    }

    fn request_render(&mut self) {
        self.render_comm.flag.cell.replace(self.canvas_context());
        self.render_task = render_srv::RenderService::request_animation_frame(
            self.link.callback(Msg::RenderFrame),
        );
        if let Some(canvas) = self.canvas_ref.cast::<web_sys::HtmlElement>() {
            canvas
                .style()
                .set_property("cursor", self.render_comm.canvas_cursor_type.get())
                .expect("Failed to set canvas cursor property");
        }
    }

    fn canvas_context(&self) -> Option<render::Canvas> {
        use wasm_bindgen::JsCast;

        let canvas = self.canvas_ref.cast::<web_sys::HtmlCanvasElement>()?;
        let width = canvas.width();
        let height = canvas.height();

        let context = canvas
            .get_context("2d")
            .expect("Failed to load 2D canvas")?
            .dyn_into()
            .expect("Failed to load 2D canvas");
        Some(render::Canvas {
            context,
            dim: render::Dimension { width, height },
        })
    }

    fn on_resize(&mut self, dim: resize::WindowDimensions) {
        {
            let mut dim = self
                .legion
                .resources
                .get_mut::<render::Dimension>()
                .expect("render::Dimension uninitialized");
            *dim = render::Dimension {
                width: dim.width,
                height: dim.height,
            };
        }

        let canvas = match self.canvas_ref.cast::<web_sys::HtmlCanvasElement>() {
            Some(canvas) => canvas,
            None => return,
        };
        canvas.set_width(dim.width as u32);
        canvas.set_height(dim.height as u32);

        self.request_render();
    }

    fn on_key(&mut self, code: &str, down: bool) {
        if let Some(event) = input::keyboard::KeyEvent::new(code, down) {
            let mut channel = self
                .legion
                .resources
                .get_mut::<shrev::EventChannel<input::keyboard::KeyEvent>>()
                .expect("EventChannel<KeyEvent> uninitialized");
            channel.single_write(event);
        }
    }

    fn on_mouse(&mut self, x: i32, y: i32, dx: i32, dy: i32) {
        let mut channel = self
            .legion
            .resources
            .get_mut::<shrev::EventChannel<input::mouse::MouseEvent>>()
            .expect("EventChannel<MouseEvent> uninitialized");

        let canvas = match self.canvas_ref.cast::<web_sys::HtmlCanvasElement>() {
            Some(canvas) => canvas,
            None => return,
        };

        let x = (x as f64) / (canvas.width() as f64);
        let y = (y as f64) / (canvas.height() as f64);
        let dx = (dx as f64) / (canvas.width() as f64);
        let dy = (dy as f64) / (canvas.height() as f64);

        channel.single_write(input::mouse::MouseEvent::Move { x, y, dx, dy });
    }
}

fn body() -> web_sys::HtmlElement {
    web_sys::window()
        .expect("Window is undefined")
        .document()
        .expect("Document is undefined")
        .body()
        .expect("document.body is undefined")
}

impl Component for Game {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let render_comm = render::Comm::default();

        let legion = SetupEcs::default()
            .resource(render_comm.clone())
            .resource({
                let window = web_sys::window().expect("Failed to get window object");
                let dim = resize::WindowDimensions::get_dimensions(&window);
                render::Dimension {
                    width: dim.width as u32,
                    height: dim.height as u32,
                }
            })
            .uses(crate::setup_ecs)
            .build(); // TODO setup depending on gamemode

        let body = body();
        let keyboard_task = [
            kb_srv::KeyboardService::register_key_down(&body, link.callback(Msg::KeyDown)),
            kb_srv::KeyboardService::register_key_up(&body, link.callback(Msg::KeyUp)),
        ];

        Self {
            props,
            legion,
            _resize_task: resize::ResizeService::new().register(link.callback(Msg::Resize)),
            render_task: render_srv::RenderService::request_animation_frame(
                link.callback(Msg::RenderFrame),
            ),
            _simulation_task: interval::IntervalService::spawn(
                Duration::from_millis(10),
                link.callback(Msg::SimulationFrame),
            ),
            _keyboard_task: keyboard_task,
            render_comm,
            canvas_ref: NodeRef::default(),
            link,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::SimulationFrame(()) => self.simulate(),
            Msg::RenderFrame(_) => self.request_render(),
            Msg::Resize(dim) => self.on_resize(dim),
            Msg::KeyDown(event) => self.on_key(&event.code(), true),
            Msg::KeyUp(event) => self.on_key(&event.code(), false),
            Msg::MouseMove(event) => self.on_mouse(
                event.client_x(),
                event.client_y(),
                event.movement_x(),
                event.movement_y(),
            ),
        }
        false
    }

    fn change(&mut self, _: Props) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <div style="margin: 0;">
                <canvas
                    ref=self.canvas_ref.clone()
                    onmousemove=self.link.callback(Msg::MouseMove)
                    style="width: 100vw; height: 100vh;"/>
            </div>
        }
    }

    fn rendered(&mut self, _first: bool) {
        let window = web_sys::window().expect("Failed to get window object");
        self.on_resize(resize::WindowDimensions::get_dimensions(&window));
    }
}

pub enum Msg {
    SimulationFrame(()),
    RenderFrame(f64),
    Resize(resize::WindowDimensions),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    MouseMove(MouseEvent),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub args: GameArgs,
    pub error_hook: Callback<String>,
}
