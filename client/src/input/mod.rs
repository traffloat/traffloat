pub mod keyboard;
pub mod mouse;

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(keyboard::setup_ecs).uses(mouse::setup_ecs)
}
