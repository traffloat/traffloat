//! Defines the mechanism of a game.

use derive_new::new;
use serde::{Deserialize, Serialize};

pub mod building;
pub mod cargo;
pub mod crime;
pub mod gas;
pub mod liquid;
pub mod reaction;
pub mod skill;
pub mod vehicle;

/// Game mechanism definition.
#[derive(Debug, Clone, new, getset::Getters, getset::MutGetters, Serialize, Deserialize)]
pub struct GameDefinition {
    /// Cargo types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    cargo: Vec<cargo::Type>,
    /// Cargo categories.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    cargo_cats: Vec<cargo::Category>,
    /// Liquid types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    liquid: Vec<liquid::Type>,
    /// Gas types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    gas: Vec<gas::Type>,
    /// Skill types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    skill: Vec<skill::Type>,
    /// Vehicle types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    vehicle: Vec<vehicle::Type>,
    /// Reaction types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    reaction: Vec<reaction::Type>,
    /// Building types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    building: Vec<building::Type>,
    /// Building categories.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    building_cats: Vec<building::Category>,
    /// List of possible crimes.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    crime: Vec<crime::Type>,
}

impl GameDefinition {
    /// Adds a cargo category.
    pub fn add_cargo_category(&mut self, cat: cargo::Category) -> cargo::CategoryId {
        self.cargo_cats.push(cat);
        cargo::CategoryId(self.cargo_cats.len() - 1)
    }
    /// Adds a cargo type.
    pub fn add_cargo(&mut self, cargo: cargo::Type) -> cargo::TypeId {
        self.cargo.push(cargo);
        cargo::TypeId(self.cargo.len() - 1)
    }
    /// Adds a liquid type.
    pub fn add_liquid(&mut self, liquid: liquid::Type) -> liquid::TypeId {
        self.liquid.push(liquid);
        liquid::TypeId(self.liquid.len() - 1)
    }
    /// Adds a gas type.
    pub fn add_gas(&mut self, gas: gas::Type) -> gas::TypeId {
        self.gas.push(gas);
        gas::TypeId(self.gas.len() - 1)
    }
    /// Adds a skill type.
    pub fn add_skill(&mut self, skill: skill::Type) -> skill::TypeId {
        self.skill.push(skill);
        skill::TypeId(self.skill.len() - 1)
    }
    /// Adds a vehicle type.
    pub fn add_vehicle(&mut self, vehicle: vehicle::Type) -> vehicle::TypeId {
        self.vehicle.push(vehicle);
        vehicle::TypeId(self.vehicle.len() - 1)
    }
    /// Adds a reaction type.
    pub fn add_reaction(&mut self, reaction: reaction::Type) -> reaction::TypeId {
        self.reaction.push(reaction);
        reaction::TypeId(self.reaction.len() - 1)
    }
    /// Adds a building category.
    pub fn add_building_category(&mut self, cat: building::Category) -> building::CategoryId {
        self.building_cats.push(cat);
        building::CategoryId(self.building_cats.len() - 1)
    }
    /// Adds a building type.
    pub fn add_building(&mut self, building: building::Type) -> building::TypeId {
        self.building.push(building);
        building::TypeId(self.building.len() - 1)
    }

    /// Returns the cargo with the specified ID
    ///
    /// # Panics
    /// Panics if the type ID is undefined
    pub fn get_cargo(&self, id: cargo::TypeId) -> &cargo::Type {
        self.cargo.get(id.0).expect("Cargo ID is out of bounds")
    }
    /// Returns the liquid with the specified ID
    ///
    /// # Panics
    /// Panics if the type ID is undefined
    pub fn get_liquid(&self, id: liquid::TypeId) -> &liquid::Type {
        self.liquid.get(id.0).expect("Liquid ID is out of bounds")
    }
    /// Returns the gas with the specified ID
    ///
    /// # Panics
    /// Panics if the type ID is undefined
    pub fn get_gas(&self, id: gas::TypeId) -> &gas::Type {
        self.gas.get(id.0).expect("Gas ID is out of bounds")
    }
    /// Returns the skill with the specified ID
    ///
    /// # Panics
    /// Panics if the type ID is undefined
    pub fn get_skill(&self, id: skill::TypeId) -> &skill::Type {
        self.skill.get(id.0).expect("Skill ID is out of bounds")
    }
    /// Returns the reaction with the specified ID
    ///
    /// # Panics
    /// Panics if the type ID is undefined
    pub fn get_reaction(&self, id: reaction::TypeId) -> &reaction::Type {
        self.reaction
            .get(id.0)
            .expect("Reaction ID is out of bounds")
    }
    /// Returns the building with the specified ID
    ///
    /// # Panics
    /// Panics if the type ID is undefined
    pub fn get_building(&self, id: building::TypeId) -> &building::Type {
        self.building
            .get(id.0)
            .expect("Building ID is out of bounds")
    }
}
