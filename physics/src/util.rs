use std::marker::PhantomData;

use bevy::app::App;
use bevy::ecs::schedule::{IntoScheduleConfigs, ScheduleLabel, SystemSet};
use bevy::ecs::system::{SystemParam, SystemParamFunction, SystemState};
use bevy::ecs::world::World;
use derivative::Derivative;
use itertools::Itertools;

mod ab;
pub use ab::{Alpha, AlphaBeta, Beta, Which};

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

pub fn run_stateless_closure<F, R, Marker>(world: &mut World, mut f: F) -> R
where
    F: SystemParamFunction<Marker, In = (), Out = R>,
    F::Param: 'static,
{
    run_stateless_closure_explicit::<F::Param, _, R>(world, |param| f.run((), param))
}

/// Similar to [`run_stateless_closure`], with less type inference.
///
/// Allows non-`'static` closures with exactly one parameter,
/// which may not satisfy `for<'w, 's> <P<'w, 's> as SystemParam>::Item<'w, 's> = P`.
/// Instead, the `SystemParam` type must be explicitly specified as a type parameter to
/// `run_stateless_closure_explicit`, but the type may be elided in the closure signature itself.
pub fn run_stateless_closure_explicit<P, F, R>(world: &mut World, f: F) -> R
where
    P: SystemParam + 'static,
    F: for<'w, 's> FnOnce(P::Item<'w, 's>) -> R,
{
    let mut state = SystemState::<P>::new(world);
    let param = match state.get_mut(world) {
        Ok(param) => param,
        Err(err) => {
            panic!("Failed to prepare system parameter {} in world: {err}", std::any::type_name::<P>())
        }
    };
    let result = f(param);
    state.apply(world);
    result
}
