//! Life stages are connected by life paths, creating a life cycle for each strain of organism.

use bevy::{
    prelude::Resource,
    utils::{Duration, HashMap},
};

use crate::items::recipe::Recipe;

use super::Species;

/// A map of the [`LifeStages`](Species::LifeStage) for an organism of type `S`, connected by [`LifePath`]s.
///
/// This type is the high-level map for the entire [`Species`], and is stored as a resource.
///
/// This forms a finite state machine.
/// Paths may diverge, loop back onto themselves, terminate and so on.
#[derive(Resource)]
pub struct LifeCycle<S: Species> {
    /// Describes how a life stage can transition to other life stages.
    ///
    /// This is a map of the outgoing paths.
    pub life_paths: HashMap<S::LifeStage, LifePath<S>>,
}

impl<S: Species> Default for LifeCycle<S> {
    fn default() -> Self {
        S::life_cycle()
    }
}

/// Paths that connect different life stages.
///
/// These are triggered when certain conditions are met for each organism,
/// causing them to change form and function.
pub struct LifePath<S: Species> {
    /// The life stage that the organism is transitioning to.
    pub target: S::LifeStage,
    /// The conditions that must be met for the transition to occur.
    pub requirements: TransitionType,
}

/// The condition that must be met for an organism to transition along a life path.
pub enum TransitionType {
    /// The organism's fundamnental metabolic [`Recipe`] must be completed a certain number of times.
    ///
    /// Generally used for organism growth or death by old age.
    Metabolic {
        /// The number of times the recipe must be completed.
        required_count: u16,
    },
    /// The organism must go a certain period of time without metabolizing.
    ///
    /// Typically used for death and dormancy.
    Starvation {
        /// The period of time without metabolizing until the organism is weakened or wilting.
        weakened_threshold: Duration,
        /// The period of time without metabolizing until the organism transitions.
        ///
        /// This transition is usually, but not always, to death.
        transition_threshold: Duration,
    },
    /// A secondary recipe is completed.
    ///
    /// This is generally used to branch off the common path, such as the use of royal jelly to make queen bees.
    AlternateRecipe {
        /// The recipe that must be completed in order for the transition to occur.
        alt_recipe: Recipe,
    },
    /// The average temperature over the past three days must be in the provided range.
    Temperature {
        /// The minimum temperature required to qualify
        min: Option<Temperature>,
        /// The maximum temperature required to qualify
        max: Option<Temperature>,
    },
    /// The average ratio of light to dark over the past three days must be in the provided range.
    Light {
        /// The minimum light level to qualify
        min: Option<LightRatio>,
        /// The maximum light level required to qualify
        max: Option<LightRatio>,
    },
}

/// Ambient or item temperature
pub struct Temperature {
    /// In degrees Celsius
    pub degrees: i16,
}

/// The ratio of light to dark over the past 72 in-game hours.
///
/// Values above 1 correspond to summer conditions, with more light than dark.
/// Values below 1 correspond to summer conditions, with more light than dark.
pub struct LightRatio {
    /// Light : Dark
    pub ratio: f32,
}
