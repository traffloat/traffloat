use std::rc::Rc;

use specs::{world::Builder, WorldExt};
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};
use yew::prelude::*;
use yew::services::render::{RenderService, RenderTask};
use yew::services::resize::WindowDimensions;

use crate::render::{Camera, RenderContext};
use common::types::Entity;

pub struct Canvas {
    link: ComponentLink<Self>,
    props: Properties,
    entity: Entity,
    id: String,
    render_task: Option<RenderTask>,
}

impl Component for Canvas {
    type Message = Message;
    type Properties = Properties;

    fn create(props: Properties, link: ComponentLink<Self>) -> Self {
        let entity = {
            let (world, _) = &mut *props.setup.borrow_mut();
            world
                .create_entity()
                .with::<Camera>(Camera::default())
                .build()
        };

        let id = rand::random::<[char; 16]>().iter().collect::<String>();

        Self {
            link,
            props,
            entity,
            id,
            render_task: None,
        }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
        match msg {
            Message::Render(_) => {
                let (world, _) = &mut *self.props.setup.borrow_mut();
                let mut ctx = world.write_component::<RenderContext>();
                let ctx = ctx
                    .get_mut(self.entity)
                    .expect("Render requested without context initialization");
                ctx.should_render = true;
                let task =
                    RenderService::request_animation_frame(self.link.callback(Message::Render));
                self.render_task = Some(task);
                false
            }
        }
    }

    fn change(&mut self, props: Properties) -> ShouldRender {
        self.props = props;
        let (world, _) = &*self.props.setup.borrow();
        let mut cameras = world.write_component::<Camera>();
        let camera = cameras.get_mut(self.entity).expect("Camera was removed");
        camera.aspect = self.props.window.aspect() * self.props.width / self.props.height;

        false
    }

    fn view(&self) -> Html {
        let style = format!(
            "\
            position: absolute;
            x: {}; \
            y: {}; \
            width: {}vw; \
            height: {}vh; \
            display: block",
            self.props.x, self.props.y, self.props.width, self.props.height,
        );
        html! {
            <canvas id=self.id
                width={self.props.window.width as f32 * self.props.width}
                height={self.props.window.height as f32 * self.props.height}
                style=style.as_str(),
                />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        fn canvas() -> (HtmlCanvasElement, WebGlRenderingContext) {
            use wasm_bindgen::JsCast;

            let document = web_sys::window().unwrap().document().unwrap();
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

        let (_, gl) = canvas();

        let (world, _) = &*self.props.setup.borrow();
        let canvas = RenderContext::new(gl, self.props.server_seed);

        world
            .write_component::<RenderContext>()
            .insert(self.entity, canvas)
            .expect("Could not insert render context");

        let mut camera = world.write_component::<Camera>();
        let camera = camera
            .get_mut(self.entity)
            .expect("Camera should be initialized");

        #[allow(clippy::cast_precision_loss)]
        {
            camera.aspect = (self.props.width as f32) / (self.props.height as f32);
        }

        if first_render {
            let task = RenderService::request_animation_frame(self.link.callback(Message::Render));
            self.render_task = Some(task);
        }
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        let (world, _) = &mut *self.props.setup.borrow_mut();
        world
            .delete_entity(self.entity)
            .expect("entity killed before canvas drop");
    }
}

pub enum Message {
    Render(f64),
}

#[derive(Clone, Properties)]
pub struct Properties {
    setup: super::Setup,
    server_seed: u64,
    window: Dim,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Properties {
    fn x_offset(&self) -> i32 {
        (self.window.width as f32 * self.x) as i32
    }
    fn y_offset(&self) -> i32 {
        (self.window.height as f32 * self.y) as i32
    }
    fn width_offset(&self) -> i32 {
        (self.window.width as f32 * self.width) as i32
    }
    fn height_offset(&self) -> i32 {
        (self.window.height as f32 * self.height) as i32
    }
    fn aspect(&self) -> f32 {
        self.window.aspect() * self.width / self.height
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dim {
    pub width: i32,
    pub height: i32,
}

impl Dim {
    pub fn aspect(self) -> f32 {
        (self.width as f32) / (self.height as f32)
    }
}

impl From<(i32, i32)> for Dim {
    fn from((width, height): (i32, i32)) -> Self {
        Self { width, height }
    }
}

impl From<WindowDimensions> for Dim {
    fn from(dim: WindowDimensions) -> Self {
        Self {
            width: dim.width,
            height: dim.height,
        }
    }
}
