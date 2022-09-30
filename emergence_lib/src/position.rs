use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HexDirection;
use rand::distributions::Distribution;
use rand::Rng;

/// Marker component for tiles that are not passable
#[derive(Component)]
pub struct Impassable;

/// Generates a random hexagonal direction using the `rng` and `distribution` provided.
#[allow(unused)]
fn random_direction<R: Rng + ?Sized, D: Distribution<usize>>(
    mut rng: &mut R,
    distribution: D,
) -> HexDirection {
    let choice = distribution.sample(&mut rng);
    HexDirection::from(choice)
}
