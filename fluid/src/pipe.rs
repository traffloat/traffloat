//! A pipe is a link between two containers.
//!
//! Despite a possible implication of its name,
//! pipes exist within a building or between a building and a corridor,
//! but not in the corridor.
//!
//! Each pipe is the parent entity of a number of "pipe element" child entities,
//! corresponding to all active fluid types across the link.

use bevy::ecs::bundle;
use bevy::prelude::{Commands, Component, Entity, Event, EventReader, Query, Res};
use bevy::{app, hierarchy};
use traffloat_graph::corridor::{Binary, Endpoint};
use typed_builder::TypedBuilder;

/// Executes fluid mass transfer between containers.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(app::Update, contribute_shape_resistance_system);
    }
}

/// Components to construct a pipe.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    containers:       Containers,
    shape_resistance: ShapeResistance,
}

/// The containers connected by the pipe.
///
/// Although this reuses the [`Binary`] type from the corridor module,
/// this has nothing to do with corridors.
#[derive(Component)]
pub struct Containers {
    pub containers: Binary<Entity>,
}

/// Resistance due to shape of a pipe.
///
/// This is affected by factors like corridor length, corridor radius and pipe radius.
/// The value is determined during construction;
/// it is not computed under any systems in this plugin.
#[derive(Component)]
pub struct ShapeResistance {
    pub resistance: f32,
}

/// Sum of all static resistance over a pipe.
///
/// Static resistance are the sources of resistance that do not change in magnitude frequently.
/// As such, we only cache the sum of all such sources.
///
/// Modules providing a static resistance source should handle [`RecomputeStaticResistanceEvent`]
/// by adding their contributed resistance to the component.
#[derive(Component)]
pub struct StaticResistance {
    pub resistance: f32,
}

fn contribute_shape_resistance_system(
    mut events: EventReader<RecomputeStaticResistanceEvent>,
    mut query: Query<(&ShapeResistance, &mut StaticResistance)>,
) {
    for event in events.read() {
        let (shape, mut sum) = query
            .get_mut(event.entity)
            .expect("RecomputeStaticResistanceEvent must contain a valid pipe entity");
        sum.resistance += shape.resistance;
    }
}

/// Notifies that the static resistance for a pipe needs to be recomputed.
///
/// Modules that want to change a static resistance should produce this event once
/// AFTER setting [`StaticResistance`] to 0.
/// Then they can add their desired new value to [`StaticResistance`] in the event handler.
#[derive(Event)]
pub struct RecomputeStaticResistanceEvent {
    /// A pipe entity.
    pub entity: Entity,
}
