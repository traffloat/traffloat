use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Command, ParamSet, Query, SystemParam};
use bevy::ecs::world::World;
use egui_material_icons::icons;

use crate::dock::{self, TabPlacement, viewable_info};
use crate::scene::{GenericViewable, ViewableKind};

mod building;
mod corridor;

pub struct Tab {
    pub entity: Entity,
}

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = Query<'w, 's, &'static GenericViewable>;
    fn title(&self, param: Self::TitleSystemParam<'_, '_>) -> String {
        let Ok(viewable) = param.get(self.entity) else {
            return "[Invalid entity]".to_string();
        };
        match viewable.kind {
            ViewableKind::Building => format!("Building: {}", viewable.name),
            ViewableKind::Corridor => format!("Corridor: {}", viewable.name),
        }
    }

    type UiSystemParam<'w, 's> = UiSystemParam<'w, 's>;
    fn ui(
        &mut self,
        mut param: Self::UiSystemParam<'_, '_>,
        ui: &mut egui::Ui,
        dock: dock::Context,
    ) {
        let Ok(generic) = param.generic.get(self.entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        ui.horizontal(|ui| {
            ui.heading(match generic.kind {
                ViewableKind::Building => "Building",
                ViewableKind::Corridor => "Corridor",
            });
            ui.heading(&generic.name);
            if ui.button(icons::ICON_EDIT).clicked() {
                // TODO edit text
            }
        });

        match generic.kind {
            ViewableKind::Building => {
                let mut building_param = param.viewable_query.p0();
                building_param.ui(self.entity, ui, dock);
            }
            ViewableKind::Corridor => {
                let mut corridor_param = param.viewable_query.p1();
                corridor_param.ui(self.entity, ui, dock);
            }
        }
    }

    type OnCloseSystemParam<'w, 's> = ();
    type BeforeRenderSystemParam<'w, 's> = ();
}

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    generic:        Query<'w, 's, &'static GenericViewable>,
    viewable_query:
        ParamSet<'w, 's, (building::UiSystemParam<'w, 's>, corridor::UiSystemParam<'w, 's>)>,
}

pub struct OpenCommand {
    pub entity:    Entity,
    pub force_new: bool,
}

impl OpenCommand {
    pub fn from_click(entity: Entity, ctx: &egui::Context) -> Self {
        let force_new = ctx.input(|input| input.modifiers.command);
        Self { entity, force_new: false }
    }
}

impl Command for OpenCommand {
    fn apply(self, world: &mut World) {
        world.resource_mut::<dock::State>().focus_or_create(
            || viewable_info::Tab { entity: self.entity }.into(),
            dock::ReplaceTab(|state| state.tab.is_viewable_info())
                .only_if(!self.force_new)
                .or(dock::Split { split: egui_dock::Split::Right, ratio: 0.7 }
                    .at(|state| state.tab.is_camera()))
                .or_always(dock::Split { split: egui_dock::Split::Right, ratio: 0.7 }),
        );
    }
}
