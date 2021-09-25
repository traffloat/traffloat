//! Options menu for Traffloat client.

use std::{cell::RefCell, rc::Rc};

use yew::{prelude::*, services::storage};

use crate::options::{self, Options};

mod toggle;

/// Options menu for Traffloat client.
pub struct Comp {
    props: Props,
    link: ComponentLink<Self>,
    options: Options,
}

impl Comp {
    fn save_options(&mut self) {
        let mut storage = storage::StorageService::new(storage::Area::Local)
            .expect("Failed to fetch localStorage");
        storage.store(options::STORAGE_KEY, yew::format::Json(&self.options));

        if let Some(legion) = self.props.legion.as_ref() {
            let legion = legion.borrow_mut();
            let mut options = legion.resources.get_mut::<Options>();
            if let Some(options) = options.as_mut() {
                **options = self.options.clone();
            }
        }
    }
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
                self.save_options();
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
                    title="Render stars"
                    value=self.options.graphics().render_stars()
                    callback=Msg::update_bool(&self.link, |options| options.graphics_mut().render_stars_mut())
                    />
                <toggle::Comp
                    title="Render reticle"
                    value=self.options.graphics().render_reticle()
                    callback=Msg::update_bool(&self.link, |options| options.graphics_mut().render_reticle_mut())
                    />
                { for cfg!(feature = "render-debug").then(|| html! {
                    <toggle::Comp
                        title="Render debug info"
                        value=self.options.graphics().render_debug_info()
                        callback=Msg::update_bool(&self.link, |options| options.graphics_mut().render_debug_info_mut())
                        />
                }) }
            </div>
        }
    }
}

type OptionsField = fn(&mut Options) -> &mut bool;

/// Events for [`Comp`].
pub enum Msg {
    UpdateBool(OptionsField, bool),
}

impl Msg {
    fn update_bool(link: &ComponentLink<Comp>, f: OptionsField) -> Callback<bool> {
        link.callback(move |b| Self::UpdateBool(f, b))
    }
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The legion setup, if opened in-game.
    pub legion: Option<Rc<RefCell<traffloat::Legion>>>,
}
