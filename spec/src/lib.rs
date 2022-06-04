//! Traffloat gamerule specification, used for world simulation, scenario editor and documentation generation.

#![deny(
    anonymous_parameters,
    bare_trait_objects,
    clippy::clone_on_ref_ptr,
    clippy::float_cmp_const,
    clippy::if_not_else,
    clippy::unwrap_used
)]
#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, clippy::match_single_binding,)
)]
#![cfg_attr(any(doc, not(debug_assertions)), deny(missing_docs))]
#![cfg_attr(
    not(debug_assertions),
    deny(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]

use serde::{Deserialize, Serialize};
use xylem::Xylem;

xylem::declare_schema! {
    /// The xylem schema for traffloat scenario builder.
    pub Schema : xylem::SchemaExt
}

/// Declares `Id` and `IdString` for an identifiable object.
macro_rules! impl_identifiable {
    ($(@$prefix:ident)? $ty:ty, $scope:ty) => {
        ::paste::paste! {
            #[doc = concat!("A copy-safe identifier of `", stringify!($ty), "`, used in most in-game logic and network transfer.")]
            pub type [<$($prefix)? Id>] = ::xylem::Id<crate::Schema, $ty>;
            #[doc = concat!("A string identifier of `", stringify!($ty), "`, typically used in scenario builder cross references and generated URLs.")]
            pub type [<$($prefix)? IdString>] = ::xylem::IdString<crate::Schema, $ty>;
        }

        impl ::xylem::Identifiable<crate::Schema> for $ty {
            type Scope = $scope;

            fn id(&self) -> ::xylem::Id<crate::Schema, $ty> { self.id }
        }
    };
    ($(@$prefix:ident)? $ty:ty) => {
        impl_identifiable!($(@$prefix)? $ty, ());
    };
}

pub mod expr;
pub mod i18n;
pub mod unit;
use i18n::I18n;
pub mod attribute;
pub mod building;
pub mod cargo;
pub mod edge;
pub mod field;
pub mod fluid;
pub mod node;
pub mod population;
pub mod skill;

/// Metadata of a scenario.
#[derive(Serialize, Deserialize)]
pub struct ScenarioMetadata {
    /// Display name of the scenario.
    pub name:        I18n<()>,
    /// Description of the scenario.
    pub description: I18n<()>,
}

/// Specifies a game item.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize), serde(tag = "type"))]
pub enum Spec {
    /// Specifies a field type.
    Field(field::Field),

    /// Specifies a fluid type.
    Fluid(fluid::Fluid),
    /// Specifies a cargo type.
    Cargo(cargo::Cargo),

    /// Specifies a building type.
    Building(building::Building),
    /// Specifies an attribute type.
    Attribute(attribute::Attribute),
    /// Creates a node.
    Node(node::Node),

    /// Creates an edge.
    Edge(edge::Edge),

    /// Specifies a skill.
    Skill(skill::Skill),
    /// Creates an inhabitant.
    Inhabitant(population::Inhabitant),
}
