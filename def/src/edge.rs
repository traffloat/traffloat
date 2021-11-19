//! Edge states.

use derive_new::new;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::units;
use typed_builder::TypedBuilder;

use crate::building;
use crate::node::NodeId;

/// The state of an edge.
#[derive(Debug, Clone, Getters, CopyGetters, TypedBuilder, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Edge {
    /// The endpoints of an edge.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(flatten)))]
    endpoints: AlphaBeta,
    /// The radius of an edge.
    #[getset(get_copy = "pub")]
    radius:    f64,
    /// The hitpoint portion of the edge.
    #[getset(get_copy = "pub")]
    hitpoint:  units::Portion<units::Hitpoint>,
    /// The ducts built in the edge.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    ducts:     Vec<Duct>,
}

/// The endpoints of an edge.
#[derive(Debug, Clone, Copy, CopyGetters, new, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), process))]
pub struct AlphaBeta {
    /// The "alpha" endpoint of an edge.
    #[getset(get_copy = "pub")]
    alpha: NodeId,
    /// The "beta" endpoint of an edge.
    #[getset(get_copy = "pub")]
    beta:  NodeId,
}

/// The state of a duct.
#[derive(Debug, Clone, Getters, CopyGetters, TypedBuilder, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
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
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), serde(tag = "type")))]
pub enum DuctType {
    /// A rail that vehicles can move along.
    ///
    /// The first parameter is the direction of the rail,
    /// or [`None`] if the rail is disabled.
    Rail(RailDuctType),
    /// A pipe that liquids can be transferred through.
    ///
    /// The first parameter is the direction of the pipe,
    /// or [`None`] if the rail is disabled.
    ///
    /// The second and third parameters are the liquid storage IDs in the endpoints.
    Liquid(LiquidDuctType),
    /// A cable that electricity can pass through.
    ///
    /// The first parameter specifies whether the cable is enabled.
    Electricity(ElectricityDuctType),
}

impl DuctType {
    /// Whether the duct is active.
    pub fn active(self) -> bool {
        match self {
            Self::Rail(ty) => ty.direction.is_some(),
            Self::Liquid(ty) => !matches!(ty, LiquidDuctType::Disabled),
            Self::Electricity(ty) => !ty.disabled,
        }
    }

    /// The direction of the duct, if any.
    pub fn direction(self) -> Option<Direction> {
        match self {
            Self::Rail(ty) => ty.direction,
            Self::Liquid(LiquidDuctType::AlphaToBeta { .. }) => Some(Direction::AlphaToBeta),
            Self::Liquid(LiquidDuctType::BetaToAlpha { .. }) => Some(Direction::BetaToAlpha),
            _ => None,
        }
    }
}

/// Details of a rail duct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters, Serialize, Deserialize)]
pub struct RailDuctType {
    /// The direction of the rail.
    #[getset(get_copy = "pub")]
    direction: Option<Direction>,
}

/// Details of a liquid duct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiquidDuctType {
    /// A duct from alpha node to beta node.
    AlphaToBeta {
        /// The source storage ID in the alpha node.
        alpha_storage: building::storage::liquid::Id,
        /// The destination storage ID in the beta node.
        beta_storage:  building::storage::liquid::Id,
    },
    /// A duct from beta node to alpha node.
    BetaToAlpha {
        /// The source storage ID in the beta node.
        beta_storage:  building::storage::liquid::Id,
        /// The destination storage ID in the alpha node.
        alpha_storage: building::storage::liquid::Id,
    },
    /// A disabled duct.
    Disabled,
}

/// Details of a electricity duct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct ElectricityDuctType {
    /// Whether the duct is disabled.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    disabled: bool,
}

/// A direction across an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub enum Direction {
    /// A direction starting from the alpha endpoint and ending at the beta endpoint
    AlphaToBeta,
    /// A direction starting from the beta endpoint and ending at the alpha endpoint
    BetaToAlpha,
}

#[cfg(feature = "xy")]
mod xy {
    use std::any::TypeId;

    use anyhow::Context as _;
    use serde::Deserialize;
    use xylem::id::GlobalIdStore;
    use xylem::{Context as _, DefaultContext, NoArgs, Processable, Xylem};

    use super::Direction;
    use crate::node::xy::NodeBuildingMap;
    use crate::{building, Schema};

    struct EdgeEndpointBuildings {
        alpha: building::Id,
        beta:  building::Id,
    }

    impl Processable<Schema> for super::AlphaBeta {
        fn postprocess(&mut self, context: &mut DefaultContext) -> anyhow::Result<()> {
            let map = context
                .get::<NodeBuildingMap>(TypeId::of::<()>())
                .expect("NodeBuildingMap not initialized");

            let alpha = map.get(self.alpha).expect("Dangling node reference");
            let beta = map.get(self.beta).expect("Dangling node reference");

            context.get_mut::<EdgeEndpointBuildings, _>(TypeId::of::<super::Edge>(), || {
                EdgeEndpointBuildings { alpha, beta }
            });

            Ok(())
        }
    }

    impl Xylem<Schema> for super::RailDuctType {
        type From = RailDuctTypeXylem;
        type Args = NoArgs;

        fn convert_impl(
            from: Self::From,
            _: &mut DefaultContext,
            _: &NoArgs,
        ) -> anyhow::Result<Self> {
            Ok(Self {
                direction: match from.dir {
                    MaybeDirection::AlphaToBeta => Some(Direction::AlphaToBeta),
                    MaybeDirection::BetaToAlpha => Some(Direction::BetaToAlpha),
                    MaybeDirection::Disabled => None,
                },
            })
        }
    }

    #[derive(Deserialize)]
    pub struct RailDuctTypeXylem {
        dir: MaybeDirection,
    }

    #[derive(Deserialize)]
    pub enum MaybeDirection {
        AlphaToBeta,
        BetaToAlpha,
        Disabled,
    }

    impl Xylem<Schema> for super::LiquidDuctType {
        type From = LiquidDuctTypeXylem;
        type Args = NoArgs;

        fn convert_impl(
            from: Self::From,
            context: &mut DefaultContext,
            _: &NoArgs,
        ) -> anyhow::Result<Self> {
            Ok(match from {
                LiquidDuctTypeXylem::AlphaToBeta { ref alpha_storage, ref beta_storage }
                | LiquidDuctTypeXylem::BetaToAlpha { ref beta_storage, ref alpha_storage } => {
                    fn resolve_storage(
                        building: building::Id,
                        storage: &str,
                        context: &mut DefaultContext,
                    ) -> anyhow::Result<building::storage::liquid::Id> {
                        let store = context
                            .get::<GlobalIdStore<Schema, building::storage::liquid::Def>>(
                                TypeId::of::<()>(),
                            )
                            .expect("Liquid storage definitions are not getting tracked");
                        let ids = store
                            .ids()
                            .get([building.index()].as_ref())
                            .context("No liquid storages in building")?;
                        let id = ids.iter().position(|id| id == storage).with_context(|| {
                            format!("No liquid storage named {} in building", storage)
                        })?;
                        Ok(building::storage::liquid::Id::new(id))
                    }

                    let &EdgeEndpointBuildings { alpha, beta } = context
                        .get::<EdgeEndpointBuildings>(TypeId::of::<super::Edge>())
                        .expect("Edge endpoints was not initialized");

                    let alpha = resolve_storage(alpha, alpha_storage, context)?;
                    let beta = resolve_storage(beta, beta_storage, context)?;
                    match from {
                        LiquidDuctTypeXylem::AlphaToBeta { .. } => {
                            Self::AlphaToBeta { alpha_storage: alpha, beta_storage: beta }
                        }
                        LiquidDuctTypeXylem::BetaToAlpha { .. } => {
                            Self::AlphaToBeta { beta_storage: beta, alpha_storage: alpha }
                        }
                        _ => unreachable!("Within match arm"),
                    }
                }
                LiquidDuctTypeXylem::Disabled => Self::Disabled,
            })
        }
    }

    #[derive(Deserialize)]
    #[serde(tag = "direction")]
    pub enum LiquidDuctTypeXylem {
        AlphaToBeta { alpha_storage: String, beta_storage: String },
        BetaToAlpha { beta_storage: String, alpha_storage: String },
        Disabled,
    }
}
