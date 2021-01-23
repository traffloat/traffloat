use shrev::EventChannel;

use common::proto::Packet;

pub struct NetSystem {
    net_reader: shrev::ReaderId<Packet>,
}

impl NetSystem {
    pub fn new(world: &mut specs::World) -> Self {
        use specs::SystemData;

        <Self as specs::System<'_>>::SystemData::setup(world);
        Self {
            net_reader: world
                .get_mut::<EventChannel<Packet>>()
                .expect("Packet channel initialized in setup")
                .register_reader(),
        }
    }
}

impl<'a> specs::System<'a> for NetSystem {
    type SystemData = (
        specs::Read<'a, EventChannel<Packet>>,
    );

    fn run(&mut self, (
        packets,
    ): Self::SystemData) {

    }
}

pub fn setup_specs((mut world, mut dispatcher): common::Setup) -> common::Setup {
    dispatcher = dispatcher.with(NetSystem::new(&mut world), "net", &[]);

    (world, dispatcher)
}
