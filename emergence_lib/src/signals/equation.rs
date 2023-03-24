use std::{collections::VecDeque, f32::consts};

use bevy::prelude::trace;

use crate::simulation::geometry::TilePos;

use super::{SignalStrength, DECAY_RATE, DECAY_THRESHOLD, DIFFUSIVITY};

#[derive(Clone, Copy, Debug)]
pub struct SignalEmission {
    /// Time at which the signal was emitted.
    pub time: f32,
    /// Strength of the emitted signal.
    pub strength: f32,
    /// Position of the signal source.
    pub source: TilePos,
}

/// A solver for the diffusion equation, optimized towards emergence's use case.
///
/// The characteristics of emergence relevant to the problem can be summarized as the following:
/// - Few occasional sources emitting "packets" of signal
// NOTE: One equation per signal type implies they don't interact together.
#[derive(Default, Debug)]
pub struct DiffusionEquation {
    /// A list of signal sources
    emissions: VecDeque<SignalEmission>,
    /// The time at which the equation is evaluated.
    time: f32,
}

impl DiffusionEquation {
    /// Integrate contributions to the signal field to compute its value.
    #[inline]
    pub fn evaluate_signal(&self, at_tile: TilePos) -> SignalStrength {
        let mut sum = 0.0;
        for SignalEmission {
            time,
            strength,
            source,
        } in &self.emissions
        {
            let time = self.time - time;
            debug_assert!(time > 0.0);
            // Using hex's distance makes distances a little bit "squished" in directions of the
            // polygons' vertices, and a contour-line plot of signal strength would show hexagons
            // instead of circles.
            // TODO: Should we use another distance function ? e.g. one that computes a distance
            //       equivalent to world distance.
            let x = source.distance_to(at_tile.hex) as f32;
            sum += strength / (4.0 * (1.0 + DECAY_RATE * time) * consts::PI * DIFFUSIVITY * time)
                * (-0.25 * x * x / (DIFFUSIVITY * time)).exp();
            // Same expression, with precision loss optimized by Herbie (though it should be negligible):
            // sum += 0.25 * consts::FRAC_1_PI * (strength / DIFFUSIVITY)
            //     * ((x / DIFFUSIVITY * (-0.25 * x / time)).exp() / time);
        }
        SignalStrength(sum)
    }

    /// Start emitting a signal of given `strength` at the `source_tile` position.
    #[inline]
    pub fn emit_signal(&mut self, source: TilePos, SignalStrength(strength): SignalStrength) {
        self.emissions.push_back(SignalEmission {
            time: self.time,
            strength,
            source,
        });
    }

    /// Sets the time at which the equation will be evaluated to the given `current_time`. This
    // doesn't recompute anything and is very cheap.
    #[inline]
    pub fn advance_time(&mut self, current_time: f32) {
        debug_assert!(self.time <= current_time);
        self.time = current_time;
    }

    /// Get rid of signal emissions that do contribute to negligible amounts to the global
    /// signal.
    ///
    /// If `true` is returned, any further evaluation of this equation will be
    /// [`SignalStrength::ZERO`][super::SignalStrength::ZERO] unless
    /// [`emit_signal`][DiffusionEquation::emit_signal] is called.
    pub fn trim_neglible_emissions(&mut self) -> bool {
        trace!("Attempting trimming among {} sources", self.emissions.len());
        let mut visited = 0;
        while visited < self.emissions.len() {
            match self.emissions.pop_back() {
                // If the emission has been there long enough, decay will eventually take over.
                // The comparison here checks whether the area under the curve of the solution
                // (which could be seen as the amount of pheromones) is lower than some threshold
                // proportional to the strength of the signal. Since this area is directly
                // proportional to the strength as well, this implies the signal emission will
                // be trimmed after some time inversely proportional to the decay rate.
                // Intuitively, this would be like waiting until the most of pheromones
                // dispersed by the emitted signal decay, the precise amount we consider
                // negligible being more for fast decaying signals (we'll trim it earlier), and
                // less for slowly decaying ones (we'll trim it later)
                Some(SignalEmission { time, strength, .. })
                    if DECAY_THRESHOLD * (1.0 + DECAY_RATE * (self.time - time)) > strength => {}
                Some(emission) => {
                    trace!(
                        "Total strength left for emission {:?}: {}",
                        emission,
                        emission.strength / (1.0 + DECAY_RATE * emission.time)
                    );
                    self.emissions.push_front(emission);
                    visited += 1;
                }
                None => {
                    return true;
                }
            }
        }
        return self.emissions.is_empty();
    }
}
