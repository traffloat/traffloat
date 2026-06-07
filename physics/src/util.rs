use std::marker::PhantomData;

use bevy::app::App;
use bevy::ecs::schedule::{IntoScheduleConfigs, ScheduleLabel, SystemSet};
use derivative::Derivative;
use itertools::Itertools;

mod ab;
pub use ab::{Alpha, AlphaBeta, Beta, GetAb, Which};

#[macro_use]
mod try_log;
pub use try_log::{EntityRefExt, EntityWorldMutExt, QueryExt, SliceGet, TryLog, WorldExt};

mod merge_sort;
pub use merge_sort::{MergeSortedItem, merge_sorted};

mod throttle;
pub use throttle::Throttle;

pub fn configure_enum_system_set<T>(app: &mut App, schedule: impl ScheduleLabel + Clone)
where
    T: 'static + Send + Sync + SystemSet + strum::IntoEnumIterator + Copy,
{
    for (prev, next) in <T as strum::IntoEnumIterator>::iter().tuple_windows() {
        app.configure_sets(schedule.clone(), prev.before(next));
    }

    for set in <T as strum::IntoEnumIterator>::iter() {
        app.configure_sets(schedule.clone(), set.in_set(AllSystemSets::<T>::default()));
    }
}

#[derive(SystemSet, Derivative)]
#[derivative(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct AllSystemSets<T>(PhantomData<T>);
