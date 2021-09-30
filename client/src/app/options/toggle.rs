//! Options menu for Traffloat client.

use yew::prelude::*;

/// Options menu for Traffloat client.
pub struct Comp {
    props: Props,
    link:  ComponentLink<Self>,
}
impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self { Self { props, link } }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::Change(_) => {
                self.props.callback.emit(!self.props.value);
                false // parent will update us anyway
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <tr>
                <th>{ self.props.title }</th>
                <td>
                    <input
                        type="checkbox"
                        checked=self.props.value
                        onchange=self.link.callback(Msg::Change)
                        />
                    { for self.props.value.then(|| self.props.on_message) }
                    { for (!self.props.value).then(|| self.props.off_message) }
                </td>
            </tr>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    Change(ChangeData),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    pub title:       &'static str,
    pub value:       bool,
    pub callback:    Callback<bool>,
    #[prop_or("On")]
    pub on_message:  &'static str,
    #[prop_or("Off")]
    pub off_message: &'static str,
}
