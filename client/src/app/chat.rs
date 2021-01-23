use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

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
                { for self.props.messages.messages().map(|chat| html! {
                    <div>
                        { "<" }
                        <span>{ chat.speaker.as_str() }</span>
                        { "> " }
                        <span>{ chat.content.as_str() }</span>
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
    pub messages: List,
}

pub struct Chat {
    pub speaker: String,
    pub content: String,
}

#[derive(Clone)]
pub struct List {
    pub deque: Rc<RefCell<VecDeque<Chat>>>,
    pub size: usize,
}

impl List {
    pub fn push(&self, message: Chat) {
        let deque = &mut *self.deque.borrow_mut();
        deque.push_back(message);
    }

    pub fn push_system(&self, content: String) {
        self.push(Chat {
            speaker: String::from("Traffloat"),
            content,
        })
    }

    pub fn messages(&self) -> impl Iterator<Item = std::cell::Ref<'_, Chat>> {
        use std::cell::Ref;

        let borrow = self.deque.borrow();
        let size = borrow.len();
        (0..size).map(move |index| {
            Ref::map(Ref::clone(&borrow), |deque| {
                deque.get(index).expect("index < size")
            })
        })
    }
}
