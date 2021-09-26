use std::cell::RefCell;
use std::rc::Rc;

use yew::prelude::*;

use super::{duct_editor, edge_preview, node_preview, toolbar};
use crate::app::options;
use crate::input::keyboard;

/// Wrapper for UI elements.
pub struct Wrapper {
    props: Props,
    link: ComponentLink<Self>,
    node_preview_args: Option<node_preview::Args>,
    edge_preview_args: Option<edge_preview::Args>,
    duct_editor_args: Option<duct_editor::Args>,
    options_opened: bool,
    display_toolbar: bool,
}

impl Component for Wrapper {
    type Message = Update;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        props.updater_ref().set(link.callback(|update| update));

        Self {
            props,
            link,
            node_preview_args: None,
            edge_preview_args: None,
            duct_editor_args: None,
            options_opened: false,
            display_toolbar: true,
        }
    }

    fn update(&mut self, msg: Update) -> ShouldRender {
        match msg {
            Update::SetNodePreview(args) => {
                match (&self.node_preview_args, &args) {
                    (None, None) => return false,
                    (Some(old), Some(new)) if old.entity == new.entity => return false,
                    _ => (),
                }
                self.node_preview_args = args;
                true
            }
            Update::SetEdgePreview(args) => {
                match (&self.edge_preview_args, &args) {
                    (None, None) => return false,
                    (Some(old), Some(new)) if old.entity == new.entity => return false,
                    _ => (),
                }
                self.edge_preview_args = args;
                true
            }
            Update::SetDuctEditor(args) => {
                self.duct_editor_args = args;
                true
            }
            Update::OpenOptions => {
                self.options_opened = true;
                true
            }
            Update::Edit => {
                if let Some(args) = self.edge_preview_args.as_ref() {
                    let args = duct_editor::Args { entity: args.entity };
                    self.link.send_message(Update::SetDuctEditor(Some(args)));
                }
                false
            }
            Update::Cancel => {
                if let Some(args) = self.duct_editor_args.as_ref() {
                    let mut legion = self.props.legion.borrow_mut();
                    args.save(&mut *legion);
                    self.duct_editor_args = None;
                }
                if self.options_opened {
                    self.options_opened = false;
                }
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        props.updater_ref().set(self.link.callback(|update| update));
        self.props = props;
        false // we just modified the setter, but there is nothing to render yet
    }

    fn view(&self) -> Html {
        html! {
            <div style=style!(
                "z-index": "3",
                "position": "absolute",
                "width": "100vw", "height": "100vh",
                "pointer-events": "none",
                "x": "0", "y": "0",
                "font-family": "Helvetica, sans-serif",
            )>
                { for self.node_preview_args.as_ref().map(|args| html! {
                    <node_preview::Comp
                        args=args.clone()
                        />
                }) }
                { for self.edge_preview_args.as_ref().map(|args| html! {
                    <edge_preview::Comp
                        args=args.clone() edit_duct=self.link.callback(Update::SetDuctEditor)
                        />
                }) }
                { for self.duct_editor_args.as_ref().map(|args| html! {
                    <duct_editor::Comp
                        args=args.clone()
                        legion=Rc::clone(&self.props.legion)
                        />
                }) }
                { for self.display_toolbar.then(|| html! {
                    <toolbar::Comp
                        legion=Rc::clone(&self.props.legion)
                        open_options=self.link.callback(|()| Update::OpenOptions)
                        cancel=self.duct_editor_args.is_some().then(|| self.link.callback(|_| Update::Cancel))
                        />
                }) }
                { for self.options_opened.then(|| html! {
                    <div style=style!(
                        "background-color": "white",
                        "width": "300px",
                        "pointer-events": "auto",
                        "position": "fixed",
                        "top": "50%", "left": "50%",
                        "transform": "translate(-50%, -50%)",
                        "margin": "2em",
                    )>
                        <options::Comp
                            legion=Some(Rc::clone(&self.props.legion))
                            />
                    </div>
                }) }
            </div>
        }
    }
}

/// Events for [`Wrapper`].
pub enum Update {
    /// Sets the node preview args to display.
    SetNodePreview(Option<node_preview::Args>),
    /// Sets the edge preview args to display.
    SetEdgePreview(Option<edge_preview::Args>),
    /// Sets the duct editor args to display.
    SetDuctEditor(Option<duct_editor::Args>),
    /// Open the options menu.
    OpenOptions,
    /// Trigger the edit action.
    Edit,
    /// Cancels the opened interfaces.
    Cancel,
}

/// Yew properties for [`Wrapper`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The legion setup.
    pub legion: Rc<RefCell<traffloat::Legion>>,
}

impl Props {
    fn updater_ref(&self) -> UpdaterRef {
        let legion = self.legion.borrow();
        let updater_ref: &UpdaterRef =
            &*legion.resources.get().expect("UpdaterRef was not initialized");
        updater_ref.clone()
    }
}

/// An interiorly-mutable reference to update the yew callback for UI messages [`Update`].
#[derive(Clone, Default)]
pub struct UpdaterRef {
    cell: Rc<RefCell<Option<Callback<Update>>>>,
}

impl UpdaterRef {
    /// Updates the callback.
    pub fn set(&self, callback: Callback<Update>) {
        let _ = self.cell.replace(Some(callback));
    }

    /// Invokes the callback if it exists.
    pub fn call(&self, update: Update) {
        if let Some(callback) = &*self.cell.borrow() {
            callback.emit(update);
        }
    }
}

#[codegen::system(Response)]
#[thread_local]
fn cancel_trigger(
    #[subscriber] click_sub: impl Iterator<Item = keyboard::SingleClick>,
    #[resource] updater_ref: &UpdaterRef,
) {
    let has_click =
        click_sub.filter(|click| click.command() == keyboard::Command::Cancel).count() > 0; // consume the whole iterator without short-circuiting
    if has_click {
        updater_ref.call(Update::Cancel);
    }
}

#[codegen::system(Response)]
#[thread_local]
fn edit_trigger(
    #[subscriber] click_sub: impl Iterator<Item = keyboard::SingleClick>,
    #[resource] updater_ref: &UpdaterRef,
) {
    let has_click =
        click_sub.filter(|click| click.command() == keyboard::Command::Edit).count() > 0; // consume the whole iterator without short-circuiting
    if has_click {
        updater_ref.call(Update::Edit);
    }
}

/// Sets up legion ECS for edge info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(cancel_trigger_setup).uses(edit_trigger_setup)
}
