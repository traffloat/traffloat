use std::time::Duration;

use bevy::ecs::system::{Local, Res, SystemParam};
use bevy::time::Time;

#[derive(SystemParam)]
pub struct Throttle<'w, 's, T: Default + Send + Sync + 'static> {
    time: Res<'w, Time<T>>,
    last: Local<'s, Option<Duration>>,
}

impl<T: Default + Send + Sync + 'static> Throttle<'_, '_, T> {
    pub fn should_run(&mut self, interval: Duration) -> bool {
        let should_run = match *self.last {
            None => true,
            Some(last) => self.time.elapsed().checked_sub(last).unwrap_or_default() >= interval,
        };
        if should_run {
            *self.last = Some(self.time.elapsed());
            true
        } else {
            false
        }
    }
}
