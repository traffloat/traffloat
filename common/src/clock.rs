//! Game clock management

use crate::time::{Instant, Time};
use crate::SetupEcs;

/// The interval between simulation frames.
pub const SIMULATION_PERIOD: Time = Time(100);

/// A resource for time read/write.
#[derive(Debug, Default, getset::CopyGetters)]
pub struct Clock {
    /// The current time
    #[getset(get_copy = "pub")]
    now: Instant, // TODO multiplayer calibration
    /// Time since the last frame
    #[getset(get_copy = "pub")]
    delta: Time,
}

impl Clock {
    /// Increases the time for the specified span.
    pub fn inc_time(&mut self, time: Time) {
        self.now += time;
        self.delta = time;
    }

    /// Sets the time to the specified instant.
    pub fn set_time(&mut self, now: Instant) {
        self.delta = now - self.now;
        self.now = now;
    }
}

/// Subscribe to this event to execute updates.
///
/// Subscribers should only handle the event once every time,
/// i.e. with the following code:
///
/// ```no_run
/// # use traffloat::clock::SimulationEvent;
/// #
/// #[codegen::system]
/// fn execute(
///     #[subscriber] simul_sub: impl Iterator<Item = SimulationEvent>,
/// ) {
///     if simul_sub.next().is_none() {
///         return;
///     }
/// }
/// ```
pub struct SimulationEvent;

#[codegen::system]
fn sim_trigger(
    #[publisher] sim_pub: impl FnMut(SimulationEvent),
    #[resource] clock: &Clock,
    #[state(Instant::default())] last_sim_time: &mut Instant,
) {
    let now = clock.now().since_epoch().int_div(SIMULATION_PERIOD);
    let last = last_sim_time.since_epoch().int_div(SIMULATION_PERIOD);
    if now != last {
        sim_pub(SimulationEvent);
        *last_sim_time = clock.now();
    }
}

/// Initializes the time module.
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(sim_trigger_setup)
}
