use std::rc::Rc;
use std::time::Duration;

use yew::prelude::*;
use yew::services::{interval, keyboard as kb_srv, render as render_srv, resize};

use super::GameArgs;
use crate::input;
use crate::render;
use crate::util;
use safety::Safety;
use traffloat::time::{Clock, Instant, Time};
use traffloat::SetupEcs;

/// HTML interface of the game page
pub struct Game {
    _props: Props,
    link: ComponentLink<Self>,
    legion: traffloat::Legion,
    _resize_task: resize::ResizeTask,
    _render_task: render_srv::RenderTask,
    _simulation_task: interval::IntervalTask,
    _keyboard_task: [kb_srv::KeyListenerHandle; 2],
    render_comm: render::Comm,
    bg_canvas_ref: NodeRef,
    scene_canvas_ref: NodeRef,
    ui_canvas_ref: NodeRef,
    debug_ref: NodeRef,
    layers_cache: Option<(render::Layers, render::Dimension)>,
    clock_epoch: u64,
}

impl Game {
    fn simulate(&mut self) {
        {
            let mut clock = self
                .legion
                .resources
                .get_mut::<Clock>()
                .expect("Clock was uninitialized");

            let delta = (util::high_res_time() - self.clock_epoch).small_float() / 10000.;
            clock.set_time(Instant(Time(delta.trunc_int())));
        }

        let time = util::measure(|| self.legion.run());
        self.render_comm.perf.push_exec_us(time);
    }

    fn request_render(&mut self) {
        if let Some((layers, dim)) = self.canvas_context() {
            let layers = Rc::clone(layers);
            let dim = *dim;

            let layers_ref = &mut *self
                .legion
                .resources
                .get_mut::<Option<render::Layers>>()
                .expect("render::Layers resource not initialized");
            *layers_ref = Some(layers);
            self.legion
                .resources
                .get_mut::<shrev::EventChannel<render::RenderFlag>>()
                .expect("RenderFlag EventChannel not initialized")
                .single_write(render::RenderFlag);
            let dim_ref = &mut *self
                .legion
                .resources
                .get_mut::<render::Dimension>()
                .expect("Uninitialized Dimension resource");
            *dim_ref = dim;
        }
        self._render_task = render_srv::RenderService::request_animation_frame(
            self.link.callback(Msg::RenderFrame),
        );
    }

    fn canvas_context(&mut self) -> Option<&(render::Layers, render::Dimension)> {
        use wasm_bindgen::JsCast;

        let seed = rand::random::<[u8; 32]>(); // TODO compute based on multiplayer hostname

        if self.layers_cache.is_none() {
            let bg_canvas = self.bg_canvas_ref.cast::<web_sys::HtmlCanvasElement>()?;
            let scene_canvas = self.scene_canvas_ref.cast::<web_sys::HtmlCanvasElement>()?;
            let ui_canvas = self.ui_canvas_ref.cast::<web_sys::HtmlCanvasElement>()?;
            let debug_div = self.debug_ref.cast::<web_sys::HtmlElement>()?;
            let width = ui_canvas.width();
            let height = ui_canvas.height();

            let bg_context = bg_canvas
                .get_context("webgl")
                .expect("Failed to load WebGL canvas")?
                .dyn_into()
                .expect("Failed to load WebGL canvas");
            let scene_context = scene_canvas
                .get_context("webgl")
                .expect("Failed to load WebGL canvas")?
                .dyn_into()
                .expect("Failed to load WebGL canvas");
            let ui_context = ui_canvas
                .get_context("2d")
                .expect("Failed to load 2D canvas")?
                .dyn_into()
                .expect("Failed to load 2D canvas");
            let debug_writer = util::DebugWriter::new(debug_div);
            let dim = render::Dimension { width, height };

            self.layers_cache = Some((
                render::LayersStruct::new(
                    bg_context,
                    scene_context,
                    ui_context,
                    debug_writer,
                    seed,
                ),
                dim,
            ));
        }

        self.layers_cache.as_ref()
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

        for node_ref in &[
            &self.bg_canvas_ref,
            &self.scene_canvas_ref,
            &self.ui_canvas_ref,
        ] {
            let canvas = match node_ref.cast::<web_sys::HtmlCanvasElement>() {
                Some(canvas) => canvas,
                None => return,
            };
            canvas.set_width(dim.width as u32);
            canvas.set_height(dim.height as u32);
        }

        self.request_render();
    }

    fn on_key(&mut self, code: &str, down: bool) {
        let event = input::keyboard::RawKeyEvent::builder()
            .code(code.to_string())
            .down(down)
            .build();
        self.legion.publish(event);
    }

    fn on_mouse_move(&mut self, x: i32, y: i32) {
        let canvas = match self.ui_canvas_ref.cast::<web_sys::HtmlCanvasElement>() {
            Some(canvas) => canvas,
            None => return,
        };

        let x = (x as f64) / (canvas.width() as f64);
        let y = (y as f64) / (canvas.height() as f64);

        let mut pos = self
            .legion
            .resources
            .get_mut::<input::mouse::CursorPosition>()
            .expect("CursorPosition is uninitialized");
        *pos = input::mouse::CursorPosition::new(x, y);
    }

    fn on_mouse_click(&mut self, _button: i16, _down: bool) {
        // TODO!("Send the event to ECS")
        /*
        if let Some(event) = input::keyboard::KeyEvent::new_mouse(button, down) {
            let mut channel = self
                .legion
                .resources
                .get_mut::<shrev::EventChannel<input::keyboard::KeyEvent>>()
                .expect("EventChannel<KeyEvent> uninitialized");
            channel.single_write(event);
        }
        */
    }

    fn on_wheel(&mut self, _delta: f64) {
        // TODO!("Send the event to ECS")
        /*
        let mut channel = self
            .legion
            .resources
            .get_mut::<shrev::EventChannel<input::mouse::WheelEvent>>()
            .expect("EventChannel<WheelEvent> uninitialized");
        channel.single_write(input::mouse::WheelEvent { delta });
        */
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
            _props: props,
            legion,
            _resize_task: resize::ResizeService::register(link.callback(Msg::Resize)),
            _render_task: render_srv::RenderService::request_animation_frame(
                link.callback(Msg::RenderFrame),
            ),
            _simulation_task: interval::IntervalService::spawn(
                Duration::from_millis(15),
                link.callback(Msg::SimulationFrame),
            ),
            _keyboard_task: keyboard_task,
            render_comm,
            bg_canvas_ref: NodeRef::default(),
            scene_canvas_ref: NodeRef::default(),
            ui_canvas_ref: NodeRef::default(),
            debug_ref: NodeRef::default(),
            layers_cache: None,
            clock_epoch: util::high_res_time(),
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
            Msg::MouseMove(event) => self.on_mouse_move(event.client_x(), event.client_y()),
            Msg::MouseDown(event) => self.on_mouse_click(event.button(), true),
            Msg::MouseUp(event) => self.on_mouse_click(event.button(), false),
            Msg::Wheel(event) => self.on_wheel(event.delta_y()),
            Msg::TouchMove(event) => {
                if let Some(touch) = event.target_touches().item(0) {
                    self.on_mouse_move(touch.client_x(), touch.client_y());
                }
            }
            Msg::TouchDown(event) => {
                if event.target_touches().length() == 1 {
                    self.on_mouse_click(0, true)
                }
            }
            Msg::TouchUp(event) => {
                if event.target_touches().length() == 0 {
                    self.on_mouse_click(0, false)
                }
            }
        }
        false
    }

    fn change(&mut self, _: Props) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <div style="margin: 0; background-color: black;">
                <canvas
                    ref=self.bg_canvas_ref.clone()
                    style="width: 100vw; height: 100vh; z-index: 1; position: absolute; x: 0; y: 0;"
                    />
                <canvas
                    ref=self.scene_canvas_ref.clone()
                    style="width: 100vw; height: 100vh; z-index: 2; position: absolute; x: 0; y: 0;"
                    />
                <canvas
                    ref=self.ui_canvas_ref.clone()
                    onmousemove=self.link.callback(Msg::MouseMove)
                    onmousedown=self.link.callback(Msg::MouseDown)
                    onmouseup=self.link.callback(Msg::MouseUp)
                    onwheel=self.link.callback(Msg::Wheel)
                    ontouchmove=self.link.callback(Msg::TouchMove)
                    ontouchstart=self.link.callback(Msg::TouchDown)
                    ontouchend=self.link.callback(Msg::TouchUp)
                    style="width: 100vw; height: 100vh; z-index: 3; position: absolute; x: 0; y: 0;"
                    />

                <div
                    ref=self.debug_ref.clone()
                    style="\
                        padding-left: 10px; padding-top: 10px; \
                        z-index: 4; \
                        position: absolute; \
                        x: 0; y: 0; \
                        color: white; \
                        pointer-events: none; \
                        font-family: Helvetica, sans-serif; \
                        font-size: x-small;"
                    />
            </div>
        }
    }

    fn rendered(&mut self, _first: bool) {
        let window = web_sys::window().expect("Failed to get window object");
        self.on_resize(resize::WindowDimensions::get_dimensions(&window));
    }
}

/// Events in the game page.
pub enum Msg {
    /// Schedule a simulation frame.
    SimulationFrame(()),
    /// Schedule a render.
    RenderFrame(f64),
    /// Updates the window size.
    Resize(resize::WindowDimensions),
    /// Starts pressing a button.
    KeyDown(KeyboardEvent),
    /// Stops pressing a button.
    KeyUp(KeyboardEvent),
    /// Updates the mouse cursor position.
    MouseMove(MouseEvent),
    /// Starts pressing the mouse.
    MouseDown(MouseEvent),
    /// Stops pressing the mouse.
    MouseUp(MouseEvent),
    /// Scrolls the wheel.
    Wheel(WheelEvent),
    /// Starts touching the screen.
    TouchDown(TouchEvent),
    /// Stops touching the screen.
    TouchUp(TouchEvent),
    /// Moves the touched position of the screen.
    TouchMove(TouchEvent),
}

/// yew properties for [`Game`][Game].
#[derive(Clone, Properties)]
pub struct Props {
    /// Arguments for the game.
    pub args: GameArgs,
    /// Error handler.
    pub error_hook: Callback<Option<String>>,
}
