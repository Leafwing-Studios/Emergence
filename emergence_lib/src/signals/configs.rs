use crate::signals::emitters::Emitter;
use indexmap::IndexMap;

/// A dictionary of available [`SignalConfig`]s.
///
/// Internally, this uses an [`IndexMap`], so that there is also a notion of order: the order
/// in which elements are inserted into the dictionary. Some notion of order is necessary in order
/// to color tiles consistently.
#[derive(Default, Clone, Debug)]
pub struct SignalConfigs {
    configs: IndexMap<Emitter, SignalConfig>,
}

impl SignalConfigs {
    /// Get the signal configuration for the specified [`Emitter`], if present.
    pub fn get(&self, emitter: &Emitter) -> Option<&SignalConfig> {
        self.configs.get(emitter)
    }

    /// Insert a [`SignalConfig`] into the dictionary.
    ///
    /// If one is already associated with the specified [`Emitter`], this function follows
    /// [`HashMap`](HashMap::insert) semantics by replacing the pre-existing configuration with the
    /// specified configuration, and then returning the pre-existing configuration.
    pub fn insert(&mut self, emitter: Emitter, config: SignalConfig) -> Option<SignalConfig> {
        self.configs.insert(emitter, config)
    }

    /// Iterate over the signals at this tile, in the order they were inserted.
    pub fn iter(&self) -> impl Iterator<Item = (&Emitter, &SignalConfig)> {
        self.configs.iter()
    }
}

/// Configuration settings for a particular [`Signal`].
#[derive(Default, Clone, Copy, Debug)]
pub struct SignalConfig {
    /// The factor with which a unit of signal diffuses to a neighboring tile per tick.
    ///
    /// Note that this is not a probability, as it is un-normalized.
    pub diffusion_factor: f32,
    /// The probability with which a signal naturally decays per tick.
    pub decay_probability: f32,
    /// Color settings.
    pub color_config: SignalColorConfig,
}

/// Color configuration for a [`Signal`].
///
/// The final coloration for a [`Signal`] at a particular tile position will depend upon the
/// `rgb_color` specified in the color configuration, and the `value` of the [`Signal`], which will
/// affect the computed color's `alpha`.
///
/// If `value` is below `zero_value`, then the computed color will have alpha `0.0`, otherwise if
/// the value is above `one_value`, then the computed color will have alpha `1.0`. If `value` is
/// between `zero_value` and `one_value`, then `alpha` will be mapped to some point between these
/// two.
#[derive(Default, Clone, Copy, Debug)]
pub struct SignalColorConfig {
    /// The three primary colour values (rgb) defining the colour used.
    pub rgb_color: [f32; 3],
    /// The signal value below (inclusive) which an alpha of 0.0 will be returned.
    pub zero_value: f32,
    /// The signal value above (inclusive) which an alpha of 1.0 will be returned.
    pub one_value: f32,
    /// Should this signal be visible on the map?
    pub is_visible: bool,
}
