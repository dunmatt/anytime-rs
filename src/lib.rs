//! Anytime results are useful for long running computations where additional computation time can
//! improve the quality of the result and the consumer(s) don't know in advance when they will need
//! the results.
//!
//! Anytime results are by their nature `Sync`, and so it is possible to share one between many
//! consumers.  If you do share an Anytime all consumers are guaranteed to get the same result(s).
//!
//! TODO: add an example usage here!!!

#![deny(
    dead_code,
    missing_docs,
    unused_imports,
    unused_must_use,
    unused_parens,
    unused_qualifications
)]
#![forbid(unsafe_code)]

use std::sync::{atomic::{AtomicBool, Ordering}, Mutex};

use log::{debug, error};

/// A result that could improve until a consumer looks at it, after which it will never change.
pub struct Anytime<T: Clone> {
    current_best: Mutex<Option<T>>,
    value_locked: AtomicBool,
}

impl<T: Clone> Anytime<T> {
    /// Creates an empty, unlocked Anytime.
    pub fn new() -> Anytime<T> {
        Anytime {
            current_best: Mutex::new(None),
            value_locked: AtomicBool::new(false),
        }
    }

    /// Returns true iff a consumer somewhere has read this result (thereby freezing it).
    pub fn is_final(&self) -> bool {
        self.value_locked.load(Ordering::Relaxed)
    }

    /// Returns true if a preliminary result has been found, or if the search has been called off.
    pub fn is_ready(&self) -> bool {
        self.is_final() || self.current_best.lock().unwrap().is_some()
    }

    /// Commits to and returns the best option currently available.  After calling this, calling
    /// update_result is a no-op.
    pub fn get_result(&self) -> Option<T> {
        if let Ok(guard) = self.current_best.lock() {
            self.value_locked.store(true, Ordering::Relaxed);
            guard.clone()
        } else {
            error!("Attempted to lock a poisoned mutex!  This anytime result cannot recover.");
            None
        }
    }

    /// Stores an updated result in this anytime, if possible.
    pub fn update_result(&self, better_result: T) {
        if let Ok(mut guard) = self.current_best.lock() {
            if !self.value_locked.load(Ordering::Relaxed) {
                guard.replace(better_result);
            } else {
                debug!("Attempted to overwrite a locked value.");
            }
        } else {
            error!("Attempted to lock a poisoned mutex!  This anytime result cannot recover.");
        }
    }
}
