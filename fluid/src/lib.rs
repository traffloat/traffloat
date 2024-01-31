//! Fluids are items that can diffuse between adjacent storages and conduits.
//!
//! "Fluid" is the generalization of gases and liquids.

use derive_more::{Add, AddAssign, Neg, Sub, SubAssign, Sum};
use dynec::Discrim;

mod desc;
pub use desc::*;

pub mod pipe;
pub mod storage;

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
    defs: Vec<TypeDef>,
}

impl TypeDefs {
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

    /// Compressibility coefficient. For simplicity we assume constant compressibility.
    ///
    /// Compressibility is the ratio of relative volume decrease and absolute pressure increase,
    /// i.e. (1 - V2/V1) / (P2 - P1).
    pub compress: f64,

    /// Volume of 1.0 mass of the fluid at vacuum.
    ///
    /// This value actually makes no sense in realistic physics,
    /// but is used as a baseline value to estimate the fluid pressure based on volume.
    pub molar_volume_vacuum: Volume,

    /// compress multiplied by molar_volume_vacuum,
    /// used to compute the pressure.
    pub cmvv: f64,
}
