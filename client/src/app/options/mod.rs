//! Options menu for Traffloat client.

use std::cell::RefCell;
use std::rc::Rc;

use yew::prelude::*;
use yew::services::storage;

use crate::options::{self, ColorMap, Options};

mod color_picker;
mod toggle;

/// Options menu for Traffloat client.
pub struct Comp {
    props:   Props,
    link:    ComponentLink<Self>,
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
        let options = options.unwrap_or_else(|_| Options::default());
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
            Msg::UpdateColor(key, value) => {
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
            <div style=style!("margin": "50px")>
                <h2 style=&*HEADER_STYLE>{ "Graphics" }</h2>
                <table>
                    <toggle::Comp
                        title="Background stars"
                        value=self.options.graphics().render_stars()
                        callback=Msg::update_bool(&self.link, |options| options.graphics_mut().render_stars_mut())
                        on_message="Show"
                        off_message="Hide"
                        />
                    <toggle::Comp
                        title="Axis reticle"
                        value=self.options.graphics().render_reticle()
                        callback=Msg::update_bool(&self.link, |options| options.graphics_mut().render_reticle_mut())
                        on_message="Show"
                        off_message="Hide"
                        />
                    { for cfg!(feature = "render-debug").then(|| html! {
                        <toggle::Comp
                            title="Debug info"
                            value=self.options.graphics().render_debug_info()
                            callback=Msg::update_bool(&self.link, |options| options.graphics_mut().render_debug_info_mut())
                            on_message="Show"
                            off_message="Hide"
                            />
                    }) }
                </table>

                <h3 style=&*HEADER_STYLE>{ "Buildings" }</h3>
                <toggle::Comp
                    title="Render"
                    value=self.options.graphics().node().render()
                    callback=Msg::update_bool(&self.link, |options| options.graphics_mut().node_mut().render_mut())
                    on_message="Show"
                    off_message="Hide"
                    />
                <color_picker::Comp
                    title="Brightness shading"
                    value=self.options.graphics().node().brightness()
                    callback=Msg::update_color_map(&self.link, |options| options.graphics_mut().node_mut().brightness_mut())
                    disabled=!self.options.graphics().node().render()
                    />
                <color_picker::Comp
                    title="Hitpoint shading"
                    value=self.options.graphics().node().hitpoint()
                    callback=Msg::update_color_map(&self.link, |options| options.graphics_mut().node_mut().hitpoint_mut())
                    disabled=!self.options.graphics().node().render()
                    />

                <h3 style=&*HEADER_STYLE>{ "Corridors" }</h3>
                <toggle::Comp
                    title="Render"
                    value=self.options.graphics().edge().render()
                    callback=Msg::update_bool(&self.link, |options| options.graphics_mut().edge_mut().render_mut())
                    on_message="Show"
                    off_message="Hide"
                    />
            </div>
        }
    }
}

type OptionsField<T> = fn(&mut Options) -> &mut T;

/// Events for [`Comp`].
pub enum Msg {
    UpdateBool(OptionsField<bool>, bool),
    UpdateColor(OptionsField<Option<ColorMap>>, Option<ColorMap>),
}

impl Msg {
    fn update_bool(link: &ComponentLink<Comp>, f: OptionsField<bool>) -> Callback<bool> {
        link.callback(move |b| Self::UpdateBool(f, b))
    }

    fn update_color_map(
        link: &ComponentLink<Comp>,
        f: OptionsField<Option<ColorMap>>,
    ) -> Callback<Option<ColorMap>> {
        link.callback(move |b| Self::UpdateColor(f, b))
    }
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The legion setup, if opened in-game.
    pub legion: Option<Rc<RefCell<traffloat::Legion>>>,
}

style! { static HEADER_STYLE =
    "margin-bottom": "0.5em",
}
style! { static ROW_STYLE =
    "margin": "0.2em",
}
style! { static ROW_KEY_STYLE =
    "text-align": "left",
    "padding-right": "2ch",
    "width": "200px",
}
