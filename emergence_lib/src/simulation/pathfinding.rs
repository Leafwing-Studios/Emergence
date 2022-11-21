//! Various odds and ends useful for pathfinding

use bevy::prelude::Component;

/// Marker struct specifying that an entity is impassable for pathfinding
#[derive(Component, Clone, Copy, Default)]
pub struct PathfindingImpassable;
