use bevy::app::{self, App, Plugin};
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{ParamSet, SystemParam};
use traffloat_macro_util::fan_out;
use traffloat_proto::proto;

use crate::view;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_message::<Approved>();
        app.add_systems(app::Update, handle_requests_system.in_set(HandlerSystemSet));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct HandlerSystemSet;

#[derive(Message)]
pub struct Approved {
    pub viewer: Entity,
    pub body:   proto::Request,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandlerClass {
    Mutate,
}

pub trait Handler {
    type Request;

    fn classify(request: &Self::Request) -> HandlerClass;

    fn handle(&mut self, viewer: Entity, request: &Self::Request);
}

fn handle_requests_system(mut approved: MessageReader<Approved>, mut params: HandlerMux) {
    let mut messages: Vec<_> = approved.read().collect();
    messages.sort_by_key(|message| HandlerMux::classify(&message.body));

    for &Approved { viewer, ref body } in messages {
        tracing::info_span!("handle_request", request = ?<&'static str>::from(body)).in_scope(
            || {
                tracing::debug!("Handle request {body:?}");
                params.handle(viewer, body);
            },
        );
    }
}

macro_rules! define_params {
    (
        [$w:lifetime, $s:lifetime]
        $paramset_tuple:ty;
        {
            $(
                $message:ident ($param:ty) $path:tt,
            )*
        }
    ) => {
        #[derive(SystemParam)]
        struct HandlerMux<$w, $s> {
            ps: $paramset_tuple,
        }

        impl<$w, $s> Handler for HandlerMux<$w, $s> {
            type Request = proto::Request;

            fn classify(request: &Self::Request) -> HandlerClass {
                match request {
                    $(
                        proto::Request::$message(request) => <$param as Handler>::classify(request),
                    )*
                }
            }

            fn handle(&mut self, viewer: Entity, request: &proto::Request) {
                match request {
                    $(
                        proto::Request::$message(request) => {
                            define_params_handle_let!(self, param, $path);
                            Handler::handle(&mut param, viewer, request)
                        }
                    )*
                }
            }
        }
    }
}

macro_rules! define_params_handle_let {
    ($mux:ident, $var:ident, ($($path:ident)*)) => {
        let $var = &mut $mux.ps;
        $(
            let mut $var = $var.$path();
        )*
    }
}

macro_rules! define_params_item {
    (
        [$w:lifetime, $s:lifetime]
        $message:ident ($param:ty)
    ) => {
        $param
    };
}

macro_rules! define_params_tuple {
    (
        [$w:lifetime, $s:lifetime]
        $($params:ty,)*
    ) => {
        ParamSet<$w, $s, (
            $($params,)*
        )>
    }
}

fan_out! {
    ['w, 's]
    define_params, define_params_tuple, define_params_item;
    8, 2;
    SetSubscription(view::SetSubscriptionHandler<'w, 's>),
    SetViewFocus(view::SetViewFocusHandler<'w, 's>),
}
