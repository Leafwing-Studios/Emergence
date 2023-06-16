//! Code that deals with rotation in hexagonal grids.

use std::f32::consts::PI;

use bevy::prelude::*;
use core::fmt::Display;
use derive_more::Display;
use hexx::Direction;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use super::MAP_LAYOUT;

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
    pub(crate) fn random(rng: &mut impl Rng) -> Self {
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
        let angle = facing.direction.angle(MAP_LAYOUT.orientation) + PI / 6.;
        let target = Quat::from_axis_angle(Vec3::Y, angle);
        transform.rotation = target;
    }
}
