//! Renders the common toolbar.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use yew::prelude::*;

use traffloat::save;

/// Displays common toolbar buttons.
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
        match msg {
            Msg::SaveButton(format) => {
                let mut legion = self.props.legion.borrow_mut();
                legion.publish(traffloat::save::Request::builder().format(format).build());
                false
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let style = "
            position: absolute;
            top: 0;
            right: 0;
        ";
        html! {
            <nav style=style>
                <button
                    style="pointer-events: auto;"
                    onclick=self.link.callback(|_| Msg::SaveButton(save::Format::Text))
                >{ "Save as text" }</button>
                <button
                    style="pointer-events: auto;"
                    onclick=self.link.callback(|_| Msg::SaveButton(save::Format::Binary))
                >{ "Save as binary" }</button>
                { for self.props.cancel.as_ref().map(|cancel| html! {
                    <button
                        style="pointer-events: auto;"
                        onclick=cancel
                    >{ "Cancel" }</button>
                })}
            </nav>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    /// The user clicks the save button.
    SaveButton(save::Format),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The legion setup.
    pub legion: Rc<RefCell<traffloat::Legion>>,
    /// The cancel callback, or [`None`] if it should not be rendered.
    pub cancel: Option<Callback<MouseEvent>>,
}

#[codegen::system]
fn post_save(#[subscriber] responses: impl Iterator<Item = save::Response>) {
    for resp in responses {
        let array = js_sys::Uint8Array::from(&resp.data()[..]);

        let mut options = web_sys::BlobPropertyBag::new();
        options.type_(match resp.format() {
            save::Format::Text => "text/yaml",
            save::Format::Binary => "application/octet-stream",
        });
        let seq = std::iter::once(array).collect::<js_sys::Array>();
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&seq, &options)
            .expect("Cannot create Blob from Uint8Array");

        let url =
            web_sys::Url::create_object_url_with_blob(&blob).expect("Cannot create URL from Blob");

        let window = web_sys::window().expect("Window is undefined");
        let document = window.document().expect("Document is undefined");
        let elem = document
            .create_element("a")
            .expect("Cannot create element")
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .expect("<a> is not HtmlAnchorElement");
        elem.set_href(&url);
        elem.set_download(match resp.format() {
            save::Format::Text => "save.tsvt",
            save::Format::Binary => "save.tsvb",
        });
        elem.click();
    }
}

/// Sets up legion ECS for node info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(post_save_setup)
}
