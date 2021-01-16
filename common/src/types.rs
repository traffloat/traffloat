//! The common types imported everywhere.

use std::any::Any;
use std::cmp::Ordering;

pub use specs::{storage, Component, Entity, ReadStorage, System, WriteStorage};

use crate::proto::{self, BinRead, BinWrite, ProtoType};
pub use crate::time::*;

/// Standard vector type
pub type Vector = nalgebra::Vector3<f32>;

/// Standard homogenous matrix type
pub type Matrix = nalgebra::Matrix4<f32>;

/// A generic variant-identifier mechanism
pub trait Id: Any + Sized {
    /// Returns the network ID value
    fn network_id_u32(self) -> u32;

    /// Finds the specs entity ID for this entity.
    fn entity(self) -> Entity {
        todo!()
    }
}

macro_rules! ids {
    ($($(#[$meta:meta])* $id:ident;)*) => {
        mod id_impl {
            $(
                #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, codegen::Gen)]
                #[repr(transparent)]
                pub(super) struct $id(pub(super) u32);
            )*
        }

        $(
            #[derive(Debug, Clone, Copy, Default)]
            $(#[$meta])*
            pub struct $id {
                network_id: id_impl::$id,
                entity: Option<Entity>,
            }

            impl Id for $id {
                fn network_id_u32(self) -> u32 {
                    self.network_id.0
                }
            }

            impl PartialEq for $id {
                fn eq(&self, other: &Self) -> bool { self.network_id == other.network_id }
            }

            impl Eq for $id {}

            impl PartialOrd for $id {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl Ord for $id {
                fn cmp(&self, other: &Self) -> Ordering {
                    self.network_id.cmp(&other.network_id)
                }
            }

            impl ProtoType for $id {
                const CHECKSUM: u128 = <id_impl::$id as ProtoType>::CHECKSUM;
            }

            impl BinWrite for $id {
                fn write(&self, vec: &mut Vec<u8>) {
                    self.network_id.write(vec);
                }
            }

            impl BinRead for $id {
                fn read(buf: &mut &[u8]) -> Result<Self, proto::Error> {
                    Ok(Self {
                        network_id: id_impl::$id::read(buf)?,
                        entity: None,
                    })
                }
            }
        )*
    }
}

ids! {
    /// Identifies a node (building)
    NodeId;

    /// Identifies a liquid type
    LiquidId;

    /// Identifies a rail
    PipeId;

    /// Identifies a gas type
    GasId;

    /// Identifies a cargo type
    CargoId;

    /// Identifies a rail
    RailId;

    /// Identifies a reaction
    ReactionId;
}

ratio_def::units! {
    /// A common unit type
    Unit(std::fmt::Debug + Clone + Copy + Default + PartialEq + PartialOrd + ProtoType + BinWrite + BinRead);

    #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, codegen::Gen)] f32:

    /// An amount of liquid
    LiquidVolume;

    /// The pressure of air in a room
    GasPressure;

    /// An absolute amount of gas
    GasVolume;

    /// The standard size for cargo
    CargoSize;

    /// Specific heat capacity
    HeatCapacity;

    /// Heat energy
    HeatEnergy;

    /// Dynamic electricity consumed immediately
    ElectricPower;

    /// Static electricity in stored form
    ElectricEnergy;
}

/// The endpoints of an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Component, codegen::Gen)]
#[storage(storage::VecStorage)]
pub struct EdgeId {
    /// The source node of the edge
    pub first: NodeId,
    /// The destination node of the edge
    pub second: NodeId,
}
