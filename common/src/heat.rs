//! Manages temperature and sunlight

use crate::types::*;
use crate::Setup;

/// The sun
#[derive(Debug, codegen::Gen)]
pub struct Sun {
    /// The position of the sun
    pub position: Vector,
}

/// Moves the sun
pub struct SunSystem(());

impl SunSystem {
    /// Initializes a SunSystem
    pub fn new(world: &mut specs::World) -> Self {
        use specs::SystemData;

        <Self as specs::System<'_>>::SystemData::setup(world);
        Self(())
    }
}

impl<'a> specs::System<'a> for SunSystem {
    type SystemData = (
        specs::WriteExpect<'a, Sun>,
        specs::Read<'a, Clock>,
    );

    fn run(&mut self, (mut sun, clock): Self::SystemData) {
        use nalgebra::dimension as dim;

        let axis = nalgebra::Unit::new_normalize(Vector::new(1., 0., 0.));
        let transform: Matrix = Matrix::from_axis_angle(&axis, clock.delta.as_secs() / 100.0);
        sun.position = transform.fixed_slice::<dim::U3, dim::U3>(0, 0) * sun.position;
    }
}

/// Initializes the reaction module
pub fn setup_specs((mut world, mut dispatcher): Setup) -> Setup {
    world.insert(Sun { position: Vector::new(0., 0., -1.) });
    dispatcher = dispatcher.with(SunSystem::new(&mut world), "sun", &[]);
    (world, dispatcher)
}
