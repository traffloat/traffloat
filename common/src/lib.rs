//! Common library for server and client

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
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        clippy::match_single_binding,
    )
)]
#![cfg_attr(
    not(debug_assertions),
    deny(
        missing_docs,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]

pub mod proto;
pub mod types;
mod util;
pub use util::*;

/// The standard setup parameters
#[derive(Default)]
pub struct SetupEcs {
    builder: legion::systems::Builder,
    world: legion::World,
    resources: legion::Resources,
}

impl SetupEcs {
    /// Register a bundle
    pub fn uses(self, setup_ecs: impl FnOnce(Self) -> Self) -> Self {
        setup_ecs(self)
    }

    /// Add a system
    pub fn system(mut self, sys: impl legion::systems::ParallelRunnable + 'static) -> Self {
        self.builder.add_system(sys);
        self
    }
    /// Add a thread-local system
    pub fn system_local(mut self, sys: impl legion::systems::Runnable + 'static) -> Self {
        self.builder.add_thread_local(sys);
        self
    }

    /// Add an entity
    pub fn entity<T>(mut self, components: T) -> Self
    where
        Option<T>: legion::storage::IntoComponentSource,
    {
        let _ = self.world.push(components);
        self
    }
    /// Add entities
    pub fn entities<T>(mut self, components: impl legion::storage::IntoComponentSource) -> Self {
        let _ = self.world.extend(components);
        self
    }

    /// Add a resource
    pub fn resource(mut self, res: impl legion::systems::Resource) -> Self {
        self.resources.insert(res);
        self
    }

    /// Build the setup into a legion
    pub fn build(mut self) -> Legion {
        Legion {
            world: self.world,
            resources: self.resources,
            schedule: self.builder.build(),
        }
    }
}

/// The set of values required to run legion
pub struct Legion {
    /// The legion world storing entities and components
    pub world: legion::World,
    /// The resource set storing legion resources
    pub resources: legion::Resources,
    /// The legion scheduler running systems
    pub schedule: legion::Schedule,
}

impl Legion {
    /// Spins all systems once.
    pub fn run(&mut self) {
        self.schedule.execute(&mut self.world, &mut self.resources)
    }
}

/// Initializes common modules.
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(types::setup_ecs)
}
