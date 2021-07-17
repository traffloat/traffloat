//! Defines the mechanism of a game.

use typed_builder::TypedBuilder;

pub mod building;
pub mod cargo;
pub mod gas;
pub mod liquid;
pub mod reaction;
pub mod skill;

/// Game mechanism definition.
#[derive(TypedBuilder, getset::Getters, getset::MutGetters)]
pub struct GameDefinition {
    /// Cargo types.
    #[getset(get = "pub", get_mut = "pub")]
    cargo: Vec<cargo::Type>,
    /// Cargo categories.
    #[getset(get = "pub", get_mut = "pub")]
    cargo_cats: Vec<cargo::Category>,
    /// Liquid types.
    #[getset(get = "pub", get_mut = "pub")]
    liquid: Vec<liquid::Type>,
    /// Gas types.
    #[getset(get = "pub", get_mut = "pub")]
    gas: Vec<gas::Type>,
    /// Skill types.
    #[getset(get = "pub", get_mut = "pub")]
    skill: Vec<skill::Type>,
    /// Reaction types.
    #[getset(get = "pub", get_mut = "pub")]
    reaction: Vec<reaction::Type>,
    /// Reaction categories.
    #[getset(get = "pub", get_mut = "pub")]
    reaction_cats: Vec<reaction::Category>,
    /// Building types.
    #[getset(get = "pub", get_mut = "pub")]
    building: Vec<building::Type>,
    /// Building categories.
    #[getset(get = "pub", get_mut = "pub")]
    building_cats: Vec<building::Category>,
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
    /// Adds a reaction category.
    pub fn add_reaction_category(&mut self, cat: reaction::Category) -> reaction::CategoryId {
        self.reaction_cats.push(cat);
        reaction::CategoryId(self.reaction_cats.len() - 1)
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
}
