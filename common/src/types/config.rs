//! A configuration is the special rules defined by the game host in a world.
//!
//! For example, each texture is a configuration, and each liquid type is a configuration.
//!
//! Configurations are stored as resources in the Legion.
//! They are referenced using IDs.

use std::cmp;
use std::convert::TryFrom;
use std::marker::PhantomData;

/// A marker trait for configuration types
pub trait Config: std::any::Any + 'static + Send + Sync + Sized {}

/// The ID of a mostly-fixed set of metadata
#[derive(Debug)]
pub struct Id<T: Config> {
    value: u32,
    _ph: PhantomData<&'static T>, // we don't own the configuration
}

impl<T: Config> Id<T> {
    /// Gets the configuration represented by this ID
    pub fn get(self, store: &ConfigStore<T>) -> &T {
        store.get(self)
    }

    /// Creates an ID, checking whether it is actually in the store
    pub fn new(value: u32, store: &ConfigStore<T>) -> Option<Self> {
        if !store.exists(value) {
            return None;
        }
        Some(Self::new_unchecked(value))
    }

    /// Creates an ID without checking its existence
    pub fn new_unchecked(value: u32) -> Self {
        Self {
            value,
            _ph: PhantomData::default(),
        }
    }
}

impl<T: Config> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self::new_unchecked(self.value)
    }
}

impl<T: Config> Copy for Id<T> {}

impl<T: Config> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Config> Eq for Id<T> {}

impl<T: Config> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Config> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

/// A storage for a configuration type
#[derive(Debug)]
pub struct ConfigStore<T: Config> {
    values: Vec<Option<T>>,
}

impl<T: Config> Default for ConfigStore<T> {
    fn default() -> Self {
        Self { values: Vec::new() }
    }
}

impl<T: Config> ConfigStore<T> {
    /// Check shwether a configuration ID is valid
    pub fn exists(&self, id: u32) -> bool {
        (id as usize) < self.values.len()
    }

    /// Retrieves a configuration by ID
    pub fn get(&self, id: Id<T>) -> &T {
        self.values
            .get(id.value as usize)
            .map(Option::as_ref)
            .flatten()
            .expect("Use of uninitialized ID")
    }

    /// Adds a new configuration to the store.
    pub fn add(&mut self, value: T) -> Id<T> {
        let id = u32::try_from(self.values.len()).expect("Too many items stored in config");
        self.values.push(Some(value));
        Id::new_unchecked(id)
    }

    /// Adds a configuration by ID, or override the existing value.
    pub fn insert(&mut self, id: Id<T>, value: T) {
        if self.values.len() <= id.value as usize {
            self.values.resize_with(id.value as usize + 1, || None);
        }
        *self
            .values
            .get_mut(id.value as usize)
            .expect("just resized") = Some(value);
    }
}
