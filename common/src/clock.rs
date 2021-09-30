//! Game clock management

use std::convert::TryFrom;

use safety::Safety;

use crate::time::{Instant, Time};
use crate::SetupEcs;

/// The interval between simulation frames.
pub const SIMULATION_PERIOD: Time = Time(100);

/// Number of microseconds per discrete unit time.
pub const MICROS_PER_TICK: u64 = 10000;

/// A resource for time read/write.
#[derive(Debug, Default, getset::CopyGetters)]
pub struct Clock {
    /// The current time
    #[getset(get_copy = "pub")]
    now:           Instant, // TODO multiplayer calibration
    /// Time since the last frame
    #[getset(get_copy = "pub")]
    delta:         Time,
    /// The reference time value at epoch.
    epoch_instant: Instant,
    /// The epoch in microseconds used for calibration.
    epoch_micros:  i64,
}

impl Clock {
    /// Sets the time to the specified instant.
    pub fn update_micros(&mut self, micros: i64) {
        let delta_micros = (micros - self.epoch_micros).homosign();
        let delta_time =
            Time(u32::try_from(delta_micros / MICROS_PER_TICK).expect("micros is not monotonic"));
        let now = self.epoch_instant + delta_time;
        self.delta = now - self.now;
        self.now = now;
    }

    /// Reset the time without involving any simulation offset.
    pub fn reset_time(&mut self, now: Instant, micros: i64) {
        self.now = now;
        self.delta = Time::zero();
        self.epoch_instant = now;
        self.epoch_micros = micros;
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
/// #[codegen::system(Simulate)]
/// fn execute(#[subscriber] simul_sub: impl Iterator<Item = SimulationEvent>) {
///     if simul_sub.next().is_none() {
///         return;
///     }
/// }
/// ```
pub struct SimulationEvent;

#[codegen::system(Schedule)]
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
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs { setup.uses(sim_trigger_setup) }
