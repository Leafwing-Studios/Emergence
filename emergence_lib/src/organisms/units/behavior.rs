//! What are units doing, and why?
//!
//! The AI model of Emergence.

use bevy::prelude::Component;

/// Enumerates possible tasks a unit may engage in.
///
/// Units will be fully concentrated on any task other than [`CurrentTask::Wander`] until it is complete (or overridden).
///
/// This component serves as a state machine.
#[derive(Component, PartialEq, Eq, Clone)]
pub enum CurrentTask {
    /// Attempting to find something useful to do
    ///
    /// Units will try and follow a signal, if they can pick up a trail, but will not fixate on it until the signal is strong enough.
    Wander,
    /// Attempting to pick up an object
    Pickup,
    /// Attempting to drop off an object
    DropOff,
    /// Attempting to perform work at a structure
    Work,
}
