use bevy::ecs::entity::Entity;
use bevy::ecs::system::{ParamSet, Query, SystemParam};
use egui_material_icons::icons;

use crate::dock;
use crate::scene::{GenericViewable, ViewableKind};

mod building;

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
            ui.heading(&generic.name);
            if ui.button(icons::ICON_EDIT).clicked() {
                // TODO switch to editor
            }
        });

        match generic.kind {
            ViewableKind::Building => {
                let mut building_param = param.viewable_query.p0();
                building_param.ui(self.entity, ui, dock);
            }
        }
    }

    type OnCloseSystemParam<'w, 's> = ();
    type BeforeRenderSystemParam<'w, 's> = ();
}

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    generic:        Query<'w, 's, &'static GenericViewable>,
    viewable_query: ParamSet<'w, 's, (building::UiSystemParam<'w, 's>,)>,
}
