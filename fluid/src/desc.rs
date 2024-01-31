use dynec::global;

use crate::Type;

/// Describes the properties of a fluid type.
pub struct Descriptor {
    pub viscosity: f64,
}

/// Stores all known fluid types.
#[global(initial)]
#[derive(Default)]
pub struct Descriptors {
    types: Vec<Descriptor>,
}

impl Descriptors {
    /// Iterates over all fluid types.
    pub fn iter(&self) -> impl Iterator<Item = (Type, &Descriptor)> {
        self.types.iter().enumerate().map(|(i, desc)| (Type(i), desc))
    }

    /// Gets the descriptor of a fluid type.
    pub fn get(&self, ty: Type) -> Option<&Descriptor> { self.types.get(ty.0) }
}
