use std::collections::HashMap;

use dynec::tracer;

use crate::{container, Mass, Pressure, Type, TypeDef, TypeDefs, Viscosity, Volume};

use super::Container;

struct ContainerSetup {
    pressure: f64,
    volume: f64,
    liquids: HashMap<Type, f64>,
}

struct TestBundle(Vec<ContainerSetup>);

impl dynec::Bundle for TestBundle {
    fn register(&mut self, builder: &mut dynec::world::Builder) {
        builder.schedule(super::reconcile_container.build());
        builder.global(TypeDefs{
            defs: vec![
                TypeDef {
                    viscosity: Viscosity{quantity: 1.0},
                    vacuum_specific_volume: 1.0,
                    critical_pressure: Pressure{quantity: 1.0},
                },
            ],
        });
    }

    fn populate(&mut self, world: &mut dynec::World) {
        for setup in &self.0 {
            world.create(dynec::comps![ Container =>
                container::MaxVolume{volume: Volume{quantity: setup.volume }},
                container::MaxPressure{pressure: Pressure{quantity: setup.pressure }},
                @?setup.liquids.iter().map(|(&ty, &quantity)| (ty, container::TypedMass{mass: Mass{quantity}})),
            ]);
        }
    }
}

#[test]
fn test_zero_mass() {
    let mut world = dynec::world::new([Box::new(TestBundle(vec![])) as Box<dyn dynec::Bundle>]);
    world.execute(&tracer::Noop);
}
