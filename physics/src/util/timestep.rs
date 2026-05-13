use std::time::Duration;

use bevy::ecs::system::{Local, Res, SystemParam};
use bevy::time::{self, Time};

#[derive(SystemParam)]
pub struct TimeStep<'w, 's, Clock: Default + Send + Sync + 'static> {
    virt_time: Res<'w, Time<time::Virtual>>,
    real_time: Res<'w, Time<Clock>>,
    last_step: Local<'s, Option<LastTimeStep>>,
}

impl<Clock: Default + Send + Sync + 'static> TimeStep<'_, '_, Clock> {
    pub fn should_run(&mut self, timestep: Duration) -> Option<f32> {
        if self.virt_time.is_paused() {
            return None;
        }
        let dt = if let Some(last_step_time) = &mut *self.last_step {
            last_step_time.virt_accum += self.virt_time.delta();
            let since_last_real = self.real_time.elapsed().checked_sub(last_step_time.last_real);
            if since_last_real.is_some_and(|t| t < timestep) {
                return None;
            }
            last_step_time.virt_accum
        } else {
            Duration::ZERO
        };
        *self.last_step =
            Some(LastTimeStep { virt_accum: Duration::ZERO, last_real: self.real_time.elapsed() });
        Some(dt.as_secs_f32())
    }
}

struct LastTimeStep {
    virt_accum: Duration,
    last_real:  Duration,
}
