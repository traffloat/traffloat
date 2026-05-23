use std::hint::black_box;
use std::time::Instant;

use bevy::app::App;
use bevy::time;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use traffloat_physics::fluid::{self, Moles};

fn base_app(types: u32) -> App {
    let mut app = App::new();
    app.add_plugins((time::TimePlugin, fluid::Plug));
    app.insert_resource(time::TimeUpdateStrategy::FixedTimesteps(1));
    app.insert_resource(fluid::Conf { transfer_timestep: 1 });

    app.insert_resource(fluid::Types {
        types: (0..types)
            .map(|ty| fluid::TypeDef {
                molar_heat_capacity:  2.0,
                advective_fluidity:   0.2,
                diffusive_fluidity:   0.1,
                molar_density:        3.0,
                thermal_conductivity: 1e-4,
                optical_extinction:   [0.0; 3],
            })
            .collect(),
    });
    app
}

fn bench_long_chain(c: &mut Criterion) {
    let mut g = c.benchmark_group("Long Chain");
    for length in [2, 16, 128, 1024, 8192] {
        for types in [2, 8, 32, 128] {
            g.bench_with_input(
                BenchmarkId::new(format!("Length={length}"), format!("Types={types}")),
                &(length, types),
                |b, &(length, types)| {
                    b.iter_custom(|iters| {
                        let mut app = base_app(types);

                        let mut last = app
                            .world_mut()
                            .spawn({
                                let mut storage = fluid::Storage::vacuum(100.0, 1.5)
                                    .with_heat(fluid::Energy(30000.0));
                                for ty in 0..types {
                                    storage
                                        .set_fluid(fluid::TypeId(ty), Moles(100.0 + (ty as f32)));
                                }
                                storage
                            })
                            .id();

                        for _ in 0..length {
                            let next = app
                                .world_mut()
                                .spawn({
                                    let mut storage = fluid::Storage::vacuum(100.0, 1.5)
                                        .with_heat(fluid::Energy(30000.0));
                                    for ty in 0..types {
                                        storage.set_fluid(
                                            fluid::TypeId(ty),
                                            Moles(100.0 + (ty as f32)),
                                        );
                                    }
                                    storage
                                })
                                .id();

                            app.world_mut().spawn((
                                fluid::Edge::new(2.0, 3.0),
                                fluid::EdgeAlpha(last),
                                fluid::EdgeBeta(next),
                            ));

                            last = next;
                        }

                        app.update();
                        app.update();

                        let start = Instant::now();
                        for _ in 0..iters {
                            app.update();
                        }
                        let duration = start.elapsed();
                        drop(black_box(app));
                        duration
                    });
                },
            );
        }
    }
}

criterion_group!(benches, bench_long_chain,);
criterion_main!(benches);
