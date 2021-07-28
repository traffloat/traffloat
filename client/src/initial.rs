/// Sets up legion ECS.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    let (def, ids) = traffloat_vanilla::get();
    let mut setup = setup.resource(def.clone());

    setup = traffloat_vanilla::default_setup(setup, &def, &ids);

    setup
}
