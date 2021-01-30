pub mod keyboard;

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(keyboard::setup_ecs)
}
