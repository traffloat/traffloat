use std::collections::{BTreeMap, VecDeque};
use std::sync::{Mutex, RwLock};

pub use traffloat_codegen_raw::*;

/// The standard setup parameters
#[derive(Default)]
pub struct SetupEcs {
    /// Whether to enable server-only systems
    pub server: bool,
    /// The legion::Scheduler builder
    pub builder: legion::systems::Builder,
    /// The legion world storing entities and components
    pub world: legion::World,
    /// The resource set storing legion resources
    pub resources: legion::Resources,
}

impl SetupEcs {
    /// Register a bundle
    pub fn uses(self, setup_ecs: impl FnOnce(Self) -> Self) -> Self {
        setup_ecs(self)
    }

    /// Add a system
    pub fn system(mut self, sys: impl legion::systems::ParallelRunnable + 'static) -> Self {
        self.builder.add_system(sys);
        self
    }
    /// Add a thread-local system
    pub fn system_local(mut self, sys: impl legion::systems::Runnable + 'static) -> Self {
        self.builder.add_thread_local(sys);
        self
    }

    /// Add an entity
    pub fn entity<T>(mut self, components: T) -> Self
    where
        Option<T>: legion::storage::IntoComponentSource,
    {
        let _ = self.world.push(components);
        self
    }
    /// Add entities
    pub fn entities<T>(mut self, components: impl legion::storage::IntoComponentSource) -> Self {
        let _ = self.world.extend(components);
        self
    }

    /// Add a resource
    pub fn resource(mut self, res: impl legion::systems::Resource) -> Self {
        let _ = self.resources.get_or_insert(res);
        self
    }
    /// Declare a published event
    pub fn publish<T: shrev::Event>(mut self) -> Self {
        let _ = self
            .resources
            .get_or_insert_with(shrev::EventChannel::<T>::new);
        self
    }
    /// Declare a subscribed event
    pub fn subscribe<T: shrev::Event>(&mut self) -> shrev::ReaderId<T> {
        let mut channel = self
            .resources
            .get_mut_or_insert_with(shrev::EventChannel::<T>::new);
        channel.register_reader()
    }

    /// Build the setup into a legion
    pub fn build(mut self) -> Legion {
        Legion {
            world: self.world,
            resources: self.resources,
            schedule: self.builder.build(),
        }
    }
}

/// The set of values required to run legion
pub struct Legion {
    /// The legion world storing entities and components
    pub world: legion::World,
    /// The resource set storing legion resources
    pub resources: legion::Resources,
    /// The legion scheduler running systems
    pub schedule: legion::Schedule,
}

impl Legion {
    /// Spins all systems once.
    pub fn run(&mut self) {
        self.schedule.execute(&mut self.world, &mut self.resources)
    }

    pub fn publish<T: shrev::Event>(&mut self, event: T) {
        let mut channel = match self.resources.get_mut::<shrev::EventChannel<T>>() {
            Some(channel) => channel,
            None => panic!(
                "EventChannel<{}> has not been initialized",
                std::any::type_name::<T>()
            ),
        };
        channel.single_write(event);
    }
}

/// Performance tracking
#[derive(Default)]
pub struct Perf {
    pub map: PerfMap,
}

pub type PerfMap = RwLock<BTreeMap<&'static str, Mutex<VecDeque<i64>>>>;

const MAX_FRAMES: usize = 100;

impl Perf {
    /// Update a timer
    pub fn push(&self, name: &'static str, value: i64) {
        loop {
            {
                let map = self.map.read().expect("Perf poisoned");
                if let Some(deque) = map.get(name) {
                    let mut deque = deque.lock().expect("Perf poisoned");
                    while deque.len() >= MAX_FRAMES {
                        deque.pop_front();
                    }
                    deque.push_back(value);
                    return;
                }
            }

            {
                let mut map = self.map.write().expect("Perf poisoned");
                let _ = map.entry(name).or_default();
            }
        }
    }
}

/// The high-resolution clock in microseconds
#[cfg(target_arch = "wasm32")]
pub fn hrtime() -> i64 {
    (web_sys::window()
        .expect("Window uninitialized")
        .performance()
        .expect("window.performance uninitialized")
        .now()
        * 1000.) as i64
}

/// The high-resolution clock in microseconds
#[cfg(not(target_arch = "wasm32"))]
pub fn hrtime() -> i64 {
    use std::time::Instant;

    lazy_static::lazy_static! {
        static ref EPOCH: Instant = Instant::now();
    }

    EPOCH.elapsed().as_micros() as i64
}
