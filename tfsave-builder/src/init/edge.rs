use std::convert::TryInto;

use codegen::{Definition, IdStr};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use traffloat_def::building;
use traffloat_types::units;

use super::node::{self, NodeBuildingMap};

#[derive(Default)]
struct EdgeFromTo {
    from: building::Id,
    to:   building::Id,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Def {
    pub(super) from:     node::Id,
    pub(super) to:       node::Id,
    pub(super) radius:   f64,
    pub(super) hitpoint: units::Portion<units::Hitpoint>,
    pub(super) ducts:    Vec<Duct>,
}

impl Definition for Def {
    type HumanFriendly = DefHumanFriendly;

    fn convert(
        hf: Self::HumanFriendly,
        context: &mut codegen::ResolveContext,
    ) -> anyhow::Result<Self> {
        let from = context.resolve_id::<node::Def>(hf.from.as_str())?;
        let from = node::Id::from_index(from.try_into().expect("Too many items"));

        let to = context.resolve_id::<node::Def>(hf.to.as_str())?;
        let to = node::Id::from_index(to.try_into().expect("Too many items"));

        let (from_building, to_building) = {
            let node_building_map = context.get_other::<NodeBuildingMap>();
            (
                *node_building_map.0.get(&from).expect("Node building not indexed"),
                *node_building_map.0.get(&to).expect("Node building not indexed"),
            )
        };
        {
            let mut ft = context.get_other::<EdgeFromTo>();
            *ft = EdgeFromTo { from: from_building, to: to_building };
        }

        Ok(Self {
            from,
            to,
            radius: hf.radius,
            hitpoint: hf.hitpoint,
            ducts: hf
                .ducts
                .into_iter()
                .map(|hf| Definition::convert(hf, context))
                .collect::<anyhow::Result<Vec<_>>>()?,
        })
    }
}

#[derive(Deserialize)]
pub struct DefHumanFriendly {
    from:     IdStr,
    to:       IdStr,
    radius:   f64,
    hitpoint: units::Portion<units::Hitpoint>,
    #[serde(default)]
    ducts:    Vec<DuctHumanFriendly>,
}

#[derive(Clone, Serialize, Deserialize, Definition)]
#[hf_always]
pub struct Duct {
    pub(super) center: Vector2<f64>,
    pub(super) radius: f64,
    pub(super) ty:     DuctType,
}

#[derive(Clone, Copy, Serialize, Deserialize, Definition)]
#[hf_always]
pub enum DuctType {
    Rail { dir: MaybeDirection },
    Liquid(LiquidDuctType),
    Electricity { disabled: bool },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum LiquidDuctType {
    From2To {
        src_storage:  building::storage::liquid::Id,
        dest_storage: building::storage::liquid::Id,
    },
    To2From {
        src_storage:  building::storage::liquid::Id,
        dest_storage: building::storage::liquid::Id,
    },
    Disabled,
}

impl Definition for LiquidDuctType {
    type HumanFriendly = LiquidDuctTypeHumanFriendly;

    fn convert(
        hf: Self::HumanFriendly,
        context: &mut codegen::ResolveContext,
    ) -> anyhow::Result<Self> {
        let EdgeFromTo { from, to } = *context.get_other::<EdgeFromTo>();

        fn convert_storage(
            building: building::Id,
            storage: &str,
            context: &mut codegen::ResolveContext,
        ) -> anyhow::Result<building::storage::liquid::Id> {
            context.reuse_scoped::<building::storage::liquid::Def, _>(building)?;
            let id = context.resolve_id::<building::storage::liquid::Def>(storage)?;
            context.stop_tracking::<building::storage::liquid::Def>();
            Ok(building::storage::liquid::Id::from_index(id.try_into().expect("Too many items")))
        }

        let ret = match hf {
            LiquidDuctTypeHumanFriendly::From2To { src_storage, dest_storage } => Self::From2To {
                src_storage:  convert_storage(from, src_storage.as_str(), context)?,
                dest_storage: convert_storage(to, dest_storage.as_str(), context)?,
            },
            LiquidDuctTypeHumanFriendly::To2From { src_storage, dest_storage } => Self::To2From {
                src_storage:  convert_storage(to, src_storage.as_str(), context)?,
                dest_storage: convert_storage(from, dest_storage.as_str(), context)?,
            },
            LiquidDuctTypeHumanFriendly::Disabled => Self::Disabled,
        };
        Ok(ret)
    }
}

#[derive(Deserialize)]
#[serde(tag = "dir")]
pub enum LiquidDuctTypeHumanFriendly {
    From2To { src_storage: IdStr, dest_storage: IdStr },
    To2From { src_storage: IdStr, dest_storage: IdStr },
    Disabled,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum MaybeDirection {
    From2To,
    To2From,
    Disabled,
}

codegen::impl_definition_by_self!(MaybeDirection);
