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
use super::chat;
use crate::keymap::{Action, ActionEvent};
use crate::session::{self, Session};
use common::types::{Clock, Time};

pub struct Game {
    link: ComponentLink<Self>,
    props: Properties,
    _resize_task: ResizeTask,
    _dispatch_task: IntervalTask,
    ws_addr: String,
    key_handles: Vec<KeyListenerHandle>,
    dim: WindowDimensions,
    setup: super::Setup,
    chat_list: chat::List,
    session: Session,
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

        let chat_list = chat::List {
            deque: Rc::default(),
            size: 100,
        };

        let ws_addr = format!("ws://{}:{}", props.addr, props.port); // TODO change this back to wss
        chat_list.push_system(format!("Connecting to {}", ws_addr));
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

        let session = Session::new(
            props.allow_insecure,
            ws,
            props.name.clone(),
            props.hashed_identity(),
        );
        Self {
            link,
            props,
            _resize_task: resize_task,
            _dispatch_task: dispatch_task,
            ws_addr,
            key_handles,
            dim: WindowDimensions::get_dimensions(&web_sys::window().unwrap()),
            setup,
            chat_list,
            session,
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
                let message = match message {
                    Ok(message) => message,
                    Err(err) => {
                        // this should never happen anyway, but let's try to handle this properly
                        let err = format!("Unexpected receive error {}", err);
                        self.chat_list.push_system(err.clone());
                        self.props.error_hook.emit(Some(err));
                        return false;
                    }
                };
                let message = match self.session.handle_message(&message, {
                    let chat_list = self.chat_list.clone(); // chat::List is backed by an Rc
                    move |msg| chat_list.push_system(msg)
                }) {
                    Ok(Some(message)) => {
                        let (world, _) = &mut *self.setup.borrow_mut();
                        // TODO send message to message handler
                    }
                    Ok(None) => (),
                    Err(err) => {
                        let err = format!("Error receiving packet: {}", err);
                        self.chat_list.push_system(err.clone());
                        self.props.error_hook.emit(Some(err));
                        return false;
                    }
                };
                false
            }
            Message::WsStatus(status) => match status {
                WebSocketStatus::Opened => {
                    self.session.handle_opened();
                    false
                }
                WebSocketStatus::Error => {
                    self.chat_list
                        .push_system(String::from("Websocket connection error"));
                    match self.session.handle_error() {
                        session::ErrorHandler::RetryInsecure => {
                            self.chat_list.push_system(String::from(
                                "Retrying connection with insecure connection",
                            ));
                            let ws_addr = format!("ws://{}:{}", self.props.addr, self.props.port);
                            self.chat_list
                                .push_system(format!("Connecting to {}", ws_addr));
                            let ws = WebSocketService::connect_binary(
                                &ws_addr,
                                self.link.callback(Message::WsReceive),
                                self.link.callback(Message::WsStatus),
                            )
                            .unwrap();
                            self.session.ws = ws;
                            false
                        }
                        session::ErrorHandler::Close => {
                            self.props
                                .error_hook
                                .emit(Some(String::from("Websocket connection error")));
                            false
                        }
                    }
                }
                WebSocketStatus::Closed => {
                    self.chat_list
                        .push_system(String::from("Websocket connection closed"));
                    self.session.handle_closed();
                    self.props
                        .error_hook
                        .emit(Some(String::from("Websocket connection closed")));
                    false
                }
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <chat::ChatComp messages=&self.chat_list/>
        }
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
    pub error_hook: Callback<Option<String>>,
}

impl Properties {
    fn server_seed(&self) -> u64 {
        use crc64fast::Digest;
        let mut c = Digest::new();
        c.write(self.addr.as_bytes());
        c.write(&self.port.to_le_bytes());
        c.sum64()
    }

    fn hashed_identity(&self) -> Vec<u8> {
        use sha2::Digest;

        let mut digest = sha2::Sha512::new();
        digest.update(&self.identity);
        digest.update(&self.port.to_le_bytes());
        digest.update(self.addr.as_bytes());

        digest.finalize().as_slice().to_vec()
    }
}
