//! Options menu for Traffloat client.

use yew::prelude::*;

/// Options menu for Traffloat client.
pub struct Comp {
    props: Props,
    link: ComponentLink<Self>,
}
impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <tr>
                <th>{ self.props.name }</th>
                <td>
                    <input type="checkbox" checked=self.props.value />
                    { for self.props.value.then(|| self.props.on_message) }
                    { for (!self.props.value).then(|| self.props.off_message) }
                </td>
            </tr>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    pub name: &'static str,
    pub key: super::OptionsField,
    pub value: bool,
    #[prop_or("On")]
    pub on_message: &'static str,
    #[prop_or("Off")]
    pub off_message: &'static str,
}
