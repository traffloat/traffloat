//! Vanilla data definition

use traffloat_types::def::GameDefinition;

pub mod building;
pub mod cargo;
pub mod gas;
pub mod liquid;
pub mod reaction;

/// Define vanilla game rules
pub fn get() -> GameDefinition {
    let mut def = GameDefinition::default();
    let liquid = liquid::populate(&mut def);
    let gas = gas::populate(&mut def);
    let cargo = cargo::populate(&mut def);
    reaction::populate(&mut def, cargo, liquid, gas);
    building::populate(&mut def);
    def
}
