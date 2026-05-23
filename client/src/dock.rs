use std::mem;

use bevy::app::{self, App, Plugin};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, Camera2d};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{self, Schedulable, ScheduleConfigs};
use bevy::ecs::system::{Commands, Local, ParamSet, ResMut, SystemParam};
use bevy::ecs::world::World;
use bevy_egui::{EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext};
use egui::WidgetText;
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabIndex};

mod camera;
mod new_level;
mod open_mode;
mod settings;
mod startup;
pub use open_mode::*;

use crate::dock;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::Plug, startup::Plug));
        app.init_resource::<State>();
        app.add_systems(app::Startup, setup_system);
        app.add_systems(EguiPrimaryContextPass, render_system);
    }
}

#[derive(Resource)]
pub struct State(DockState<TabState>);

pub struct TabState {
    tab:      TabEnum,
    location: Option<(SurfaceIndex, NodeIndex, TabIndex)>,
}

impl<T: Into<TabEnum>> From<T> for TabState {
    fn from(tab: T) -> Self { Self { tab: tab.into(), location: None } }
}

impl Default for State {
    fn default() -> Self { Self(DockState::new(vec![startup::Tab.into()])) }
}

impl State {
    pub fn focus_or_create(
        &mut self,
        tab_fn: impl FnOnce() -> TabEnum,
        placement: impl AlwaysTabPlacement,
    ) {
        let path =
            placement.always_place(&mut self.0, || TabState { tab: tab_fn(), location: None });
        self.0.set_focused_node_and_surface((path.0, path.1));
        self.0.set_active_tab(path);
    }

    pub fn reset_all(&mut self, tab: TabEnum) { self.0 = DockState::new([tab.into()].into()); }
}

pub trait Tab {
    fn schedule_configs<T>(configs: ScheduleConfigs<T>) -> ScheduleConfigs<T>
    where
        T: Schedulable<Metadata = schedule::GraphInfo, GroupMetadata = schedule::Chain>,
    {
        configs
    }

    type TitleSystemParam<'w, 's>: SystemParam;
    fn title(&self, param: Self::TitleSystemParam<'_, '_>) -> String;

    type UiSystemParam<'w, 's>: SystemParam;
    fn ui(&mut self, param: Self::UiSystemParam<'_, '_>, ui: &mut egui::Ui, dock: Context);

    fn closeable(&self) -> bool { true }
    type OnCloseSystemParam<'w, 's>: SystemParam;
    fn on_close(&mut self, _params: Self::OnCloseSystemParam<'_, '_>) -> OnCloseResponse {
        OnCloseResponse::Close
    }

    type BeforeRenderSystemParam<'w, 's>: SystemParam;
    fn before_render(
        &mut self,
        _contexts: &mut EguiContexts,
        _param: Self::BeforeRenderSystemParam<'_, '_>,
    ) {
    }
}

pub struct Context {
    pub order:    usize,
    pub location: (SurfaceIndex, NodeIndex, TabIndex),
    pub focused:  bool,
}

struct TabViewer<'w, 's> {
    params:      TabViewerParams<'w, 's>,
    next_order:  usize,
    focused_tab: Option<(SurfaceIndex, NodeIndex, TabIndex)>,
}

macro_rules! define_tabs {
    (
        $(
            #[$meta:meta]
            $ps_path:tt
            $variant:ident ($tab_type:ty)
        )*
    ) => {
        pub enum TabEnum {
            $(
                #[$meta]
                $variant($tab_type),
            )*
        }

        $(
            impl From<$tab_type> for TabEnum {
                fn from(tab: $tab_type) -> Self {
                    TabEnum::$variant(tab)
                }
            }
        )*

        impl TabEnum {
            fn schedule_configs<T>(mut configs: ScheduleConfigs<T>) -> ScheduleConfigs<T>
            where
                T: Schedulable<Metadata = schedule::GraphInfo, GroupMetadata = schedule::Chain>,
            {
                $(configs = <$tab_type>::schedule_configs(configs);)*
                configs
            }

            fn before_render(&mut self, contexts: &mut EguiContexts, viewer: &mut TabViewer) {
                match self {
                    $(
                        TabEnum::$variant(t) => do_ps_path!(viewer.params.ps.p1(), $ps_path; |p| t.before_render(contexts, p.p0())),
                    )*
                }
            }
        }

        #[derive(SystemParam)]
        struct TabViewerParams<'w, 's> {
            ps: ParamSet<'w, 's, (CommonParams<'w, 's>, recurse_param_set!(
                'w, 's,
                $(
                    (
                        <$tab_type as Tab>::BeforeRenderSystemParam<'w, 's>,
                        <$tab_type as Tab>::TitleSystemParam<'w, 's>,
                        <$tab_type as Tab>::UiSystemParam<'w, 's>,
                        <$tab_type as Tab>::OnCloseSystemParam<'w, 's>,
                    ),
                )*
            ))>,
        }

        impl<'w, 's> egui_dock::TabViewer for TabViewer<'w, 's> {
            type Tab = TabState;

            fn title(&mut self, tab: &mut TabState) -> WidgetText {
                match tab.tab {
                    $(
                        TabEnum::$variant(ref mut t) => do_ps_path!(self.params.ps.p1(), $ps_path; |p| t.title(p.p1())) ,
                    )*
                }.into()
            }

            fn ui(&mut self, ui: &mut egui::Ui, tab: &mut TabState) {
                let order_mut = &mut self.next_order;
                let order = mem::replace(order_mut, *order_mut + 1);
                let location = tab.location.expect("location must be initialzed before DockArea rendering");
                let dock = Context { order, location, focused: Some(location) == self.focused_tab };
                match tab.tab {
                    $(
                        TabEnum::$variant(ref mut t) => do_ps_path!(self.params.ps.p1(), $ps_path; |p| t.ui(p.p2(), ui, dock)),
                    )*
                }
            }

            fn is_closeable(&self, tab: &TabState) -> bool {
                match tab.tab {
                    $(
                        TabEnum::$variant(ref t) => t.closeable(),
                    )*
                }
            }

            fn on_close(&mut self, tab: &mut TabState) -> OnCloseResponse {
                match tab.tab {
                    $(
                        TabEnum::$variant(ref mut t) => do_ps_path!(self.params.ps.p1(), $ps_path; |p| t.on_close(p.p3())),
                    )*
                }
            }

            fn add_popup(&mut self, ui: &mut egui::Ui, _: SurfaceIndex, _: NodeIndex) {
                self.params.ps.p0().add_popup(ui);
            }
        }
    }
}

macro_rules! do_ps_path {
    ($ps:expr, $p0:tt; |$var:ident| $closure:expr) => {{
        let mut ps = $ps;
        do_ps_path!(@recurse ps, $p0; |$var| $closure)
    }};
    (@recurse $ps:expr, (); |$var:ident| $closure:expr) => {{
        let mut $var = $ps.p1();
        $closure
    }};
    (@recurse $ps:expr, ($p0:ident $($rest:ident)*); |$var:ident| $closure:expr) => {{
        let $var = $ps.$p0();
        do_ps_path!($var, ($($rest)*); |$var| $closure)
    }};
}

macro_rules! recurse_param_set {
    ($w:lifetime, $s: lifetime,) => { () };
    ($w:lifetime, $s: lifetime, $args:tt, $($rest:tt)*) => {
        ParamSet<'w, 's, (recurse_param_set!($w, $s, $($rest)*), ParamSet<'w, 's, $args>)>
    }
}

define_tabs! {
    /// Startup menu
    () Startup(startup::Tab)
    /// Level creation menu
    (p0) NewLevel(new_level::Tab)
    /// Settings
    (p0 p0) Settings(settings::Tab)
    /// Camera viewport
    (p0 p0 p0) Camera(camera::Tab)
}

fn setup_system(mut egui_global_settings: ResMut<EguiGlobalSettings>, mut commands: Commands) {
    egui_global_settings.auto_create_primary_context = false;
    commands.spawn((
        Camera2d,
        Camera { order: 1, ..Default::default() },
        PrimaryEguiContext,
        RenderLayers::none(),
    )); // egui camera
}

fn render_system(mut contexts: EguiContexts, mut state: ResMut<State>, params: TabViewerParams) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let focused_node = state.0.focused_leaf();
    let focused_tab = focused_node.and_then(|(surface, node)| {
        state.0[surface][node].get_leaf().map(|leaf| (surface, node, leaf.active))
    });
    if focused_tab.is_none() {
        let main_camera =
            state.0.find_tab_from(|tab| matches!(tab.tab, TabEnum::Camera(ref t) if t.is_main));
        if let Some(path) = main_camera {
            state.0.set_focused_node_and_surface((path.0, path.1));
            state.0.set_active_tab(path);
        }
    }
    let mut viewer = TabViewer { params, next_order: 0, focused_tab };
    for (surface_index, surface) in state.0.iter_surfaces_mut().enumerate() {
        for (node_index, node) in surface.iter_nodes_mut().enumerate() {
            if let egui_dock::Node::Leaf(leaf) = node {
                for (tab_index, tab) in leaf.tabs.iter_mut().enumerate() {
                    tab.location =
                        Some((surface_index.into(), node_index.into(), tab_index.into()));
                    tab.tab.before_render(&mut contexts, &mut viewer);
                }
            }
        }
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default().show(ctx, |ui| {
        DockArea::new(&mut state.0)
            .show_add_buttons(true)
            .show_add_popup(true)
            .style(egui_dock::Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut viewer);
    });
}

#[derive(SystemParam)]
struct CommonParams<'w, 's> {
    commands: Commands<'w, 's>,
}

impl CommonParams<'_, '_> {
    fn add_popup(&mut self, ui: &mut egui::Ui) {
        if ui.button("Settings").clicked() {
            self.commands.queue(|world: &mut World| {
                world.resource_mut::<State>().focus_or_create(
                    || settings::Tab.into(),
                    dock::ReplaceTab(|tab| matches!(tab.tab, TabEnum::Settings(_)))
                        .or_always(dock::NewWindow),
                );
            });
            ui.close();
        }
    }
}
