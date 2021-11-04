use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::units;
use typed_builder::TypedBuilder;

use super::NodeId;

/// The state of an edge.
#[derive(Getters, CopyGetters, TypedBuilder, Serialize, Deserialize)]
pub struct Edge {
    /// The "alpha" endpoint of an edge.
    #[getset(get_copy = "pub")]
    alpha:     NodeId,
    /// The "beta" endpoint of an edge.
    #[getset(get_copy = "pub")]
    beta:       NodeId,
    /// The radius of an edge.
    #[getset(get_copy = "pub")]
    radius:   f64,
    /// The hitpoint portion of the edge.
    #[getset(get_copy = "pub")]
    hitpoint: units::Portion<units::Hitpoint>,
    /// The ducts built in the edge.
    #[getset(get = "pub")]
    design:   Vec<Duct>,
}

/// The state of a duct.
#[derive(Getters, CopyGetters, TypedBuilder, Serialize, Deserialize)]
pub struct Duct {
    /// The position of the center of the duct, relative to the center of the edge.
    #[getset(get_copy = "pub")]
    center: nalgebra::Vector2<f64>,
    /// The radius of the duct.
    #[getset(get_copy = "pub")]
    radius: f64,
    /// The type of the duct.
    #[getset(get_copy = "pub")]
    ty:     DuctType,
}

/// The type of a duct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuctType {
    /// A rail that vehicles can move along.
    ///
    /// The first parameter is the direction of the rail,
    /// or [`None`] if the rail is disabled.
    Rail(Option<Direction>),
    /// A pipe that liquids can be transferred through.
    ///
    /// The first parameter is the direction of the pipe,
    /// or [`None`] if the rail is disabled.
    ///
    /// The second and third parameters are the liquid storage IDs in the endpoints.
    Liquid {
        /// The direction that the pipe runs in
        dir:          Option<Direction>,
        /// The storage ordinal in the "from" node.
        ///
        /// This value does **not** swap with [`to_storage`] when the direction is flipped.
        src_storage:  usize,
        /// The storage ordinal in the "to" node.
        ///
        /// This value does **not** swap with [`from_storage`] when the direction is flipped.
        dest_storage: usize,
    },
    /// A cable that electricity can pass through.
    ///
    /// The first parameter specifies whether the cable is enabled.
    Electricity(bool),
}

impl DuctType {
    /// Whether the duct is active.
    pub fn active(self) -> bool {
        match self {
            Self::Rail(option) => option.is_some(),
            Self::Liquid { dir, .. } => dir.is_some(),
            Self::Electricity(enabled) => enabled,
        }
    }

    /// The direction of the duct, if any.
    pub fn direction(self) -> Option<Direction> {
        match self {
            Self::Rail(dir) | Self::Liquid { dir, .. } => dir,
            _ => None,
        }
    }
}

/// A direction across an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// A direction starting from [`Edge::alpha`] and ending at [`Edge::beta`]
    AlphaBeta,
    /// A direction starting from [`Edge::beta`] and ending at [`Edge::alpha`]
    BetaAlpha,
}

codegen::impl_definition_by_self!(Direction);
