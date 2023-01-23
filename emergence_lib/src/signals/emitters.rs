//! Data for entities which can emit a signal.

use crate::player_interaction::abilities::IntentAbility;
/// Varieties of signals
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Emitter {
    /// An ant
    Ant,
    /// A fungus
    Fungus,
    /// A plant
    Plant,
    /// Ability-driven effects.
    Ability(IntentAbility),
}
