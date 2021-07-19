//! Manages factory building logic.

use smallvec::SmallVec;

use crate::def;
use crate::SetupEcs;

/// A component attached to buildings that can perform reactions.
#[derive(getset::Getters)]
pub struct Factory {
    /// List of reactions supported by this factory.
    #[getset(get = "pub")]
    reactions: SmallVec<[def::reaction::TypeId; 2]>,
}
/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
