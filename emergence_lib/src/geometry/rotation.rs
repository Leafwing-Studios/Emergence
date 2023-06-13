//! Code that deals with rotation in hexagonal grids.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use core::fmt::Display;
use derive_more::Display;
use hexx::{Direction, HexLayout, HexOrientation};
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

/// The hex direction that this entity is facing.
///
/// Stored as a component on each entity with a grid-aligned rotation.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Deref, DerefMut)]
pub(crate) struct Facing {
    /// The desired direction.
    ///
    /// Defaults to [`Direction::Top`].
    pub direction: Direction,
}

impl Facing {
    /// Generates a random facing.
    #[inline]
    #[must_use]
    pub(crate) fn random(rng: &mut ThreadRng) -> Self {
        let direction = *Direction::ALL_DIRECTIONS.choose(rng).unwrap();

        Self { direction }
    }

    /// Rotates this facing one 60 degree step counterclockwise.
    #[inline]
    pub(crate) fn rotate_counterclockwise(&mut self) {
        self.direction = self.direction.counter_clockwise();
    }

    /// Rotates this facing one 60 degree step clockwise.
    #[inline]
    pub(crate) fn rotate_clockwise(&mut self) {
        self.direction = self.direction.clockwise();
    }

    /// Returns the number of clockwise 60 degree rotations needed to face this direction, starting from [`Direction::Top`].
    ///
    /// This is intended to be paired with [`Hex::rotate_clockwise`](hexx::Hex) to rotate a hex to face this direction.
    #[inline]
    pub(crate) const fn rotation_count(&self) -> u32 {
        match self.direction {
            Direction::Top => 0,
            Direction::TopLeft => 1,
            Direction::BottomLeft => 2,
            Direction::Bottom => 3,
            Direction::BottomRight => 4,
            Direction::TopRight => 5,
        }
    }
}

impl Default for Facing {
    fn default() -> Self {
        Facing {
            direction: Direction::Top,
        }
    }
}

impl Display for Facing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self.direction {
            Direction::TopRight => "Top-right",
            Direction::Top => "Top",
            Direction::TopLeft => "Top-left",
            Direction::BottomLeft => "Bottom-left",
            Direction::Bottom => "Bottom",
            Direction::BottomRight => "Bottom-right",
        };

        write!(f, "{str}")
    }
}

/// The direction of a [`Facing`] rotation
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
pub(crate) enum RotationDirection {
    /// Counterclockwise
    Left,
    /// Clockwise
    Right,
}

impl RotationDirection {
    /// Picks a direction to rotate in at random
    #[inline]
    #[must_use]
    pub(crate) fn random(rng: &mut ThreadRng) -> Self {
        match rng.gen::<bool>() {
            true => RotationDirection::Left,
            false => RotationDirection::Right,
        }
    }
}

// BLOCKED: manual implementation of https://github.com/ManevilleF/hexx/issues/84
// PERF: this is terrible, use a lookup table
/// Converts an angle in radians to a [`Direction`].
pub(crate) fn direction_from_angle(radians: f32, orientation: HexOrientation) -> Direction {
    // Clamp to [0, 2π)
    let radians = radians.rem_euclid(TAU);

    let direction_angle_pairs = Direction::ALL_DIRECTIONS.map(|direction| {
        let angle = direction.angle(&orientation).rem_euclid(TAU);
        (direction, angle)
    });

    let mut current_best_direction = Direction::Top;
    let mut current_best_delta = f32::MAX;

    let mut lowest_direction = Direction::Top;
    let mut lowest_angle = f32::MAX;

    let mut highest_direction = Direction::Top;
    let mut highest_angle = 0.;

    for (direction, angle) in direction_angle_pairs {
        if angle > highest_angle {
            highest_direction = direction;
            highest_angle = angle;
        }

        if angle < lowest_angle {
            lowest_direction = direction;
            lowest_angle = angle;
        }

        let delta = (angle - radians).abs();
        if delta < current_best_delta {
            current_best_direction = direction;
            current_best_delta = delta;
        }
    }

    // Handle the case where the angle is between the highest and lowest angles
    if radians > highest_angle {
        let lowest_angle = lowest_angle + TAU;
        if (radians - highest_angle).abs() < (radians - lowest_angle).abs() {
            highest_direction
        } else {
            lowest_direction
        }
    } else {
        current_best_direction
    }
}

/// Rotates objects so they are facing the correct direction.
pub(crate) fn sync_rotation_to_facing(
    // Camera requires different logic, it rotates "around" a central point
    // PERF: re-enable change detection. For some reason this wasn't working on structures,
    // but was on ghosts.
    mut query: Query<(&mut Transform, &Facing), Without<Camera3d>>,
) {
    for (mut transform, &facing) in query.iter_mut() {
        // Rotate the object in the correct direction
        // We want to be aligned with the faces of the hexes, not their points
        let angle = facing.direction.angle(&HexLayout::default().orientation) + PI / 6.;
        let target = Quat::from_axis_angle(Vec3::Y, angle);
        transform.rotation = target;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexx::{Direction, HexOrientation};

    #[test]
    fn direction_from_angle_works_for_exact_values() {
        for direction in Direction::ALL_DIRECTIONS {
            let pointy_radians = direction.angle(&HexOrientation::pointy());
            let pointy_direction = direction_from_angle(pointy_radians, HexOrientation::pointy());

            let flat_radians = direction.angle(&HexOrientation::flat());
            let flat_direction = direction_from_angle(flat_radians, HexOrientation::flat());

            assert_eq!(direction, pointy_direction, "Failed for {pointy_radians:?}");
            assert_eq!(direction, flat_direction, "Failed for {flat_radians:?}");
        }
    }

    #[test]
    fn direction_from_angle_works_for_large_and_small_values() {
        for direction in Direction::ALL_DIRECTIONS {
            let radians = direction.angle(&HexOrientation::flat());

            let large_radians = radians + TAU;
            let small_radians = radians - TAU;

            let large_direction = direction_from_angle(large_radians, HexOrientation::flat());
            let small_direction = direction_from_angle(small_radians, HexOrientation::flat());

            assert_eq!(direction, large_direction, "Failed for {large_radians:?}");
            assert_eq!(direction, small_direction, "Failed for {small_radians:?}");
        }
    }

    #[test]
    fn direction_from_angle_works_with_small_offsets() {
        let epsilon = 0.001;
        assert!(epsilon < TAU / 12.);

        for direction in Direction::ALL_DIRECTIONS {
            let radians = direction.angle(&HexOrientation::flat());

            let large_radians = radians + epsilon;
            let small_radians = radians - epsilon;

            let large_direction = direction_from_angle(large_radians, HexOrientation::flat());
            let small_direction = direction_from_angle(small_radians, HexOrientation::flat());

            assert_eq!(direction, large_direction, "Failed for {large_radians:?}");
            assert_eq!(direction, small_direction, "Failed for {small_radians:?}");
        }
    }
}
