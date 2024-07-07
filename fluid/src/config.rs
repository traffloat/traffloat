//! Fluid definitions.

use std::sync::Arc;

use bevy::prelude::{Component, Resource};

use crate::units;

/// Identifies a type of fluid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct Type(pub u16);

/// A bevy Resource storing all available fluid types.
#[derive(Resource, Clone)]
pub struct Config(Arc<ConfigBuilder>);

impl Config {
    /// Gets the definition of a fluid type
    pub fn get_type(&self, ty: Type) -> &TypeDef {
        self.0.defs.get(usize::from(ty.0)).expect("reference to unknown fluid type")
    }

    /// Iterates over all fluid types.
    pub fn iter_types(&self) -> impl Iterator<Item = (Type, &TypeDef)> {
        self.0.defs.iter().enumerate().map(|(index, def)| (Type(index as u16), def))
    }

    /// Transferring fluid less than this amount would not trigger container element creation.
    pub fn creation_threshold(&self) -> units::Mass { self.0.creation_threshold }

    /// Remaining fluid less than this amount would trigger container element deletion.
    pub fn deletion_threshold(&self) -> units::Mass { self.0.deletion_threshold }
}

/// Constructs the [`Config`] resource.
pub struct ConfigBuilder {
    defs:                   Vec<TypeDef>,
    /// Transferring fluid less than this amount would not trigger container element creation.
    pub creation_threshold: units::Mass,
    /// Remaining fluid less than this amount would trigger container element deletion.
    pub deletion_threshold: units::Mass,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            defs:               Vec::new(),
            creation_threshold: units::Mass { quantity: 1e-3 },
            deletion_threshold: units::Mass { quantity: 1e-6 },
        }
    }
}

impl ConfigBuilder {
    /// Registers a new fluid type. Only for constructing test cases.
    pub fn register_type(&mut self, def: TypeDef) -> Type {
        let ret = Type(u16::try_from(self.defs.len()).expect("too many types"));
        self.defs.push(def);
        ret
    }

    /// Converts the builder into a resource.
    pub fn build(self) -> Config { Config(Arc::new(self)) }
}

/// Defines the properties of a fluid.
pub struct TypeDef {
    /// Viscosity coefficient.
    ///
    /// Viscosity is inversely proportional to flow rate in fluid flow
    /// and diffusion rate in diffusion respectively.
    pub viscosity: units::Viscosity,

    /// The specific volume (reciprocal of density) of the fluid during vacuum phase.
    pub vacuum_specific_volume: units::SpecificVolume,

    /// The pressure above which the fluid exhibits saturation phase properties.
    pub critical_pressure: units::Pressure,

    /// The amplitification coefficient for saturated fluids.
    pub saturation_gamma: f32,
}
