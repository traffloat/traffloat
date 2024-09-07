use bevy::app::App;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Query;
use bevy::ecs::world::{Command, World};
use serde::{Deserialize, Serialize};

use crate::save;

#[test]
fn e2e_json() { e2e(save::Format::Json); }

#[test]
fn e2e_msgpack() { e2e(save::Format::Msgpack); }

fn e2e(format: save::Format) {
    fn init() -> App {
        let mut app = App::new();
        app.add_plugins(save::Plugin);
        save::add_def::<Parent>(&mut app);
        save::add_def::<Child>(&mut app);
        app
    }

    let mut app = init();

    let parent = app.world_mut().spawn((ParentName("Parent".into()),)).id();
    app.world_mut().spawn((ChildParent(parent), ChildLabel("Child".into())));

    save::StoreCommand {
        format,
        on_complete: Box::new(move |_, result| {
            let result = result.unwrap();

            // test entity creation in a new world
            let mut app = init();

            save::LoadCommand {
                data:        result,
                on_complete: Box::new(|world, result| {
                    result.unwrap();

                    let (parent_entity, parent_name) =
                        world.query::<(Entity, &ParentName)>().single(world);
                    assert_eq!(parent_name.0, "Parent");

                    let (child_parent, child_label) =
                        world.query::<(&ChildParent, &ChildLabel)>().single(world);
                    assert_eq!(child_parent.0, parent_entity);
                    assert_eq!(child_label.0, "Child");
                }),
            }
            .apply(app.world_mut());
        }),
    }
    .apply(app.world_mut());
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Parent {
    name: String,
}

#[derive(Component)]
struct ParentName(String);

impl save::Def for Parent {
    const TYPE: &'static str = "parent";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Parent>,
            (): (),
            query: Query<(Entity, &ParentName)>,
        ) {
            writer.write_all(
                query.iter().map(|(entity, name)| (entity, Parent { name: name.0.clone() })),
            );
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(world: &mut World, def: Parent, (): &()) -> anyhow::Result<Entity> {
            Ok(world.spawn(ParentName(def.name)).id())
        }

        save::LoadFn::new(loader)
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Child {
    parent: save::Id<Parent>,
    label:  String,
}

#[derive(Component)]
struct ChildParent(Entity);

#[derive(Component)]
struct ChildLabel(String);

impl save::Def for Child {
    const TYPE: &'static str = "child";

    type Runtime = ();

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Child>,
            (parent_ids,): (save::StoreDepend<Parent>,),
            query: Query<(&ChildParent, &ChildLabel)>,
        ) {
            writer.write_all(query.iter().map(|(parent, label)| {
                ((), Child { parent: parent_ids.get(parent.0).unwrap(), label: label.0.clone() })
            }));
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        fn loader(
            world: &mut World,
            def: Child,
            (parent_dep,): &(save::load::Depend<Parent>,),
        ) -> anyhow::Result<()> {
            world.spawn((ChildParent(parent_dep.get(def.parent)?), ChildLabel(def.label)));
            Ok(())
        }

        save::LoadFn::new(loader)
    }
}
