//! A pipe element corresponds to an active fluid type transferring across a pipe.
//!
//! A container element is created when an adjacent pipe wants to transfer
//! a new fluid type into this container.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use traffloat_graph::corridor::Binary;
use typed_builder::TypedBuilder;

use crate::{config, units};

/// Components to construct a pipe element.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    ty:                 config::Type,
    container_elements: ContainerElements,
    #[builder(default = AbTransferMass { mass: <_>::default() })]
    ab_transfer_mass:   AbTransferMass,
    #[builder(default = TransferWeight { output: <_>::default() })]
    transfer_weight:    TransferWeight,
}

/// A base coefficient for the volumetric flow rate in each direction,
/// specific to the element before considering pipe-wide resistance.
///
/// This coefficient is used as the weight to
/// distribute the available volumetric flow rate to different elements.
#[derive(Component)]
pub struct TransferWeight {
    /// This is the output weight,
    /// i.e. `output.alpha` is the output from alpha to beta,
    /// and `output.beta` is the output from beta to alpha.
    pub output: Binary<f32>,
}

/// Net transfer of this type from alpha to beta in the current cycle.
#[derive(Component)]
pub struct AbTransferMass {
    /// Net transfer value.
    pub mass: units::Mass,
}

/// The container elements connected by the pipe.
///
/// Although this reuses the [`Binary`] type from the corridor module,
/// this has nothing to do with corridors.
///
/// One of the endpoint containers may not have the element created yet.
/// The element is only created when the fluid spreads to the container;
/// this ensures fluids do not immediately propagate to the entire network.
#[derive(Component)]
pub struct ContainerElements {
    /// At least one of `alpha` and `beta` must be set,
    /// i.e. `{alpha: None, beta: None}` is invalid.
    ///
    /// Due to rustc limitations, `enum { Entity, Entity, (Entity, Entity) }`
    /// currently does not have niche optimizations and occupies 24 bytes,
    /// so for now we use a pair of `Option`s for more compact representation with 16 bytes.
    pub containers: Binary<Option<Entity>>,
}
