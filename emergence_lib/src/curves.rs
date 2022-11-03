//! Curves that are commonly useful when defining interesting game mechanics.

use bevy::math::Vec2;

/// A type which maps from an input value to an output value that lies on a curve.
pub trait Mapping {
    /// Given some input value, map to an output so that `(input, output)` lie on the curve.
    fn map(&self, x: f32) -> f32;
}

/// Represents a line with slope `m` and y-intercept `b`.
#[derive(Clone, Copy, Debug)]
pub struct Line {
    /// The slope of the line.
    slope: f32,
    /// The y-intercept of the line. In other words, the output value when the input value is `0.0`.
    y_intercept: f32,
}

impl Line {
    /// Create a new line with specified slope and y-intercept.
    pub fn new(m: f32, b: f32) -> Line {
        Line {
            slope: m,
            y_intercept: b,
        }
    }

    /// Create a new line that goes through the given points.
    pub fn new_from_points(p0: Vec2, p1: Vec2) -> Line {
        let m = (p1.y - p0.y) / (p1.x - p0.x);
        let b = p0.y - m * p0.x;

        Line {
            slope: m,
            y_intercept: b,
        }
    }

    /// Maps an input value to an output value so that `(x, output)` is a point on the line.
    pub fn map(&self, x: f32) -> f32 {
        self.slope * x + self.y_intercept
    }
}

impl Mapping for Line {
    /// Maps an input value to an output value so that `(x, output)` is a point on the line.
    fn map(&self, x: f32) -> f32 {
        self.slope * x + self.y_intercept
    }
}

/// Represents a line that is clamped to specified `min` and `max` values.
#[derive(Clone, Copy, Debug)]
pub struct ClampedLine {
    /// Minimum output value.
    min: f32,
    /// Maximum output value.
    max: f32,
    /// Underlying [`Line`] whose output values are clamped.
    line: Line,
}

impl ClampedLine {
    /// Create a new clamped line.
    pub fn new(slope: f32, y_intercept: f32, min: f32, max: f32) -> ClampedLine {
        let line = Line { slope, y_intercept };
        ClampedLine { line, min, max }
    }

    /// Create a new clamped line that goes through the points `p0` and `p1`. The minimum value to
    /// which output will be clamped is `p1.y.min(p0.y)` and the maximum value to which output
    /// will be clamped is `p1.y.max(p0.y)`.
    pub fn new_from_points(p0: Vec2, p1: Vec2) -> ClampedLine {
        let m = (p1.y - p0.y) / (p1.x - p0.x);
        let b = p0.y - m * p0.x;

        let line = Line {
            slope: m,
            y_intercept: b,
        };
        ClampedLine {
            line,
            min: p1.y.min(p0.y),
            max: p1.y.max(p0.y),
        }
    }
}

impl Mapping for ClampedLine {
    /// Maps an input value `x` to an output value so that `(x, output)` is a point on the line if
    /// `min <= output <= max`. Otherwise, output is clamped to `min` and `max`.
    fn map(&self, x: f32) -> f32 {
        self.line.map(x).clamp(self.min, self.max)
    }
}

/// Represents a line that is clamped to specified minimum at the "bottom", but unclamped with
/// respect to a maximum at the "top".
#[derive(Clone, Copy, Debug)]
pub struct BottomClampedLine {
    /// Minimum output value.
    min: f32,
    /// Underlying [`Line`] whose output values are clamped.
    line: Line,
}

impl BottomClampedLine {
    /// Create a new bottom clamped line.
    pub fn new(slope: f32, y_intercept: f32, min: f32) -> BottomClampedLine {
        let line = Line { slope, y_intercept };
        BottomClampedLine { line, min }
    }

    /// Create a new bottom clamped line that goes through the points `p0` and `p1`. The minimum
    /// value to which output will be clamped is `p1.y.min(p0.y)`.
    pub fn new_from_points(p0: Vec2, p1: Vec2) -> BottomClampedLine {
        let m = (p1.y - p0.y) / (p1.x - p0.x);
        let b = p0.y - m * p0.x;

        let line = Line {
            slope: m,
            y_intercept: b,
        };
        BottomClampedLine {
            line,
            min: p1.y.min(p0.y),
        }
    }
}

impl Mapping for BottomClampedLine {
    /// Maps an input value `x` to an output value so that `(x, output)` is a point on the line if
    /// `min <= output`. Otherwise, output is clamped to `min`.
    fn map(&self, x: f32) -> f32 {
        self.line.map(x).max(self.min)
    }
}

/// Take a linear combination of `x` and `y`, by the factor `c`.
///
/// Formally: `linear_combination(x, y, c) = x * c + y * (1.0 - c)`.
pub fn linear_combination(x: f32, y: f32, c: f32) -> f32 {
    x * c + y * (1.0 - c)
}

/// The sigmoid function in its "normal form": without any scaling or translation.
///
/// Formally: `normal_sigmoid(x) == 1.0 / (1.0 + f32::exp(-x))`.
pub fn normal_sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + f32::exp(-x))
}

/// Inverse of the [`normal_sigmoid`].
///
/// Formally: `inverse_normal_sigmoid(normal_sigmoid(x)) == x` (barring floating point error).
pub fn inverse_normal_sigmoid(y: f32) -> f32 {
    // y = 1/(1 + e^(-x))
    // y(1 + e^(-x)) = 1
    // 1 + e^(-x) = 1/y
    // e^(-x) = (1/y) - 1
    // -x = ln((1/y) - 1)
    // x = -ln((1/y) - 1)
    -f32::ln((1.0 / y) - 1.0)
}

/// Represents a scaled and translated version of [`normal_sigmoid`].
///
/// Formally: `sigmoid.map(x) == sigmoid.vertical_scale * normal_sigmoid((x - sigmoid.horizontal_offset)/sigmoid.horizontal_scale) + sigmoid.vertical_offset`.
///
/// Use [`new`](Sigmoid::new), an ergonomic interface to smooth the process of defining the
/// necessary numbers governing the curve.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sigmoid {
    /// The distance between asymptotic maximum (output value produced at `+infinity`) and asymptotic
    /// minimum value (output value produced at `-infinity`).
    ///
    /// Given some [`horizontal_scale`](Sigmoid::horizontal_scale), this effectively defines the
    /// asymptotic maximum produced by this sigmoid.
    vertical_scale: f32,
    /// Distance between input values which produce `0.99 * asymptotic_max` and `0.01 * asymptotic_min`,
    /// where `asymptotic_max` is the output value produced at `+infinity` and `asymptotic_min`
    /// is the output value produced at `-infinity`.
    horizontal_scale: f32,
    /// Defines the asymptotic minimum (output value attained at `-infinity`) produced by this sigmoid.
    vertical_offset: f32,
    /// Defines which input value produces an output of `0.5 * (`asymptotic_max` + `asymptotic_min`)
    /// where `asymptotic_max` is the output value produced at `+infinity` and `asymptotic_min`
    /// is the output value produced at `-infinity`.
    horizontal_offset: f32,
}

impl Sigmoid {
    /// An ergonomic interface to create a create new sigmoid function which [`map`](Sigmoid::map)s
    /// from an input value (`f32`) to an output value bounded between `(min, max)`.
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

        let normal_first_percentile = inverse_normal_sigmoid(0.01);
        let normal_last_percentile = inverse_normal_sigmoid(0.99);

        let normal_delta_x = normal_last_percentile - normal_first_percentile;
        let horizontal_scale = delta_x / normal_delta_x;

        let midpoint = (first_percentile + last_percentile) / 2.0;
        let normal_midpoint = 0.0;
        let horizontal_offset = midpoint - normal_midpoint;

        Sigmoid {
            vertical_scale,
            horizontal_scale,
            vertical_offset,
            horizontal_offset,
        }
    }
}

impl Mapping for Sigmoid {
    /// Maps an input value to an output lies on the sigmoid curve.
    fn map(&self, x: f32) -> f32 {
        self.vertical_scale * normal_sigmoid((x - self.horizontal_offset) / self.horizontal_scale)
            + self.vertical_offset
    }
}
