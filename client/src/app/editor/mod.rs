//! The scenraio viewer/editor.

use std::rc::Rc;

use yew::prelude::*;

use traffloat::save;

/// Displays an editor for ducts in an edge.
pub struct Comp {
    file: save::SaveFile,
    link: ComponentLink<Self>,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let file = match save::parse(&props.buf) {
            Ok(file) => file,
            Err(err) => {
                props
                    .close_hook
                    .emit(Some(format!("Error reading save file: {}", err)));
                return Self {
                    file: Default::default(), // this value shouldn't be used anyway.
                    link,
                };
            }
        };

        Self { file, link }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        unimplemented!("I don't know when editor::Comp would be re-rendered")
    }

    fn view(&self) -> Html {
        html! {
            <main>
                // TODO
            </main>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub buf: Rc<[u8]>,
    pub close_hook: Callback<Option<String>>,
}
