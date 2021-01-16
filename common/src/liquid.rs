//! Liquid system

use shrev::EventChannel;
use smallvec::SmallVec;
use specs::{Join, WorldExt};

use crate::terminal::Terminal;
use crate::types::*;
use crate::Setup;

#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::VecStorage)]
pub struct Pipe {
    /// The edge on which this pipe is built
    pub edge: EdgeId,
    /// The liquid type transferred through the pipe
    pub variant: LiquidId,
    /// The volume of liquid currently n the pipe
    pub volume: LiquidVolume,
}

/// A component that stores the information of a liquid type.
///
/// Attached to the entity representing the liquid type.
#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct Liquid {
    /// The texture of the liquid
    pub texture: [f32; 4],
}

/// Indicates that the liquid type demonstrates refrigerant behaviour.
///
/// Attached to the entity representing the liquid type.
#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct Refrigerant {
    /// The specific heat capacity of the refrigerant
    pub capacity: HeatCapacity,
}

/// Lists the liquids produced by a node.
///
/// Applied on a node entity.
#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct SourceList {
    /// The liquids produced by the node
    pub sources: SmallVec<[(LiquidId, Rate<LiquidVolume>); 8]>,
}

/// Lists the liquids consumed by a node.
///
/// Applied on a node entity.
#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct SinkList {
    /// The liquids consumed by the node
    pub sinks: SmallVec<[(LiquidId, Rate<LiquidVolume>); 8]>,
}

fn find_rate(ty: LiquidId, list: &[(LiquidId, Rate<LiquidVolume>)]) -> Rate<LiquidVolume> {
    for &(id, rate) in list {
        if id == ty {
            return rate;
        }
    }
    Rate::default()
}

/// Manages liquid flow logic
pub struct LiquidSystem(());

impl LiquidSystem {
    pub fn new(world: &mut specs::World) -> Self {
        use specs::SystemData;

        <Self as specs::System<'_>>::SystemData::setup(world);
        Self(())
    }
}

impl<'a> System<'a> for LiquidSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Pipe>,
        ReadStorage<'a, Terminal>,
        ReadStorage<'a, SourceList>,
        ReadStorage<'a, SinkList>,
        specs::Write<'a, EventChannel<WithdrawEvent>>,
        specs::Write<'a, EventChannel<SupplyEvent>>,
        specs::Read<'a, Clock>,
    );

    fn run(
        &mut self,
        (mut pipes, terminals, sources, sinks, mut withdraw_events, mut supply_events, clock) : Self::SystemData,
    ) {
        let mut withdraw_vec = Vec::with_capacity(pipes.count());
        let mut supply_vec = Vec::with_capacity(pipes.count());

        for (pipe,) in (&mut pipes,).join() {
            let src = pipe.edge.first.entity();
            let dest = pipe.edge.second.entity();

            let mut force = 0_f32;
            for &ent in &[src, dest] {
                let terminal = terminals.get(ent);
                if let Some(terminal) = terminal {
                    force += terminal.pump_force;
                }
            }

            let delta = compute_pump_amount(force, pipe.volume);

            let provider = match sources.get(src) {
                Some(source_list) => find_rate(pipe.variant, &source_list.sources),
                None => Rate::default(),
            } * clock.delta;
            let consumer = match sinks.get(dest) {
                Some(sink_list) => find_rate(pipe.variant, &sink_list.sinks),
                None => Rate::default(),
            } * clock.delta;

            let pull = if provider > delta { delta } else { provider };
            let mut push = if consumer > delta { delta } else { consumer };

            pipe.volume += pull;
            if push > pipe.volume {
                push = pipe.volume;
            }
            pipe.volume -= push;

            withdraw_vec.push(WithdrawEvent {
                node: pipe.edge.first,
                volume: pull,
            });
            supply_vec.push(SupplyEvent {
                node: pipe.edge.second,
                volume: push,
            });
        }

        withdraw_events.iter_write(withdraw_vec);
        supply_events.iter_write(supply_vec);
    }
}

fn compute_pump_amount(force: f32, volume: LiquidVolume) -> LiquidVolume {
    todo!("I expect some more complex algorithm here")
}

/// An event that indicates the specified amount of liquid is supplied.
pub struct SupplyEvent {
    /// The node receiving the liquid
    pub node: NodeId,
    /// The liquid volume sent to the node
    pub volume: LiquidVolume,
}

/// An event that indicates the specified amount of liquid is withdrawn.
pub struct WithdrawEvent {
    /// The node receiving the liquid
    pub node: NodeId,
    /// The liquid volume taken from the node
    pub volume: LiquidVolume,
}

pub fn setup_specs((mut world, mut dispatcher): Setup) -> Setup {
    world.register::<Liquid>();
    world.register::<Refrigerant>();
    world.register::<Pipe>();
    world.register::<SourceList>();
    world.register::<SinkList>();
    dispatcher = dispatcher.with(LiquidSystem::new(&mut world), "liquid", &[]);
    (world, dispatcher)
}
