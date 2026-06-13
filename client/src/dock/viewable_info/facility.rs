use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{Commands, Query, Res, SystemParam};
use egui_material_icons::icons;
use traffloat_physics::util::QueryExt;

use crate::dock::viewable_info::{show_fluid, show_link, show_link_small};
use crate::dock::{self, viewable_info};
use crate::scene::building::FluidConnectionPeer;
use crate::scene::conduit::ConduitCorridor;
use crate::scene::{FluidTypes, GenericViewable, IdRegistry, ProtoId, building, facility};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    facility_query:          Query<'w, 's, FacilityData>,
    building_query:          Query<'w, 's, (&'static GenericViewable, &'static building::Info)>,
    fluid_types:             Res<'w, FluidTypes>,
    commands:                Commands<'w, 's>,
    show_connections_params: ShowConnectionsParams<'w, 's>,
}

#[derive(QueryData)]
struct FacilityData {
    id:       &'static ProtoId,
    info:     &'static facility::Info,
    building: &'static facility::FacilityBuilding,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(facility_data) = self.facility_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        let Some((building_viewable, building_info)) =
            self.building_query.log_get(facility_data.building.0)
        else {
            return;
        };

        ui.heading("Located in");
        show_building(ui, dock.id, building_viewable, &mut self.commands, facility_data.building.0);

        let mut connections =
            show_connections(building_info, &facility_data, &self.show_connections_params, dock.id)
                .peekable();
        if connections.peek().is_some() {
            ui.heading("Connections");
            for connection in connections {
                connection(ui, &mut self.commands);
            }
        }

        if let Some(ambient_fluid) = &facility_data.info.stored_fluid {
            egui::CollapsingHeader::new("Stored fluid").id_salt(new_id!(dock.id)).show(ui, |ui| {
                show_fluid(ui, dock.id, ambient_fluid, &self.fluid_types);
            });
        }
    }
}

fn show_building(
    ui: &mut egui::Ui,
    id: egui::Id,
    building_viewable: &GenericViewable,
    commands: &mut Commands,
    building_entity: Entity,
) {
    ui.horizontal(|ui| {
        show_link(ui, commands, building_entity);
        ui.label(&building_viewable.name);
    });
}

#[derive(SystemParam)]
struct ShowConnectionsParams<'w, 's> {
    id_registry:    Res<'w, IdRegistry>,
    viewable_query: Query<'w, 's, &'static GenericViewable>,
    conduit_query:  Query<'w, 's, &'static ConduitCorridor>,
}

fn show_connections(
    building_info: &building::Info,
    facility_data: &FacilityDataItem,
    params: &ShowConnectionsParams,
    id: egui::Id,
) -> impl Iterator<Item = impl FnOnce(&mut egui::Ui, &mut Commands)> {
    building_info
        .facility_fluid_connections(facility_data.id.0, &params.id_registry)
        .enumerate()
        .map(move |(id_salt, (conn, peer))| {
            let id = new_id!(id, id_salt);

            move |ui: &mut egui::Ui, commands: &mut Commands| {
                ui.horizontal(|ui| match peer {
                    FluidConnectionPeer::Facility(peer) => {
                        show_link(ui, commands, peer);
                        ui.label("Neighbor facility:");
                        if let Some(peer_viewable) = params.viewable_query.log_get(peer) {
                            ui.label(&peer_viewable.name);
                        }
                    }
                    FluidConnectionPeer::Building(peer) => {
                        show_link(ui, commands, peer);
                        ui.label("Parent building");
                    }
                    FluidConnectionPeer::Pipe(peer) => {
                        show_link(ui, commands, peer);
                        ui.label("Pipe:");
                        if let Some(peer_viewable) = params.viewable_query.log_get(peer) {
                            ui.label(&peer_viewable.name);
                        }

                        if let Some(corridor) = params.conduit_query.log_get(peer) {
                            ui.label("in corridor:");
                            show_link_small(ui, commands, corridor.0);
                            if let Some(corridor_viewable) =
                                params.viewable_query.log_get(corridor.0)
                            {
                                ui.label(&corridor_viewable.name);
                            }
                        }
                    }
                });

                ui.indent(new_id!(id), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Openness:");
                        let mut proportion = conn.current_area / conn.max_area * 100.0;
                        ui.add(egui::Slider::new(&mut proportion, 0.0..=100.0).suffix("%"));
                        // TODO send proportion changes to the server if resp.changed()
                    });
                });
            }
        })
}
