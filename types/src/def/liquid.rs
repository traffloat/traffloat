//! Liquid definitions.

use std::borrow::Borrow;
use std::cmp;
use std::collections::BTreeMap;

use arcstr::ArcStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typed_builder::TypedBuilder;

use crate::units;

/// Identifies a liquid type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeId(pub ArcStr);

/// A type of liquid.
#[derive(
    Debug, Clone, TypedBuilder, getset::Getters, getset::CopyGetters, Serialize, Deserialize,
)]
pub struct Type {
    /// Name of the liquid type.
    #[getset(get = "pub")]
    name:         ArcStr,
    /// Short summary of the liquid type.
    #[getset(get = "pub")]
    summary:      ArcStr,
    /// Long description of the liquid type.
    #[getset(get = "pub")]
    description:  ArcStr,
    /// Viscosity of a liquid.
    #[getset(get_copy = "pub")]
    viscosity:    units::LiquidViscosity,
    /// The texture source path of the liquid.
    #[getset(get = "pub")]
    texture_src:  ArcStr,
    /// The texture name of the liquid.
    #[getset(get = "pub")]
    texture_name: ArcStr,
}

/// A data structure storing liquid mixing behaviour.
#[derive(Debug, Clone, getset::Getters)]
pub struct Mixer {
    /// The default type for mixing.
    #[getset(get = "pub")]
    default:   TypeId,
    /// Specific addition formulas.
    #[getset(get = "pub")]
    specifics: BTreeMap<Pair, TypeId>,
}

impl Mixer {
    /// Get a specific output.
    pub fn specific<'b>(&self, a: &'b TypeId, b: &'b TypeId) -> Option<&TypeId> {
        trait PairRef {
            fn less(&self) -> &TypeId;
            fn greater(&self) -> &TypeId;

            fn tuple(&self) -> (&TypeId, &TypeId) { (self.less(), self.greater()) }
        }

        impl PairRef for Pair {
            fn less(&self) -> &TypeId { &self.types[0] }
            fn greater(&self) -> &TypeId { &self.types[1] }
        }

        impl PairRef for (&TypeId, &TypeId) {
            fn less(&self) -> &TypeId { self.0 }
            fn greater(&self) -> &TypeId { self.1 }
        }

        impl<'b> Borrow<dyn PairRef + 'b> for Pair {
            fn borrow(&self) -> &(dyn PairRef + 'b) { self }
        }

        impl<'b> Borrow<dyn PairRef + 'b> for (&'b TypeId, &'b TypeId) {
            fn borrow(&self) -> &(dyn PairRef + 'b) { self }
        }

        impl PartialEq for dyn PairRef + '_ {
            fn eq(&self, other: &Self) -> bool {
                self.less() == other.less() && self.greater() == other.greater()
            }
        }
        impl Eq for dyn PairRef + '_ {}

        impl PartialOrd for dyn PairRef + '_ {
            fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> { Some(self.cmp(other)) }
        }
        impl Ord for dyn PairRef + '_ {
            fn cmp(&self, other: &Self) -> cmp::Ordering { self.tuple().cmp(&other.tuple()) }
        }

        self.specifics.get::<dyn PairRef + 'b>(&(a, b))
    }

    /// Returns the output liquid type when two liquids are mixed together.
    // TODO allow side effects other than liquid types.
    pub fn mix<'t>(&'t self, a: &'t TypeId, b: &'t TypeId) -> &'t TypeId {
        if a == b {
            return a;
        }

        self.specific(a, b).unwrap_or_else(|| self.default())
    }
}

#[derive(Serialize, Deserialize)]
struct Serde {
    default:  TypeId,
    formulas: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    augend: TypeId,
    addend: TypeId,
    sum:    TypeId,
}

impl Serialize for Mixer {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let serde = Serde {
            default:  self.default.clone(),
            formulas: self
                .specifics
                .iter()
                .map(|(pair, sum)| {
                    let [augend, addend] = pair.types.clone();
                    Entry { augend, addend, sum: sum.clone() }
                })
                .collect(),
        };
        serde.serialize(ser)
    }
}
impl<'de> Deserialize<'de> for Mixer {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let serde = Serde::deserialize(de)?;
        Ok(Self {
            default:   serde.default,
            specifics: serde
                .formulas
                .into_iter()
                .map(|entry| {
                    let pair = Pair::new(entry.augend, entry.addend);
                    (pair, entry.sum)
                })
                .collect(),
        })
    }
}

/// A pair of liquids.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pair {
    types: [TypeId; 2],
}

impl Pair {
    /// Creates a liquid pair.
    ///
    /// Currently, augend and addend are commutative.
    pub fn new(augend: TypeId, addend: TypeId) -> Self {
        let mut types = [augend, addend];
        types.sort();
        Self { types }
    }
}
