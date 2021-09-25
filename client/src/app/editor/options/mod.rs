//! Options menu for Traffloat client.

use yew::{prelude::*, services::storage};

use crate::options;

mod toggle;

/// Options menu for Traffloat client.
pub struct Comp {
    props: Props,
    link: ComponentLink<Self>,
    options: options::Options,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let storage = storage::StorageService::new(storage::Area::Local)
            .expect("Failed to fetch localStorage");
        let yew::format::Json(options) = storage.restore(options::STORAGE_KEY);
        let options = options.unwrap_or_default();
        Self { props, link, options }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::UpdateBool(key, value) => {
                let field = key(&mut self.options);
                *field = value;
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
            <div>
                <h2>{ "Graphics" }</h2>
                <toggle::Comp
                    name="Render stars"
                    key={|options| options.graphics_mut().render_stars_mut()}
                    value=self.options.graphics().render_stars()
                    />
                <toggle::Comp
                    name="Render reticle"
                    key={|options| options.graphics_mut().render_reticle_mut()}
                    value=self.options.graphics().render_reticle()
                    />
            </div>
        }
    }
}

type OptionsField = fn(&mut options::Options) -> &mut bool;

/// Events for [`Comp`].
pub enum Msg {
    UpdateBool(OptionsField, bool),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {}
