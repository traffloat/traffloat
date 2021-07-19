//! Vanilla data definition

use traffloat::graph;
use traffloat_types::def::GameDefinition;

pub mod building;
pub mod cargo;
mod default;
pub mod gas;
pub mod liquid;
pub mod reaction;

pub const VANILLA_TEXTURE: &str = "textures.png";

/// Define vanilla game rules
pub fn get() -> (
    GameDefinition,
    Vec<graph::NodeComponents>,
    Vec<(usize, usize, f64)>,
) {
    let mut def = GameDefinition::default();
    let liquid = liquid::populate(&mut def);
    let gas = gas::populate(&mut def);
    let cargo = cargo::populate(&mut def);
    let reaction = reaction::populate(&mut def, &cargo, &liquid, &gas);
    let building = building::populate(&mut def, &reaction);

    let (nodes, edges) = default::default_setup(&def, &building);

    (def, nodes, edges)
}
