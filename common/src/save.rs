//! Saving game definition and state.

use std::ops;
use std::sync::{Mutex, MutexGuard};

use codegen::SetupEcs;
use getset::Getters;

use crate::def::{self, state};
use crate::liquid;

/// Stores all scenario gamerule definitions.
#[derive(Getters)]
pub struct GameDefinition {
    /// Liquid types.
    #[getset(get = "pub")]
    liquid:            Vec<def::liquid::Def>,
    /// The liquid recipe map.
    #[getset(get = "pub")]
    liquid_recipes:    liquid::RecipeMap,
    /// Gas types.
    #[getset(get = "pub")]
    gas:               Vec<def::gas::Def>,
    /// Cargo category.
    #[getset(get = "pub")]
    cargo_category:    Vec<def::cargo::category::Def>,
    /// Cargo types.
    #[getset(get = "pub")]
    cargo:             Vec<def::cargo::Def>,
    /// Skill types.
    #[getset(get = "pub")]
    skill:             Vec<def::skill::Def>,
    /// Vehicle types.
    #[getset(get = "pub")]
    vehicle:           Vec<def::vehicle::Def>,
    /// Building category.
    #[getset(get = "pub")]
    building_category: Vec<def::building::category::Def>,
    /// Building types.
    #[getset(get = "pub")]
    building:          Vec<def::building::Def>,
    /// Crime types.
    #[getset(get = "pub")]
    crime:             Vec<def::crime::Def>,
}

impl GameDefinition {
    /// Pack the [`GameDefinition`] into a vector of [`def::Def`] for serialization.
    pub fn pack(&self) -> Vec<def::Def> {
        use def::Def; // don't import this globally because everything is called `Def`.

        self.liquid
            .iter()
            .cloned()
            .map(Def::Liquid)
            .chain(self.gas.iter().cloned().map(Def::Gas))
            .chain(self.cargo_category.iter().cloned().map(Def::CargoCategory))
            .chain(self.cargo.iter().cloned().map(Def::Cargo))
            .chain(self.skill.iter().cloned().map(Def::Skill))
            .chain(self.vehicle.iter().cloned().map(Def::Vehicle))
            .chain(self.building_category.iter().cloned().map(Def::BuildingCategory))
            .chain(self.building.iter().cloned().map(Def::Building))
            .chain(self.crime.iter().cloned().map(Def::Crime))
            .collect()
    }
}

macro_rules! impl_index {
    ($field:ident, $find:ident, $($module:tt)*) => {
        impl ops::Index<$($module)*::Id> for GameDefinition {
            type Output = $($module)*::Def;

            fn index(&self, index: $($module)*::Id) -> &Self::Output {
                let index = index.as_index();
                self.$field.get(index).expect("Corrupted game file")
            }
        }

        impl GameDefinition {
            pub fn $find(&self, name: &str) -> Option<&$($module)*::Def> {
                self.$field.iter()
                    .find(|def| def.id_str().as_str() == name)
            }
        }
    }
}

impl_index!(liquid, find_liquid, def::liquid);
impl_index!(gas, find_gas, def::gas);
impl_index!(cargo_category, find_cargo_category, def::cargo::category);
impl_index!(cargo, find_cargo, def::cargo);
impl_index!(skill, find_skill, def::skill);
impl_index!(vehicle, find_vehicle, def::vehicle);
impl_index!(building_category, find_building_category, def::building::category);
impl_index!(building, find_building, def::building);
impl_index!(crime, find_crime, def::crime);

/// Parameters for saving game state.
///
/// Publish a [`Params`] to initiate a save process.
/// The save module will initilize a [`Request`] and
/// publish it in the [`PreVisualize`][codegen::SystemClass::PreVisualize] stage.
/// Other modules can subscribe to [`Request`]
/// to populate the data they manage.
/// This requires interior mutability via mutexes,
/// but the lock overhead should be negligible since saves are relatively rare.
/// The save is eventually flushed in the
/// [`PostVisualize`][codegen::SystemClass::PostVisualize] stage,
/// and the result is written to the [`Response`] event stream.
#[derive(Clone)]
pub struct Params {}

/// A request to save game state.
#[derive(Getters)]
pub struct Request {
    /// Request parameters.
    #[getset(get = "pub")]
    params: Params,
    file:   Mutex<def::Schema>,
}

impl Request {
    /// Locks the mutex on the serialization target and returns the wrapped value.
    pub fn file(&self) -> MutexGuard<def::Schema> { self.file.lock().expect("Previous panic") }
}

/// Response type for [`Request`].
#[derive(Getters)]
pub struct Response {
    /// The params in the initial save request.
    #[getset(get = "pub")]
    params: Params,
    /// The encoded save buffer.
    #[getset(get = "pub")]
    buffer: Vec<u8>,
}

#[codegen::system(PreVisualize)]
fn save_scenario(
    #[subscriber] params_sub: impl Iterator<Item = Params>,
    #[publisher] requests: impl FnMut(Request),
    #[resource(no_init)] scenario: &def::Scenario,
    #[resource(no_init)] config: &def::Config,
    #[resource(no_init)] def: &GameDefinition,
) {
    for params in params_sub {
        requests(Request {
            params: params.clone(),
            file:   Mutex::new(
                def::Schema::builder()
                    .scenario(scenario.clone())
                    .config(config.clone())
                    .def(def.pack())
                    .state(state::State::default())
                    .build(),
            ),
        });
    }
}

#[codegen::system(PostVisualize)]
fn save_request(
    #[subscriber] requests: impl Iterator<Item = Request>,
    #[publisher] responses: impl FnMut(Response),
) {
    fn handle_request(request: &Request) -> anyhow::Result<Response> {
        use anyhow::Context;

        let file = request.file.lock().expect("Previous panic");
        let mut buffer = Vec::new();
        file.write(&mut buffer).context("Error encoding save file")?;
        Ok(Response { params: request.params.clone(), buffer })
    }

    for request in requests {
        match handle_request(request) {
            Ok(resp) => responses(resp),
            Err(err) => log::error!("Error saving file: {:?}", err),
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(save_request_setup).uses(save_scenario_setup)
}
