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

/// A solver for the diffusion equation.
///
/// The diffusion equation can be formulated in its generic form as:
///
/// ∂ϕ(x, t)/∂t = ∇⋅(D(ϕ, x) ∇ϕ(x, t))
///
/// where:
/// - ϕ is the density of the diffusing material. In emergence's case, ϕ is the signal strength.
/// - D is the diffusivity of the environment. For emergence, this is a constant for now, making
///   our case equivalent to the classic [heat equation](https://en.wikipedia.org/wiki/Heat_equation).
/// - x is a location.
/// - t is the time.
/// - ∇⋅ is the divergence operator.
/// - ∇ is the gradient operator.
///
/// To solve the equation, one can use the [fundamental solution](https://en.wikipedia.org/wiki/Heat_equation#Fundamental_solutions)
/// of the heat equation, in 2D:
///
/// ϕ(x, t) = 1/(4 π D t) * exp(−x⋅x / (4 D t))
///
/// where x⋅x is a dot product.
///
/// The fundamental solution solves a problem with no boundary conditions and
/// an initial density given by the [Dirac delta function](https://en.wikipedia.org/wiki/Dirac_delta_function).
/// The general solution, for any initial condition is a convolution with the fundamental solution:
///
/// s(x, t) = ∫ Φ(x−y, t) g(y) dy
///
/// where:
/// - g is the initial condition, i.e. s(x, 0).
/// - y is a surface element in the same space as x.
///
/// Practically, we don't want to deal with costly convolutions, so we'll try to avoid them.
/// Because signals are emitted in pulses, and convolution is distributive, we can separate each
/// emission and sum their contributions:
///
/// s(x, t) = ∫ Φ(x−y, t-T0) e0(y) dy + ∫ Φ(x−y, t-T1) e1(y) dy + ... + ∫ Φ(x−y, t-Tn) en(y) dy
///
/// where:
/// - en is the initial condition of the nth emission.
/// - Tn is the time at which the nth emission was emitted.
///
/// It would be very practical to get rid of all these integrals, and one way to do so is for
/// the initial conditions of each emissions to be Dirac delta functions. This also fits very well
/// the analogy of a pulse of signal being emitted before being diffused. The solution is
/// drastically simplified:
///
/// s(x, t) = Φ(x-y0, t-T0) + Φ(x-y1, t-T1) + ... + Φ(x-yn, t-Tn)
///
/// where yn is the location of the nth signal emission.
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
