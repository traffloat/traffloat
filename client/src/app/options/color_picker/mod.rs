//! A color picker component in the options menu.

use std::borrow::Cow;

use itertools::Itertools;
use traffloat::Finite;
use yew::prelude::*;

use crate::options::ColorMap;
use crate::style::{self, Style};

mod dialog;
mod gradient;
mod palette;
mod preset;
mod tab;

/// A color picker component in the options menu.
pub struct Comp {
    props:       Props,
    link:        ComponentLink<Self>,
    dialog_open: bool,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self { props, link, dialog_open: false }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::Change(_) => {
                todo!()
            }
            Msg::OpenDialog(_) => {
                self.dialog_open = true;
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        style! { static PREVIEW_STYLE =
            "display": "inline-block",
            "width": "8ch",
            "height": "1em",
            "text-align": "center",
            "cursor": "pointer",
        }
        fn preview(this: &Comp, color: ColorMap) -> Html {
            let mut style: Style = PREVIEW_STYLE.clone();
            style.rules.push(match color {
                ColorMap::Linear(mut from, mut to) => {
                    from *= 255.;
                    to *= 255.;
                    (
                        "background",
                        Cow::Owned(format!(
                            "rgba(0, 0, 0, 0) linear-gradient(to right, rgb({}, {}, {}), rgb({}, \
                             {}, {}))",
                            from[0], from[1], from[2], to[0], to[1], to[2],
                        )),
                    )
                }
                ColorMap::Trapeziums([r, g, b]) => {
                    #[rustfmt::skip]
                    let mut nodes = [
                        Finite::new(0.), Finite::new(1.),
                        Finite::new(r.min_start()), Finite::new(r.max_start()), Finite::new(r.min_end()), Finite::new(r.max_end()),
                        Finite::new(g.min_start()), Finite::new(g.max_start()), Finite::new(g.min_end()), Finite::new(g.max_end()),
                        Finite::new(b.min_start()), Finite::new(b.max_start()), Finite::new(b.min_end()), Finite::new(b.max_end()),
                    ];
                    nodes.sort_unstable();
                    let gradient = IntoIterator::into_iter(nodes)
                        .map(|node| {
                            let nr = r.convert(node.value()) * 255.;
                            let ng = g.convert(node.value()) * 255.;
                            let nb = b.convert(node.value()) * 255.;

                            format!("rgb({}, {}, {}) {}%", nr, ng, nb, node.value() * 100.)
                        })
                        .join(", ");

                    (
                        "background",
                        Cow::Owned(format!(
                            "rgba(0, 0, 0, 0) linear-gradient(to right, {})",
                            gradient,
                        )),
                    )
                }
            });
            style.reset_lazy();
            html! {
                <span
                    style=style::NonStatic(style)
                    onclick=this.link.callback(Msg::OpenDialog)
                    />
            }
        }

        html! {
            <tr style=&*super::ROW_STYLE>
                <th style=&*super::ROW_KEY_STYLE>{ self.props.title }</th>
                <td>
                    { self.props.value.map_or_else(|| html! {
                        <span
                            style=&*PREVIEW_STYLE
                            onclick=self.link.callback(Msg::OpenDialog)
                        >
                            { "Disabled" }
                        </span>
                    }, |color| preview(self, color))}
                </td>

                { for self.dialog_open.then(|| html! {
                    <dialog::Comp />
                })}
            </tr>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    Change(ChangeData),
    OpenDialog(MouseEvent),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    pub title:    &'static str,
    pub value:    Option<ColorMap>,
    pub callback: Callback<Option<ColorMap>>,
    #[prop_or(false)]
    pub disabled: bool,
}
