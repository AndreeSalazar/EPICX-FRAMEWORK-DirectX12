//! State management for EPICX components

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::fmt::Debug;
use std::sync::Arc;

/// Trait for component state
///
/// State is mutable data owned by a component that triggers re-renders when changed.
pub trait State: Clone + Debug + Default + Send + Sync + 'static {}

/// Default implementation for unit type (no state)
impl State for () {}

/// Implement State for common primitive types
impl State for i32 {}
impl State for i64 {}
impl State for u32 {}
impl State for u64 {}
impl State for f32 {}
impl State for f64 {}
impl State for bool {}
impl State for String {}

/// A reactive state container
pub struct ReactiveState<T: State> {
    value: Arc<RwLock<T>>,
    version: Arc<RwLock<u64>>,
    subscribers: Arc<RwLock<Vec<Box<dyn Fn(&T) + Send + Sync>>>>,
}

impl<T: State> Debug for ReactiveState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactiveState")
            .field("value", &*self.value.read())
            .field("version", &*self.version.read())
            .field("subscribers_count", &self.subscribers.read().len())
            .finish()
    }
}

impl<T: State> ReactiveState<T> {
    /// Create a new reactive state with initial value
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(initial)),
            version: Arc::new(RwLock::new(0)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get a read guard to the state
    pub fn read(&self) -> RwLockReadGuard<T> {
        self.value.read()
    }

    /// Get a write guard to the state
    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.value.write()
    }

    /// Get the current version (increments on each update)
    pub fn version(&self) -> u64 {
        *self.version.read()
    }

    /// Update the state with a function
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        {
            let mut value = self.value.write();
            updater(&mut value);
            *self.version.write() += 1;
        }
        self.notify_subscribers();
    }

    /// Set a new value
    pub fn set(&self, new_value: T) {
        {
            *self.value.write() = new_value;
            *self.version.write() += 1;
        }
        self.notify_subscribers();
    }

    /// Subscribe to state changes
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.subscribers.write().push(Box::new(callback));
    }

    /// Notify all subscribers of a state change
    fn notify_subscribers(&self) {
        let value = self.value.read();
        let subscribers = self.subscribers.read();
        for subscriber in subscribers.iter() {
            subscriber(&value);
        }
    }
}

impl<T: State> Clone for ReactiveState<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            version: Arc::clone(&self.version),
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}

impl<T: State> Default for ReactiveState<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A simple state atom (like Recoil atoms)
pub struct Atom<T: State> {
    key: String,
    state: ReactiveState<T>,
}

impl<T: State> Atom<T> {
    /// Create a new atom with a key and default value
    pub fn new(key: impl Into<String>, default: T) -> Self {
        Self {
            key: key.into(),
            state: ReactiveState::new(default),
        }
    }

    /// Get the atom's key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the current value
    pub fn get(&self) -> RwLockReadGuard<T> {
        self.state.read()
    }

    /// Set a new value
    pub fn set(&self, value: T) {
        self.state.set(value);
    }

    /// Update with a function
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        self.state.update(updater);
    }
}

impl<T: State> Clone for Atom<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            state: self.state.clone(),
        }
    }
}
