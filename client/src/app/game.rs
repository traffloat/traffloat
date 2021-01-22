use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use specs::WorldExt;
use yew::prelude::*;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::services::keyboard::{KeyListenerHandle, KeyboardService};
use yew::services::resize::{ResizeService, ResizeTask, WindowDimensions};
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

use super::canvas;
use crate::keymap::{Action, ActionEvent};
use common::types::{Clock, Time};

pub struct Game {
    link: ComponentLink<Self>,
    props: Properties,
    _resize_task: ResizeTask,
    _dispatch_task: IntervalTask,
    ws_addr: String,
    ws: WebSocketTask,
    key_handles: Vec<KeyListenerHandle>,
    dim: WindowDimensions,
    setup: super::Setup,
}

impl Game {
    fn document() -> web_sys::Document {
        let window = web_sys::window().unwrap();
        window.document().unwrap()
    }
}

impl Component for Game {
    type Message = Message;
    type Properties = Properties;

    fn create(props: Properties, link: ComponentLink<Self>) -> Self {
        let resize_task = ResizeService::new().register(link.callback(Message::WindowResize));
        let dispatch_task =
            IntervalService::spawn(Duration::from_millis(10), link.callback(Message::Dispatch));

        let ws_addr = format!("wss://{}:{}", props.addr, props.port);
        let ws = WebSocketService::connect_binary(
            &ws_addr,
            link.callback(Message::WsReceive),
            link.callback(Message::WsStatus),
        )
        .unwrap();

        let body = Self::document().body().unwrap();

        let mut key_handles = Vec::new();
        key_handles.push(KeyboardService::register_key_down(
            &body,
            link.callback(Message::KeyDown),
        ));
        key_handles.push(KeyboardService::register_key_up(
            &body,
            link.callback(Message::KeyUp),
        ));

        let setup: super::Setup = Rc::new(RefCell::new(crate::setup_specs()));

        Self {
            link,
            props,
            _resize_task: resize_task,
            _dispatch_task: dispatch_task,
            ws_addr,
            ws,
            key_handles,
            dim: WindowDimensions::get_dimensions(&web_sys::window().unwrap()),
            setup,
        }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
        fn update_key(setup: &super::Setup, key: KeyboardEvent, down: bool) -> ShouldRender {
            let (world, _) = &mut *setup.borrow_mut();

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
            Message::KeyDown(key) => update_key(&self.setup, key, true),
            Message::KeyUp(key) => update_key(&self.setup, key, false),
            Message::Dispatch(()) => {
                let (world, dispatcher) = &mut *self.setup.borrow_mut();
                let clock: &mut Clock = world.get_mut().expect("Clock was initialized at setup");
                clock.inc_time(Time(1));
                dispatcher.dispatch(world);
                world.maintain();
                false
            }
            Message::WsReceive(message) => {
                let (world, _) = &mut *self.setup.borrow_mut();
                // TODO send message to message handler
                false
            }
            Message::WsStatus(status) => {
                // TODO
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {}
    }
}

pub enum Message {
    WindowResize(WindowDimensions),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    Dispatch(()),
    WsReceive(anyhow::Result<Vec<u8>>),
    WsStatus(WebSocketStatus),
}

#[derive(Clone, Debug, Properties)]
pub struct Properties {
    pub addr: String,
    pub port: u16,
    pub allow_insecure: bool,
    pub name: String,
    pub identity: Vec<u8>,
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
