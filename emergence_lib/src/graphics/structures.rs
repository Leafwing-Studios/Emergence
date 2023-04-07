//! Graphics and animation code for structures.

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use crate::construction::ghosts::{Ghost, Preview};

/// Adds [`NotShadowCaster`] and [`NotShadowReceiver`] to all ghosts and previews
pub(super) fn remove_ghostly_shadows(
    root_query: Query<Entity, Or<(With<Ghost>, With<Preview>)>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    for root_entity in root_query.iter() {
        for child in children.iter_descendants(root_entity) {
            commands
                .entity(child)
                .insert((NotShadowCaster, NotShadowReceiver));
        }
    }
}
