//! A color palette for choosing colors.

use std::borrow::Cow;

use traffloat::space::Vector;
use yew::prelude::*;

use crate::options::ColorMap;
use crate::style::Style;

/// The gradient tab of the color picker dialog.
pub struct Comp {
    props:   Props,
    link:    ComponentLink<Self>,
    rgb_ref: [NodeRef; 3],
}

impl Comp {
    /// Compute the selected RGB of the components.
    fn rgb(&self) -> Option<[f64; 3]> {
        let mut ret = [0., 0., 0.];
        #[allow(clippy::indexing_slicing)]
        for ch in 0..3 {
            let elem = self.rgb_ref[ch].cast::<web_sys::HtmlInputElement>()?;
            let value = elem.value();
            let parsed: f64 = value.parse().ok()?;
            if parsed < 0. || parsed > 1. {
                return None;
            }
            ret[ch] = parsed;
        }
        Some(ret)
    }
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self { props, link, rgb_ref: Default::default() }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::UpdateRgb => true, // just recalculate
            Msg::Confirm => {
                if let Some(rgb) = self.rgb() {
                    self.props.callback.emit(ColorMap::Linear(
                        Vector::from_iterator(rgb),
                        Vector::from_iterator(rgb),
                    ));
                }
                false
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let mut button_style: Style = style! {
            "width": "100%",
            "height": "3em",
        }
        .clone();
        let rgb = self.rgb();
        button_style.rules.push((
            "background-color",
            if let Some([r, g, b]) = rgb {
                Cow::Owned(format!("rgb({}, {}, {})", r, g, b,))
            } else {
                Cow::Borrowed("white")
            },
        ));
        button_style.reset_lazy();
        html! {
            <>
                <table>
                    <tr>
                        <td>{ "Red" }</td>
                        <td>
                            <input
                                type="text"
                                ref=self.rgb_ref[0].clone()
                                onchange=self.link.callback(|_| Msg::UpdateRgb)
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Green" }</td>
                        <td>
                            <input
                                type="text"
                                ref=self.rgb_ref[1].clone()
                                onchange=self.link.callback(|_| Msg::UpdateRgb)
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Blue" }</td>
                        <td>
                            <input
                                type="text"
                                ref=self.rgb_ref[2].clone()
                                onchange=self.link.callback(|_| Msg::UpdateRgb)
                            />
                        </td>
                    </tr>
                </table>
                <button
                    onclick=self.link.callback(|_| Msg::Confirm)
                    disabled=rgb.is_none()
                >
                    { "Confirm" }
                </button>
            </>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    UpdateRgb,
    Confirm,
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    pub callback: Callback<ColorMap>,
}
