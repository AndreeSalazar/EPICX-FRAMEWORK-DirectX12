//! React-style hooks for EPICX
//!
//! Provides familiar React hooks for state management and side effects.

use parking_lot::RwLock;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

thread_local! {
    static HOOK_STATE: RefCell<HookState> = RefCell::new(HookState::new());
}

/// Internal hook state
struct HookState {
    current_component: Option<String>,
    hook_index: usize,
    states: HashMap<String, Vec<Arc<dyn Any + Send + Sync>>>,
    effects: HashMap<String, Vec<EffectState>>,
}

impl HookState {
    fn new() -> Self {
        Self {
            current_component: None,
            hook_index: 0,
            states: HashMap::new(),
            effects: HashMap::new(),
        }
    }
}

struct EffectState {
    cleanup: Option<Box<dyn FnOnce() + Send>>,
    deps: Option<Vec<u64>>,
}

/// State hook - similar to React's useState
#[derive(Clone)]
pub struct UseState<T> {
    value: Arc<RwLock<T>>,
    setter: Arc<dyn Fn(T) + Send + Sync>,
}

impl<T: Clone> UseState<T> {
    /// Get the current value
    pub fn get(&self) -> T {
        self.value.read().clone()
    }

    /// Set a new value
    pub fn set(&self, value: T) {
        (self.setter)(value);
    }

    /// Update the value with a function
    pub fn update<F: FnOnce(&T) -> T>(&self, f: F) {
        let new_value = f(&self.value.read());
        self.set(new_value);
    }
}

/// Create a state hook
pub fn use_state<T: Clone + Send + Sync + 'static>(initial: T) -> UseState<T> {
    let value = Arc::new(RwLock::new(initial));
    let value_clone = Arc::clone(&value);
    
    let setter: Arc<dyn Fn(T) + Send + Sync> = Arc::new(move |new_value: T| {
        *value_clone.write() = new_value;
    });

    UseState { value, setter }
}

/// Effect hook - similar to React's useEffect
pub fn use_effect<F, D>(effect: F, deps: D)
where
    F: FnOnce() + Send + 'static,
    D: Into<Option<Vec<u64>>>,
{
    let deps = deps.into();
    
    // In a full implementation, this would:
    // 1. Compare deps with previous deps
    // 2. Run cleanup from previous effect if deps changed
    // 3. Schedule effect to run after render
    
    // For now, just run the effect
    effect();
}

/// Memo hook - similar to React's useMemo
pub fn use_memo<T, F, D>(compute: F, deps: D) -> T
where
    T: Clone + Send + Sync + 'static,
    F: FnOnce() -> T,
    D: Into<Option<Vec<u64>>>,
{
    // In a full implementation, this would cache the computed value
    // and only recompute when deps change
    compute()
}

/// Callback hook - similar to React's useCallback
pub fn use_callback<F, D>(callback: F, deps: D) -> Arc<F>
where
    F: Send + Sync + 'static,
    D: Into<Option<Vec<u64>>>,
{
    // In a full implementation, this would memoize the callback
    Arc::new(callback)
}

/// Ref hook - similar to React's useRef
pub struct UseRef<T> {
    current: Arc<RwLock<T>>,
}

impl<T> UseRef<T> {
    /// Get a reference to the current value
    pub fn current(&self) -> parking_lot::RwLockReadGuard<T> {
        self.current.read()
    }

    /// Get a mutable reference to the current value
    pub fn current_mut(&self) -> parking_lot::RwLockWriteGuard<T> {
        self.current.write()
    }

    /// Set the current value
    pub fn set(&self, value: T) {
        *self.current.write() = value;
    }
}

impl<T: Clone> UseRef<T> {
    /// Get a clone of the current value
    pub fn get(&self) -> T {
        self.current.read().clone()
    }
}

/// Create a ref hook
pub fn use_ref<T: Send + Sync + 'static>(initial: T) -> UseRef<T> {
    UseRef {
        current: Arc::new(RwLock::new(initial)),
    }
}

/// Reducer hook - similar to React's useReducer
pub struct UseReducer<S, A> {
    state: Arc<RwLock<S>>,
    dispatch: Arc<dyn Fn(A) + Send + Sync>,
}

impl<S: Clone, A> UseReducer<S, A> {
    /// Get the current state
    pub fn state(&self) -> S {
        self.state.read().clone()
    }

    /// Dispatch an action
    pub fn dispatch(&self, action: A) {
        (self.dispatch)(action);
    }
}

/// Create a reducer hook
pub fn use_reducer<S, A, R>(reducer: R, initial_state: S) -> UseReducer<S, A>
where
    S: Clone + Send + Sync + 'static,
    A: Send + 'static,
    R: Fn(&S, A) -> S + Send + Sync + 'static,
{
    let state = Arc::new(RwLock::new(initial_state));
    let state_clone = Arc::clone(&state);
    let reducer = Arc::new(reducer);

    let dispatch: Arc<dyn Fn(A) + Send + Sync> = Arc::new(move |action: A| {
        let current = state_clone.read().clone();
        let new_state = reducer(&current, action);
        *state_clone.write() = new_state;
    });

    UseReducer { state, dispatch }
}

/// Context hook - similar to React's useContext
pub fn use_context<T: Clone + Send + Sync + 'static>(
    context: &crate::core::Context,
) -> Option<Arc<T>> {
    context.get::<T>()
}
