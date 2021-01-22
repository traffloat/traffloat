use yew::prelude::*;

pub struct ChatComp {
    link: ComponentLink<Self>,
    props: Properties,
}

impl Component for ChatComp {
    type Message = Message;
    type Properties = Properties;

    fn create(props: Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                { for self.props.messages.iter().map(|chat| html! {
                    <div>
                        <div>{ chat.speaker.as_str() }</div>
                        <div>{ chat.content.as_str() }</div>
                    </div>
                }) }
                // TODO chat input
            </div>
        }
    }
}

pub enum Message {}

#[derive(Clone, Properties)]
pub struct Properties {
    messages: Vec<Chat>,
}

#[derive(Clone)]
pub struct Chat {
    pub speaker: String,
    pub content: String,
}
