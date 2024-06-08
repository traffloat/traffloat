//! Fluids are items that can diffuse between adjacent storages and conduits.
//!
//! "Fluid" is the generalization of gases and liquids.

use derive_more::{Add, AddAssign, Neg, Sub, SubAssign, Sum};
use dynec::Discrim;

pub mod container;
pub use container::Container;

// pub mod pipe;

/// Identifies a type of liquid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Discrim)]
pub struct Type(pub usize);

// Common units to describe liquids.

/// The mass of fluid.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign, Sub, SubAssign, Sum, Neg)]
pub struct Mass {
    pub quantity: f64,
}

/// The space occupied by fluid.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign, Sub, SubAssign, Sum, Neg)]
pub struct Volume {
    pub quantity: f64,
}

/// The pressure of fluid.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign, Sub, SubAssign, Sum, Neg)]
pub struct Pressure {
    pub quantity: f64,
}

/// The viscosity of a liquid, inversely proportional to flow rate.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign, Sub, SubAssign, Sum, Neg)]
pub struct Viscosity {
    pub quantity: f64,
}

#[dynec::global]
pub struct TypeDefs {
    pub defs: Vec<TypeDef>,
}

impl TypeDefs {
    /// Registers a new fluid type. Only for constructing test cases.
    pub fn register(&mut self, def: TypeDef) -> Type {
        let ret = Type(self.defs.len()-1);
        self.defs.push(def);
        ret
    }

    /// Gets the definition of a fluid type
    pub fn get(&self, ty: Type) -> &TypeDef {
        self.defs.get(ty.0).expect("reference to unknown fluid type")
    }

    /// Iterates over all fluid types.
    pub fn iter(&self) -> impl Iterator<Item = (Type, &TypeDef)> {
        self.defs.iter().enumerate().map(|(index, def)| (Type(index), def))
    }
}

pub struct TypeDef {
    /// Viscosity coefficient.
    ///
    /// Viscosity is inversely proportional to flow rate in fluid flow
    /// and diffusion rate in diffusion respectively.
    pub viscosity: Viscosity,

    /// The specific volume (reciprocal of density) of the fluid during vacuum phase.
    pub vacuum_specific_volume: f64,

    /// The critical pressure above which the fluid exhibits saturation phase properties.
    pub critical_pressure: Pressure,
}

pub struct Bundle;

impl dynec::Bundle for Bundle {
    fn register(&mut self, builder: &mut dynec::world::Builder) {
        builder.schedule(container::reconcile_container.build());
    }
}
