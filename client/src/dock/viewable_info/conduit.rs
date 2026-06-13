use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{Commands, Query, Res, SystemParam};
use egui_material_icons::icons;
use traffloat_physics::util::{Alpha, Beta, QueryExt};
use traffloat_proto::proto;

use crate::dock::viewable_info::{show_fluid, show_link, show_link_small};
use crate::dock::{self, viewable_info};
use crate::scene::{FluidTypes, GenericViewable, IdRegistry, ProtoId, building, conduit, corridor};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    conduit_query:           Query<'w, 's, ConduitData>,
    corridor_query:          Query<'w, 's, &'static GenericViewable>,
    fluid_types:             Res<'w, FluidTypes>,
    commands:                Commands<'w, 's>,
    show_connections_params: ShowConnectionsParams<'w, 's>,
}

#[derive(QueryData)]
struct ConduitData {
    info:     &'static conduit::Info,
    corridor: &'static conduit::ConduitCorridor,
    id:       &'static ProtoId,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(conduit_data) = self.conduit_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        ui.heading("Located in");
        show_corridor(
            ui,
            dock.id,
            &self.corridor_query,
            &mut self.commands,
            conduit_data.corridor.0,
        );

        let mut connections =
            show_connections(&self.show_connections_params, &conduit_data, dock.id).peekable();
        if connections.peek().is_some() {
            ui.heading("Connections");
            for connection in connections {
                connection(ui, &mut self.commands);
            }
        }

        if let Some(ambient_fluid) = &conduit_data.info.stored_fluid {
            egui::CollapsingHeader::new("Stored fluid").id_salt(new_id!(dock.id)).show(ui, |ui| {
                show_fluid(ui, dock.id, ambient_fluid, &self.fluid_types);
            });
        }
    }
}

fn show_corridor(
    ui: &mut egui::Ui,
    id: egui::Id,
    corridor_query: &Query<&GenericViewable>,
    commands: &mut Commands,
    corridor_entity: Entity,
) {
    let Some(corridor_info) = corridor_query.log_get(corridor_entity) else { return };

    ui.horizontal(|ui| {
        show_link(ui, commands, corridor_entity);
        ui.label("Corridor:");
        ui.label(&corridor_info.name);
    });
}

#[derive(SystemParam)]
struct ShowConnectionsParams<'w, 's> {
    id_registry:    Res<'w, IdRegistry>,
    viewable_query: Query<'w, 's, &'static GenericViewable>,
    corridor_query: Query<
        'w,
        's,
        (
            Option<&'static corridor::EndpointRef<Alpha>>,
            Option<&'static corridor::EndpointRef<Beta>>,
        ),
    >,
    building_query: Query<'w, 's, &'static building::Info>,
}

fn show_connections(
    params: &ShowConnectionsParams,
    conduit_data: &ConduitDataItem,
    id: egui::Id,
) -> impl Iterator<Item = impl FnOnce(&mut egui::Ui, &mut Commands)> {
    params
        .corridor_query
        .log_get(conduit_data.corridor.0)
        .into_iter()
        .flat_map(|(a, b)| [a.map(|a| a.0), b.map(|b| b.0)])
        .flatten()
        .flat_map(move |building| {
            params
                .building_query
                .log_get(building)
                .into_iter()
                .flat_map(|building_info| &building_info.connections)
                .filter_map(|conn| match conn.pair {
                    proto::BuildingFluidConnectionPair::FacilityPipe { facility, pipe }
                        if pipe == conduit_data.id.0 =>
                    {
                        Some((conn, params.id_registry.get_facility(facility)?))
                    }
                    _ => None,
                })
                .enumerate()
                .map(move |(id_salt, (conn, facility))| {
                    move |ui: &mut egui::Ui, commands: &mut Commands| {
                        let id = new_id!(id, id_salt);
                        show_connection_ui(
                            ui,
                            commands,
                            facility,
                            building,
                            conn,
                            &params.viewable_query,
                            id,
                        );
                    }
                })
        })
}

fn show_connection_ui(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    facility: Entity,
    building: Entity,
    conn: &proto::BuildingFluidConnection,
    viewable_query: &Query<&GenericViewable>,
    id: egui::Id,
) {
    ui.horizontal(|ui| {
        show_link(ui, commands, facility);
        ui.label("Facility:");
        if let Some(facility_viewable) = viewable_query.log_get(facility) {
            ui.label(&facility_viewable.name);
        }
        ui.label("in building:");
        show_link_small(ui, commands, building);
        if let Some(building_viewable) = viewable_query.log_get(building) {
            ui.label(&building_viewable.name);
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
