//! Resistance is an undirected coefficient inversely proportional to the flow rate across a pipe.
//!
//! # Static resistance
//! A resistance source can be static or dynamic.
//! A static resistance source changes infrequently and only needs to be recomputed on demand.
//! Systems contributing static resistance should only update the [`Static`] of an entity
//! when a [`RecomputeStaticEvent`] for the entity is received
//! in the [`SystemSets::Static`] system set:
//!
//! ```
//! use bevy::prelude::*;
//! use traffloat_fluid::pipe::resistance;
//!
//! #[derive(Component)]
//! struct MyData(f32);
//!
//! fn example_contributor_system(
//!     mut events: EventReader<resistance::RecomputeStaticEvent>,
//!     mut query: Query<(&MyData, &mut resistance::Static)>,
//! ) {
//!     for event in events.read() {
//!         let (data, mut sum) = query
//!             .get_mut(event.entity)
//!             .expect("RecomputeStaticEvent must contain a valid pipe entity");
//!         sum.resistance.quantity += data.0;
//!     }
//! }
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn build(&self, app: &mut App) {
//!         app.add_systems(
//!             Update,
//!             example_contributor_system.in_set(resistance::SystemSets::Static),
//!         );
//!     }
//! }
//! ```
//!
//! When a static resistance value for a pipe needs to be recomputed,
//! emit a [`RecomputeStaticEvent`] for the pipe entity
//! in a system scheduled **before** the [`SystemSets::Compute`] system set.
//!
//! ```
//! use bevy::prelude::*;
//! use traffloat_fluid::pipe::resistance;
//!
//! #[derive(Resource)]
//! struct MyFavoritePipe(Entity);
//!
//! fn example_trigger_system(
//!     pipe: Res<MyFavoritePipe>,
//!     mut ev_writer: EventWriter<resistance::RecomputeStaticEvent>,
//! ) {
//!     ev_writer.send(resistance::RecomputeStaticEvent { entity: pipe.0 });
//! }
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn build(&self, app: &mut App) {
//!         app.add_systems(Update, example_trigger_system.before(resistance::SystemSets::Compute));
//!     }
//! }
//! ```
//!
//! # Dynamic resistance
//! Dynamic resistance sources are recomputed every cycle.
//! Systems contributing dynamic resistance should update the [`Dynamic`] of an entity
//! every cycle in the [`SystemSets::Dynamic`] system set:
//!
//! ```
//! use bevy::prelude::*;
//! use traffloat_fluid::pipe::resistance;
//!
//! #[derive(Component)]
//! struct MyData(f32);
//!
//! fn example_contributor_system(mut query: Query<(&MyData, &mut resistance::Dynamic)>) {
//!     for (data, mut sum) in query.iter_mut() {
//!         sum.resistance.quantity += data.0;
//!     }
//! }
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn build(&self, app: &mut App) {
//!         app.add_systems(
//!             Update,
//!             example_contributor_system.in_set(resistance::SystemSets::Dynamic),
//!         );
//!     }
//! }
//! ```

use bevy::app;
use bevy::prelude::{
    App, Component, Entity, Event, EventReader, IntoSystemConfigs, IntoSystemSetConfigs, Query,
    SystemSet,
};
use derive_more::From;

use crate::units;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RecomputeStaticEvent>();
        app.add_systems(
            app::Update,
            (
                static_to_dynamic_system.after(SystemSets::Static).before(SystemSets::Dynamic),
                init_static.before(SystemSets::Static).in_set(SystemSets::Compute),
            ),
        );
        app.configure_sets(
            app::Update,
            (SystemSets::Static, SystemSets::Dynamic).in_set(SystemSets::Compute),
        );
    }
}

/// System sets for resistance processing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum SystemSets {
    /// A system set wrapping all systems for initializing and computing resistance.
    Compute,
    /// Systems that update static resistance.
    Static,
    /// Systems that update dynamic resistance.
    Dynamic,
}

fn init_static(
    mut events: EventReader<RecomputeStaticEvent>,
    mut query: Query<(&FromShape, &mut Static)>,
) {
    for event in events.read() {
        let (shape, mut sum) = query
            .get_mut(event.entity)
            .expect("RecomputeStaticEvent must contain a valid pipe entity");
        // Reset directly, overwriting any previous value since we are first.
        // This is not a regular static contributor;
        // it is always first to avoid a useless zero write.
        sum.resistance = shape.resistance;
    }
}

/// Contributes static resistance as a dynamic resistance,
/// and acts as a partition between static and dynamic resistance.
///
/// - All static resistance contributors must execute before this.
/// - All dynamic resistance contributors must execute after this.
fn static_to_dynamic_system(mut query: Query<(&Static, &mut Dynamic)>) {
    query.iter_mut().for_each(|(static_, mut dynamic)| {
        dynamic.resistance += static_.resistance;
    });
}

/// Resistance due to shape of a pipe.
///
/// This is affected by factors like corridor length, corridor radius and pipe radius.
/// The value is determined during construction;
/// it is not computed under any systems in this plugin.
#[derive(Component, From)]
pub struct FromShape {
    /// Resistance value from shape.
    pub resistance: units::Resistance,
}

/// Sum of all static resistance over a pipe.
///
/// Static resistance is the sources of resistance that do not change in magnitude frequently.
/// As such, we only cache the sum of all such sources.
///
/// Modules providing a static resistance source should handle [`RecomputeStaticEvent`]
/// by adding their contributed resistance to this component
/// in a system in the [`SystemSets::Static`] set.
#[derive(Component)]
pub struct Static {
    /// Total static resistance.
    pub resistance: units::Resistance,
}

/// Sum of all resistance types.
///
/// Dynamic resistance is the sources of resistance that have to be recomputed every cycle.
///
/// Modules providing a dynamic resistance source should
/// add their contributed resistance to this component
/// every cycle in a system in the [`SystemSets::Dynamic`] set.
///
/// The value should only be read after [`SystemSets::Compute`].
#[derive(Component)]
pub struct Dynamic {
    /// Total dynamic resistance.
    pub resistance: units::Resistance,
}

/// Notifies that the static resistance for a pipe needs to be recomputed.
///
/// See module-level documentation for details.
#[derive(Event)]
pub struct RecomputeStaticEvent {
    /// A pipe entity.
    pub entity: Entity,
}
