use yew::prelude::*;

pub struct Menu {
    link: ComponentLink<Self>,
    props: Properties,
}

impl Component for Menu {
    type Message = Message;
    type Properties = Properties;

    fn create(props: Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
        match msg {
            Message::UpdateAddress(data) => {
                self.props.addr = data.value;
                true
            }
            Message::UpdatePort(data) => {
                let value = match data {
                    ChangeData::Value(value) => value,
                    _ => unreachable!("Input change data should be value"),
                };
                let value = match value.parse::<u16>() {
                    Ok(value) => value,
                    Err(_) => self.props.port,
                };
                self.props.port = value;
                true
            }
            Message::UpdateInsecure(_) => {
                self.props.allow_insecure = !self.props.allow_insecure;
                true
            }
            Message::UpdateName(data) => {
                self.props.name = data.value;
                true
            }
            Message::Connect => {
                self.props.connect_hook.emit(super::ClientArgs {
                    addr: self.props.addr.clone(),
                    port: self.props.port,
                    allow_insecure: self.props.allow_insecure,
                    name: self.props.name.clone(),
                });
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div style="max-width: 640px; margin: 0 auto;">
                <h1>{ "traffloat" }</h1>
                <table>
                    <tr>
                        <td><label for="address">{ "Address" }</label></td>
                        <td><input id="address" oninput=self.link.callback(Message::UpdateAddress) value=self.props.addr /></td>
                    </tr>
                    <tr>
                        <td><label for="port">{ "Port" }</label></td>
                        <td><input id="port" onchange=self.link.callback(Message::UpdatePort) value=self.props.port /></td>
                    </tr>
                    <tr>
                        <td><label for="insecure">{ "Allow insecure" }</label></td>
                        <td><input id="insecure" type="checkbox" onchange=self.link.callback(Message::UpdateInsecure) checked=self.props.allow_insecure /></td>
                    </tr>
                    <tr>
                        <td><label for="name">{ "Name" }</label></td>
                        <td><input id="name" oninput=self.link.callback(Message::UpdateName) value=self.props.name /></td>
                    </tr>
                </table>
                <button disabled=!common::is_valid_name(&self.props.name) onclick=self.link.callback(|_| Message::Connect)>{ "Connect" }</button>
                { for self.props.err.iter().map(|err| html! {
                    <div style="color: red">
                        <h3>{ "Error" }</h3>
                        <p>{ err }</p>
                    </div>
                }) }
            </div>
        }
    }
}

pub enum Message {
    UpdateAddress(InputData),
    UpdatePort(ChangeData),
    UpdateInsecure(ChangeData),
    UpdateName(InputData),
    Connect,
}

#[derive(Clone, Debug, Properties)]
pub struct Properties {
    pub err: Option<String>,
    pub connect_hook: Callback<super::ClientArgs>,
    #[prop_or("localhost".to_string())]
    pub addr: String,
    #[prop_or(common::DEFAULT_PORT)]
    pub port: u16,
    #[prop_or(false)]
    pub allow_insecure: bool,
    #[prop_or_default]
    pub name: String,
}
