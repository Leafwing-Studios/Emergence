//! Curves that are commonly useful when defining interesting game mechanics.

/// Take a linear combination of `x` and `y`, by the factor `c`.
///
/// Formally: `linear_combination(x, y, c) = x * c + y * (1.0 - c)`.
pub fn linear_combination(x: f32, y: f32, c: f32) -> f32 {
    x * c + y * (1.0 - c)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sigmoid {
    vertical_scale: f32,
    horizontal_scale: f32,
    vertical_offset: f32,
    horizontal_offset: f32,
}

impl Sigmoid {
    /// Create new sigmoid function which [`map`](Sigmoid::map)s from an input value (`f32`) to an
    /// output value bounded between `(min, max)`.
    ///
    /// `first_percentile` is the input value below which outputs approach close to the `min`.
    /// More precisely: `input <= first_percentile` implies `output <= min + 0.01 * (max - min)`.
    ///
    /// `last_percentile` is the input value above which outputs approach close to the `max`.
    /// More precisely: `input <= last_percentile` implies `output >=  min + 0.99 * (max - min)`.
    pub fn new(min: f32, max: f32, first_percentile: f32, last_percentile: f32) -> Sigmoid {
        let vertical_scale = max - min;
        let vertical_offset = min;
        let delta_x = last_percentile - first_percentile;

        // y = 1/(1 + e^-x)
        // y(1 + e^-x) = 1
        // 1 + e^-x = 1/y
        // e^-x = 1/y - 1
        // -x = ln(1/y - 1)
        // x = -ln(1/y - 1)
        // Substituting y = 0.01 gives the answer
        const BASE_FIRST_PERCENTILE: f32 = -4.595_12;
        const BASE_LAST_PERCENTILE: f32 = 4.595_12;

        const BASE_DELTA_X: f32 = BASE_LAST_PERCENTILE - BASE_FIRST_PERCENTILE;
        let horizontal_scale = delta_x / BASE_DELTA_X;

        let midpoint = (first_percentile + last_percentile) / 2.0;
        const BASE_MIDPOINT: f32 = 0.;
        let horizontal_offset = midpoint - BASE_MIDPOINT;

        Sigmoid {
            vertical_scale,
            horizontal_scale,
            vertical_offset,
            horizontal_offset,
        }
    }

    /// Maps input value to an output value using stored settings.
    pub fn map(&self, x: f32) -> f32 {
        self.vertical_scale
            / (1. + f32::exp(-1.0 * (x - self.horizontal_offset) / self.horizontal_scale))
            + self.vertical_offset
    }
}

pub struct LinearSigmoid {
    min: f32,
    max: f32,
    m: f32,
    b: f32,
    input_at_min: f32,
}

impl LinearSigmoid {
    /// Create new sigmoid function which [`map`](Sigmoid::map)s from an input value (`f32`) to an
    /// output value bounded between `(min, max)`.
    ///
    /// `first_percentile` is the input value below which outputs approach close to the `min`.
    /// More precisely: `input <= first_percentile` implies `output <= min + 0.01 * (max - min)`.
    ///
    /// `last_percentile` is the input value above which outputs approach close to the `max`.
    /// More precisely: `input <= last_percentile` implies `output >=  min + 0.99 * (max - min)`.
    pub fn new(input_at_min: f32, input_at_max: f32, min: f32, max: f32) -> LinearSigmoid {
        let m = (max - min) / (input_at_max - input_at_min);
        let b = min - m * input_at_min;

        LinearSigmoid {
            min,
            max,
            m,
            b,
            input_at_min,
        }
    }

    /// Maps input value to an output value using stored settings.
    pub fn map(&self, x: f32) -> f32 {
        (self.m * (x - self.input_at_min + self.b))
            .max(self.min)
            .min(self.max)
    }
}
