//! Population-related components.

use derive_new::new;
use gusket::Gusket;
use legion::Entity;
use smallvec::SmallVec;

use crate::def::building::storage::population as storage;
use crate::SetupEcs;

/// List of population storages in a node.
///
/// This component is applied on all nodes.
#[derive(Debug, Gusket)]
pub struct StorageList {
    /// The entities containing the Storage.
    #[gusket(immut)]
    storages: SmallVec<[Entity; 2]>, // usually only 2 storages, "transit" and "operator".
}

/// A storage of population.
#[derive(new, Gusket)]
pub struct Storage {
    /// The capacity of the storage.
    #[gusket(immut, copy)]
    capacity: u32,
}

codegen::component_depends! {
    Storage = (
        Storage,
    ) + ?()
}

/// Copmonent applied on nodes to indicate housing provision.
#[derive(new, Gusket)]
pub struct Housing {
    /// The population storage used as housing.
    #[gusket(immut, copy)]
    storage: storage::Id,
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs { setup }
