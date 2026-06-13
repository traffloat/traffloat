use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::system::{Commands, Query, Res, SystemParam};
use egui_material_icons::icons;
use traffloat_physics::util::{Alpha, Beta, QueryExt, Which};
use traffloat_proto::proto::AlphaOrBeta;

use crate::dock::viewable_info::corridor::display_gate;
use crate::dock::viewable_info::{show_fluid, show_link, show_link_small};
use crate::dock::{self, viewable_info};
use crate::scene::facility::{BuildingFacilities, FacilityBuilding};
use crate::scene::{FluidTypes, GenericViewable, building, corridor, resident};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    commands:        Commands<'w, 's>,
    building_query:  Query<'w, 's, BuildingData>,
    corridor_query:  Query<'w, 's, CorridorData>,
    facility_query:  Query<'w, 's, FacilityData>,
    resident_params: ShowResidentsParams<'w, 's>,
    fluid_types:     Res<'w, FluidTypes>,
}

#[derive(QueryData)]
struct BuildingData {
    generic:         &'static GenericViewable,
    info:            &'static building::Info,
    corridors_alpha: Option<&'static corridor::IsEndpointOf<Alpha>>,
    corridors_beta:  Option<&'static corridor::IsEndpointOf<Beta>>,
    facilities:      Option<&'static BuildingFacilities>,
}

#[derive(QueryData)]
struct CorridorData {
    generic:        &'static GenericViewable,
    alpha_building: Option<(
        &'static corridor::EndpointRef<Alpha>,
        &'static corridor::GenericEndpointDetails<Alpha>,
    )>,
    beta_building: Option<(
        &'static corridor::EndpointRef<Beta>,
        &'static corridor::GenericEndpointDetails<Beta>,
    )>,
}

#[derive(QueryData)]
struct FacilityData {
    generic: &'static GenericViewable,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(data) = self.building_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        if let Some(facilities) = data.facilities
            && !facilities.is_empty()
        {
            ui.heading("Facilities");
            for facility in facilities.iter() {
                show_facility(ui, dock.id, &mut self.commands, &self.facility_query, facility);
            }
        }

        if data.corridors_alpha.is_some_and(|c| !c.is_empty())
            || data.corridors_beta.is_some_and(|c| !c.is_empty())
        {
            ui.heading("Connections");
            for corridor in data.corridors_alpha.iter().flat_map(|c| c.iter()) {
                show_connection(
                    ui,
                    dock.id,
                    &mut self.commands,
                    &self.building_query,
                    &self.corridor_query,
                    corridor,
                    Alpha,
                );
            }
            for corridor in data.corridors_beta.iter().flat_map(|c| c.iter()) {
                show_connection(
                    ui,
                    dock.id,
                    &mut self.commands,
                    &self.building_query,
                    &self.corridor_query,
                    corridor,
                    Beta,
                );
            }
        }

        let mut residents = show_residents(entity, &self.resident_params).peekable();
        if residents.peek().is_some() {
            ui.heading("Residents");
            for resident in residents {
                resident(ui, &mut self.commands);
            }
        }

        if let Some(ambient_fluid) = &data.info.ambient_fluid {
            egui::CollapsingHeader::new("Ambient fluid").id_salt(new_id!(dock.id)).show(ui, |ui| {
                show_fluid(ui, dock.id, ambient_fluid, &self.fluid_types);
            });
        }
    }
}

fn show_connection<Ab: Which>(
    ui: &mut egui::Ui,
    id: egui::Id,
    commands: &mut Commands,
    building_query: &Query<BuildingData>,
    corridor_query: &Query<CorridorData>,
    corridor: Entity,
    which: Ab,
) {
    fn get_corridor_data<'a>(
        data: &'a CorridorDataItem,
        which: impl Which,
    ) -> Option<(Entity, &'a corridor::EndpointDetails)> {
        match which.proto() {
            AlphaOrBeta::Alpha => {
                data.alpha_building.as_ref().map(|(endpoint, details)| (endpoint.0, &details.0))
            }
            AlphaOrBeta::Beta => {
                data.beta_building.as_ref().map(|(endpoint, details)| (endpoint.0, &details.0))
            }
        }
    }

    let Ok(corridor_data) = corridor_query.get(corridor) else { return };
    let (_, near_detail) = get_corridor_data(&corridor_data, which)
        .expect("IsEndpointOf implies EndpointRef presence");
    let peer = get_corridor_data(&corridor_data, which.other());

    ui.horizontal(|ui| {
        if let Some((peer_building, peer_detail)) = peer {
            show_link(ui, commands, peer_building);

            if let Some(peer_building_data) = building_query.log_get(peer_building) {
                ui.label(&peer_building_data.generic.name);
            }
        }
    });

    ui.indent(new_id!(id), |ui| {
        ui.horizontal(|ui| {
            ui.label("through corridor");
            show_link_small(ui, commands, corridor);
            ui.label(&corridor_data.generic.name);
        });

        display_gate(ui, near_detail.open, "Proximal gate");
        if let Some((_, peer_detail)) = peer {
            display_gate(ui, peer_detail.open, "Distal gate");
        }
    });
}

fn show_facility(
    ui: &mut egui::Ui,
    id: egui::Id,
    commands: &mut Commands,
    facility_query: &Query<FacilityData>,
    facility_entity: Entity,
) {
    let Some(facility_data) = facility_query.log_get(facility_entity) else { return };

    ui.horizontal(|ui| {
        show_link(ui, commands, facility_entity);

        ui.label(&facility_data.generic.name);
    });
}

#[derive(SystemParam)]
struct ShowResidentsParams<'w, 's> {
    resident_query: Query<'w, 's, (Entity, &'static GenericViewable, &'static resident::Info)>,
    facility_query: Query<'w, 's, (&'static GenericViewable, &'static FacilityBuilding)>,
}

fn show_residents<'q>(
    building: Entity,
    params: &'q ShowResidentsParams,
) -> impl Iterator<Item = impl FnOnce(&mut egui::Ui, &mut Commands)> + 'q {
    enum LocRef<'a> {
        Building,
        Facility(Entity, &'a GenericViewable),
    }

    let in_building = params.resident_query.iter().filter_map(move |(resident, viewable, info)| {
        (info.location == resident::Location::Building(building))
            .then(|| (resident, viewable, LocRef::Building))
    });
    let in_facility = params.resident_query.iter().filter_map(move |(resident, viewable, info)| {
        if let resident::Location::Facility(facility) = info.location
            && let Some((facility_viewable, fb)) = params.facility_query.log_get(facility)
            && fb.0 == building
        {
            Some((resident, viewable, LocRef::Facility(facility, facility_viewable)))
        } else {
            None
        }
    });

    in_building.chain(in_facility).map(|(resident, viewable, loc_ref)| {
        move |ui: &mut egui::Ui, commands: &mut Commands| {
            ui.horizontal(|ui| {
                show_link(ui, commands, resident);
                match loc_ref {
                    LocRef::Building => {
                        ui.label(&viewable.name);
                    }
                    LocRef::Facility(facility, facility_viewable) => {
                        ui.label(&viewable.name);

                        ui.label("working in");
                        show_link_small(ui, commands, facility);
                        ui.label(&facility_viewable.name);
                    }
                }
            });
        }
    })
}
