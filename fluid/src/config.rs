//! Fluid definitions.

use std::sync::Arc;

use bevy::prelude::{Component, Resource};

use crate::units;

/// Identifies a type of fluid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct Type(pub u16);

/// A bevy Resource storing all available fluid types.
#[derive(Resource, Clone)]
pub struct TypeDefs(Arc<TypeDefsBuilder>);

impl TypeDefs {
    /// Gets the definition of a fluid type
    pub fn get(&self, ty: Type) -> &TypeDef {
        self.0.defs.get(usize::from(ty.0)).expect("reference to unknown fluid type")
    }

    /// Iterates over all fluid types.
    pub fn iter(&self) -> impl Iterator<Item = (Type, &TypeDef)> {
        self.0.defs.iter().enumerate().map(|(index, def)| (Type(index as u16), def))
    }
}

/// Constructs the [`TypeDefs`] resource.
#[derive(Default)]
pub struct TypeDefsBuilder {
    pub(crate) defs: Vec<TypeDef>,
}

impl TypeDefsBuilder {
    /// Registers a new fluid type. Only for constructing test cases.
    pub fn register(&mut self, def: TypeDef) -> Type {
        let ret = Type(u16::try_from(self.defs.len()).expect("too many types"));
        self.defs.push(def);
        ret
    }

    /// Converts the builder into a resource.
    pub fn build(self) -> TypeDefs { TypeDefs(Arc::new(self)) }
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
