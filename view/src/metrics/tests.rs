use std::time::Duration;

use bevy::app::{self, App};
use bevy::ecs::event::{Event, Events, ManualEventReader};
use bevy::ecs::system::{Res, Resource};
use bevy::ecs::world::{Command, World};
use bevy::math::Vec3;
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy::utils::HashMap;

use super::{
    create_type, make_value_feeder_system, SubscribeCommand, Subscription, TypeDef,
    UpdateMetricEvent,
};
use crate::viewable::{self, ShowEvent};
use crate::viewer;

#[test]
fn test_report() {
    let mut app = App::new();
    app.add_plugins((super::Plugin, viewable::Plugin));
    app.insert_resource({
        let mut time: Time = Time::default();
        time.advance_to(Duration::from_millis(500));
        time
    });

    let ty1 = create_type(app.world_mut(), TypeDef { update_frequency: Duration::from_secs(5) });
    let ty2 = create_type(app.world_mut(), TypeDef { update_frequency: Duration::from_secs(2) });

    // Generate random values that can be validated for each cycle.
    let value_generator = ValueGenerator(0.);
    app.insert_resource(value_generator);

    let feeders = (
        make_value_feeder_system::<&Transform, (), Res<ValueGenerator>, _>(
            app.world_mut(),
            move |entity, value_generator| {
                let tf = entity.get::<Transform>().unwrap();

                value_generator.generate(tf.translation.x, 1.)
            },
            ty1,
        ),
        make_value_feeder_system::<&Transform, (), Res<ValueGenerator>, _>(
            app.world_mut(),
            move |entity, value_generator| {
                let tf = entity.get::<Transform>().unwrap();

                value_generator.generate(tf.translation.x, -1.)
            },
            ty2,
        ),
    );
    app.add_systems(app::Update, feeders);

    let viewer = app
        .world_mut()
        .spawn(
            viewer::Bundle::builder()
                .range(viewer::Range { distance: 100. })
                .position(Transform { translation: Vec3::ZERO, ..<_>::default() })
                .build(),
        )
        .id();

    let viewable = app
        .world_mut()
        .spawn((
            viewable::Bundle::builder().position(Transform::from_xyz(50., 0., 0.)).build(),
            viewable::Static,
        ))
        .id();

    SubscribeCommand { viewer, ty: ty1, subscription: Subscription { noise_sd: 0. } }
        .apply(app.world_mut());
    SubscribeCommand { viewer, ty: ty2, subscription: Subscription { noise_sd: 0. } }
        .apply(app.world_mut());

    let mut show_event_reader = event_reader::<ShowEvent>(app.world());
    let mut metric_event_reader = event_reader::<UpdateMetricEvent>(app.world());

    for time in 0_u16..12 {
        app.world_mut().resource_mut::<ValueGenerator>().0 = time.into();
        app.update();

        let show_events: Vec<_> = get_events(app.world(), &mut show_event_reader).collect();
        if time == 0 {
            assert_eq!(show_events.len(), 1);
            assert_eq!(show_events[0].viewer, viewer);
            assert_eq!(show_events[0].viewable, viewable);
        } else {
            assert_eq!(show_events.len(), 0);
        }

        {
            let metric_events: HashMap<_, _> = get_events(app.world(), &mut metric_event_reader)
                .map(|event| (event.ty, event.magnitude))
                .collect();

            let actual = [ty1, ty2].map(|ty| metric_events.get(&ty).copied());

            let value_generator = app.world_mut().resource::<ValueGenerator>();

            let mut expected = [None, None];

            if time > 0 {
                if time % 5 == 0 {
                    expected[0] = Some(value_generator.generate(50., 1.));
                }

                if time % 2 == 0 {
                    expected[1] = Some(value_generator.generate(50., -1.));
                }
            }

            assert_eq!(expected, actual);
        }

        let mut time_res = app.world_mut().resource_mut::<Time>();
        time_res.advance_by(Duration::from_secs(1));
    }
}

#[derive(Resource)]
struct ValueGenerator(f32);

impl ValueGenerator {
    fn generate(&self, pos_x: f32, multiplier: f32) -> f32 { (pos_x * 100. + self.0) * multiplier }
}

fn event_reader<E: Event>(world: &World) -> ManualEventReader<E> {
    world.resource::<Events<E>>().get_reader()
}

fn get_events<'a, E: Event>(
    world: &'a World,
    reader: &'a mut ManualEventReader<E>,
) -> impl Iterator<Item = &'a E> {
    reader.read(world.resource::<Events<E>>())
}
