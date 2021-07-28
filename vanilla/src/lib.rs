//! Vanilla data definition

use traffloat::def::GameDefinition;

pub mod building;
pub mod cargo;
pub mod crime;
pub mod gas;
pub mod liquid;
pub mod reaction;
pub mod skill;
pub mod vehicle;

mod default;
pub use default::default_setup;

pub const VANILLA_TEXTURE: &str = "textures-x-fancy.png"; // TODO allow customization of resolution

/// All IDs assigned to different vanilla definitions.
pub struct AllIds {
    /// All skill IDS.
    pub skill: skill::Ids,
    /// All vehicle IDS.
    pub vehicle: vehicle::Ids,
    /// All liquid IDS.
    pub liquid: liquid::Ids,
    /// All gas IDS.
    pub gas: gas::Ids,
    /// All cargo IDS.
    pub cargo: cargo::Ids,
    /// All reaction IDS.
    pub reaction: reaction::Ids,
    /// All building IDS.
    pub building: building::Ids,
}

/// Define vanilla game rules
pub fn get() -> (GameDefinition, AllIds) {
    let mut def = GameDefinition::new();
    let skill = skill::populate(&mut def);
    let vehicle = vehicle::populate(&mut def, &skill);
    let liquid = liquid::populate(&mut def);
    let gas = gas::populate(&mut def);
    let cargo = cargo::populate(&mut def);
    let reaction = reaction::populate(&mut def, &cargo, &liquid, &gas, &skill);
    let building = building::populate(&mut def, &reaction, &skill);
    crime::populate(&mut def, &skill);

    (
        def,
        AllIds {
            skill,
            vehicle,
            liquid,
            gas,
            cargo,
            reaction,
            building,
        },
    )
}
