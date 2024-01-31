use dynec::{archetype, comp, Discrim, Entity};
use traffloat_graph::Node;

use crate::{Mass, Pressure, Type, Volume};

archetype! {
    /// A storage is a container that holds zero or more types of fluids.
    ///
    /// A node may have multiple storages, which may be connected to adjacent storages or pipes
    /// depending on the configuration of the node.
    pub Storage;
}

/// The ordinal number of a storage within a node.
///
/// `StorageNumber(0)` is always the "ambient storage".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Discrim)]
pub struct StorageNumber(pub usize);

/// The list of storages in a node, stored as isotopes.
/// Our assumption is that there is only a small number of storages in a node,
/// and accessing them by the maximum constant is not too memory expensive.
#[comp(of = Node, isotope = StorageNumber)]
pub struct RefFromNode {
    #[entity]
    pub storage: Entity<Storage>,
}

/// The position of a storage.
#[comp(of = Storage)]
pub struct Owner {
    /// The owner node entity.
    #[entity]
    pub node:   Entity<Node>,
    /// The number of this storage in the owner.
    pub number: StorageNumber,
}

/// The space occupied by fluid of a type in a storage.
#[comp(of = Storage, isotope = Type, required, init = || TypedMass{mass: Mass{quantity: 0.0}})]
pub struct TypedMass {
    pub mass: Mass,
}

/// The space occupied by fluid of a type in a storage.
#[comp(of = Storage, isotope = Type, required, init = || TypedVolume{volume: Volume{quantity: 0.0}})]
pub struct TypedVolume {
    pub volume: Volume,
}

/// Sum of TypedVolume of all fluids in the storage.
#[comp(of = Storage, required, init = || VolumeSum{volume: Volume{quantity: 0.0}})]
pub struct VolumeSum {
    pub volume: Volume,
}

/// The pressure of the fluids in a storage.
///
/// For simplicity we assume that all fluids, liquid or not, share the same pressure.
///
/// A storage may have negative pressure, e.g. if the storage volume is greater than the fluid vacuum volume.
#[comp(of = Storage, required)]
pub struct CurrentPressure {
    pub pressure: Pressure,
}

/// The maximum capacity of fluid in a storage.
#[comp(of = Storage)]
pub struct MaxVolume {
    pub volume: Volume,
}

/// The maximum pressure of fluid in a storage.
#[comp(of = Storage)]
pub struct MaxPressure {
    pub pressure: Pressure,
}
