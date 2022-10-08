//! The metaphorical junk drawer.

/// Take a linear combination of `x` and `y`, by the factor `c`.
///
/// Formally: `linear_combination(x, y, c) = x * c + y * (1.0 - c)`.
pub fn linear_combination(x: f32, y: f32, c: f32) -> f32 {
    x * c + y * (1.0 - c)
}

/// A signmoid curve
pub fn sigmoid(
    x: f32,
    vertical_scale: f32,
    horizontal_scale: f32,
    vertical_offset: f32,
    horizontal_offset: f32,
) -> f32 {
    vertical_scale / (1. + f32::exp(-1.0 * (x - horizontal_offset) / horizontal_scale))
        + vertical_offset
}

/// A sigmoid curve with more human-comprehensible parameterization
pub fn ergonomic_sigmoid(
    x: f32,
    min: f32,
    max: f32,
    x_first_percentile: f32,
    x_last_percentile: f32,
) -> f32 {
    let vertical_scale = max - min;
    let vertical_offset = min;
    let delta_x = x_last_percentile - x_first_percentile;

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

    let midpoint = (x_first_percentile + x_last_percentile) / 2.0;
    const BASE_MIDPOINT: f32 = 0.;
    let horizontal_offset = midpoint - BASE_MIDPOINT;
    sigmoid(
        x,
        vertical_scale,
        horizontal_scale,
        vertical_offset,
        horizontal_offset,
    )
}
