//! Data for characterizing entities as terrain
use bevy::ecs::component::Component;

/// Component representing plain terrain
#[derive(Component, Clone, Copy)]
pub struct PlainTerrain;

/// Component representing impassable terrain.
#[derive(Component, Clone, Copy)]
pub struct RockyTerrain;

/// The marker component for high terrain.
#[derive(Component, Clone, Copy, Default)]
pub struct HighTerrain;
