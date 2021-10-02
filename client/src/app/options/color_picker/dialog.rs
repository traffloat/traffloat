//! The color picker dialog.

use std::borrow::Cow;

use yew::prelude::*;

use super::{gradient, preset, tab};
use crate::options::ColorMap;
use crate::style::{self, Style};

/// A color picker component in the options menu.
pub struct Comp {
    props: Props,
    link:  ComponentLink<Self>,
    body:  Body,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self { props, link, body: Body::Gradient }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::SwitchGradient => {
                self.body = Body::Gradient;
                true
            }
            Msg::SwitchPreset => {
                self.body = Body::Preset;
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div style=style! {
                "position": "fixed",
                "top": "0",
                "left": "0",
                "width": "100%",
                "height": "100%",
                "background-color": "rgba(0, 0, 0, 0.8)",
            }>
                <div style=style! {
                    "position": "fixed",
                    "width": "80%",
                    "height": "80%",
                    "top": "50%",
                    "left": "50%",
                    "transform": "translate(-50%, -50%)",
                    "background-color": "white",
                }>
                    <div style=style! {
                        "display": "flex",
                    }>
                        <tab::Comp
                            title="Scale by value"
                            callback=self.link.callback(|()| Msg::SwitchGradient) />
                        <tab::Comp
                            title="Colorbar preset"
                            callback=self.link.callback(|()| Msg::SwitchPreset) />
                    </div>
                </div>

                { self.body.view() }
            </div>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    SwitchGradient,
    SwitchPreset,
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {}

enum Body {
    Gradient,
    Preset,
}

impl Body {
    fn view(&self) -> Html {
        match self {
            Self::Gradient => html! {
                <gradient::Comp />
            },
            Self::Preset => html! {
                <preset::Comp />
            },
        }
    }
}
