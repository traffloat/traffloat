use std::time::Duration;

use yew::prelude::*;
use yew::services::{interval, render as render_srv, resize};

use super::{GameArgs, SpGameArgs};
use crate::render;
use traffloat::types::{Clock, Time};
use traffloat::SetupEcs;

pub struct Game {
    props: Props,
    link: ComponentLink<Self>,
    legion: traffloat::Legion,
    _resize_task: resize::ResizeTask,
    render_task: render_srv::RenderTask,
    _simulation_task: interval::IntervalTask,
    render_flag: render::RenderFlag,
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
        self.legion.run();
    }

    fn request_render(&mut self) {
        self.render_flag.cell.replace(self.canvas_context());
        self.render_task = render_srv::RenderService::request_animation_frame(
            self.link.callback(Msg::RenderFrame),
        );
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
        let canvas = match self.canvas_ref.cast::<web_sys::HtmlCanvasElement>() {
            Some(canvas) => canvas,
            None => return,
        };
        canvas.set_width(dim.width as u32);
        canvas.set_height(dim.height as u32);

        self.request_render();
    }
}

impl Component for Game {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let render_flag = render::RenderFlag::default();
        let legion = SetupEcs::default()
            .uses(crate::setup_ecs)
            .uses(|setup| render::setup_ecs(setup, &render_flag))
            .build(); // TODO setup depending on gamemode

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
            render_flag,
            canvas_ref: NodeRef::default(),
            link,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::SimulationFrame(()) => self.simulate(),
            Msg::RenderFrame(_) => self.request_render(),
            Msg::Resize(dim) => self.on_resize(dim),
        }
        false
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <div style="margin: 0;">
                <canvas
                    ref=self.canvas_ref.clone()
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
}

#[derive(Clone, Properties)]
pub struct Props {
    pub args: GameArgs,
    pub error_hook: Callback<String>,
}
