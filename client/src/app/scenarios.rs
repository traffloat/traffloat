pub struct ScenarioDef {
    pub id:   &'static str,
    pub name: &'static str,
    pub path: &'static str,
}

pub const OPTIONS: &[ScenarioDef] = &[ScenarioDef {
    id:   "vanilla",
    name: "Vanilla",
    path: "gen/scenarios/vanilla/scenario.tfsave",
}];
