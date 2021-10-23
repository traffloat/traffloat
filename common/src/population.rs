//! Population-related components.

use derive_new::new;
use getset::Getters;
use legion::Entity;
use smallvec::SmallVec;

use crate::def::building::storage::population as storage;
use crate::SetupEcs;

/// List of population storages in a node.
///
/// This component is applied on all nodes.
#[derive(Debug, Getters)]
pub struct StorageList {
    /// The entities containing the Storage.
    #[getset(get = "pub")]
    storages: SmallVec<[Entity; 2]>, // usually only 2 storages, "transit" and "operator".
}

/// A storage of population.
#[derive(Getters, new)]
pub struct Storage {
    /// The capacity of the storage.
    #[getset(get = "pub")]
    capacity: u32,
}

codegen::component_depends! {
    Storage = (
        Storage,
    ) + ?()
}

/// Copmonent applied on nodes to indicate housing provision.
#[derive(new, getset::CopyGetters)]
pub struct Housing {
    /// The population storage used as housing.
    #[getset(get_copy = "pub")]
    storage: storage::Id,
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs { setup }
