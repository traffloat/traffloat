use std::cmp;

use bevy::ecs::resource::Resource;
use bevy::ecs::world::{CommandQueue, World};
use jiff::Timestamp;

#[derive(Resource)]
pub struct Storage(pub Box<dyn Backend>);

pub trait Backend: Send + Sync + 'static {
    /// Flushes scheduled tasks to run on `&mut World`.
    fn flush(&mut self) -> CommandQueue;

    /// Reloads list of saves from the backend.
    fn reload(&mut self);

    /// Checks if a name is used. Used for duplicate detection. Does not need to be consistent.
    fn is_name_used(&mut self, name: &str) -> bool;

    fn submit_save(&mut self, name: &str, data: &[u8]);

    fn list_entries(&mut self) -> Vec<Entry>;

    fn load_file(&mut self, name: &str, callback: Box<dyn FnOnce(&mut World, &[u8]) + Send>);
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name:     String,
    pub modified: Timestamp,
    pub size:     u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    FirstModified,
    LastModified,
    Smallest,
    Largest,
}

impl SortBy {
    pub fn sort_fn(self) -> impl Fn(&Entry, &Entry) -> cmp::Ordering {
        move |a, b| match self {
            SortBy::Name => a.name.cmp(&b.name),
            SortBy::FirstModified => a.modified.cmp(&b.modified),
            SortBy::LastModified => a.modified.cmp(&b.modified).reverse(),
            SortBy::Smallest => a.size.cmp(&b.size),
            SortBy::Largest => a.size.cmp(&b.size).reverse(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod fs;
