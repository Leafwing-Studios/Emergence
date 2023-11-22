//! Contains [`FallibleEntityCommandExt`] and related code.

use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
};

/// An extension trait for [`EntityCommands`] that has fallible versions of
/// the most commonly used commands.

pub trait FallibleEntityCommandExt<'w, 's, 'a> {
    /// Attempts to add a child entity to the entity.
    ///
    /// Fails silently (rather than panicking) if the entity does not exist.
    ///
    /// Fallible version of [`BuildChildren::add_child`].
    fn try_add_child(&mut self, child: Entity) -> &mut EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> FallibleEntityCommandExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn try_add_child(&mut self, child: Entity) -> &mut EntityCommands<'w, 's, 'a> {
        self.add(TryAddChild { child });
        self
    }
}

/// Command for [`FallibleEntityCommandExt::try_add_child`].
struct TryAddChild {
    /// The child entity to add.
    child: Entity,
}

impl EntityCommand for TryAddChild {
    fn apply(self, id: Entity, world: &mut World) {
        if let Some(mut entity_mut) = world.get_entity_mut(id) {
            entity_mut.add_child(self.child);
        }
    }
}
