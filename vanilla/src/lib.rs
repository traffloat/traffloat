//! Vanilla data definition

use traffloat::def::GameDefinition;
use traffloat::node;

pub mod building;
pub mod cargo;
pub mod crime;
pub mod default;
pub mod gas;
pub mod liquid;
pub mod reaction;
pub mod skill;
pub mod vehicle;

pub const VANILLA_TEXTURE: &str = "textures-x-fancy.png"; // TODO allow customization of resolution

/// Define vanilla game rules
pub fn get() -> (GameDefinition, Vec<node::Components>, default::EdgeSetup) {
    let mut def = GameDefinition::default();
    let skill = skill::populate(&mut def);
    let _vehicle = vehicle::populate(&mut def, &skill);
    let liquid = liquid::populate(&mut def);
    let gas = gas::populate(&mut def);
    let cargo = cargo::populate(&mut def);
    let reaction = reaction::populate(&mut def, &cargo, &liquid, &gas, &skill);
    let building = building::populate(&mut def, &reaction, &skill);
    crime::populate(&mut def, &skill);

    let (nodes, edges) = default::default_setup(&def, &building);

    (def, nodes, edges)
}
