use std::time::Duration;

use bevy::app::{self, App};
use bevy::ecs::event::{Event, Events, ManualEventReader};
use bevy::ecs::system::{Res, Resource};
use bevy::ecs::world::{Command, World};
use bevy::hierarchy::BuildWorldChildren;
use bevy::math::Vec3;
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy::utils::HashMap;

use super::{
    create_type, make_value_feeder_system, SubscribeCommand, Subscription, Type, TypeDef,
    UpdateMetricEvent,
};
use crate::viewable::{self, ShowEvent};
use crate::{appearance, viewer};

#[test]
fn report() {
    let mut app = App::new();
    app.add_plugins(crate::Plugin);
    let setup = setup_world(&mut app);

    let mut show_event_reader = event_reader::<ShowEvent>(app.world());
    let mut metric_event_reader = event_reader::<UpdateMetricEvent>(app.world());

    for time in 0_u16..12 {
        prepare_update(app.world_mut(), time);
        app.update();
        check_world(
            app.world_mut(),
            &setup,
            &mut show_event_reader,
            &mut metric_event_reader,
            time,
        );
    }
}

struct WorldSetup {
    ty1:                Type,
    ty2:                Type,
    viewer_id:          viewer::Sid,
    parent_viewable_id: viewable::Sid,
    child_viewable_id:  viewable::Sid,
}

fn setup_world(app: &mut App) -> WorldSetup {
    app.insert_resource({
        let mut time: Time = Time::default();
        time.advance_to(Duration::from_millis(500));
        time
    });

    // Generate random values that can be validated for each cycle.
    let value_generator = ValueGenerator(0.);
    app.insert_resource(value_generator);

    let ty1 = create_type(app.world_mut(), TypeDef { update_frequency: Duration::from_secs(5) });
    let ty2 = create_type(app.world_mut(), TypeDef { update_frequency: Duration::from_secs(2) });

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

    let viewer_id = viewer::next_sid(app.world_mut());
    let viewer = app
        .world_mut()
        .spawn(
            viewer::Bundle::builder()
                .id(viewer_id)
                .range(viewer::Range { distance: 100. })
                .position(Transform { translation: Vec3::ZERO, ..<_>::default() })
                .build(),
        )
        .id();

    let parent_viewable_id = viewable::next_sid(app.world_mut());
    let child_viewable_id = viewable::next_sid(app.world_mut());

    app.world_mut()
        .spawn(
            viewable::StationaryBundle::builder()
                .base(
                    viewable::BaseBundle::builder()
                        .sid(parent_viewable_id)
                        .appearance(appearance::Layers::null())
                        .build(),
                )
                .transform(Transform::from_xyz(50., 50., 50.).with_scale(Vec3::splat(0.01)))
                .build(),
        )
        .with_children(|builder| {
            builder.spawn(
                viewable::StationaryChildBundle::builder()
                    .base(
                        viewable::BaseBundle::builder()
                            .sid(child_viewable_id)
                            .appearance(appearance::Layers::null())
                            .build(),
                    )
                    .inner_transform(Transform::from_xyz(200., 200., 200.))
                    .build(),
            );
        });

    SubscribeCommand { viewer, ty: ty1, subscription: Subscription { noise_sd: 0. } }
        .apply(app.world_mut());
    SubscribeCommand { viewer, ty: ty2, subscription: Subscription { noise_sd: 0. } }
        .apply(app.world_mut());

    WorldSetup { ty1, ty2, viewer_id, parent_viewable_id, child_viewable_id }
}

fn prepare_update(world: &mut World, time: u16) {
    world.resource_mut::<ValueGenerator>().0 = time.into();
}
fn check_world(
    world: &mut World,
    setup: &WorldSetup,
    show_event_reader: &mut ManualEventReader<ShowEvent>,
    metric_event_reader: &mut ManualEventReader<UpdateMetricEvent>,
    time: u16,
) {
    let show_events: Vec<_> = get_events(world, show_event_reader).collect();
    if time == 0 {
        assert_eq!(show_events.len(), 2);
        assert_eq!(show_events[0].viewer, setup.viewer_id);
        assert_eq!(show_events[0].viewable, setup.parent_viewable_id);
        assert_eq!(show_events[1].viewer, setup.viewer_id);
        assert_eq!(show_events[1].viewable, setup.child_viewable_id);
    } else {
        assert_eq!(show_events.len(), 0);
    }

    {
        let metric_events: HashMap<_, _> = get_events(world, metric_event_reader)
            .map(|event| (event.ty, event.magnitude))
            .collect();

        let actual = [setup.ty1, setup.ty2].map(|ty| metric_events.get(&ty).copied());

        let value_generator = world.resource::<ValueGenerator>();

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

    let mut time_res = world.resource_mut::<Time>();
    time_res.advance_by(Duration::from_secs(1));
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
