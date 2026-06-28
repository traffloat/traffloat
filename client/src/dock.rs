use std::mem;
use std::sync::atomic::{self, AtomicU32};

use bevy::app::{self, App, Plugin};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, Camera2d};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{self, IntoScheduleConfigs, Schedulable, ScheduleConfigs};
use bevy::ecs::system::{Command, Commands, ParamSet, Res, ResMut, RunSystemOnce, SystemParam};
use bevy::ecs::world::World;
use bevy_egui::{EguiContexts, EguiGlobalSettings, EguiPreUpdateSet, EguiPrimaryContextPass, PrimaryEguiContext};
use egui::WidgetText;
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, TabPath};

pub mod camera;
mod menu;
mod new_level;
mod open_mode;
mod save;
mod settings;
mod startup;
pub mod viewable_info;
pub use open_mode::*;
use traffloat_macro_util::fan_out;

use crate::scene::LevelState;
use crate::util::new_id;

static NEXT_TAB_ID: AtomicU32 = AtomicU32::new(0);

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::Plug, startup::Plug, save::Plug));
        app.init_resource::<State>();
        app.init_resource::<Toasts>();
        app.add_systems(app::Startup, setup_system);
        app.add_systems(app::PreUpdate, init_egui_system.after(EguiPreUpdateSet::InitContexts).before(EguiPreUpdateSet::BeginPass));
        app.add_systems(EguiPrimaryContextPass, (render_system, render_toasts_system).chain());
    }
}

#[derive(Resource)]
pub struct State(DockState<TabState>);

#[derive(Resource, Default)]
pub struct Toasts(egui_notify::Toasts);

pub struct TabState {
    id:       u32,
    pub tab:  TabEnum,
    location: Option<TabPath>,
}

impl<T: Into<TabEnum>> From<T> for TabState {
    fn from(tab: T) -> Self {
        Self {
            id:       NEXT_TAB_ID.fetch_add(1, atomic::Ordering::Relaxed),
            tab:      tab.into(),
            location: None,
        }
    }
}

impl Default for State {
    fn default() -> Self { Self(DockState::new(vec![startup::Tab.into()])) }
}

impl State {
    pub fn focus_or_create(
        &mut self,
        tab_fn: impl FnOnce() -> TabEnum,
        placement: impl AlwaysTabPlacement,
    ) -> &mut TabEnum {
        let path = placement.always_place(&mut self.0, || TabState::from(tab_fn()));
        self.0.set_focused_node_and_surface(path.node_path());
        self.0.set_active_tab(path).expect("AlwaysPlacement ensures path must exist");
        let leaf =
            self.0.leaf_mut(path.node_path()).expect("AlwaysPlacement ensures path must exist");
        &mut leaf.tabs[path.tab.0].tab
    }

    pub fn focus_tab(&mut self, tab_fn: impl Fn(&TabEnum) -> bool) -> bool {
        if let Some(path) = self.0.find_tab_from(|tab| tab_fn(&tab.tab)) {
            self.0.set_focused_node_and_surface(path.node_path());
            self.0.set_active_tab(path).expect("find_tab_from result must exist");
            true
        } else {
            false
        }
    }

    pub fn tabs(&self) -> impl Iterator<Item = &TabEnum> {
        self.0.iter_all_tabs().map(|(_, tab)| &tab.tab)
    }

    pub fn tabs_mut(&mut self) -> impl Iterator<Item = &mut TabEnum> {
        self.0.iter_all_tabs_mut().map(|(_, tab)| &mut tab.tab)
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
    pub id:       egui::Id,
    pub location: TabPath,
    pub focused:  bool,
}

struct TabViewer<'w, 's> {
    params:      TabViewerParams<'w, 's>,
    next_order:  usize,
    focused_tab: Option<TabPath>,
}

macro_rules! define_tabs {
    (
        [$w:lifetime, $s:lifetime]
        $paramset_tuple:ty;
        {
            $(
                $variant:ident ($tab_type:ty) $path:tt,
            )*
        }
    ) => {
        #[derive(strum::EnumIs)]
        pub enum TabEnum {
            $(
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
                        TabEnum::$variant(t) => {
                            define_tabs_handle_let!(viewer.params.ps.p1(), param, $path);
                            t.before_render(contexts, param.p0())
                        },
                    )*
                }
            }
        }

        #[derive(SystemParam)]
        struct TabViewerParams<$w, $s> {
            ps: ParamSet<$w, $s, (menu::Params<$w, $s>, $paramset_tuple)>,
        }

        impl<$w, $s> egui_dock::TabViewer for TabViewer<$w, $s> {
            type Tab = TabState;

            fn title(&mut self, tab: &mut TabState) -> WidgetText {
                match tab.tab {
                    $(
                        TabEnum::$variant(ref mut t) => {
                            define_tabs_handle_let!(self.params.ps.p1(), param, $path);
                            t.title(param.p1())
                        }
                    )*
                }.into()
            }

            fn ui(&mut self, ui: &mut egui::Ui, tab: &mut TabState) {
                let order_mut = &mut self.next_order;
                let order = mem::replace(order_mut, *order_mut + 1);
                let location = tab.location.expect("location must be initialized before DockArea rendering");
                let dock = Context { order, id: new_id!(tab.id), location, focused: Some(location) == self.focused_tab };
                match tab.tab {
                    $(
                        TabEnum::$variant(ref mut t) => {
                            define_tabs_handle_let!(self.params.ps.p1(), param, $path);
                            t.ui(param.p2(), ui, dock)
                        }
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
                        TabEnum::$variant(ref mut t) => {
                            define_tabs_handle_let!(self.params.ps.p1(), param, $path);
                            t.on_close(param.p3())
                        }
                    )*
                }
            }
        }
    }
}

macro_rules! define_tabs_handle_let {
    ($ps:expr, $var:ident, ($($path:ident)*)) => {
        let $var = &mut $ps;
        $(
            let mut $var = $var.$path();
        )*
    }
}

macro_rules! define_tabs_item {
    (
        [$w:lifetime, $s:lifetime]
        $message:ident ($tab_type:ty)
    ) => {
        ParamSet<$w, $s, (
            <$tab_type as Tab>::BeforeRenderSystemParam<'w, 's>,
            <$tab_type as Tab>::TitleSystemParam<'w, 's>,
            <$tab_type as Tab>::UiSystemParam<'w, 's>,
            <$tab_type as Tab>::OnCloseSystemParam<'w, 's>,
        )>
    };
}

macro_rules! define_tabs_tuple {
    (
        [$w:lifetime, $s:lifetime]
        $($params:ty,)*
    ) => {
        ParamSet<$w, $s, (
            $($params,)*
        )>
    }
}

fan_out! {
    ['w, 's]
    define_tabs, define_tabs_tuple, define_tabs_item;
    8, 2;
    Startup(startup::Tab),
    NewLevel(new_level::Tab),
    OpenSave(save::OpenTab),
    Settings(settings::Tab),
    Camera(camera::Tab),
    ViewableInfo(viewable_info::Tab),
    SaveAs(save::SaveAsTab),
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

fn init_egui_system(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui_material_icons::initialize(ctx);
}

fn render_system(
    mut contexts: EguiContexts,
    mut state: ResMut<State>,
    mut params: TabViewerParams,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let ui = &mut egui::Ui::new(
        ctx.clone(),
        new_id!(),
        egui::UiBuilder::new().layer_id(egui::LayerId::background()).max_rect(ctx.viewport_rect()),
    );

    egui::Panel::top(new_id!()).show_inside(ui, |ui| {
        ui.horizontal(|ui| params.ps.p0().show_menu_buttons(ui));
    });

    let focused_node = state.0.focused_leaf();
    let focused_tab = focused_node
        .and_then(|path| state.0[path].get_leaf().map(|leaf| TabPath::from((path, leaf.active))));
    if focused_tab.is_none() {
        let main_camera =
            state.0.find_tab_from(|tab| matches!(tab.tab, TabEnum::Camera(ref t) if t.is_main));
        if let Some(path) = main_camera {
            state.0.set_focused_node_and_surface(path.node_path());
            state.0.set_active_tab(path).expect("find_tab_from result must exist");
        }
    }
    let mut viewer = TabViewer { params, next_order: 0, focused_tab };
    for (path, tab) in state.0.iter_all_tabs_mut() {
        tab.location = Some(path);
        tab.tab.before_render(&mut contexts, &mut viewer);
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default().show_inside(ui, |ui| {
        DockArea::new(&mut state.0)
            .show_add_buttons(false)
            .style(egui_dock::Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut viewer);
        viewer.params.ps.p0().global_dock_shortcuts(ui.ctx());
    });
}

#[derive(SystemParam)]
struct MenuParams<'w, 's> {
    commands:    Commands<'w, 's>,
    src:         Res<'w, save::LoadSource>,
    level_state: Res<'w, bevy::state::state::State<LevelState>>,
}

pub struct DockCommand<F: FnOnce(&mut State)>(pub F);

impl<F: FnOnce(&mut State) + Send + 'static> Command for DockCommand<F> {
    type Out = ();

    fn apply(self, world: &mut World) {
        let mut state = world.resource_mut::<State>();
        (self.0)(&mut state);
    }
}

fn render_toasts_system(mut contexts: EguiContexts, mut toasts: ResMut<Toasts>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    toasts.0.show(ctx);
}

pub fn init_camera_view(world: &mut World) {
    world
        .run_system_once(|mut new_camera: camera::NewTabParams, mut dock: ResMut<State>| {
            let tab = camera::Tab::new(true, "main".into(), &mut new_camera);
            dock.reset_all(tab.into());
        })
        .unwrap();
}
