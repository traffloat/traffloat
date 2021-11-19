//! Saving game definition and state.

use std::ops;
use std::sync::{Mutex, MutexGuard};

use codegen::SetupEcs;
use getset::Getters;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::clock::Clock;
use crate::{def, edge, liquid, node};

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
    pub fn new(defs: impl IntoIterator<Item = def::AnyDef>) -> anyhow::Result<Self> {
        let mut ret = GameDefinition::default();
        for def in defs {
            match def {
                def::AnyDef::LangBundle(def) => ret.lang.push(def),
                def::AnyDef::Liquid(def) => ret.liquid.push(def),
                def::AnyDef::LiquidFormula(def) => ret.liquid_recipes.define(&def)?,
                def::AnyDef::DefaultLiquidFormula(def) => {
                    ret.liquid_recipes.define_default(&def)?
                }
                def::AnyDef::Gas(def) => ret.gas.push(def),
                def::AnyDef::CargoCategory(def) => ret.cargo_category.push(def),
                def::AnyDef::Cargo(def) => ret.cargo.push(def),
                def::AnyDef::Skill(def) => ret.skill.push(def),
                def::AnyDef::Vehicle(def) => ret.vehicle.push(def),
                def::AnyDef::BuildingCategory(def) => ret.building_category.push(def),
                def::AnyDef::Building(def) => ret.building.push(def),
                def::AnyDef::Crime(def) => ret.crime.push(def),
                def::AnyDef::Atlas(_) => (), // unused in runtime
            }
        }
        Ok(ret)
    }

    /// Pack the [`GameDefinition`] into a vector of [`def::Def`] for serialization.
    pub fn pack(&self) -> Vec<def::AnyDef> {
        use def::AnyDef; // don't import this globally because everything is called `Def`.

        self.liquid
            .iter()
            .cloned()
            .map(AnyDef::Liquid)
            .chain(self.gas.iter().cloned().map(AnyDef::Gas))
            .chain(self.cargo_category.iter().cloned().map(AnyDef::CargoCategory))
            .chain(self.cargo.iter().cloned().map(AnyDef::Cargo))
            .chain(self.skill.iter().cloned().map(AnyDef::Skill))
            .chain(self.vehicle.iter().cloned().map(AnyDef::Vehicle))
            .chain(self.building_category.iter().cloned().map(AnyDef::BuildingCategory))
            .chain(self.building.iter().cloned().map(AnyDef::Building))
            .chain(self.crime.iter().cloned().map(AnyDef::Crime))
            .collect()
    }

    /// Finds a def object by string or integer ID.
    pub fn find<T: GameDefObject>(&self, id: &MixedId<T>) -> Option<&T> {
        let list = <T as GameDefObject>::get_list(self);

        match id {
            MixedId::Int(int) => list.get(int.index()),
            MixedId::Str(str) => list.iter().find(|def| def.id_str().value() == str),
        }
    }
}

/// Either integer or string ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MixedId<T> {
    /// The integer form.
    Int(def::Id<T>),
    /// The string form.
    Str(String),
}

impl<T> MixedId<T> {
    /// Shorthand for `MixedId::Str(IdStr::new(...))`
    pub fn new_str(str: &str) -> Self { Self::Str(String::from(str)) }
}

impl<T> From<def::Id<T>> for MixedId<T> {
    fn from(t: def::Id<T>) -> Self { Self::Int(t) }
}

impl<T> From<def::IdString<T>> for MixedId<T> {
    fn from(t: def::IdString<T>) -> Self { Self::new_str(t.value()) }
}

/// Implemented by def types that can be uniquely indexed in the [`GameDefinition`] scope.
pub trait GameDefObject: Sized {
    /// Gets the slice in the [`GameDefinition`] that holds the objects of this type.
    fn get_list(def: &GameDefinition) -> &[Self];

    /// Returns the runtime integer ID of the object.
    fn id(&self) -> def::Id<Self>;

    /// Returns the original string ID of the object.
    fn id_str(&self) -> &def::IdString<Self>;
}

macro_rules! impl_game_def_object {
    ($($field:ident: [$def:ty],)* $(,)?) => {
        $(
            impl GameDefObject for $def {
                fn get_list(def: &GameDefinition) -> &[Self] {
                    &def.$field[..]
                }

                fn id(&self) -> def::Id<Self> {
                    <$def>::id(self)
                }

                fn id_str(&self) -> &def::IdString<Self> {
                    <$def>::id_str(self)
                }
            }
        )*
    };
}

impl_game_def_object! {
    liquid: [def::liquid::Def],
    gas: [def::gas::Def],
    cargo_category: [def::cargo::category::Def],
    cargo: [def::cargo::Def],
    skill: [def::skill::Def],
    vehicle: [def::vehicle::Def],
    building_category: [def::building::category::Def],
    building: [def::building::Def],
    crime: [def::crime::Def],
}

impl<T: GameDefObject> ops::Index<def::Id<T>> for GameDefinition {
    type Output = T;

    fn index(&self, id: def::Id<T>) -> &Self::Output {
        let list = <T as GameDefObject>::get_list(self);
        list.get(id.index()).expect("Reference out of bounds")
    }
}

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
    file:   Mutex<def::TfsaveFile>,
}

impl Request {
    /// Locks the mutex on the serialization target and returns the wrapped value.
    pub fn file(&self) -> MutexGuard<def::TfsaveFile> { self.file.lock().expect("Previous panic") }
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
                def::TfsaveFile::builder()
                    .scenario(scenario.clone())
                    .config(config.clone())
                    .def(def.pack())
                    .state(def::State::default())
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
pub fn load_scenario(mut setup: SetupEcs, scenario: &def::TfsaveFile, micros_now: i64) -> SetupEcs {
    let def = match GameDefinition::new(scenario.def().iter().cloned()) {
        Ok(def) => def,
        Err(err) => {
            log::error!("Error loading scenario: {:?}", err);
            return setup;
        }
    };

    {
        let mut clock = setup.resources.get_mut_or_default::<Clock>();
        clock.reset_time(scenario.state().time(), micros_now);
    }

    for node in scenario.state().nodes() {
        setup =
            setup.publish_event(node::LoadRequest::builder().save(Box::new(node.clone())).build());
    }
    for edge in scenario.state().edges() {
        setup =
            setup.publish_event(edge::LoadRequest::builder().save(Box::new(edge.clone())).build());
    }

    setup.resource(def).resource(scenario.scenario().clone()).resource(scenario.config().clone())
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(save_request_setup).uses(save_scenario_setup)
}
