//! Saving game definition and state.

use std::ops;
use std::sync::{Mutex, MutexGuard};

use arcstr::ArcStr;
use codegen::{IdStr, Identifiable, Identifier, SetupEcs};
use getset::Getters;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::def::{self, state};
use crate::liquid;

/// Stores all scenario gamerule definitions.
#[derive(Getters, Default)]
pub struct GameDefinition {
    /// Language bundles.
    #[getset(get = "pub")]
    lang:              Vec<def::lang::Def>,
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
    /// Creates the object from raw definitions.
    pub fn new(defs: impl IntoIterator<Item = def::Def>) -> anyhow::Result<Self> {
        let mut ret = GameDefinition::default();
        for def in defs {
            match def {
                def::Def::LangBundle(def) => ret.lang.push(def),
                def::Def::Liquid(def) => ret.liquid.push(def),
                def::Def::LiquidFormula(def) => ret.liquid_recipes.define(&def)?,
                def::Def::DefaultLiquidFormula(def) => ret.liquid_recipes.define_default(&def)?,
                def::Def::Gas(def) => ret.gas.push(def),
                def::Def::CargoCategory(def) => ret.cargo_category.push(def),
                def::Def::Cargo(def) => ret.cargo.push(def),
                def::Def::Skill(def) => ret.skill.push(def),
                def::Def::Vehicle(def) => ret.vehicle.push(def),
                def::Def::BuildingCategory(def) => ret.building_category.push(def),
                def::Def::Building(def) => ret.building.push(def),
                def::Def::Crime(def) => ret.crime.push(def),
                def::Def::Atlas(_) => (), // unused in runtime
            }
        }
        Ok(ret)
    }

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

    /// Finds a def object by string or integer ID.
    pub fn find<I: IdListExt>(&self, id: &MixedId<I>) -> Option<&I::Def> {
        match id {
            MixedId::Int(int) => int.index(I::get_list(self)),
            MixedId::Str(str) => I::get_list(self).iter().find(|def| def.id_str() == str),
        }
    }
}

/// Either integer or string ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MixedId<T: Identifier> {
    /// The integer form.
    Int(T),
    /// The string form.
    Str(IdStr),
}

impl<T: Identifier> MixedId<T> {
    /// Shorthand for `MixedId::Str(IdStr::new(...))`
    pub fn new_str(str: &str) -> Self { Self::Str(IdStr::new(ArcStr::from(str))) }
}

impl<T: IdListExt> From<T> for MixedId<T> {
    fn from(t: T) -> Self { Self::Int(t) }
}

impl<T: IdListExt> From<IdStr> for MixedId<T> {
    fn from(t: IdStr) -> Self { Self::Str(t) }
}

/// An extension trait for getting the corresponding Vec field from a [`GameDefinition`].
pub trait IdListExt: Identifier {
    /// Gets the list of the identified type from the definition.
    fn get_list(def: &GameDefinition) -> &[<Self as Identifier>::Def];
}

impl<I: IdListExt> ops::Index<I> for GameDefinition {
    type Output = I::Def;

    fn index(&self, id: I) -> &Self::Output {
        id.index(I::get_list(self)).expect("Corrupted definition")
    }
}

macro_rules! impl_id_list_ext {
    ($field:ident, $($module:tt)*) => {
        impl IdListExt for $($module)*::Id {
            fn get_list(def: &GameDefinition) -> &[$($module)*::Def] {
                &def.$field[..]
            }
        }
    }
}

impl_id_list_ext!(liquid, def::liquid);
impl_id_list_ext!(gas, def::gas);
impl_id_list_ext!(cargo_category, def::cargo::category);
impl_id_list_ext!(cargo, def::cargo);
impl_id_list_ext!(skill, def::skill);
impl_id_list_ext!(vehicle, def::vehicle);
impl_id_list_ext!(building_category, def::building::category);
impl_id_list_ext!(building, def::building);
impl_id_list_ext!(crime, def::crime);

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
#[derive(Clone, TypedBuilder)]
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

/// Loads a scenario file.
pub fn load_scenario(setup: SetupEcs, scenario: &def::Schema) -> SetupEcs {
    let def = match GameDefinition::new(scenario.def().iter().cloned()) {
        Ok(def) => def,
        Err(err) => {
            log::error!("Error loading scenario: {:?}", err);
            return setup;
        }
    };

    setup.resource(def).resource(scenario.scenario().clone()).resource(scenario.config().clone())
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(save_request_setup).uses(save_scenario_setup)
}
